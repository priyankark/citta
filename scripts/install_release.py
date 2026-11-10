#!/usr/bin/env python3
"""Verify and install a Citta archive without starting services or replacing installs."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import shutil
import sys
import tempfile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def native_target():
    arch = {'arm64': 'aarch64', 'aarch64': 'aarch64', 'x86_64': 'x86_64', 'AMD64': 'x86_64'}.get(platform.machine())
    system = {'Darwin': 'apple-darwin', 'Linux': 'unknown-linux-gnu'}.get(platform.system())
    if not arch or not system:
        raise ValueError('Citta archives support macOS and Linux on the listed architectures.')
    return f'{arch}-{system}'


def verify(bundle):
    bundle = bundle.resolve()
    manifest = json.loads((bundle / 'manifest.json').read_text())
    if manifest.get('schema_version') != 1:
        raise ValueError('Unsupported archive manifest version')
    if manifest.get('target') != native_target():
        raise ValueError(f"Archive target {manifest.get('target')} does not match this device ({native_target()})")
    minimum = manifest.get('minimum_os_version')
    if minimum and platform.system() == 'Darwin':
        current = tuple(int(part) for part in platform.mac_ver()[0].split('.'))
        required = tuple(int(part) for part in minimum.split('.'))
        if current < required:
            raise ValueError(f'Archive requires macOS {minimum} or later')
    minimum_glibc = manifest.get('glibc_version')
    if minimum_glibc and platform.system() == 'Linux':
        libc, version = platform.libc_ver()
        if libc != 'glibc' or tuple(int(p) for p in version.split('.')) < tuple(int(p) for p in minimum_glibc.split('.')):
            raise ValueError(f'Archive requires glibc {minimum_glibc} or later; use a compatible archive or build from source')
    expected = manifest.get('files', {})
    for required in ['bin/citta', 'LICENSE.md', 'NOTICE', 'install.py', 'THIRD-PARTY.json']:
        if required not in expected:
            raise ValueError(f'Missing required manifest entry: {required}')
    for name, checksum in expected.items():
        relative = PurePosixPath(name)
        if relative.is_absolute() or '..' in relative.parts or '\\' in name:
            raise ValueError(f'Unsafe archive path: {name}')
        path = bundle / relative
        if any(parent.is_symlink() for parent in [path, *path.parents] if parent != bundle.parent):
            raise ValueError(f'Archive symlink is not allowed: {name}')
        if not path.is_file() or digest(path) != checksum:
            raise ValueError(f'Archive integrity check failed: {name}')
    actual = {str(p.relative_to(bundle)) for p in bundle.rglob('*') if p.is_file()}
    if actual != set(expected) | {'manifest.json'}:
        raise ValueError('Archive contains files not listed in its manifest')
    for path in bundle.rglob('*'):
        if path.is_symlink() or not (path.is_file() or path.is_dir()):
            raise ValueError(f'Archive contains an unsupported file: {path.name}')
    return manifest


def install(bundle, prefix, apply=False):
    manifest = verify(bundle)
    version = manifest['version']
    if not version or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.-+' for c in version):
        raise ValueError('Invalid archive version')
    prefix = prefix.expanduser().absolute()
    binary = prefix / 'bin/citta'
    share = prefix / 'share/citta' / version
    if binary.exists() or binary.is_symlink() or share.exists() or share.is_symlink():
        raise ValueError('Citta already exists at this destination. Use docs/operations.md for an offline upgrade.')
    on_path = shutil.which('citta')
    if on_path:
        raise ValueError(f'Citta is already on PATH at {on_path}. Use docs/operations.md for an offline upgrade.')
    configs = {Path.home() / '.config/citta/config.toml'}
    if os.environ.get('XDG_CONFIG_HOME'):
        configs.add(Path(os.environ['XDG_CONFIG_HOME']) / 'citta/config.toml')
    for config in configs:
        if config.exists():
            raise ValueError(f'Existing Citta configuration at {config}. Use the offline upgrade procedure; no files changed.')
    report = {'state': 'verified', 'binary': str(binary), 'assets': str(share), 'source_commit': manifest['source_commit'], 'target': manifest['target']}
    if not apply:
        return report
    if hasattr(os, 'geteuid') and os.geteuid() == 0:
        raise ValueError('Install as your normal user, without sudo.')
    binary.parent.mkdir(parents=True, exist_ok=True)
    share.parent.mkdir(parents=True, exist_ok=True)
    # Serialize fresh installs and avoid replacing an existing binary in a race.
    lock = prefix / 'share/citta/.install-lock'
    lock.mkdir()
    try:
        if binary.exists() or binary.is_symlink() or share.exists() or share.is_symlink():
            raise ValueError('Another installation created the destination; no files replaced.')
        with tempfile.TemporaryDirectory(prefix='.citta-install-', dir=share.parent) as tmp:
            staged = Path(tmp) / 'bundle'
            shutil.copytree(bundle, staged)
            verify(staged)
            staged_binary = staged / 'bin/citta'
            staged_binary.chmod(0o755)
            staged.rename(share)
            try:
                os.link(share / 'bin/citta', binary)
            except BaseException:
                shutil.rmtree(share)
                raise
        report['state'] = 'installed'
    finally:
        lock.rmdir()
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prefix', type=Path, default=Path.home() / '.local')
    parser.add_argument('--verify-only', action='store_true', help='Validate the archive and destination without installing')
    args = parser.parse_args()
    try:
        report = install(Path(__file__).resolve().parent, args.prefix, not args.verify_only)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f'Citta installation stopped: {error}\n')
    print(json.dumps(report, indent=2))
    if report['state'] == 'installed':
        print('\nAdd the bin directory above to PATH, then run citta setup and citta doctor on each device.')
        print('No configuration, services, app permissions or shell startup files were changed by this installer.')


if __name__ == '__main__':
    main()
