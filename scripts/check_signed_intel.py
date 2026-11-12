#!/usr/bin/env python3
"""Verify the exact pinned signed Intel draft; never build, sign, or publish."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import platform
import re
import subprocess
import sys
import tarfile
import tempfile

REPOSITORY = 'priyankark/citta'
REPOSITORY_ID = 1448601034
RELEASE = 442541115
SOURCE = 'c767ce5b5c1ad283c00afad9684a4f809a7c857e'
TARGET = 'x86_64-apple-darwin'
NAME = f'citta-0.1.0-{TARGET}'
TEAM = '9ZSYA62D6L'
IDENTIFIER = 'io.citta.daemon'
CERTIFICATE = '6C0C0715C06427D3D33C15C22FC0A25F1E126304'
BINARY_SHA256 = '4df05409f3e009242d836a73a89329ebcdf73386bc0978a6854d47b79e4819d6'
ASSETS = {
    NAME + '.tar.gz': (738265168, '689c14b36826ab9cf52bb7f90dd79e18052bc0ef8ac1851d0f0b847094dc7200'),
    NAME + '.tar.gz.sha256': (738265213, '2dd670bb1b8a265e4c7d3cc54f06f23ff66c427a24245274befbf2a26f2e0776'),
    NAME + '.manifest.json': (738265238, '3babacd8be92e3fefd0527af7d83392d0737bb982f92b806d58bc1ddc7ed86e7'),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_assets(directory):
    for name, (_, expected) in ASSETS.items():
        require(digest(directory / name) == expected, f'Pinned asset checksum mismatch: {name}')


def check_release(repository, release):
    require(repository.get('full_name') == REPOSITORY and repository.get('id') == REPOSITORY_ID,
            'Repository identity mismatch')
    require(isinstance(repository.get('private'), bool), 'Missing repository visibility')
    require(release.get('id') == RELEASE and release.get('draft') is True, 'Expected pinned unpublished draft release')
    require(release.get('published_at') is None, 'Release was published; inspect before reusing this rehearsal')
    require(release.get('target_commitish') == SOURCE, 'Draft source commit changed')
    for name, (asset_id, checksum) in ASSETS.items():
        matches = [a for a in release.get('assets', []) if a.get('id') == asset_id and a.get('name') == name]
        require(len(matches) == 1 and matches[0].get('state') == 'uploaded', f'Expected uploaded asset: {name}')
        require(matches[0].get('digest') == 'sha256:' + checksum, f'GitHub asset digest changed: {name}')


def download(directory, report):
    def api(endpoint):
        return json.loads(subprocess.check_output(['gh', 'api', '--method', 'GET', endpoint], timeout=45))
    repository = api(f'repos/{REPOSITORY}')
    release = api(f'repos/{REPOSITORY}/releases/{RELEASE}')
    check_release(repository, release)
    report.update(repository=REPOSITORY, repository_id=REPOSITORY_ID,
                  repository_private=repository['private'], release_draft=True, release_id=RELEASE)
    for name, (asset_id, _) in ASSETS.items():
        # Exclusive creation preserves a partial failure instead of silently replacing it.
        with (directory / name).open('xb') as output:
            subprocess.run(['gh', 'api', '--method', 'GET', '-H', 'Accept: application/octet-stream',
                            f'repos/{REPOSITORY}/releases/assets/{asset_id}'],
                           stdout=output, check=True, timeout=60)
    check_assets(directory)
    report['assets'] = {name: {'id': value[0], 'sha256': value[1]} for name, value in ASSETS.items()}


def check_signature_details(details, requirement):
    require(re.search(r'^Identifier=' + re.escape(IDENTIFIER) + r'$', details, re.M), 'Signing identifier mismatch')
    require(re.search(r'^TeamIdentifier=' + TEAM + r'$', details, re.M), 'Signing team mismatch')
    require(re.search(r'^Authority=Developer ID Application:', details, re.M), 'Developer ID Application required')
    flags = re.search(r'flags=0x([0-9a-fA-F]+)', details)
    require(flags is not None and int(flags.group(1), 16) & 0x10000, 'Hardened runtime missing')
    timestamp = re.search(r'^Timestamp=(.+)$', details, re.M)
    require(timestamp is not None and timestamp.group(1).lower() not in {'none', 'not set'}, 'Secure timestamp missing')
    require(f'identifier "{IDENTIFIER}"' in requirement and TEAM in requirement
            and 'anchor apple generic' in requirement and '1.2.840.113635.100.6.1.13' in requirement,
            'Unexpected designated requirement')


def extract(archive, destination):
    with tarfile.open(archive) as tar:
        names = set()
        for member in tar.getmembers():
            path = PurePosixPath(member.name)
            require(path.parts and not path.is_absolute() and '..' not in path.parts and '\\' not in member.name
                    and path.parts[0] == NAME and (member.isfile() or member.isdir()), 'Unsafe archive member')
            require(member.name not in names, 'Duplicate archive member')
            names.add(member.name)
        tar.extractall(destination, filter='data')
    return destination / NAME


def verify(directory, report):
    require(platform.system() == 'Darwin' and platform.machine() == 'x86_64', 'Native Intel macOS required')
    translated = subprocess.run(['/usr/sbin/sysctl', '-in', 'sysctl.proc_translated'], capture_output=True, text=True)
    require(translated.stdout.strip() != '1', 'Rosetta does not qualify native Intel execution')
    report['host'] = {'system': platform.system(), 'architecture': platform.machine(), 'macos': platform.mac_ver()[0]}
    check_assets(directory)
    with tempfile.TemporaryDirectory(prefix='citta-signed-intel-') as temporary:
        temporary = Path(temporary)
        home = temporary / 'home'
        home.mkdir()
        # The tested executable/installer gets no GitHub token, inherited account HOME, or service setup.
        environment = {'HOME': str(home), 'XDG_CONFIG_HOME': str(home / '.config'),
                       'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'TMPDIR': str(temporary), 'LANG': 'en_US.UTF-8'}
        commands = report.setdefault('commands', [])
        def run(*args, timeout=45):
            result = subprocess.run(args, env=environment, capture_output=True, text=True, timeout=timeout)
            commands.append({'command': list(args), 'exit_code': result.returncode,
                             'stdout': result.stdout, 'stderr': result.stderr})
            require(result.returncode == 0, f'Command failed: {Path(args[0]).name}; see verification.json')
            return result.stdout + result.stderr
        archive = directory / (NAME + '.tar.gz')
        bundle = extract(archive, temporary / 'unpacked')
        raw_manifest = (bundle / 'manifest.json').read_bytes()
        require(raw_manifest == (directory / (NAME + '.manifest.json')).read_bytes(), 'Embedded manifest mismatch')
        manifest = json.loads(raw_manifest)
        require(manifest['source_commit'] == SOURCE and manifest['source_dirty'] is False
                and manifest['target'] == TARGET and manifest['release_qualified'] is False, 'Unexpected source manifest')
        binary = bundle / 'bin/citta'
        require(digest(binary) == BINARY_SHA256 == manifest['files']['bin/citta'], 'Signed binary hash mismatch')
        require(run('/usr/bin/lipo', '-archs', str(binary)).strip() == 'x86_64', 'Intel-only executable expected')
        run('/usr/bin/codesign', '--verify', '--strict', '--verbose=2', str(binary))
        details = run('/usr/bin/codesign', '-d', '--verbose=4', str(binary))
        requirement = run('/usr/bin/codesign', '-d', '-r-', str(binary))
        check_signature_details(details, requirement)
        cert_prefix = str(temporary / 'public-signing-cert')
        run('/usr/bin/codesign', '-d', '--extract-certificates=' + cert_prefix, str(binary))
        require(hashlib.sha1(Path(cert_prefix + '0').read_bytes()).hexdigest().upper() == CERTIFICATE,
                'Developer ID certificate fingerprint mismatch')
        run('/usr/bin/codesign', '--verify', '--strict', '--verbose=4', '-R=notarized', '--check-notarization', str(binary), timeout=60)
        run(str(binary), 'setup', '--help')
        installer_report = directory / 'installer.json'
        run(sys.executable, str(Path(__file__).with_name('check_release_archive.py')), str(archive),
            '--report', str(installer_report), timeout=90)
        report['installer'] = json.loads(installer_report.read_text())
        require(not list(home.iterdir()), 'Verification created unexpected HOME state')
        check_assets(directory)
        report.update(archive_sha256=ASSETS[NAME + '.tar.gz'][1], binary_sha256=BINARY_SHA256,
                      certificate_sha1=CERTIFICATE, signing_identifier=IDENTIFIER, signing_team=TEAM,
                      native_intel_execution=True, online_notarization=True, release_qualified=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('phase', choices=['download', 'verify'])
    parser.add_argument('--directory', type=Path, default=Path('local/signed-intel-check'))
    args = parser.parse_args()
    directory = args.directory.resolve()
    directory.mkdir(parents=True, exist_ok=True)
    report_path = directory / ('download.json' if args.phase == 'download' else 'verification.json')
    require(not report_path.exists(), 'Existing report must be preserved; use a new directory for another attempt')
    report = {'phase': args.phase, 'source_commit': SOURCE, 'target': TARGET, 'success': False,
              'scope': 'Exact existing signed artifact: native CLI and isolated installer; no desktop, browser-download, or mesh acceptance'}
    try:
        (download if args.phase == 'download' else verify)(directory, report)
        report['success'] = True
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, tarfile.TarError) as error:
        report['error'] = str(error)
    finally:
        report_path.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'success': report['success'], 'report': str(report_path), 'phase': args.phase}))
    return 0 if report['success'] else 1


if __name__ == '__main__':
    sys.exit(main())
