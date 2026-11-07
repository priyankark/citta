#!/usr/bin/env python3
"""Offline, device-local binary upgrade with private backups (Python 3.11+).

Stop the user service first. This does not change policy, restart services,
install remote software, or automatically roll back a migrated database.
"""
import argparse
import contextlib
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import stat
import subprocess
import tempfile
import tomllib


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def sync_dir(path):
    fd = os.open(path, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def private_copy(source, target):
    if source.is_symlink() or not source.is_file():
        raise ValueError(f'backup source must be a regular file: {source}')
    with source.open('rb') as inp, target.open('xb') as out:
        os.fchmod(out.fileno(), 0o600 | (stat.S_IMODE(source.stat().st_mode) & 0o100))
        shutil.copyfileobj(inp, out)
        out.flush()
        os.fsync(out.fileno())


def copy_data(source, target, root=True):
    target.mkdir(mode=0o700)
    for entry in source.iterdir():
        if entry.name in {'daemon.lock', 'citta.db', 'citta.db-wal', 'citta.db-shm'} and root:
            continue
        if entry.is_symlink():
            raise ValueError(f'data backup refuses symlinks: {entry}')
        dest = target / entry.name
        if entry.is_dir():
            copy_data(entry, dest, root=False)
        else:
            private_copy(entry, dest)
    sync_dir(target)


def write_manifest(backup, manifest):
    path = backup / 'manifest.json'
    temporary = backup / 'manifest.new'
    with temporary.open('w') as out:
        os.fchmod(out.fileno(), 0o600)
        json.dump(manifest, out, indent=2)
        out.write('\n')
        out.flush()
        os.fsync(out.fileno())
    temporary.replace(path)
    sync_dir(backup)


def upgrade(config_path, installed, candidate, expected_sha256, apply=False):
    config_path = config_path.resolve(strict=True)
    if installed.is_symlink():
        raise ValueError('installed binary must not be a symlink')
    installed = installed.resolve(strict=True)
    candidate = candidate.resolve(strict=True)
    config = tomllib.loads(config_path.read_text())
    data = Path(config['data_dir']).resolve(strict=True)
    backup_root = config_path.parent / 'upgrade-backups'
    if data == backup_root.resolve() or data in backup_root.resolve().parents:
        raise ValueError('upgrade backups must be outside the data directory')
    if not installed.is_file() or not os.access(installed, os.X_OK):
        raise ValueError('installed binary must be executable')
    expected_sha256 = expected_sha256.lower()
    if len(expected_sha256) != 64 or any(c not in '0123456789abcdef' for c in expected_sha256):
        raise ValueError('provide the SHA-256 from the reviewed build')
    # Use the same flock protocol as Rust fs2. Never unlink this lock file.
    with (data / 'daemon.lock').open('a+b') as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ValueError('daemon or maintenance owns the data directory; stop the service first') from error
        dbpath = data / 'citta.db'
        with contextlib.closing(sqlite3.connect(dbpath.as_uri() + '?mode=ro', uri=True)) as db:
            version = db.execute('PRAGMA user_version').fetchone()[0]
            if version not in (1, 2, 3):
                raise ValueError(f'unsupported source storage version: {version}')
            node = db.execute("SELECT value FROM meta WHERE key='node'").fetchone()
            if node != (config['node_id'],):
                raise ValueError('database belongs to a different device')
            pending = db.execute("SELECT COUNT(*) FROM tasks WHERE status IN ('queued','running')").fetchone()[0]
            if pending:
                raise ValueError(f'{pending} queued/running tasks remain; reconcile them before upgrading')
            fd, name = tempfile.mkstemp(prefix='.citta-upgrade-', dir=installed.parent)
            staged = Path(name)
            os.close(fd)
            backup = None
            try:
                with candidate.open('rb') as inp, staged.open('wb') as out:
                    shutil.copyfileobj(inp, out)
                    out.flush()
                    os.fsync(out.fileno())
                if digest(staged) != expected_sha256:
                    raise ValueError('candidate SHA-256 mismatch; nothing installed')
                staged.chmod(0o700)
                checked = subprocess.run([str(staged), '--config', str(config_path), 'check'],
                                         capture_output=True, text=True, timeout=30)
                if checked.returncode:
                    raise ValueError('candidate configuration check failed; nothing installed')
                result = json.loads(checked.stdout)
                if result.get('ok') is not True or result.get('node_id') != config['node_id']:
                    raise ValueError('candidate check returned an unexpected identity')
                report = {'node_id': config['node_id'], 'storage_version_before': version,
                          'installed_path': str(installed), 'config_path': str(config_path),
                          'old_sha256': digest(installed), 'candidate_sha256': expected_sha256,
                          'state': 'validated', 'credentials': []}
                if not apply:
                    return report
                backup_root.mkdir(mode=0o700, exist_ok=True)
                if backup_root.is_symlink() or backup_root.stat().st_mode & 0o077:
                    raise ValueError('backup directory must be private and not a symlink')
                backup = Path(tempfile.mkdtemp(prefix=datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ-'), dir=backup_root))
                report['backup'] = str(backup)
                write_manifest(backup, report)
                private_copy(installed, backup / 'citta')
                private_copy(config_path, backup / 'config.toml')
                copy_data(data, backup / 'data')
                with contextlib.closing(sqlite3.connect(backup / 'data/citta.db')) as target:
                    db.backup(target)
                    if target.execute('PRAGMA integrity_check').fetchall() != [('ok',)]:
                        raise ValueError('backup database integrity check failed')
                (backup / 'data/citta.db').chmod(0o600)
                with (backup / 'data/citta.db').open('rb') as stream:
                    os.fsync(stream.fileno())
                credentials = [config['token_file']] + [p['token_file'] for p in config.get('peers', [])]
                (backup / 'credentials').mkdir(mode=0o700)
                for index, path in enumerate(dict.fromkeys(credentials)):
                    target = backup / 'credentials' / str(index)
                    private_copy(Path(path), target)
                    report['credentials'].append({'original': path, 'backup': str(target.relative_to(backup))})
                for directory in (backup / 'data', backup / 'credentials', backup_root):
                    sync_dir(directory)
                report['state'] = 'backup_complete'
                write_manifest(backup, report)
                staged.chmod(stat.S_IMODE(installed.stat().st_mode) & 0o777)
                with staged.open('rb') as stream:
                    os.fsync(stream.fileno())
                staged.replace(installed)
                sync_dir(installed.parent)
                report['state'] = 'installed_service_stopped'
                write_manifest(backup, report)
                return report
            except BaseException as error:
                if backup:
                    raise RuntimeError(f'upgrade stopped ({error}); inspect {backup}/manifest.json and installed digest before recovery; no automatic rollback') from error
                raise
            finally:
                staged.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--installed', type=Path, required=True)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--apply', action='store_true', help='back up and replace; otherwise validate only')
    args = parser.parse_args()
    if os.geteuid() == 0:
        parser.error('run as the existing Citta service user, not root')
    try:
        report = upgrade(args.config, args.installed, args.candidate, args.sha256, args.apply)
    except Exception as error:
        parser.exit(1, f'{error}\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
