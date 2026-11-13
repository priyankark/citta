#!/usr/bin/env python3
import copy
import importlib.util
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('session', Path(__file__).with_name('desktop-session.py'))
s = importlib.util.module_from_spec(spec)
spec.loader.exec_module(s)


def ready():
    return {'permissions': {'accessibility': True, 'screenCapture': True},
            'context': {'screen_locked': False, 'display_error': 0, 'displays': [{}],
                        'frontmost_pid': 23, 'hid_idle_seconds': 30}}


class Sessions(unittest.TestCase):
    def setUp(self):
        self.now = 100
        self.session = s.Session('secret', 60, lambda: self.now)

    def test_readiness_refusals(self):
        for key, value, reason in [('screen_locked', True, 'locked'), ('screen_locked', None, 'unlock_unknown'),
                                   ('displays', [], 'no_active_display_or_unknown'),
                                   ('hid_idle_seconds', None, 'human_activity_unknown'),
                                   ('hid_idle_seconds', 0, 'recent_human_activity')]:
            status = ready()
            status['context'][key] = value
            self.assertEqual(s.readiness(status), reason)
        for key in ('accessibility', 'screenCapture'):
            status = ready()
            status['permissions'][key] = False
            self.assertEqual(s.readiness(status), 'permission_refused_or_unknown')
        self.assertIsNotNone(s.readiness({}))
        self.assertEqual(s.readiness({'probe_error': 'addon missing'}), 'desktop_probe_failed')

    def test_probe_preserves_installation_failure(self):
        with patch.object(s, 'native', side_effect=RuntimeError('addon missing')):
            self.assertEqual(s.probe(), {'probe_error': 'addon missing'})

    def test_capture_clients_have_distinct_private_executables_and_cleanup(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / 'original-node'
            source.write_bytes(b'exact runtime bytes')
            with patch.object(s, 'ROOT', root), patch.object(s.sys, 'platform', 'darwin'), \
                    patch.object(s.shutil, 'which', return_value=str(source)):
                with s.capture_runtime({'action': 'capture'}) as first:
                    with s.capture_runtime({'action': 'capture', 'backend': None}) as second:
                        self.assertNotEqual(first, second)
                        for name in (first, second):
                            executable = Path(name)
                            self.assertFalse(executable.is_symlink())
                            self.assertEqual(executable.read_bytes(), source.read_bytes())
                            self.assertEqual(executable.stat().st_mode & 0o777, 0o700)
                            self.assertEqual(executable.parent.stat().st_mode & 0o777, 0o700)
                    self.assertFalse(Path(second).parent.exists())
                    self.assertTrue(Path(first).exists())
                self.assertFalse(Path(first).parent.exists())
                self.assertTrue(source.exists())

    def test_capture_timeout_keeps_permit_and_cleans_without_replay(self):
        original_run = subprocess.run
        calls = []
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / 'original-node'
            source.write_bytes(b'runtime')
            fd = s.acquire(root)
            def run(command, **kwargs):
                if command[0] == '/bin/cp':
                    return original_run(command, **kwargs)
                calls.append(command)
                self.assertEqual(kwargs['timeout'], 20)
                self.assertIn(fd, kwargs['pass_fds'])
                permit = int(kwargs['env']['CITTA_DESKTOP_PERMIT_FD'])
                self.assertIn(permit, kwargs['pass_fds'])
                self.assertEqual(os.read(permit, 100), b'guarded-session\n')
                raise subprocess.TimeoutExpired(command, 20)
            try:
                with patch.object(s, 'ROOT', root), patch.object(s.sys, 'platform', 'darwin'), \
                        patch.object(s.shutil, 'which', return_value=str(source)), \
                        patch.object(s.subprocess, 'run', side_effect=run):
                    with self.assertRaises(subprocess.TimeoutExpired):
                        s.native({'action': 'capture'}, fd)
                self.assertEqual(len(calls), 1)
                self.assertFalse(Path(calls[0][0]).parent.exists())
                self.assertTrue(source.exists())
            finally:
                os.close(fd)

    def test_isolation_failure_never_runs_shared_capture(self):
        with tempfile.TemporaryDirectory() as tmp:
            with patch.object(s, 'ROOT', Path(tmp)), patch.object(s.sys, 'platform', 'darwin'), \
                    patch.object(s.shutil, 'which', return_value='/original/node'), \
                    patch.object(s.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'cp')) as run:
                with self.assertRaisesRegex(RuntimeError, 'capture_runtime_isolation_failed'):
                    s.native({'action': 'capture'})
                self.assertEqual(len(run.call_args_list), 2)
                self.assertTrue(all(call.args[0][0] == '/bin/cp' for call in run.call_args_list))
                self.assertEqual(list(Path(tmp).iterdir()), [])

    def test_copy_fallback_supports_filesystems_without_cloning(self):
        original_run = subprocess.run
        calls = []
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / 'original-node'
            source.write_bytes(b'unchanged executable')
            def run(command, **kwargs):
                calls.append(command)
                if '-c' in command:
                    raise subprocess.CalledProcessError(1, command)
                return original_run(command, **kwargs)
            with patch.object(s, 'ROOT', Path(tmp)), patch.object(s.sys, 'platform', 'darwin'), \
                    patch.object(s.shutil, 'which', return_value=str(source)), \
                    patch.object(s.subprocess, 'run', side_effect=run):
                with s.capture_runtime({'action': 'capture'}) as executable:
                    self.assertEqual(Path(executable).read_bytes(), source.read_bytes())
            self.assertEqual(len(calls), 2)
            self.assertFalse(Path(executable).exists())

    def test_other_operations_and_platforms_do_not_copy_node(self):
        with patch.object(s.shutil, 'which', side_effect=AssertionError('unexpected runtime copy')):
            with patch.object(s.sys, 'platform', 'darwin'):
                for request in ({'action': 'status'}, {'action': 'type'},
                                {'action': 'capture', 'backend': 'system-window'}):
                    with s.capture_runtime(request) as executable:
                        self.assertEqual(executable, 'node')
            with patch.object(s.sys, 'platform', 'linux'):
                with s.capture_runtime({'action': 'capture'}) as executable:
                    self.assertEqual(executable, 'node')

    def test_wait_unlock_pause_resume(self):
        status = ready()
        status['context']['screen_locked'] = True
        self.assertEqual(self.session.check(status)['state'], 'waiting')
        self.assertEqual(self.session.check(ready())['state'], 'active')
        self.assertEqual(self.session.check(status)['state'], 'paused')
        self.assertEqual(self.session.check(ready())['state'], 'paused')
        self.assertEqual(self.session.check(ready(), resume=True)['state'], 'active')

    def test_takeover_and_focus_require_resume(self):
        for key, value, reason in [('hid_idle_seconds', 5, 'human_activity_detected'),
                                   ('frontmost_pid', 24, 'foreground_changed')]:
            session = s.Session('secret', 60, lambda: self.now)
            session.check(ready())
            changed = ready()
            changed['context'][key] = value
            self.assertEqual(session.check(changed)['reason'], reason)
            self.assertEqual(session.check(ready())['state'], 'paused')
            self.assertEqual(session.check(ready(), resume=True)['state'], 'active')

    def test_expiry_and_wrong_owner(self):
        for token in (None, 'wrong'):
            with self.assertRaises(RuntimeError):
                self.session.authorize(token)
        self.session.authorize('secret')
        self.now += 61
        with self.assertRaises(RuntimeError):
            self.session.authorize('secret')
        self.assertEqual(self.session.check(ready())['state'], 'expired')

    def test_os_contention_crashed_owner_and_stale_socket(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fd = s.acquire(root)
            with self.assertRaisesRegex(RuntimeError, 'desktop_busy'):
                s.acquire(root)
            os.close(fd)
            child = subprocess.Popen([sys.executable, '-c',
                'import fcntl,os,sys,time; f=open(sys.argv[1],"a"); fcntl.flock(f,fcntl.LOCK_EX); print("ready",flush=True); time.sleep(30)',
                str(root / 'lease.lock')], stdout=subprocess.PIPE, text=True)
            try:
                self.assertEqual(child.stdout.readline().strip(), 'ready')
                with self.assertRaises(RuntimeError):
                    s.acquire(root)
            finally:
                child.kill()
                child.wait()
                child.stdout.close()
            (root / 'control.sock').write_text('stale')
            fd = s.acquire(root)
            os.close(fd)

    def test_inflight_child_retains_lock_after_owner_exits(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fd = s.acquire(root)
            child = subprocess.Popen([sys.executable, '-c',
                'import time; print("ready",flush=True); time.sleep(30)'],
                pass_fds=(fd,), stdout=subprocess.PIPE, text=True)
            os.close(fd)
            try:
                self.assertEqual(child.stdout.readline().strip(), 'ready')
                with self.assertRaisesRegex(RuntimeError, 'desktop_busy'):
                    s.acquire(root)
            finally:
                child.kill()
                child.wait()
                child.stdout.close()
            fd = s.acquire(root)
            os.close(fd)

    def test_unowned_cli_refuses_capture_and_input(self):
        env = os.environ.copy()
        env.pop('CITTA_DESKTOP_SESSION', None)
        env.pop('CITTA_DESKTOP_PERMIT_FD', None)
        for action in ('capture', 'type', 'click', 'key', 'scroll', 'verify-fixture'):
            result = subprocess.run(['node', str(s.HERE / 'desktop-control.cjs'), '-'],
                                    input=json.dumps({'action': action}), text=True,
                                    capture_output=True, env=env, timeout=5)
            self.assertNotEqual(result.returncode, 0, action)
            self.assertIn('required', result.stdout + result.stderr)

    def test_broker_wait_unlock_takeover_and_no_action_without_lease(self):
        with tempfile.TemporaryDirectory(prefix='guard-') as tmp:
            root = Path(tmp)
            status_file = root / 'status.json'
            status = ready()
            status['context']['screen_locked'] = True
            status_file.write_text(json.dumps(status))
            credential = root / 'owner.json'
            credential.write_text(json.dumps({'token': 'secret'}))
            actions = root / 'actions'
            fd = s.acquire(root)
            pid = os.fork()
            if pid == 0:
                s.probe = lambda: json.loads(status_file.read_text())
                def fake_native(request, lease_fd):
                    actions.write_text(request['action'])
                    return {'issued': True}
                s.native = fake_native
                try:
                    s.serve(root, 'secret', 30, fd)
                finally:
                    os._exit(0)
            os.close(fd)
            try:
                for _ in range(100):
                    if (root / 'control.sock').exists():
                        break
                    time.sleep(.01)
                request = {'action': 'type', 'expect_pid': 23, 'text': 'x'}
                self.assertFalse(s.client(root, credential, 'action', request)['ok'])
                self.assertFalse(actions.exists())
                self.assertEqual(s.client(root, credential, 'status')['state'], 'waiting')
                before = time.monotonic()
                with patch.object(s, 'ROOT', root), patch.object(sys, 'argv',
                        ['desktop-session.py', 'wait', '--session', str(credential), '--timeout', '0.05']):
                    waiting = s.main()
                self.assertEqual(waiting['error'], 'wait_timeout')
                self.assertEqual(waiting['reason'], 'locked')
                self.assertLess(time.monotonic() - before, 2)
                self.assertFalse(actions.exists())
                status_file.write_text(json.dumps(ready()))
                self.assertEqual(s.client(root, credential, 'status')['state'], 'active')
                self.assertTrue(s.client(root, credential, 'action', request)['ok'])
                status = ready()
                status['context']['frontmost_pid'] = 24
                status_file.write_text(json.dumps(status))
                self.assertFalse(s.client(root, credential, 'action', request)['ok'])
                status_file.write_text(json.dumps(ready()))
                self.assertEqual(s.client(root, credential, 'status')['state'], 'paused')
                self.assertEqual(s.client(root, credential, 'resume')['state'], 'active')
                other = root / 'other.json'
                other.write_text(json.dumps({'token': 'wrong'}))
                self.assertFalse(s.client(root, other, 'action', request)['ok'])
                self.assertTrue(s.client(root, credential, 'end')['ok'])
            finally:
                os.kill(pid, signal.SIGTERM)
                os.waitpid(pid, 0)
            fd = s.acquire(root)
            os.close(fd)


if __name__ == '__main__':
    unittest.main()
