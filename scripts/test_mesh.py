#!/usr/bin/env python3
"""Exercise real daemon processes and MCP stdio using only Python's standard library."""
import argparse
import concurrent.futures
import json
import http.cookiejar
import io
import os
from pathlib import Path
import queue
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest import mock
import urllib.error
import urllib.parse
import urllib.request
import uuid

BINARY = None

def free_port():
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        return sock.getsockname()[1]

def wait_for(fn, timeout=15):
    deadline = time.monotonic() + timeout
    last = None
    while time.monotonic() < deadline:
        try:
            value = fn()
            if value:
                return value
        except (OSError, urllib.error.URLError) as exc:
            last = exc
        time.sleep(0.1)
    raise AssertionError(f'condition timed out: {last}')

class Mesh(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory(prefix='citta-test-')
        cls.root = Path(cls.tmp.name)
        cls.nodes = {}
        cls.logs = []
        cls.processes = []
        cls.addClassCleanup(cls.cleanup)
        worker_script = cls.root / 'worker.py'
        worker_script.write_text('''import json, os, subprocess, sys, time
v = json.load(sys.stdin)
if v.get("stderr_output"):
    sys.stderr.buffer.write(v.get("stderr_character", "x").encode() * v["stderr_output"] + b"\\nCITTA_DIAGNOSTIC_END\\n")
    sys.stderr.buffer.flush()
if v.get("spawn"):
    child = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(300)"])
    with open(v["pid_file"], "w") as f: f.write(str(child.pid))
if v.get("output"):
    print(v.get("character", "x") * v["output"])
    if v.get("both_streams"): print(v.get("character", "x") * v["output"], file=sys.stderr)
else:
    time.sleep(v.get("sleep", 20 if v.get("prompt") == "wait-long" else 2 if v.get("prompt") == "wait" else 0))
    print(json.dumps({"worker":os.environ["CITTA_WORKER_NODE"], "input":v,
                      "secret":os.environ.get("CITTA_TEST_SECRET"), "cwd":os.getcwd()}))
sys.exit(v.get("exit_code", 0))
''')
        for name in ('alpha', 'beta'):
            directory = cls.root / name
            port = free_port()
            subprocess.run([str(BINARY), 'init', '--dir', str(directory), '--node', name, '--port', str(port)], check=True, capture_output=True)
            cls.nodes[name] = {'dir': directory, 'port': port, 'token': (directory / 'node.token').read_text(), 'process': None}
        for name, node in cls.nodes.items():
            peer_name = 'beta' if name == 'alpha' else 'alpha'
            peer = cls.nodes[peer_name]
            # Only beta has the test capability. Alpha cannot execute it locally.
            text = f'''node_id = "{name}"
listen = "127.0.0.1:{node['port']}"
data_dir = {json.dumps(str(node['dir'] / 'data'))}
token_file = {json.dumps(str(node['dir'] / 'node.token'))}
slots = 1
lease_secs = 6
max_pending = 100
max_history = 200
[[peers]]
node_id = "{peer_name}"
url = "http://127.0.0.1:{peer['port']}"
token_file = {json.dumps(str(peer['dir'] / 'node.token'))}
'''
            if name == 'beta':
                text += f'''[[capabilities]]
name = "compute"
description = "Remote test worker"
program = {json.dumps(sys.executable)}
args = [{json.dumps(str(worker_script))}]
slots = 1
max_timeout_secs = 30
'''
                text += f'''[[capabilities]]
name = "agent-test"
harness = "fixture-harness"
model = "fixture-model"
kind = "agent"
description = "Fixed test agent profile"
program = {json.dumps(sys.executable)}
args = [{json.dumps(str(worker_script))}]
slots = 1
max_timeout_secs = 30
'''
                text += f'''[[capabilities]]
name = "alternate-agent"
kind = "agent"
harness = "alternate-harness"
model = "alternate-model"
description = "Different harness using the same neutral execution contract"
program = {json.dumps(sys.executable)}
args = [{json.dumps(str(worker_script))}]
slots = 1
max_timeout_secs = 30
'''
            (node['dir'] / 'config.toml').write_text(text)
        cls.start('alpha')
        cls.start('beta')

    @classmethod
    def start(cls, name):
        node = cls.nodes[name]
        log = open(node['dir'] / 'daemon.log', 'ab')
        cls.logs.append(log)
        env = dict(os.environ, CITTA_TEST_SECRET='must-not-reach-child')
        proc = subprocess.Popen([str(BINARY), '--config', str(node['dir'] / 'config.toml'), 'daemon'], stdout=log, stderr=log, env=env)
        node['process'] = proc
        cls.processes.append(proc)
        wait_for(lambda: cls.api(name, '/healthz')['ok'])

    @classmethod
    def stop(cls, name):
        proc = cls.nodes[name]['process']
        if proc.poll() is None:
            proc.terminate()
            proc.wait(timeout=25)
        cls.nodes[name]['process'] = None

    @classmethod
    def cleanup(cls):
        for proc in cls.processes:
            if proc.poll() is None:
                proc.terminate()
                try: proc.wait(timeout=25)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait()
        for log in cls.logs:
            log.close()
        cls.tmp.cleanup()

    @classmethod
    def api(cls, node, path, data=None, token=None, headers=None):
        info = cls.nodes[node]
        request = urllib.request.Request(f'http://127.0.0.1:{info["port"]}{path}', data=None if data is None else json.dumps(data).encode(), headers={'Authorization': f'Bearer {info["token"] if token is None else token}', 'Content-Type':'application/json', **(headers or {})})
        started = time.monotonic()
        try:
            with urllib.request.urlopen(request, timeout=10) as response:
                return json.load(response)
        except urllib.error.HTTPError as error:
            error.close()
            raise
        except (TimeoutError, urllib.error.URLError) as error:
            if isinstance(error, TimeoutError) or isinstance(error.reason, TimeoutError):
                # Preserve the original timeout and never replay a possibly
                # committed mutation. Capture evidence before fixture cleanup.
                try:
                    cls.timeout_diagnostics(node, request.get_method(), path, time.monotonic() - started)
                except Exception:
                    pass
            raise

    @classmethod
    def timeout_diagnostics(cls, node, method, path, elapsed):
        secrets = [info['token'].strip() for info in cls.nodes.values()]
        for info in cls.nodes.values():
            try:
                with (info['dir'] / 'data/control.token').open() as stream:
                    secrets.append(stream.read(512).strip())
            except OSError:
                pass
        daemons = []
        for name, info in cls.nodes.items():
            process = info['process']
            diagnostic = {'node':name, 'pid':process.pid if process else None,
                          'exit_status':process.poll() if process else None}
            try:
                with (info['dir'] / 'daemon.log').open('rb') as stream:
                    stream.seek(0, os.SEEK_END)
                    stream.seek(max(0, stream.tell() - 4096))
                    tail = stream.read(4096).decode('utf-8', errors='replace')
                for secret in secrets:
                    if secret:
                        tail = tail.replace(secret, '[redacted]')
                diagnostic['log_tail'] = '\n'.join(tail.splitlines()[-30:])
            except OSError:
                diagnostic['log_tail'] = 'unavailable'
            daemons.append(diagnostic)
        print(json.dumps({'event':'request_timeout', 'node':node, 'method':method,
                          'path':urllib.parse.urlsplit(path).path,
                          'elapsed_seconds':round(elapsed, 3), 'daemons':daemons}),
              file=sys.stderr, flush=True)

    def submit(self, input=None, **kwargs):
        body = {'capability':'compute', 'input':input or {}, 'idempotency_key':str(uuid.uuid4()), 'target_node':'beta', 'timeout_secs':20, 'queue_ttl_secs':60, **kwargs}
        return self.api('alpha', '/v1/tasks', body)

    def terminal(self, task):
        def done():
            result = self.api('alpha', '/v1/tasks/' + task['id'])
            return result if result['status'] not in ('queued', 'running') else None
        return wait_for(done, 25)

    def test_15_agent_profiles_discovery_and_remote_spawn(self):
        discovery = wait_for(lambda: (d if (d := self.api('alpha', '/v1/agents'))['local'].get('inventory') else None))
        resources = discovery['local']['inventory']['resources']
        self.assertGreater(resources['logical_cpus'], 0)
        self.assertGreater(resources['memory_total_bytes'], 0)
        self.assertLessEqual(resources['memory_available_bytes'], resources['memory_total_bytes'])
        body = dict(profile='agent-test', prompt='wait', idempotency_key=str(uuid.uuid4()), target_node='beta', timeout_secs=20)
        for changes, status in [({'profile':'compute'},422), ({'prompt':' '},400), ({'program':'/bin/sh'},422)]:
            with self.assertRaises(urllib.error.HTTPError) as error:
                self.api('alpha', '/v1/agents/spawn', {**body, **changes})
            self.assertEqual(error.exception.code, status)
        task = self.api('alpha', '/v1/agents/spawn', body)
        repeated = self.api('alpha', '/v1/agents/spawn', body)
        self.assertEqual(task['id'], repeated['id'])
        self.assertTrue(task['request']['agent_only'])
        def agent_state():
            return next((a['state'] for a in self.api('beta', '/v1/node')['managed_agents'] if a['task_id'] == task['id']), None)
        wait_for(lambda: agent_state() == 'busy')
        result = self.terminal(task)
        self.assertEqual(result['status'], 'succeeded', result)
        self.assertEqual(json.loads(result['outcome']['stdout'])['worker'], 'beta')
        wait_for(lambda: agent_state() == 'completed')
        cancelled = self.api('alpha', '/v1/agents/spawn', {**body, 'prompt':'wait-long', 'idempotency_key':str(uuid.uuid4())})
        wait_for(lambda: any(a['task_id'] == cancelled['id'] and a['state'] == 'busy' for a in self.api('beta', '/v1/node')['managed_agents']))
        self.api('alpha', f'/v1/tasks/{cancelled["id"]}/cancel', {})
        self.assertEqual(self.terminal(cancelled)['status'], 'cancelled')
        wait_for(lambda: any(a['task_id'] == cancelled['id'] and a['state'] == 'cancelled' for a in self.api('beta', '/v1/node')['managed_agents']))
        # Retrying an acknowledged submission still works while its worker is offline.
        self.stop('beta')
        try:
            self.assertEqual(self.api('alpha', '/v1/agents/spawn', body)['id'], task['id'])
        finally:
            self.start('beta')

    def test_16_panel_sessions_origin_checks_and_read_only_inspection(self):
        for host in ('127.0.0.1', 'localhost'):
            with self.subTest(host=host):
                self.check_panel_sessions(host)

    def check_panel_sessions(self, host):
        port = self.nodes['alpha']['port']
        base = f'http://{host}:{port}'
        jar = http.cookiejar.CookieJar()
        browser = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
        def request(path, data=None, headers=None):
            req = urllib.request.Request(base + path, data=None if data is None else json.dumps(data).encode(), headers={'Content-Type':'application/json', **(headers or {})})
            try:
                return browser.open(req, timeout=15)
            except urllib.error.HTTPError as error:
                error.close()
                raise
        with request('/') as response:
            self.assertIn("frame-ancestors 'none'", response.headers['Content-Security-Policy'])
            self.assertEqual(response.headers['Cache-Control'], 'no-store')
        with self.assertRaises(urllib.error.HTTPError) as error:
            request('/panel/overview')
        self.assertEqual(error.exception.code, 401)
        link = self.api('alpha', '/v1/panel/session', {}, token=(self.nodes['alpha']['dir'] / 'data/control.token').read_text())['url']
        code = link.split('#login=')[1]
        with self.assertRaises(urllib.error.HTTPError) as error:
            request('/panel/login', {'code':code}, {'Origin':'https://evil.example'})
        self.assertEqual(error.exception.code, 403)
        with request('/panel/login', {'code':code}, {'Origin':base}) as response:
            cookie = response.headers['Set-Cookie']
            self.assertIn('HttpOnly', cookie)
            self.assertIn('SameSite=Strict', cookie)
        with self.assertRaises(urllib.error.HTTPError) as error:
            request('/panel/login', {'code':code})
        self.assertEqual(error.exception.code, 401)
        for headers in (
            {'Host':'evil.example'},
            {'Host':f'localhost.evil.example:{port}'},
            {'Host':f'localhost:{port + 1}'},
            {'Origin':'https://evil.example'},
            {'Origin':f'http://localhost.evil.example:{port}'},
            {'Origin':f'http://localhost:{port + 1}'},
            {'Origin':f'https://localhost:{port}'},
            {'Sec-Fetch-Site':'cross-site'},
        ):
            with self.assertRaises(urllib.error.HTTPError) as error:
                request('/panel/overview', headers=headers)
            self.assertEqual(error.exception.code, 403)
        with request('/panel/overview') as response:
            overview = json.load(response)
            self.assertEqual(overview['discovery']['local']['node_id'], 'alpha')
            self.assertNotIn(self.nodes['alpha']['token'], json.dumps(overview))
        task = self.terminal(self.submit())
        with request('/panel/task?node=alpha&id=' + task['id']) as response:
            self.assertEqual(json.load(response)['status'], 'succeeded')
        # A browser cookie cannot authorize any mesh mutation.
        with self.assertRaises(urllib.error.HTTPError) as error:
            request('/v1/tasks/' + task['id'] + '/cancel', {})
        self.assertEqual(error.exception.code, 401)
        with request('/panel/permissions?node=alpha') as response:
            policy_view=json.load(response)
        edit={'node':'alpha','expected_revision':policy_view['snapshot']['revision'],'policy':{**policy_view['snapshot']['policy'],'mode':'allow_all'}}
        with self.assertRaises(urllib.error.HTTPError) as error:
            request('/panel/permissions',edit)
        self.assertEqual(error.exception.code,403)
        with request('/panel/permissions',edit,{'X-Citta-Panel':'1','Origin':base}) as response:
            saved=json.load(response)
            self.assertEqual(saved['policy']['mode'],'allow_all')
        with self.assertRaises(urllib.error.HTTPError) as error:
            request('/panel/permissions',edit,{'X-Citta-Panel':'1','Origin':base})
        self.assertEqual(error.exception.code,409)
        with request('/panel/permissions', {**edit,'expected_revision':saved['revision'],'policy':policy_view['snapshot']['policy']}, {'X-Citta-Panel':'1','Origin':base}):pass
        with request('/panel/logout', {}): pass
        with self.assertRaises(urllib.error.HTTPError) as error:
            request('/panel/overview')
        self.assertEqual(error.exception.code, 401)

    def control(self, node, body=None):
        token=(self.nodes[node]['dir'] / 'data/control.token').read_text()
        return self.api(node, '/v1/control/permissions', body, token=token)

    def test_17_permissions_enforce_scope_revocation_conflicts_and_restart(self):
        initial=self.control('beta')['snapshot']
        def change(policy, revision=None):
            current=self.control('beta')['snapshot']
            return self.control('beta', {'expected_revision':current['revision'] if revision is None else revision,'policy':policy})
        for path in ['/v1/control/permissions', '/v1/panel/session']:
            with self.assertRaises(urllib.error.HTTPError) as error:
                self.api('beta',path,{} if path.endswith('session') else None)
            self.assertEqual(error.exception.code,401)
        control_token=(self.nodes['beta']['dir'] / 'data/control.token').read_text()
        with self.assertRaises(urllib.error.HTTPError) as error:
            self.api('beta','/v1/node',token=control_token)
        self.assertEqual(error.exception.code,401)
        try:
            policy={**initial['policy'],'mode':'restricted','allowed_capabilities':[]}
            state=change(policy)
            with self.assertRaises(urllib.error.HTTPError) as error:
                change(initial['policy'],revision=initial['revision'])
            self.assertEqual(error.exception.code,409)
            # A disabled capability cannot claim work; a short queue TTL gives a durable outcome.
            task=self.submit(queue_ttl_secs=1)
            self.assertEqual(self.terminal(task)['status'],'interrupted')
            self.assertIsNone(self.api('alpha','/v1/tasks/'+task['id'])['worker_node'])
            change({**policy,'mode':'allow_all','allowed_origins':['local:beta']})
            task=self.submit(queue_ttl_secs=1)
            self.assertEqual(self.terminal(task)['status'],'interrupted')
            change({**policy,'mode':'allow_all','allowed_origins':['peer:alpha']})
            self.assertEqual(self.terminal(self.submit())['status'],'succeeded')
            task=self.submit({'sleep':20})
            wait_for(lambda:self.api('alpha','/v1/tasks/'+task['id'])['status']=='running')
            paused=change({**policy,'mode':'paused'})
            # Even a rapid re-enable cannot resurrect a run whose grant was revoked.
            change({**initial['policy'],'mode':'allow_all'})
            result=self.terminal(task)
            self.assertEqual(result['status'],'interrupted',result)
            self.assertIn('permissions',result['outcome']['error'])
            self.stop('beta');self.start('beta')
            current=self.control('beta')
            self.assertEqual(current['snapshot']['policy']['mode'],'allow_all')
            self.assertGreater(current['snapshot']['revision'],paused['revision'])
            self.assertTrue(any(row['policy']['mode']=='paused' for row in current['history']))
        finally:
            change(initial['policy'])

    def test_18_harness_and_model_metadata_follow_the_selected_profile(self):
        d=self.api('alpha','/v1/agents')
        node=next(p['node'] for p in d['peers'] if p['node_id']=='beta')
        profiles={c['name']:c for c in node['capabilities']}
        self.assertEqual(profiles['agent-test']['harness'],'fixture-harness')
        self.assertEqual(profiles['alternate-agent']['model'],'alternate-model')
        task=self.api('alpha','/v1/agents/spawn',{'profile':'alternate-agent','prompt':'wait','idempotency_key':str(uuid.uuid4()),'target_node':'beta','timeout_secs':20})
        def observed():
            return next((a for a in self.api('beta','/v1/node')['managed_agents'] if a['task_id']==task['id']),None)
        agent=wait_for(observed)
        self.assertEqual(agent['harness'],'alternate-harness')
        self.assertEqual(agent['model'],'alternate-model')
        self.assertEqual(agent['profile'],'alternate-agent')
        self.assertEqual(self.terminal(task)['status'],'succeeded')

    def test_01_discovery_and_remote_execution(self):
        found = self.api('alpha', '/v1/discover')
        self.assertEqual(found['local']['capabilities'], [])
        self.assertEqual(found['peers'][0]['node']['capabilities'][0]['name'], 'compute')
        task = self.submit({'value':42})
        result = self.terminal(task)
        self.assertEqual(result['status'], 'succeeded', result)
        output = json.loads(result['outcome']['stdout'])
        self.assertEqual(output['worker'], 'beta')
        self.assertEqual(output['input']['value'], 42)
        self.assertIsNone(output['secret'])
        self.assertFalse(Path(output['cwd']).exists())
        self.assertNotIn('lease_token', json.dumps(result))
        activity = self.api('beta', '/v1/activity')
        self.assertTrue(any(t['id'] == task['id'] for t in activity['peers'][0]['tasks']))

    def test_02_authentication_and_browser_origin_rejection(self):
        for token, headers, expected in [('wrong', {}, 401), (None, {'Origin':'https://evil.example'}, 403)]:
            with self.assertRaises(urllib.error.HTTPError) as error:
                self.api('alpha', '/v1/node', token=token, headers=headers)
            self.assertEqual(error.exception.code, expected)

    def test_03_idempotency_and_conflict(self):
        key = str(uuid.uuid4())
        first = self.submit(idempotency_key=key)
        repeated = self.submit(idempotency_key=key)
        self.assertEqual(first['id'], repeated['id'])
        with self.assertRaises(urllib.error.HTTPError) as error:
            self.submit({'different':True}, idempotency_key=key)
        self.assertEqual(error.exception.code, 409)
        self.terminal(first)

    def test_04_capacity_and_concurrent_submissions(self):
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            tasks = list(pool.map(lambda _: self.submit({'sleep':0.6}), range(4)))
        deadline = time.monotonic() + 20
        while True:
            self.assertLess(time.monotonic(), deadline, "concurrent tasks did not finish")
            records = [self.api('alpha', '/v1/tasks/' + t['id']) for t in tasks]
            self.assertLessEqual(sum(t['status'] == 'running' for t in records), 1)
            if all(t['status'] == 'succeeded' for t in records): break
            self.assertTrue(all(t['status'] in ('queued', 'running', 'succeeded') for t in records))
            time.sleep(0.1)
        self.assertEqual(self.api('beta', '/v1/node')['slots'], 1)

    def test_05_cancel_running_process_tree(self):
        pid_file = self.root / 'descendant.pid'
        task = self.submit({'spawn':True, 'pid_file':str(pid_file), 'sleep':20})
        wait_for(pid_file.exists)
        pid = int(pid_file.read_text())
        self.api('alpha', f'/v1/tasks/{task["id"]}/cancel', {})
        self.assertEqual(self.terminal(task)['status'], 'cancelled')
        def dead():
            try: os.kill(pid, 0)
            except ProcessLookupError: return True
            # A zombie is already dead, pending reaping by init on Linux.
            stat = Path(f'/proc/{pid}/stat')
            return stat.exists() and stat.read_text().split()[2] == 'Z'
        wait_for(dead, 10)

    def test_06_timeout_output_limit_and_input_validation(self):
        task = self.submit({'sleep':5}, timeout_secs=1)
        self.assertEqual(self.terminal(task)['status'], 'failed')
        task = self.submit({'output':300000})
        result = self.terminal(task)
        self.assertEqual(result['status'], 'failed')
        self.assertIn('output exceeds', result['outcome']['error'])
        with self.assertRaises(urllib.error.HTTPError) as error:
            self.submit({'data':'x' * 70000})
        self.assertEqual(error.exception.code, 400)

    def test_07_restart_persistence_and_queue_expiry(self):
        done = self.terminal(self.submit())
        queued = self.submit(capability='unavailable', queue_ttl_secs=1)
        self.stop('alpha')
        self.start('alpha')
        self.assertEqual(self.api('alpha', '/v1/tasks/' + done['id'])['status'], 'succeeded')
        self.assertEqual(self.terminal(queued)['status'], 'interrupted')
        events = self.api('alpha', '/v1/events?after=0')
        self.assertEqual([e['sequence'] for e in events], sorted({e['sequence'] for e in events}))

    def test_08_lost_lease_is_terminal_without_replay(self):
        task = self.submit(capability='manual')
        claim = self.api('alpha', '/v1/claim', {'worker_node':'beta', 'capability':'manual', 'disk':self.api('beta', '/v1/node')['disk']})
        self.assertEqual(claim['task']['id'], task['id'])
        result = self.terminal(task)
        self.assertEqual(result['status'], 'interrupted')
        self.assertIsNone(self.api('alpha', '/v1/claim', {'worker_node':'beta', 'capability':'manual', 'disk':self.api('beta', '/v1/node')['disk']}))
        with self.assertRaises(urllib.error.HTTPError) as error:
            self.api('alpha', f'/v1/tasks/{task["id"]}/renew', {'lease_token':claim['lease_token']})
        self.assertEqual(error.exception.code, 409)

    def test_09_mcp_handshake_tools_and_delegation(self):
        proc = subprocess.Popen([str(BINARY), '--config', str(self.nodes['alpha']['dir'] / 'config.toml'), 'mcp'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        messages = queue.Queue()
        def reader():
            for line in proc.stdout:
                messages.put(json.loads(line))
        thread = threading.Thread(target=reader, daemon=True)
        thread.start()
        def send(message):
            proc.stdin.write(json.dumps({'jsonrpc':'2.0', **message}) + '\n')
            proc.stdin.flush()
        def call(id, method, params):
            send({'id':id,'method':method,'params':params})
            while True:
                message = messages.get(timeout=10)
                if message.get('id') == id:
                    self.assertNotIn('error', message)
                    return message['result']
        try:
            result = call(1, 'initialize', {'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'citta-test','version':'1'}})
            self.assertIn('tools', result['capabilities'])
            send({'method':'notifications/initialized'})
            tools = call(2, 'tools/list', {})['tools']
            self.assertEqual({t['name'] for t in tools}, {'citta_discover','citta_agents','citta_spawn_agent','citta_delegate','citta_placement','citta_lookup','citta_activity','citta_submit','citta_inspect','citta_tasks','citta_cancel'})
            response = call(3, 'tools/call', {'name':'citta_submit','arguments':{'capability':'compute','input':{'via':'mcp'},'target_node':'beta','idempotency_key':str(uuid.uuid4())}})
            self.assertFalse(response.get('isError', False), response)
            task = json.loads(response['content'][0]['text'])
            self.assertEqual(self.terminal(task)['status'], 'succeeded')
        finally:
            proc.stdin.close()
            try: proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.terminate()
                proc.wait(timeout=5)
            proc.stdout.close()
            thread.join(timeout=2)

    def test_11_hard_worker_crash_stops_descendants(self):
        pid_file = self.root / 'crash-descendant.pid'
        task = self.submit({'spawn':True, 'pid_file':str(pid_file), 'sleep':20})
        wait_for(pid_file.exists)
        pid = int(pid_file.read_text())
        proc = self.nodes['beta']['process']
        proc.kill()
        proc.wait(timeout=5)
        try:
            def dead():
                try: os.kill(pid, 0)
                except ProcessLookupError: return True
                stat = Path(f'/proc/{pid}/stat')
                return stat.exists() and stat.read_text().split()[2] == 'Z'
            wait_for(dead, 5)
            self.assertEqual(self.terminal(task)['status'], 'interrupted')
        finally:
            self.start('beta')
        time.sleep(0.5)
        self.assertEqual(self.api('alpha', '/v1/tasks/' + task['id'])['status'], 'interrupted')

    def test_12_origin_disconnect_stops_remote_work(self):
        pid_file = self.root / 'disconnect-descendant.pid'
        task = self.submit({'spawn':True, 'pid_file':str(pid_file), 'sleep':20})
        wait_for(pid_file.exists)
        pid = int(pid_file.read_text())
        self.stop('alpha')
        try:
            def dead():
                try: os.kill(pid, 0)
                except ProcessLookupError: return True
                stat = Path(f'/proc/{pid}/stat')
                return stat.exists() and stat.read_text().split()[2] == 'Z'
            wait_for(dead, 8)
        finally:
            self.start('alpha')
        self.assertEqual(self.terminal(task)['status'], 'interrupted')

    def test_13_successful_command_cleans_remaining_descendants(self):
        pid_file = self.root / 'completed-descendant.pid'
        task = self.submit({'spawn':True, 'pid_file':str(pid_file)})
        result = self.terminal(task)
        self.assertEqual(result['status'], 'succeeded', result)
        pid = int(pid_file.read_text())
        def dead():
            try: os.kill(pid, 0)
            except ProcessLookupError: return True
            stat = Path(f'/proc/{pid}/stat')
            return stat.exists() and stat.read_text().split()[2] == 'Z'
        wait_for(dead, 5)

    def test_14_json_escaping_preserves_valid_bounded_output(self):
        task = self.submit({'output':250000, 'character':'\0', 'both_streams':True})
        result = self.terminal(task)
        self.assertEqual(result['status'], 'succeeded', result['status'])
        self.assertEqual(len(result['outcome']['stdout']), 250001)
        self.assertEqual(len(result['outcome']['stderr']), 250001)

    def test_20_agent_diagnostics_do_not_abort_success_or_hide_exit_status(self):
        for exit_code in (0, 17):
            with self.subTest(exit_code=exit_code):
                task = self.submit({'stderr_output':300000, 'stderr_character':'\U0001f642', 'exit_code':exit_code}, capability='agent-test')
                result = self.terminal(task)
                self.assertEqual(result['status'], 'succeeded' if exit_code == 0 else 'failed', result)
                self.assertEqual(result['outcome']['exit_code'], exit_code)
                self.assertEqual(json.loads(result['outcome']['stdout'])['worker'], 'beta')
                stderr = result['outcome']['stderr']
                self.assertLessEqual(len(stderr.encode()), 256 * 1024)
                self.assertIn('agent stderr truncated:', stderr)
                self.assertIn(str(1200000 + len(b'\nCITTA_DIAGNOSTIC_END\n')) + ' raw bytes observed', stderr)
                self.assertTrue(stderr.endswith('\nCITTA_DIAGNOSTIC_END\n'))

    def test_21_agent_timeout_retains_diagnostics_and_limits_stay_scoped(self):
        result = self.terminal(self.submit({'stderr_output':300000, 'sleep':20}, capability='agent-test', timeout_secs=1))
        self.assertEqual(result['status'], 'failed', result)
        self.assertIn('CITTA_DIAGNOSTIC_END', result['outcome']['stderr'])
        # Ordinary capability stderr and all stdout retain their strict contract.
        for body, capability in [({'stderr_output':300000}, 'compute'), ({'output':300000}, 'agent-test')]:
            with self.subTest(capability=capability):
                result = self.terminal(self.submit(body, capability=capability))
                self.assertEqual(result['status'], 'failed', result)
                self.assertIn('output exceeds', result['outcome']['error'])

    def test_22_cancelling_a_noisy_agent_stops_its_descendants(self):
        pid_file = self.root / 'noisy-agent-descendant.pid'
        task = self.submit({'stderr_output':300000, 'spawn':True, 'pid_file':str(pid_file), 'sleep':20}, capability='agent-test')
        wait_for(pid_file.exists)
        pid = int(pid_file.read_text())
        self.api('alpha', f'/v1/tasks/{task["id"]}/cancel', {})
        self.assertEqual(self.terminal(task)['status'], 'cancelled')
        def dead():
            try: os.kill(pid, 0)
            except ProcessLookupError: return True
            stat = Path(f'/proc/{pid}/stat')
            return stat.exists() and stat.read_text().split()[2] == 'Z'
        wait_for(dead, 10)
        wait_for(lambda: self.api('beta', '/v1/node')['active'] == 0)

    def test_19_resource_placement_offloads_and_retries_durably(self):
        paths = {n: node['dir'] / 'config.toml' for n, node in self.nodes.items()}
        original = {n: p.read_text() for n, p in paths.items()}
        try:
            for n in paths: self.stop(n)
            # Give alpha the same agent profile, but reserve all its memory.
            # No real memory exhaustion or production-only fixture override is needed.
            profiles = original['beta'].split('[[capabilities]]', 1)[1]
            paths['alpha'].write_text(original['alpha'] + '\n[admission]\nguard_all = true\nmemory_headroom_bytes = 1125899906842624\nmax_cpu_percent = 100\n[[capabilities]]' + profiles)
            paths['beta'].write_text(original['beta'].split('[[capabilities]]', 1)[0] + '\n[admission]\nmax_cpu_percent = 100\n[[capabilities]]' + profiles)
            for n in paths: self.start(n)
            # Policy is persisted; explicitly grant only the new test profile on alpha.
            initial = self.control('alpha')['snapshot']
            self.control('alpha', {'expected_revision': initial['revision'], 'policy': {**initial['policy'], 'allowed_capabilities': ['agent-test']}})
            req = {'profile': 'agent-test', 'requirements': {'memory_bytes': 1048576, 'cpu_cores': 0.01, 'harness':'fixture-harness', 'model':'fixture-model'}}
            def eligible():
                result = self.api('alpha', '/v1/placement', req)
                return result if any(c['node_id']=='beta' and c['eligible'] for c in result['candidates']) else None
            placement = wait_for(eligible, timeout=20)
            self.assertFalse(next(c['eligible'] for c in placement['candidates'] if c['node_id']=='alpha'))
            body = {**req, 'prompt':'resource-aware analysis', 'idempotency_key':str(uuid.uuid4()), 'timeout_secs':15, 'queue_ttl_secs':30}
            task = self.api('alpha','/v1/agents/delegate',body)
            result = self.terminal(task)
            self.assertEqual(result['status'],'succeeded',result)
            self.assertEqual(result['worker_node'],'beta')
            self.assertEqual(self.api('alpha','/v1/tasks/by-key/'+body['idempotency_key'])['id'],task['id'])
            self.stop('beta')
            self.assertEqual(self.api('alpha','/v1/agents/delegate',body)['id'],task['id'])
            self.start('beta')
            # Impossible capacity queues, never falls back to unguarded execution.
            blocked = {**body, 'idempotency_key':str(uuid.uuid4()), 'queue_ttl_secs':1, 'requirements':{**req['requirements'],'memory_bytes':1125899906842624}}
            result = self.terminal(self.api('alpha','/v1/agents/delegate',blocked))
            self.assertEqual(result['status'],'interrupted')
            self.assertIsNone(result['worker_node'])
            # An incompatible runtime is rejected before creating work.
            with self.assertRaises(urllib.error.HTTPError) as error:
                self.api('alpha','/v1/agents/delegate',{**body,'idempotency_key':str(uuid.uuid4()),'requirements':{'model':'not-installed'}})
            self.assertEqual(error.exception.code,422)
        finally:
            for n,p in paths.items():
                if self.nodes[n]['process'] is not None: self.stop(n)
                p.write_text(original[n])
                self.start(n)
            if 'initial' in locals():
                current=self.control('alpha')['snapshot']
                self.control('alpha', {'expected_revision':current['revision'],'policy':initial['policy']})

    def test_23_disk_pressure_blocks_legacy_claims_and_reports_recovery(self):
        path = self.nodes['beta']['dir'] / 'config.toml'
        original = path.read_text()
        work = self.nodes['beta']['dir'] / 'data/work'
        saved_work = work.with_name('work-disk-test-backup')
        try:
            self.stop('beta')
            # A policy reserve above host capacity simulates pressure without filling disk.
            path.write_text(original.split('[[capabilities]]', 1)[0] +
                            '\n[admission]\nguard_all = false\ndisk_headroom_bytes = 1125899906842624\n' +
                            '[[capabilities]]' + original.split('[[capabilities]]', 1)[1])
            self.start('beta')
            node = self.api('beta', '/v1/node')
            self.assertEqual(node['disk']['reason'], 'disk_low')
            self.assertFalse(node['disk']['accepting'])
            self.assertGreater(node['disk']['data_available_bytes'], 0)
            for cap in node['capabilities']:
                self.assertFalse(cap['admission']['accepting'])
                self.assertIsNone(cap['admission']['offer'])
            placement = self.api('alpha', '/v1/placement', {'profile':'agent-test', 'requirements':{}})
            candidate = next(c for c in placement['candidates'] if c['node_id'] == 'beta')
            self.assertEqual(candidate['reason'], 'disk_low')
            doctor = subprocess.run([str(BINARY), '--config', str(path), 'doctor'], check=True, capture_output=True, text=True)
            self.assertEqual(json.loads(doctor.stdout)['local']['state'], 'disk_low')
            task = self.submit(queue_ttl_secs=1)
            result = self.terminal(task)
            self.assertEqual(result['status'], 'interrupted')
            self.assertIsNone(result['worker_node'])  # No legacy claim, despite guard_all=false.
            self.stop('beta')
            # CPU/memory guards remain optional for legacy work after disk recovers.
            path.write_text(original.split('[[capabilities]]', 1)[0] +
                            '\n[admission]\nguard_all = false\nmemory_headroom_bytes = 1125899906842624\n' +
                            '[[capabilities]]' + original.split('[[capabilities]]', 1)[1])
            self.start('beta')
            self.assertTrue(self.api('beta', '/v1/node')['disk']['accepting'])
            self.assertEqual(self.terminal(self.submit())['status'], 'succeeded')
            # A non-directory work path makes its filesystem unknown, independent
            # of valid data-disk telemetry. Only manipulate this fixture's paths.
            wait_for(lambda: self.api('beta', '/v1/node')['active'] == 0)
            work.rename(saved_work)
            work.write_text('owned fixture obstruction')
            self.assertEqual(self.api('beta', '/v1/node')['disk']['reason'], 'disk_unknown')
            result = self.terminal(self.submit(queue_ttl_secs=1))
            self.assertIsNone(result['worker_node'])
            self.assertEqual(result['status'], 'interrupted')
        finally:
            if self.nodes['beta']['process'] is not None:
                self.stop('beta')
            if saved_work.exists():
                work.unlink()
                saved_work.rename(work)
            path.write_text(original)
            self.start('beta')

    def test_24_origin_rejects_old_and_unready_disk_claims_without_consuming_task(self):
        # No fixture worker exposes 'manual', so only these protocol requests
        # can claim the queued task. Verify the actual origin endpoint, not UI.
        task = self.submit(capability='manual')
        legacy = {'worker_node':'beta', 'capability':'manual'}
        good = self.api('beta', '/v1/node')['disk']
        for disk, reason in [(None, 'disk_readiness_required'),
                             ({**good, 'observed_at':0}, 'disk_unknown'),
                             ({**good, 'data_available_bytes':0, 'accepting':False, 'reason':'disk_low'}, 'disk_low'),
                             ({**good, 'work_available_bytes':None, 'accepting':False, 'reason':'disk_unknown'}, 'disk_unknown')]:
            with self.assertRaises(urllib.error.HTTPError) as error:
                self.api('alpha', '/v1/claim', legacy if disk is None else {**legacy, 'disk':disk})
            self.assertEqual(error.exception.code, 428, reason)
            stored = self.api('alpha', '/v1/tasks/' + task['id'])
            self.assertEqual(stored['status'], 'queued')
            self.assertIsNone(stored['worker_node'])
        accepted = self.api('alpha', '/v1/claim', {**legacy, 'disk':self.api('beta', '/v1/node')['disk']})
        self.assertEqual(accepted['task']['id'], task['id'])
        self.api('alpha', '/v1/tasks/' + task['id'] + '/cancel', {})

    def test_10_unreachable_peer_is_reported(self):
        self.stop('beta')
        try:
            peers = self.api('alpha', '/v1/discover')['peers']
            self.assertFalse(peers[0]['reachable'])
            queued = self.submit()
            self.assertEqual(self.api('alpha', '/v1/tasks/' + queued['id'])['status'], 'queued')
        finally:
            self.start('beta')
        self.assertEqual(self.terminal(queued)['status'], 'succeeded')

class RequestDiagnostics(unittest.TestCase):
    def test_timeout_reports_bounded_evidence_without_replaying_request(self):
        with tempfile.TemporaryDirectory(prefix='citta-timeout-diagnostics-') as temp:
            directory = Path(temp)
            (directory / 'data').mkdir()
            (directory / 'data/control.token').write_text('private-control-token')
            (directory / 'daemon.log').write_text('old line\n' * 1000 + 'private-node-token private-control-token worker ready\n')
            process = mock.Mock(pid=123)
            process.poll.return_value = None
            nodes = {'fixture':{'port':1, 'token':'private-node-token', 'dir':directory, 'process':process}}
            failure = TimeoutError('timed out')
            output = io.StringIO()
            with mock.patch.object(Mesh, 'nodes', nodes, create=True), \
                    mock.patch('urllib.request.urlopen', side_effect=failure) as opened, \
                    mock.patch('sys.stderr', output):
                with self.assertRaises(TimeoutError) as caught:
                    Mesh.api('fixture', '/v1/control/permissions?private=query', {'private':'request-payload'})
            self.assertIs(caught.exception, failure)
            opened.assert_called_once()
            self.assertEqual(opened.call_args.kwargs['timeout'], 10)
            evidence = json.loads(output.getvalue())
            self.assertEqual(evidence['method'], 'POST')
            self.assertEqual(evidence['path'], '/v1/control/permissions')
            self.assertGreaterEqual(evidence['elapsed_seconds'], 0)
            self.assertEqual(evidence['daemons'][0]['pid'], 123)
            self.assertIsNone(evidence['daemons'][0]['exit_status'])
            self.assertLessEqual(len(evidence['daemons'][0]['log_tail'].splitlines()), 30)
            self.assertIn('worker ready', evidence['daemons'][0]['log_tail'])
            for private in ['private-node-token', 'private-control-token', 'request-payload', 'private=query']:
                self.assertNotIn(private, output.getvalue())


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', default='target/debug/citta')
    args, rest = parser.parse_known_args()
    BINARY = Path(args.binary).resolve()
    unittest.main(argv=[sys.argv[0], *rest], verbosity=2)
