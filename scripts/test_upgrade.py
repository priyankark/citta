#!/usr/bin/env python3
"""Upgrade failure/recovery checks, including real-daemon lock interoperability."""
import contextlib
import fcntl
import json
import os
from pathlib import Path
import shutil
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch
import upgrade_device as upgrade

ROOT = Path(__file__).resolve().parents[1]


@contextlib.contextmanager
def database(path):
    with contextlib.closing(sqlite3.connect(path)) as db:
        with db:
            yield db


class Upgrade(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='citta-upgrade-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.data = self.root / 'data'
        self.data.mkdir()
        self.config = self.root / 'config.toml'
        self.token = self.root / 'node.token'
        self.token.write_text('test-token')
        self.token.chmod(0o600)
        self.config.write_text(f'node_id="test"\ndata_dir={json.dumps(str(self.data))}\ntoken_file={json.dumps(str(self.token))}\n')
        self.installed = self.root / 'citta'
        self.installed.write_text('old-binary')
        self.installed.chmod(0o755)
        self.candidate = self.root / 'candidate'
        self.candidate.write_text(f'#!{sys.executable}\nimport json\nprint(json.dumps({{"ok":True,"node_id":"test"}}))\n')
        self.candidate.chmod(0o700)
        with database(self.data / 'citta.db') as db:
            db.executescript("PRAGMA user_version=3; CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT); INSERT INTO meta VALUES('node','test'); CREATE TABLE tasks(id TEXT, status TEXT); INSERT INTO tasks VALUES('old-failure','failed'); CREATE TABLE device_permissions(id INTEGER, payload TEXT); INSERT INTO device_permissions VALUES(1,'paused');")
        (self.data / 'control.token').write_text('private-control-credential')
        (self.data / 'enrollment.json').write_text('{"existing":"identity"}')

    def run_upgrade(self, apply=True, checksum=None):
        return upgrade.upgrade(self.config, self.installed, self.candidate,
                               checksum or upgrade.digest(self.candidate), apply)

    def test_success_preserves_identity_policy_history_and_private_backup(self):
        config = self.config.read_bytes()
        report = self.run_upgrade()
        self.assertEqual(report['state'], 'installed_service_stopped')
        self.assertEqual(self.installed.read_bytes(), self.candidate.read_bytes())
        self.assertEqual(self.config.read_bytes(), config)
        backup = Path(report['backup'])
        self.assertEqual((backup / 'citta').read_text(), 'old-binary')
        self.assertEqual((backup / 'credentials/0').read_bytes(), self.token.read_bytes())
        self.assertEqual((backup / 'data/control.token').read_text(), 'private-control-credential')
        self.assertEqual((backup / 'data/enrollment.json').read_text(), '{"existing":"identity"}')
        for database_path in (self.data / 'citta.db', backup / 'data/citta.db'):
            with database(database_path) as db:
                self.assertEqual(db.execute('SELECT * FROM tasks').fetchall(), [('old-failure', 'failed')])
                self.assertEqual(db.execute('SELECT payload FROM device_permissions').fetchone(), ('paused',))
        for path in [backup, *backup.rglob('*')]:
            self.assertEqual(path.stat().st_mode & 0o077, 0, path)
        self.assertEqual(list(self.root.glob('.citta-upgrade-*')), [])

    def test_validation_and_wrong_digest_do_not_replace(self):
        self.assertEqual(self.run_upgrade(apply=False)['state'], 'validated')
        with self.assertRaisesRegex(ValueError, 'SHA-256 mismatch'):
            self.run_upgrade(checksum='0' * 64)
        self.assertEqual(self.installed.read_text(), 'old-binary')
        self.assertFalse((self.root / 'upgrade-backups').exists())

    def test_pending_work_and_newer_storage_are_refused(self):
        for status in ('queued', 'running'):
            with database(self.data / 'citta.db') as db:
                db.execute('UPDATE tasks SET status=?', (status,))
            with self.assertRaisesRegex(ValueError, 'tasks remain'):
                self.run_upgrade()
        with database(self.data / 'citta.db') as db:
            db.executescript("UPDATE tasks SET status='failed'; PRAGMA user_version=4;")
        with self.assertRaisesRegex(ValueError, 'unsupported source storage'):
            self.run_upgrade()
        self.assertEqual(self.installed.read_text(), 'old-binary')

    def test_wrong_identity_and_invalid_candidate_are_refused(self):
        self.candidate.write_text(f'#!{sys.executable}\nraise SystemExit(1)\n')
        with self.assertRaisesRegex(ValueError, 'configuration check failed'):
            self.run_upgrade()
        with database(self.data / 'citta.db') as db:
            db.execute("UPDATE meta SET value='different-device'")
        with self.assertRaisesRegex(ValueError, 'different device'):
            self.run_upgrade()
        self.assertEqual(self.installed.read_text(), 'old-binary')

    def test_incomplete_backup_never_replaces_binary(self):
        (self.data / 'unexpected-link').symlink_to(self.token)
        with self.assertRaisesRegex(RuntimeError, 'refuses symlinks'):
            self.run_upgrade()
        self.assertEqual(self.installed.read_text(), 'old-binary')
        self.assertEqual(list(self.root.glob('.citta-upgrade-*')), [])

    def test_failure_after_replacement_retains_new_binary_and_backup(self):
        real_write = upgrade.write_manifest
        def fail_after_install(backup, manifest):
            if manifest['state'] == 'installed_service_stopped':
                raise OSError('simulated storage failure')
            real_write(backup, manifest)
        with patch.object(upgrade, 'write_manifest', side_effect=fail_after_install):
            with self.assertRaisesRegex(RuntimeError, 'no automatic rollback'):
                self.run_upgrade()
        self.assertEqual(self.installed.read_bytes(), self.candidate.read_bytes())
        manifests = list((self.root / 'upgrade-backups').glob('*/manifest.json'))
        self.assertEqual(len(manifests), 1)
        self.assertEqual(json.loads(manifests[0].read_text())['state'], 'backup_complete')
        self.assertEqual((manifests[0].parent / 'citta').read_text(), 'old-binary')

    def test_real_daemon_lock_then_offline_upgrade_and_restart(self):
        binary = Path(os.environ.get('CITTA_TEST_BINARY', ROOT / 'target/debug/citta')).resolve(strict=True)
        device = self.root / 'real-device'
        installed = self.root / 'real-citta'
        shutil.copy2(binary, installed)
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        subprocess.run([str(installed), 'init', '--dir', str(device), '--node', 'real-test', '--port', str(port)], capture_output=True, check=True)
        config = device / 'config.toml'
        process = subprocess.Popen([str(installed), '--config', str(config), 'daemon'], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        try:
            deadline = time.monotonic() + 15
            locked = False
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    self.fail('fixture daemon failed: ' + process.stderr.read().decode())
                lockfile = device / 'data/daemon.lock'
                if lockfile.exists():
                    with lockfile.open('a+b') as lock:
                        try:
                            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                        except BlockingIOError:
                            locked = True
                    if locked and (device / 'data/control.token').exists():
                        break
                time.sleep(0.05)
            self.assertTrue(locked)
            with self.assertRaisesRegex(ValueError, 'stop the service first'):
                upgrade.upgrade(config, installed, binary, upgrade.digest(binary), apply=True)
        finally:
            process.terminate()
            process.wait(timeout=20)
            process.stderr.close()
        token_before = (device / 'node.token').read_bytes()
        result = upgrade.upgrade(config, installed, binary, upgrade.digest(binary), apply=True)
        self.assertEqual(result['state'], 'installed_service_stopped')
        self.assertEqual((device / 'node.token').read_bytes(), token_before)
        restarted = subprocess.Popen([str(installed), '--config', str(config), 'daemon'], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        try:
            time.sleep(0.5)
            self.assertIsNone(restarted.poll(), 'upgraded daemon did not stay running')
        finally:
            restarted.terminate()
            restarted.wait(timeout=20)
            restarted.stderr.close()


if __name__ == '__main__':
    unittest.main(verbosity=2)
