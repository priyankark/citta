#!/usr/bin/env python3
"""Repeatable onboarding with isolated Tailscale/harness fixtures and real Citta."""
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import tomllib
import unittest
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get('CITTA_TEST_BINARY', ROOT / 'target/debug/citta')).resolve()

class Setup(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix='citta-setup-')
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.config = self.root / 'device/config.toml'
        self.harness = self.root / 'harness'
        self.harness.mkdir()
        self.harness.joinpath('config.toml').write_text('model="fixture-model"\n[mcp_servers.other]\ncommand="preserve-me"\n[mcp_servers.citta]\nenabled=false\n')
        self.ts = self.root / 'tailscale'
        self.ts.write_text(f'''#!{sys.executable}
import json, os, pathlib, sys
root=pathlib.Path(__file__).parent
assert os.environ.get('TAILSCALE_BE_CLI') == '1'
args=sys.argv[1:]
if args == ['status', '--json']:
 print(json.dumps({{'BackendState':'Stopped' if (root/'offline').exists() else 'Running','Self':{{'ID':'other' if (root/'identity-changed').exists() else 'fixture-device','UserID':7,'DNSName':'fixture.tail-test.ts.net.','TailscaleIPs':['100.64.0.1'],'Online':True}},'CurrentTailnet':{{'StableID':'fixture-tailnet','MagicDNSSuffix':'tail-test.ts.net'}},'Peer':{{}}}}))
elif args == ['serve','status','--json']:
 print(json.dumps({{'TCP':{{'7332':{{'TCPForward':'elsewhere'}}}}}} if (root/'conflict').exists() else {{}}))
elif args[:2] == ['serve','--bg']:
 (root/'serve-added').write_text('private route')
else: sys.exit(2)
''')
        self.ts.chmod(0o700)
        self.codex = self.root / 'codex'
        self.codex.write_text(f'''#!{sys.executable}
import pathlib, sys
if sys.argv[1:] == ['login','status']: sys.exit(1 if pathlib.Path(__file__).with_name('logged-out').exists() else 0)
if '--help' in sys.argv: print('--sandbox --no-daemon'); sys.exit(0)
print(sys.stdin.read())
''')
        self.codex.chmod(0o700)
        self.env = dict(os.environ, HOME=str(self.root), CODEX_HOME=str(self.harness))
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            self.port = sock.getsockname()[1]
        self.cmd = [str(BINARY), '--config', str(self.config)]
        self.setup_args = ['setup', '--no-service', '--harness-dir', str(self.harness), '--tailscale-bin', str(self.ts), '--codex-bin', str(self.codex), '--port', str(self.port)]

    def setup_device(self, *extra, ok=True):
        result = subprocess.run(self.cmd + self.setup_args + list(extra), env=self.env, capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode == 0, ok, result.stderr)
        return json.loads(result.stdout) if ok else result.stderr

    def daemon(self):
        process = subprocess.Popen(self.cmd + ['daemon'], env=self.env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        def stop():
            if process.poll() is None:
                process.terminate()
                try: process.wait(timeout=10)
                except subprocess.TimeoutExpired: process.kill(); process.wait()
        self.addCleanup(stop)
        for _ in range(100):
            try:
                self.api('/v1/node')
                return process
            except (OSError, urllib.error.URLError): time.sleep(.05)
        self.fail('fixture daemon did not start')

    def api(self, path, payload=None):
        tokenfile = self.config.parent / ('data/control.token' if '/control/' in path else 'node.token')
        request = urllib.request.Request(f'http://127.0.0.1:{self.port}{path}', headers={'Authorization':'Bearer ' + tokenfile.read_text().strip(), 'Content-Type':'application/json'}, data=json.dumps(payload).encode() if payload is not None else None)
        with urllib.request.urlopen(request, timeout=3) as response:
            return json.load(response)

    def test_fresh_setup_rerun_preserves_identity_settings_and_explicit_disable(self):
        result = self.setup_device()
        self.assertTrue(result['agent_configured'])
        self.assertTrue(result['profile_grant_pending'])
        before = self.config.read_bytes()
        token = self.config.with_name('node.token').read_bytes()
        config = tomllib.loads(before.decode())
        self.assertEqual(config['node_id'], 'fixture')
        self.assertTrue(config['admission']['guard_all'])
        profile = config['capabilities'][1]
        self.assertEqual(profile['model'], 'fixture-model')
        args = json.loads(profile['env']['CITTA_AGENT_ARGS'])
        self.assertIn('danger-full-access', args)
        self.assertEqual(profile['name'], 'codex')
        self.assertEqual(profile['slots'], config['slots'])
        self.assertEqual(profile['max_timeout_secs'], 86400)
        self.assertNotIn('mcp_servers.citta.enabled=false', args)
        self.assertNotIn('--ignore-user-config', args)
        self.assertEqual(profile['env']['CODEX_HOME'], str(self.harness))
        self.assertEqual(args[args.index('-a')+1], 'never')
        self.setup_device()
        self.assertEqual(before, self.config.read_bytes())
        self.assertEqual(token, self.config.with_name('node.token').read_bytes())
        harness = tomllib.loads(self.harness.joinpath('config.toml').read_text())
        self.assertEqual(harness['mcp_servers']['other']['command'], 'preserve-me')
        self.assertIs(harness['mcp_servers']['citta']['enabled'], False)
        self.assertTrue(self.harness.joinpath('skills/citta-delegate/SKILL.md').exists())
        self.assertEqual(list(self.config.parent.glob('.setup-*.toml')), [])

    def test_explicit_readonly_and_full_profiles_preserve_existing_choices(self):
        self.setup_device('--agent-access', 'read-only')
        config = tomllib.loads(self.config.read_text())
        readonly = config['capabilities'][1]
        self.assertEqual(readonly['name'], 'codex-readonly')
        self.assertIn('read-only', json.loads(readonly['env']['CITTA_AGENT_ARGS']))
        self.setup_device()
        self.assertEqual(len(tomllib.loads(self.config.read_text())['capabilities']), 2)
        self.setup_device('--agent-access', 'full')
        config = tomllib.loads(self.config.read_text())
        self.assertEqual(config['capabilities'][1], readonly)
        self.assertEqual(config['capabilities'][2]['name'], 'codex')
        self.assertIn('danger-full-access', json.loads(config['capabilities'][2]['env']['CITTA_AGENT_ARGS']))
        self.setup_device('--agent-access', 'full')
        self.assertEqual(len(tomllib.loads(self.config.read_text())['capabilities']), 3)

    def test_dry_run_and_offline_preflight_do_not_initialize(self):
        original = self.harness.joinpath('config.toml').read_bytes()
        self.assertTrue(self.setup_device('--dry-run')['dry_run'])
        self.assertFalse(self.config.exists())
        self.assertFalse(self.root.joinpath('serve-added').exists())
        self.assertEqual(original, self.harness.joinpath('config.toml').read_bytes())
        self.root.joinpath('offline').touch()
        self.assertIn('Tailscale', self.setup_device(ok=False))
        self.assertFalse(self.config.exists())

    def test_conflict_and_identity_change_leave_existing_configuration_intact(self):
        self.setup_device()
        before = self.config.read_bytes()
        for marker in ['conflict', 'identity-changed']:
            self.root.joinpath(marker).touch()
            self.setup_device(ok=False)
            self.assertEqual(before, self.config.read_bytes())
            self.assertEqual(list(self.config.parent.glob('.setup-*.toml')), [])
            self.root.joinpath(marker).unlink()

    def test_login_then_rerun_adds_profile_without_reinitializing(self):
        self.root.joinpath('logged-out').touch()
        result = self.setup_device()
        self.assertFalse(result['agent_configured'])
        self.assertEqual(result['agent'], 'harness_login_required')
        token = self.config.with_name('node.token').read_bytes()
        self.root.joinpath('logged-out').unlink()
        self.assertTrue(self.setup_device()['agent_configured'])
        self.assertEqual(token, self.config.with_name('node.token').read_bytes())

    def test_pending_grant_resumes_preserving_paused_mode_and_is_not_regranted(self):
        self.setup_device('--no-agent')
        process = self.daemon()
        state = self.api('/v1/control/permissions')['snapshot']
        policy = state['policy']
        policy.update(mode='paused', allowed_capabilities=['echo'], allowed_origins=['local:fixture'])
        self.api('/v1/control/permissions', {'expected_revision':state['revision'], 'policy':policy})
        process.terminate(); process.wait(timeout=10)
        self.setup_device()
        process = self.daemon()
        result = self.setup_device()
        self.assertFalse(result['profile_grant_pending'])
        state = self.api('/v1/control/permissions')['snapshot']
        self.assertEqual(state['policy']['mode'], 'paused')
        self.assertEqual(state['policy']['allowed_origins'], ['local:fixture'])
        self.assertEqual(state['policy']['allowed_capabilities'], ['echo','codex'])
        policy = state['policy']; policy['allowed_capabilities'] = ['echo']
        self.api('/v1/control/permissions', {'expected_revision':state['revision'], 'policy':policy})
        self.setup_device()
        self.assertEqual(self.api('/v1/control/permissions')['snapshot']['policy']['allowed_capabilities'], ['echo'])
        result = subprocess.run(self.cmd + ['doctor'], env=self.env, capture_output=True, text=True, check=True)
        self.assertEqual(json.loads(result.stdout)['local']['node_id'], 'fixture')

    def test_prompt_adapter_uses_literal_stdin_and_rejects_argument_injection(self):
        env = dict(self.env, CITTA_AGENT_PROGRAM=sys.executable, CITTA_AGENT_ARGS=json.dumps(['-c','import sys; print(sys.stdin.read())']))
        prompt = 'hello; $(touch must-not-exist)'
        result = subprocess.run([str(BINARY), 'builtin', 'prompt'], input=json.dumps({'prompt':prompt}), text=True, capture_output=True, env=env, check=True)
        self.assertEqual(result.stdout.strip(), prompt)
        result = subprocess.run([str(BINARY), 'builtin', 'prompt'], input=json.dumps({'prompt':prompt,'args':['--dangerous']}), text=True, capture_output=True, env=env)
        self.assertNotEqual(result.returncode, 0)

    def test_cli_default_uses_home_config_and_ignores_working_directory_config(self):
        self.config = self.root / '.config/citta/config.toml'
        self.cmd = [str(BINARY)]
        self.setup_device('--no-agent')
        self.assertTrue(self.config.exists())
        result = subprocess.run([str(BINARY), 'check'], cwd=self.root, env=self.env, capture_output=True, text=True, check=True)
        self.assertEqual(json.loads(result.stdout)['node_id'], 'fixture')

if __name__ == '__main__':
    unittest.main(verbosity=2)
