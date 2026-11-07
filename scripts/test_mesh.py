#!/usr/bin/env python3
"""Exercise real daemon processes and MCP stdio using only Python's standard library."""
import argparse
import concurrent.futures
import json
import http.cookiejar
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
import urllib.error
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
kind = "agent"
description = "Fixed test agent profile"
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
        try:
            with urllib.request.urlopen(request, timeout=10) as response:
                return json.load(response)
        except urllib.error.HTTPError as error:
            error.close()
            raise

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
        base = f'http://127.0.0.1:{self.nodes["alpha"]["port"]}'
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
        link = self.api('alpha', '/v1/panel/session', {})['url']
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
        for headers in ({'Host':'evil.example'}, {'Origin':'https://evil.example'}, {'Sec-Fetch-Site':'cross-site'}):
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
        with request('/panel/logout', {}): pass
        with self.assertRaises(urllib.error.HTTPError) as error:
            request('/panel/overview')
        self.assertEqual(error.exception.code, 401)

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
        claim = self.api('alpha', '/v1/claim', {'worker_node':'beta', 'capability':'manual'})
        self.assertEqual(claim['task']['id'], task['id'])
        result = self.terminal(task)
        self.assertEqual(result['status'], 'interrupted')
        self.assertIsNone(self.api('alpha', '/v1/claim', {'worker_node':'beta', 'capability':'manual'}))
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
            self.assertEqual({t['name'] for t in tools}, {'citta_discover','citta_agents','citta_spawn_agent','citta_activity','citta_submit','citta_inspect','citta_tasks','citta_cancel'})
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

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', default='target/debug/citta')
    args, rest = parser.parse_known_args()
    BINARY = Path(args.binary).resolve()
    unittest.main(argv=[sys.argv[0], *rest], verbosity=2)
