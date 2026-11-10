#!/usr/bin/env python3
"""Exercise a native archive install in an isolated temporary home, without setup."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tarfile
import tempfile


def check(archive):
    archive = archive.resolve()
    checksum = Path(str(archive) + '.sha256').read_text().split()
    if len(checksum) != 2 or checksum[1] != archive.name:
        raise ValueError('Checksum file must identify exactly this archive')
    if hashlib.sha256(archive.read_bytes()).hexdigest() != checksum[0]:
        raise ValueError('Archive checksum mismatch')
    with tempfile.TemporaryDirectory(prefix='citta-fresh-archive-') as tmp:
        temporary = Path(tmp)
        unpacked = temporary / 'unpacked'
        unpacked.mkdir()
        with tarfile.open(archive) as tar:
            for member in tar.getmembers():
                path = PurePosixPath(member.name)
                if path.is_absolute() or '..' in path.parts or not (member.isfile() or member.isdir()):
                    raise ValueError(f'Unsafe archive member: {member.name}')
            tar.extractall(unpacked, filter='data')
        roots = list(unpacked.iterdir())
        if len(roots) != 1 or not roots[0].is_dir():
            raise ValueError('Archive must contain exactly one bundle directory')
        bundle = roots[0]
        manifest = json.loads((bundle / 'manifest.json').read_text())
        home = temporary / 'home'
        home.mkdir()
        environment = os.environ.copy()
        environment.update(HOME=str(home), XDG_CONFIG_HOME=str(home / '.config'), PATH='/usr/bin:/bin:/usr/sbin:/sbin')
        prefix = home / '.local'
        install = [sys.executable, str(bundle / 'install.py'), '--prefix', str(prefix)]
        subprocess.run([*install, '--verify-only'], env=environment, check=True, capture_output=True, text=True)
        subprocess.run(install, env=environment, check=True, capture_output=True, text=True)
        version = subprocess.check_output([str(prefix / 'bin/citta'), '--version'], env=environment, text=True).strip()
        if version != f"citta {manifest['version']}":
            raise ValueError('Installed version disagrees with manifest')
        subprocess.run([str(prefix / 'bin/citta'), '--help'], env=environment, check=True, capture_output=True)
        repeat = subprocess.run(install, env=environment, capture_output=True, text=True)
        if repeat.returncode == 0 or 'already exists' not in repeat.stderr:
            raise ValueError('Installer did not refuse an existing installation')
        if (home / '.config/citta').exists() or (home / 'Library/LaunchAgents/io.citta.daemon.plist').exists():
            raise ValueError('Fresh installer unexpectedly created service/config state')
        for required in ['skills/citta-desktop/scripts/desktop-session.py', 'examples/ollama_worker.py', 'LICENSE.md', 'NOTICE', 'THIRD-PARTY.json']:
            if required not in manifest['files']:
                raise ValueError(f'Missing expected distribution asset: {required}')
        if any(name.startswith(('docs/media/', 'docs/launch/')) for name in manifest['files']):
            raise ValueError('Candidate contains media or internal launch planning')
        return {'archive': str(archive), 'sha256': checksum[0], 'size_bytes': archive.stat().st_size,
                'source_commit': manifest['source_commit'], 'source_dirty': manifest['source_dirty'],
                'manifest_file_count': len(manifest['files']), 'target': manifest['target'],
                'checks': ['checksum', 'fresh install', 'installed version/help', 'existing install refused', 'no service/config changes', 'required assets'],
                'scope': 'isolated home on the current machine; not a Gatekeeper or physical-mesh acceptance test'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    try:
        report = check(args.archive)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Archive check failed: {error}\n')
    output = json.dumps(report, indent=2) + '\n'
    if args.report:
        args.report.write_text(output)
    print(output, end='')


if __name__ == '__main__':
    main()
