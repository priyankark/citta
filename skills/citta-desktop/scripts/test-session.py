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
