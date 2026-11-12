#!/usr/bin/env python3
"""Isolated onboarding checks; CITTA_TEST_BINARY can select a verified prebuilt Citta."""
import json
import fcntl
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

def require_test_binary():
    try:
        result = subprocess.run([str(BINARY), '--version'], stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, errors='replace', timeout=10)
        if result.returncode == 0 and result.stdout.startswith('citta '):
            return
        reason = f'exit status {result.returncode}' if result.returncode else 'unexpected version output'
    except subprocess.TimeoutExpired:
        reason = '--version did not finish within 10 seconds'
    except OSError as error:
        reason = str(error)
    raise SystemExit(
        f'Citta test binary unavailable or blocked: {BINARY} ({reason}). '
        'Run cargo build --locked, or set CITTA_TEST_BINARY to the absolute path of a verified prebuilt Citta binary. '
        'If the OS blocked execution, complete its approval for that binary before retrying. '
        'No setup scenarios were run.'
    )

def pending_profiles(receipt):
    if isinstance(receipt, list):
        return receipt
    return receipt['profiles'] if 'version' in receipt else [receipt]

class Setup(unittest.TestCase):
    def setUp(self):
        # Real home/config paths can contain spaces; every scenario must preserve them.
        temp = tempfile.TemporaryDirectory(prefix='citta setup-')
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.config = self.root / 'device/config.toml'
        self.harness = self.root / 'harness'
        self.harness.mkdir()
        self.harness.joinpath('config.toml').write_text('model="fixture-model"\n[mcp_servers.other]\ncommand="preserve-me"\n[mcp_servers.citta]\nenabled=false\n')
        self.ts = self.root / 'tailscale'
        self.ts.write_text(f'''#!{sys.executable}
import json, os, pathlib, sys, time
root=pathlib.Path(__file__).parent
assert os.environ.get('TAILSCALE_BE_CLI') == '1'
args=sys.argv[1:]
if args == ['status', '--json']:
 print(json.dumps({{'BackendState':'Stopped' if (root/'offline').exists() else 'Running','Self':{{'ID':'other' if (root/'identity-changed').exists() else 'fixture-device','UserID':7,'DNSName':'fixture.tail-test.ts.net.','TailscaleIPs':['100.64.0.1'],'Online':True}},'CurrentTailnet':{{'StableID':'fixture-tailnet','MagicDNSSuffix':'tail-test.ts.net'}},'Peer':{{}}}}))
elif args == ['serve','status','--json']:
 if (root/'serve-status-denied').exists():
  print('private-command-output', file=sys.stderr); sys.exit(1)
 state = json.loads((root/'serve-state').read_text()) if (root/'serve-state').exists() else {{}}
 print(json.dumps({{'TCP':{{'7332':{{'TCPForward':'elsewhere'}}}}}} if (root/'conflict').exists() else state))
elif args[:2] == ['serve','--bg']:
 if (root/'serve-denied').exists() or (root/'serve-timeout').exists():
  print('private-command-output', flush=True)
  print('private-command-output', file=sys.stderr, flush=True)
  if (root/'serve-timeout').exists(): time.sleep(20)
  sys.exit(1)
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

    def pending_agent_grant(self):
        """Create a real existing policy that has not granted the new profile."""
        self.setup_device('--no-agent')
        process = self.daemon()
        process.terminate(); process.wait(timeout=10)
        self.setup_device()
        self.daemon()
        state = self.api('/v1/control/permissions')['snapshot']
        self.assertNotIn('codex', state['policy']['allowed_capabilities'])
        return state

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

    def test_empty_harness_setup_installs_tools_without_inventing_a_model(self):
        self.harness.joinpath('config.toml').unlink()
        result = self.setup_device()
        self.assertTrue(result['agent_configured'])
        self.assertFalse(result['mcp_explicitly_disabled'])
        config = tomllib.loads(self.config.read_text())
        profile = next(c for c in config['capabilities'] if c['kind'] == 'agent')
        self.assertNotIn('--model', json.loads(profile['env']['CITTA_AGENT_ARGS']))
        harness = tomllib.loads(self.harness.joinpath('config.toml').read_text())
        self.assertEqual(harness['mcp_servers']['citta']['command'], str(BINARY))
        self.assertEqual(harness['mcp_servers']['citta']['args'], ['--config', str(self.config), 'mcp'])
        for skill in ['citta-setup', 'citta-delegate', 'citta-desktop']:
            self.assertTrue(self.harness.joinpath('skills', skill, 'SKILL.md').is_file())
            self.assertTrue(self.harness.joinpath('skills', skill, 'agents/openai.yaml').is_file())

    def test_desktop_assets_install_passively_repeatably_and_preserve_local_files(self):
        tools = self.root / 'desktop-tools'
        tools.mkdir()
        invoked = self.root / 'desktop-tool-invoked'
        for name in ['node', 'npm', 'swiftc', 'open', 'osascript', 'screencapture']:
            tool = tools / name
            tool.write_text(f'#!{sys.executable}\nfrom pathlib import Path\nPath({str(invoked)!r}).write_text("unexpected desktop setup")\nraise SystemExit(99)\n')
            tool.chmod(0o700)
        self.env['PATH'] = str(tools) + os.pathsep + self.env.get('PATH', '')
        installed = self.harness / 'skills/citta-desktop'
        source = ROOT / 'skills/citta-desktop'
        # Session tests import the Python helper before setup runs in CI. Their
        # generated bytecode is not a portable skill asset to embed or install.
        files = [p.relative_to(source) for p in source.rglob('*')
                 if p.is_file() and '__pycache__' not in p.relative_to(source).parts]
        self.setup_device('--dry-run')
        self.assertFalse(installed.exists())
        self.setup_device()
        for relative in files:
            self.assertEqual((installed / relative).read_bytes(), (source / relative).read_bytes())
        self.assertFalse(self.root.joinpath('.local/share/citta/desktop-control').exists())
        self.assertFalse(installed.joinpath('scripts/desktop-context').exists())
        self.assertFalse(installed.joinpath('scripts/owned-fixture').exists())
        self.assertFalse(invoked.exists())
        modified = {
            Path('SKILL.md'): b'previous locally edited skill',
            Path('scripts/desktop-control.cjs'): b'previous desktop adapter',
        }
        for relative, contents in modified.items():
            (installed / relative).write_bytes(contents)
        unrelated = installed / 'user-notes.txt'
        unrelated.write_text('keep my notes')
        compiled = installed / 'scripts/desktop-context'
        compiled.write_bytes(b'existing locally compiled helper')
        compiled.chmod(0o700)
        self.setup_device()
        backups = self.config.parent / 'setup-backups'
        saved_contents = [p.read_bytes() for p in backups.rglob('*') if p.is_file()]
        for contents in modified.values():
            self.assertIn(contents, saved_contents)
        for relative in files:
            self.assertEqual((installed / relative).read_bytes(), (source / relative).read_bytes())
        self.assertEqual(unrelated.read_text(), 'keep my notes')
        self.assertEqual(compiled.read_bytes(), b'existing locally compiled helper')
        self.assertEqual(compiled.stat().st_mode & 0o777, 0o700)
        before = {p.relative_to(installed): p.stat().st_mtime_ns for p in installed.rglob('*') if p.is_file()}
        backup_files = sorted(str(p) for p in backups.rglob('*') if p.is_file())
        self.setup_device()
        after = {p.relative_to(installed): p.stat().st_mtime_ns for p in installed.rglob('*') if p.is_file()}
        self.assertEqual(before, after)
        self.assertEqual(backup_files, sorted(str(p) for p in backups.rglob('*') if p.is_file()))
        self.assertFalse(invoked.exists())

    def test_multiple_pending_profiles_receive_grants_after_daemon_starts(self):
        self.setup_device('--no-agent')
        process = self.daemon()
        process.terminate(); process.wait(timeout=10)
        self.setup_device('--agent-access', 'read-only')
        self.setup_device('--agent-access', 'full')
        self.daemon()
        self.assertFalse(self.setup_device()['profile_grant_pending'])
        state = self.api('/v1/control/permissions')['snapshot']
        self.assertEqual(state['policy']['mode'], 'restricted')
        self.assertEqual(set(state['policy']['allowed_capabilities']), {'echo', 'codex-readonly', 'codex'})
        revision = state['revision']
        self.setup_device()
        self.assertEqual(self.api('/v1/control/permissions')['snapshot']['revision'], revision)

    def test_invalid_harness_settings_fail_before_device_or_network_mutation(self):
        for contents in ['not valid [toml', 'mcp_servers="not a table"\n', '[mcp_servers]\ncitta="not a table"\n']:
            with self.subTest(contents=contents):
                self.harness.joinpath('config.toml').write_text(contents)
                self.setup_device(ok=False)
                self.assertEqual(self.harness.joinpath('config.toml').read_text(), contents)
                self.assertFalse(self.config.exists())
                self.assertFalse(self.root.joinpath('serve-added').exists())

    def test_only_live_profiles_are_granted_until_external_daemon_restarts(self):
        self.setup_device('--no-agent')
        process = self.daemon()
        process.terminate(); process.wait(timeout=10)
        self.setup_device('--agent-access', 'read-only')
        process = self.daemon()
        result = self.setup_device('--agent-access', 'full')
        self.assertTrue(result['profile_grant_pending'])
        state = self.api('/v1/control/permissions')['snapshot']
        self.assertIn('codex-readonly', state['policy']['allowed_capabilities'])
        self.assertNotIn('codex', state['policy']['allowed_capabilities'])
        receipt = self.config.with_name('setup-pending-profile.json')
        self.assertEqual([c['name'] for c in pending_profiles(json.loads(receipt.read_text()))], ['codex'])
        process.terminate(); process.wait(timeout=10)
        self.daemon()
        self.assertFalse(self.setup_device()['profile_grant_pending'])
        state = self.api('/v1/control/permissions')['snapshot']
        self.assertEqual(set(state['policy']['allowed_capabilities']), {'echo', 'codex-readonly', 'codex'})
        self.assertFalse(receipt.exists())

    def test_changed_pending_profile_is_rejected_before_network_mutation(self):
        self.setup_device()
        before = self.config.read_bytes()
        receipt = self.config.with_name('setup-pending-profile.json')
        pending = json.loads(receipt.read_text())
        profiles = pending_profiles(pending)
        profiles[0]['model'] = 'different-than-configured-model'
        receipt.write_text(json.dumps(pending))
        self.root.joinpath('serve-added').unlink()
        self.assertIn('pending setup profile was changed', self.setup_device(ok=False))
        self.assertEqual(self.config.read_bytes(), before)
        self.assertFalse(self.root.joinpath('serve-added').exists())

    def record_grant_attempt(self, state):
        """Persist the durable boundary immediately before a grant API call."""
        receipt = self.config.with_name('setup-pending-profile.json')
        original = json.loads(receipt.read_text())
        profiles = pending_profiles(original)
        planned = dict(state['policy'], allowed_capabilities=state['policy']['allowed_capabilities'] + ['codex'])
        attempt = dict(original) if isinstance(original, dict) and 'version' in original else {}
        attempt.update({'version':1, 'legacy':False, 'profiles':profiles, 'grant_attempt':{
            'expected_revision':state['revision'], 'original_policy':state['policy'],
            'policy':planned, 'names':['codex'],
        }})
        receipt.write_text(json.dumps(attempt))
        return receipt, planned

    def test_crash_after_grant_then_owner_revocation_does_not_regrant(self):
        state = self.pending_agent_grant()
        receipt, planned = self.record_grant_attempt(state)
        # Commit the recorded POST, leaving its intent on disk as if setup died
        # before retiring the receipt. Then the owner deliberately revokes it.
        self.api('/v1/control/permissions', {'expected_revision':state['revision'], 'policy':planned})
        granted = self.api('/v1/control/permissions')['snapshot']
        self.api('/v1/control/permissions', {'expected_revision':granted['revision'], 'policy':state['policy']})
        revoked = self.api('/v1/control/permissions')['snapshot']
        result = self.setup_device()
        self.assertFalse(result['profile_grant_pending'])
        self.assertFalse(receipt.exists())
        self.assertEqual(self.api('/v1/control/permissions')['snapshot'], revoked)
        self.setup_device()
        self.assertEqual(self.api('/v1/control/permissions')['snapshot'], revoked)

    def test_uncommitted_grant_intent_retries_only_the_recorded_revision(self):
        state = self.pending_agent_grant()
        receipt, planned = self.record_grant_attempt(state)
        self.assertFalse(self.setup_device()['profile_grant_pending'])
        applied = self.api('/v1/control/permissions')['snapshot']
        self.assertEqual(applied['revision'], state['revision'] + 1)
        self.assertEqual(applied['policy'], planned)
        self.assertFalse(receipt.exists())

    def test_uncertain_grant_conflict_keeps_evidence_and_does_not_report_success(self):
        state = self.pending_agent_grant()
        receipt, _ = self.record_grant_attempt(state)
        def intent():
            value = json.loads(receipt.read_text())
            value.setdefault('legacy', False)
            return value
        original_receipt = intent()
        owner_policy = dict(state['policy'], mode='paused', allowed_origins=['local:fixture'])
        self.api('/v1/control/permissions', {'expected_revision':state['revision'], 'policy':owner_policy})
        changed = self.api('/v1/control/permissions')['snapshot']
        error = self.setup_device(ok=False)
        self.assertIn('pending setup grant needs manual reconciliation', error)
        self.assertEqual(intent(), original_receipt)
        self.assertEqual(self.api('/v1/control/permissions')['snapshot'], changed)
        # A repeated recovery attempt must remain unresolved rather than apply
        # a fresh policy update over the owner's intervening decision.
        self.assertIn('pending setup grant needs manual reconciliation', self.setup_device(ok=False))
        self.assertEqual(intent(), original_receipt)
        self.assertEqual(self.api('/v1/control/permissions')['snapshot'], changed)

    def test_legacy_receipt_preserves_owner_revocation_after_an_earlier_grant(self):
        state = self.pending_agent_grant()
        receipt = self.config.with_name('setup-pending-profile.json')
        profile = pending_profiles(json.loads(receipt.read_text()))[0]
        receipt.write_text(json.dumps(profile))
        planned = dict(state['policy'], allowed_capabilities=state['policy']['allowed_capabilities'] + ['codex'])
        self.api('/v1/control/permissions', {'expected_revision':state['revision'], 'policy':planned})
        granted = self.api('/v1/control/permissions')['snapshot']
        self.api('/v1/control/permissions', {'expected_revision':granted['revision'], 'policy':state['policy']})
        revoked = self.api('/v1/control/permissions')['snapshot']
        self.assertFalse(self.setup_device()['profile_grant_pending'])
        self.assertFalse(receipt.exists())
        self.assertEqual(self.api('/v1/control/permissions')['snapshot'], revoked)

    def test_initial_daemon_grant_revoked_before_setup_completion_is_preserved(self):
        self.setup_device()
        receipt = self.config.with_name('setup-pending-profile.json')
        self.assertIsNone(json.loads(receipt.read_text())['grant_attempt'])
        self.daemon()
        initial = self.api('/v1/control/permissions')['snapshot']
        self.assertIn('codex', initial['policy']['allowed_capabilities'])
        owner_policy = dict(initial['policy'], allowed_capabilities=['echo'])
        self.api('/v1/control/permissions', {'expected_revision':initial['revision'], 'policy':owner_policy})
        revoked = self.api('/v1/control/permissions')['snapshot']
        self.assertFalse(self.setup_device()['profile_grant_pending'])
        self.assertFalse(receipt.exists())
        self.assertEqual(self.api('/v1/control/permissions')['snapshot'], revoked)

    def test_unattempted_receipt_with_truncated_revocation_history_is_not_regranted(self):
        self.setup_device()
        receipt = self.config.with_name('setup-pending-profile.json')
        self.daemon()
        initial = self.api('/v1/control/permissions')['snapshot']
        owner_policy = dict(initial['policy'], allowed_capabilities=['echo'])
        self.api('/v1/control/permissions', {'expected_revision':initial['revision'], 'policy':owner_policy})
        revoked = self.api('/v1/control/permissions')['snapshot']
        for _ in range(21):
            self.api('/v1/control/permissions', {'expected_revision':revoked['revision'], 'policy':revoked['policy']})
            revoked = self.api('/v1/control/permissions')['snapshot']
        self.assertIn('pending setup grant needs manual reconciliation', self.setup_device(ok=False))
        self.assertTrue(receipt.exists())
        self.assertEqual(self.api('/v1/control/permissions')['snapshot'], revoked)

    def test_legacy_receipt_with_truncated_history_requires_reconciliation(self):
        state = self.pending_agent_grant()
        receipt = self.config.with_name('setup-pending-profile.json')
        profile = pending_profiles(json.loads(receipt.read_text()))[0]
        receipt.write_text(json.dumps(profile))
        # The API returns only recent policy history. An old legacy receipt
        # cannot prove an earlier grant never happened once that history drops out.
        for _ in range(21):
            self.api('/v1/control/permissions', {'expected_revision':state['revision'], 'policy':state['policy']})
            state = self.api('/v1/control/permissions')['snapshot']
        self.assertIn('pending setup grant needs manual reconciliation', self.setup_device(ok=False))
        self.assertTrue(receipt.exists())
        self.assertEqual(pending_profiles(json.loads(receipt.read_text())), [profile])
        self.assertEqual(self.api('/v1/control/permissions')['snapshot'], state)

    def test_new_profile_on_mature_policy_uses_its_recorded_baseline(self):
        self.setup_device('--no-agent')
        process = self.daemon()
        state = self.api('/v1/control/permissions')['snapshot']
        for _ in range(21):
            self.api('/v1/control/permissions', {'expected_revision':state['revision'], 'policy':state['policy']})
            state = self.api('/v1/control/permissions')['snapshot']
        process.terminate(); process.wait(timeout=10)
        self.setup_device()
        self.daemon()
        self.assertFalse(self.setup_device()['profile_grant_pending'])
        granted = self.api('/v1/control/permissions')['snapshot']
        self.assertEqual(granted['revision'], state['revision'] + 1)
        self.assertEqual(set(granted['policy']['allowed_capabilities']), {'echo', 'codex'})

    def test_legacy_pending_profile_receipt_resumes_without_resetting_policy(self):
        self.setup_device('--no-agent')
        process = self.daemon()
        state = self.api('/v1/control/permissions')['snapshot']
        policy = state['policy']
        policy.update(mode='paused', allowed_origins=['local:fixture'])
        self.api('/v1/control/permissions', {'expected_revision':state['revision'], 'policy':policy})
        process.terminate(); process.wait(timeout=10)
        self.setup_device()
        receipt = self.config.with_name('setup-pending-profile.json')
        pending = json.loads(receipt.read_text())
        # Prior builds persisted a single profile object rather than a list.
        receipt.write_text(json.dumps(pending_profiles(pending)[0]))
        self.daemon()
        self.assertFalse(self.setup_device()['profile_grant_pending'])
        state = self.api('/v1/control/permissions')['snapshot']
        self.assertEqual(state['policy']['mode'], 'paused')
        self.assertEqual(state['policy']['allowed_origins'], ['local:fixture'])
        self.assertIn('codex', state['policy']['allowed_capabilities'])
        self.assertFalse(receipt.exists())

    def test_unsupported_harness_fails_before_initialization(self):
        self.codex.write_text(f'#!{sys.executable}\nprint("old harness without noninteractive support")\n')
        error = self.setup_device(ok=False)
        self.assertIn('Codex', error)
        self.assertFalse(self.config.exists())
        self.assertFalse(self.root.joinpath('serve-added').exists())

    def test_explicit_missing_harness_fails_before_initialization(self):
        self.codex.unlink()
        error = self.setup_device(ok=False)
        self.assertIn('Codex', error)
        self.assertFalse(self.config.exists())
        self.assertFalse(self.root.joinpath('serve-added').exists())

    def test_simultaneous_setup_is_rejected_without_changing_existing_files(self):
        self.setup_device()
        before = self.config.read_bytes()
        harness_before = self.harness.joinpath('config.toml').read_bytes()
        self.root.joinpath('serve-added').unlink()
        with self.config.with_name('setup.lock').open('r+') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.assertIn('another setup is already running', self.setup_device(ok=False))
        self.assertEqual(self.config.read_bytes(), before)
        self.assertEqual(self.harness.joinpath('config.toml').read_bytes(), harness_before)
        self.assertFalse(self.root.joinpath('serve-added').exists())

    def test_failed_service_start_preserves_setup_for_recovery(self):
        # Never invoke the user's real service manager, even though this scenario
        # exercises setup's normal service-install path.
        manager_dir = self.root / 'service manager'
        manager_dir.mkdir()
        for manager in ['launchctl', 'systemctl']:
            executable = manager_dir / manager
            executable.write_text(f'''#!{sys.executable}
import json, pathlib, sys
with pathlib.Path(__file__).with_name('calls.jsonl').open('a') as log:
 log.write(json.dumps(sys.argv[1:]) + '\\n')
sys.exit(0 if 'daemon-reload' in sys.argv else 1)
''')
            executable.chmod(0o700)
        self.env['PATH'] = str(manager_dir) + os.pathsep + self.env.get('PATH', '')
        self.setup_args.remove('--no-service')
        self.assertIn('service', self.setup_device(ok=False))
        self.assertTrue(self.config.is_file())
        token = self.config.with_name('node.token').read_bytes()
        calls = [json.loads(line) for line in manager_dir.joinpath('calls.jsonl').read_text().splitlines()]
        self.assertTrue(any('bootstrap' in call or 'enable' in call for call in calls))
        self.assertFalse(any('bootout' in call or 'stop' in call for call in calls))
        self.setup_args.append('--no-service')
        self.daemon()
        recovered = self.setup_device()
        self.assertTrue(recovered['daemon_reachable'])
        self.assertFalse(recovered['profile_grant_pending'])
        self.assertEqual(self.config.with_name('node.token').read_bytes(), token)

    def test_permission_guidance_is_available_before_setup_and_does_not_claim_verification(self):
        for extra in [('--dry-run',), ()]:
            result = self.setup_device(*extra)
            self.assertFalse(result['execution_verified'])
            self.assertFalse(result['desktop_verified'])
            self.assertEqual(result['permissions']['desktop']['status'], 'unverified')
            self.assertEqual(result['permissions']['desktop']['invoked_executable'], str(BINARY))
            self.assertIsNone(result['permissions']['desktop']['service_executable'])
            self.assertTrue(result['permissions']['desktop']['checks'])
            self.assertEqual(result['permissions']['accounts']['status'], 'task_specific')
        self.assertTrue(result['next_steps'])
        self.assertTrue(result['mcp_explicitly_disabled'])
        self.assertFalse(result['daemon_reachable'])

    def test_intentional_pairing_only_setup_does_not_ask_to_install_a_harness(self):
        result = self.setup_device('--no-agent')
        self.assertFalse(result['agent_configured'])
        self.assertEqual(result['agent'], 'not_requested')
        self.assertNotIn('Install/sign in to Codex', result['next'])
        self.assertNotIn('Install Codex', result['next'])
        self.assertNotIn('codex login', result['next'])

    def test_doctor_with_stopped_daemon_offers_recovery(self):
        self.setup_device()
        result = subprocess.run(self.cmd + ['doctor'], env=self.env, capture_output=True, text=True, timeout=15)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('citta setup', result.stderr)
        self.assertIn(str(self.config), result.stderr)

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

    def test_first_serve_failure_retry_preserves_fresh_admission_defaults(self):
        harness = self.harness.joinpath('config.toml').read_bytes()
        self.root.joinpath('serve-denied').touch()
        self.setup_device(ok=False)
        initial = tomllib.loads(self.config.read_text())
        token = self.config.with_name('node.token').read_bytes()
        self.assertTrue(initial['admission']['guard_all'])
        self.assertEqual(initial['capabilities'][0]['resources'],
                         {'memory_bytes': 16 * 1024 * 1024, 'cpu_cores': 0.05})
        self.assertEqual(self.harness.joinpath('config.toml').read_bytes(), harness)
        self.assertFalse(self.harness.joinpath('skills').exists())
        self.assertFalse(self.root.joinpath('serve-added').exists())
        self.assertFalse(self.config.with_name('setup-pending-profile.json').exists())
        self.assertEqual(list(self.config.parent.glob('.setup-*.toml')), [])
        self.root.joinpath('serve-denied').unlink()
        self.assertTrue(self.setup_device()['agent_configured'])
        current = tomllib.loads(self.config.read_text())
        self.assertEqual(current['node_id'], initial['node_id'])
        self.assertEqual(self.config.with_name('node.token').read_bytes(), token)
        self.assertEqual(current['admission'], initial['admission'])
        self.assertEqual(current['capabilities'][0], initial['capabilities'][0])

    def test_serve_failure_retry_preserves_existing_operator_admission_choice(self):
        self.setup_device()
        self.config.write_text(self.config.read_text().replace('guard_all = true', 'guard_all = false'))
        before = self.config.read_bytes()
        token = self.config.with_name('node.token').read_bytes()
        harness = self.harness.joinpath('config.toml').read_bytes()
        routes = {'TCP': {'9000': {'TCPForward': '127.0.0.1:9001'}}}
        self.root.joinpath('serve-state').write_text(json.dumps(routes))
        self.root.joinpath('serve-added').unlink()
        self.root.joinpath('serve-denied').touch()
        self.setup_device(ok=False)
        self.assertEqual(self.config.read_bytes(), before)
        self.assertEqual(self.config.with_name('node.token').read_bytes(), token)
        self.assertEqual(self.harness.joinpath('config.toml').read_bytes(), harness)
        self.assertFalse(self.root.joinpath('serve-added').exists())
        self.root.joinpath('serve-denied').unlink()
        self.setup_device()
        self.assertEqual(self.config.read_bytes(), before)
        self.assertEqual(self.config.with_name('node.token').read_bytes(), token)
        self.assertFalse(tomllib.loads(self.config.read_text())['admission']['guard_all'])
        self.assertEqual(json.loads(self.root.joinpath('serve-state').read_text()), routes)

    def test_serve_failures_explain_recovery_without_exposing_command_output(self):
        self.setup_device()
        before = self.config.read_bytes()
        token = self.config.with_name('node.token').read_bytes()
        harness = self.harness.joinpath('config.toml').read_bytes()
        self.root.joinpath('serve-added').unlink()
        for marker in ['serve-status-denied', 'serve-denied', 'serve-timeout']:
            with self.subTest(marker=marker):
                self.root.joinpath(marker).touch()
                error = self.setup_device(ok=False)
                self.assertNotIn('private-command-output', error)
                self.assertIn('Tailscale Serve', error)
                self.assertIn('operator', error)
                self.assertIn('docs/setup.md#tailscale-https-and-local-permissions', error)
                if marker != 'serve-status-denied':
                    self.assertIn('MagicDNS', error)
                    self.assertIn('HTTPS', error)
                    self.assertIn('7332', error)
                if marker == 'serve-timeout':
                    self.assertIn('timed out', error)
                self.assertEqual(self.config.read_bytes(), before)
                self.assertEqual(self.config.with_name('node.token').read_bytes(), token)
                self.assertEqual(self.harness.joinpath('config.toml').read_bytes(), harness)
                self.assertFalse(self.root.joinpath('serve-added').exists())
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
    require_test_binary()
    unittest.main(verbosity=2)
