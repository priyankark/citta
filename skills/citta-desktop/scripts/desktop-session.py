#!/usr/bin/env python3
"""Cooperative, per-user desktop lease. No security boundary against same-user Bash."""
import argparse
from contextlib import contextmanager
import fcntl
import hmac
import json
import math
import os
from pathlib import Path
import secrets
import shutil
import socket
import subprocess
import sys
import tempfile
import time

HERE = Path(__file__).resolve().parent
ROOT = Path.home() / '.local/state/citta/desktop-session'


def private_dir(root):
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    if root.is_symlink() or root.stat().st_uid != os.getuid():
        raise RuntimeError('Unsafe session directory')
    root.chmod(0o700)


def acquire(root):
    private_dir(root)
    fd = os.open(root / 'lease.lock', os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BaseException:
        os.close(fd)
        raise RuntimeError('desktop_busy: another controller owns the lease')
    return fd  # Never unlink: every owner must lock the same inode.


def readiness(status):
    if status.get('probe_error'):
        return 'desktop_probe_failed'
    p, c = status.get('permissions', {}), status.get('context', {})
    if p.get('accessibility') is not True or p.get('screenCapture') is not True:
        return 'permission_refused_or_unknown'
    if c.get('screen_locked') is not False:
        return 'locked' if c.get('screen_locked') is True else 'unlock_unknown'
    if c.get('display_error') != 0 or not c.get('displays'):
        return 'no_active_display_or_unknown'
    idle = c.get('hid_idle_seconds')
    if not isinstance(idle, (int, float)) or not math.isfinite(idle) or idle < 0:
        return 'human_activity_unknown'
    if idle < 2:
        return 'recent_human_activity'
    if not isinstance(c.get('frontmost_pid'), int) or c['frontmost_pid'] <= 0:
        return 'foreground_unknown'
    return None


class Session:
    def __init__(self, token, ttl, clock=time.monotonic):
        self.token, self.clock = token, clock
        self.deadline = clock() + ttl
        self.state, self.reason = 'waiting', 'initial_readiness'
        self.pid = self.activity = None

    def check(self, status, resume=False):
        if self.clock() >= self.deadline:
            self.state, self.reason = 'expired', 'lease_expired'
            return self.report()
        reason = readiness(status)
        c = status.get('context', {})
        if self.state == 'paused' and not resume:
            return self.report()
        if not reason:
            activity = self.clock() - c['hid_idle_seconds']
            if self.state == 'active' and not resume:
                if c['frontmost_pid'] != self.pid:
                    reason = 'foreground_changed'
                elif activity > self.activity + 0.25:
                    reason = 'human_activity_detected'
            if not reason:
                self.state, self.reason = 'active', None
                self.pid, self.activity = c['frontmost_pid'], activity
        if reason:
            self.state = 'paused' if self.state in ('active', 'paused') or resume else 'waiting'
            self.reason = reason
        return self.report()

    def report(self):
        return {'state': self.state, 'reason': self.reason, 'foreground_pid': self.pid,
                'remaining_seconds': max(0, round(self.deadline - self.clock(), 1)),
                'desktop_verified': False}

    def authorize(self, token):
        if not isinstance(token, str) or not hmac.compare_digest(token, self.token):
            raise RuntimeError('valid owning session required')
        if self.clock() >= self.deadline:
            raise RuntimeError('lease_expired')


@contextmanager
def capture_runtime(request):
    # macOS replayd can confuse simultaneous capture clients with the same Node
    # executable path. A private, byte-identical executable keeps this operation
    # separate without stopping other apps or changing signing/TCC permissions.
    if (sys.platform != 'darwin' or request.get('action') != 'capture'
            or request.get('backend', 'rusty-robot') != 'rusty-robot'):
        yield 'node'
        return
    source = shutil.which('node')
    if source is None:
        raise RuntimeError('capture_runtime_unavailable: Node is missing from PATH')
    private_dir(ROOT)
    with tempfile.TemporaryDirectory(prefix='capture-', dir=ROOT) as directory:
        executable = Path(directory) / 'node'
        try:
            # APFS copy-on-write avoids copying the entire runtime for each shot.
            # A bounded ordinary copy supports installations on other filesystems.
            try:
                subprocess.run(['/bin/cp', '-c', str(Path(source).resolve()), str(executable)],
                               check=True, capture_output=True, timeout=2)
            except subprocess.CalledProcessError:
                subprocess.run(['/bin/cp', str(Path(source).resolve()), str(executable)],
                               check=True, capture_output=True, timeout=2)
            executable.chmod(0o700)
        except (OSError, subprocess.SubprocessError) as error:
            raise RuntimeError('capture_runtime_isolation_failed: ' + str(error)) from error
        yield str(executable)


def native(request, lease_fd=None):
    # The inherited pipe is a one-operation permit. It contains no credential.
    # The inherited lock keeps exclusivity if the broker crashes during input.
    env = os.environ.copy()
    env.pop('CITTA_DESKTOP_PERMIT_FD', None)
    read_fd = write_fd = None
    try:
        pass_fds = ()
        if lease_fd is not None:
            read_fd, write_fd = os.pipe()
            os.write(write_fd, b'guarded-session\n')
            os.close(write_fd)
            write_fd = None
            env['CITTA_DESKTOP_PERMIT_FD'] = str(read_fd)
            pass_fds = (read_fd, lease_fd)
        with capture_runtime(request) as executable:
            result = subprocess.run([executable, str(HERE / 'desktop-control.cjs'), '-'],
                                    input=json.dumps(request), text=True, capture_output=True,
                                    timeout=20, env=env, pass_fds=pass_fds)
        response = json.loads(result.stdout)
        if result.returncode or not response.get('ok'):
            raise RuntimeError(response.get('error', 'native_helper_failed'))
        return response
    finally:
        for fd in (read_fd, write_fd):
            if fd is not None:
                os.close(fd)


def probe():
    try:
        return native({'action': 'status'})
    except Exception as error:
        return {'probe_error': str(error)}  # Installation failures are not missing OS grants.


def serve(root, token, ttl, fd):
    endpoint = root / 'control.sock'
    endpoint.unlink(missing_ok=True)  # Only after acquiring the kernel lock.
    session = Session(token, ttl)
    server = socket.socket(socket.AF_UNIX)
    try:
        server.bind(str(endpoint))
        os.chmod(endpoint, 0o600)
        server.listen(8)
        server.settimeout(0.5)
        while time.monotonic() < session.deadline:
            try:
                conn, _ = server.accept()
            except socket.timeout:
                continue
            end = False
            with conn:
                conn.settimeout(3)
                try:
                    with conn.makefile('rb') as stream:
                        raw = stream.readline(1024 * 1024)
                    message = json.loads(raw)
                    session.authorize(message.get('token'))
                    command = message['command']
                    if command == 'end':
                        end = True
                        result = {'state': 'ended'}
                    elif command == 'pause':
                        session.state, session.reason = 'paused', 'explicit_pause'
                        result = session.report()
                    elif command in ('status', 'resume', 'action'):
                        result = session.check(probe(), resume=command == 'resume')
                        if command == 'action':
                            if session.state != 'active':
                                raise RuntimeError('desktop_' + session.state + ': ' + str(session.reason))
                            request = message['request']
                            if request.get('action') not in ('capture', 'type', 'click', 'key', 'scroll', 'verify-fixture'):
                                raise RuntimeError('unsupported session action')
                            if request.get('action') != 'capture' and request.get('expect_pid') != session.pid:
                                session.state, session.reason = 'paused', 'expected_foreground_mismatch'
                                raise RuntimeError(session.reason)
                            try:
                                result = native(request, fd)
                            except Exception:
                                session.state, session.reason = 'paused', 'action_failed_or_uncertain'
                                raise
                    else:
                        raise RuntimeError('unknown command')
                    response = {'ok': True, **result}
                except Exception as error:
                    response = {'ok': False, 'error': str(error)}
                try:
                    conn.sendall(json.dumps(response).encode() + b'\n')
                except OSError:
                    pass
            if end:
                break
    finally:
        server.close()
        endpoint.unlink(missing_ok=True)
        os.close(fd)


def client(root, credential, command, request=None):
    token = json.loads(credential.read_text())['token']
    with socket.socket(socket.AF_UNIX) as conn:
        conn.settimeout(25)
        conn.connect(str(root / 'control.sock'))
        conn.sendall(json.dumps({'token': token, 'command': command, 'request': request}).encode() + b'\n')
        with conn.makefile('rb') as stream:
            return json.loads(stream.readline(1024 * 1024))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['status', 'start', 'wait', 'pause', 'resume', 'end', 'action'])
    parser.add_argument('--session', type=Path, default=os.environ.get('CITTA_DESKTOP_SESSION'))
    parser.add_argument('--ttl', type=int, default=300)
    parser.add_argument('--timeout', type=float, default=30)
    parser.add_argument('--request', default='-')
    args = parser.parse_args()
    if not 1 <= args.ttl <= 3600 or not math.isfinite(args.timeout) or not 0 <= args.timeout <= 300:
        raise RuntimeError('ttl must be 1..3600; timeout must be 0..300')
    if args.command == 'status' and args.session is None:
        current = probe()
        return {'ok': True, 'state': 'unowned_observation', 'reason': readiness(current), 'status': current}
    if args.session is None:
        raise RuntimeError('--session PRIVATE_CREDENTIAL_FILE required')
    if args.command == 'start':
        fd = acquire(ROOT)
        token = secrets.token_hex(32)
        try:
            # Exclusive creation prevents accidentally overwriting another session's credential.
            with os.fdopen(os.open(args.session, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), 'w') as stream:
                json.dump({'token': token}, stream)
            pid = os.fork()
        except BaseException:
            os.close(fd)
            raise
        if pid == 0:
            os.setsid()
            null = os.open(os.devnull, os.O_RDWR)
            for target in (0, 1, 2):
                os.dup2(null, target)
            os.close(null)
            try:
                serve(ROOT, token, args.ttl, fd)
            finally:
                os._exit(0)
        os.close(fd)
        for _ in range(100):
            try:
                return client(ROOT, args.session, 'status')
            except (FileNotFoundError, ConnectionRefusedError):
                time.sleep(0.05)
        raise RuntimeError('session_start_timeout')
    if args.command == 'wait':
        deadline = time.monotonic() + args.timeout
        while True:
            result = client(ROOT, args.session, 'status')
            if not result.get('ok') or result.get('state') in ('active', 'paused', 'expired'):
                return result
            if time.monotonic() >= deadline:
                return {**result, 'ok': False, 'error': 'wait_timeout'}
            print(json.dumps(result), file=sys.stderr, flush=True)
            time.sleep(min(0.5, max(0, deadline - time.monotonic())))
    request = None
    if args.command == 'action':
        request = json.load(sys.stdin) if args.request == '-' else json.loads(Path(args.request).read_text())
    result = client(ROOT, args.session, args.command, request)
    if args.command == 'end' and result.get('ok'):
        args.session.unlink()
    return result


if __name__ == '__main__':
    try:
        answer = main()
    except Exception as error:
        answer = {'ok': False, 'error': str(error)}
    print(json.dumps(answer))
    sys.exit(0 if answer.get('ok') and answer.get('state') not in ('waiting', 'paused', 'expired') else 1)
