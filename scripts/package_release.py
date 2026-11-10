#!/usr/bin/env python3
"""Build a native Citta candidate archive, with provenance and dependency notices."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
from datetime import datetime, timezone
from install_release import native_target

ROOT = Path(__file__).resolve().parents[1]
SUPPORTED = {'aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-unknown-linux-gnu'}


def run(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dependencies(metadata):
    nodes = {n['id']: n for n in metadata['resolve']['nodes']}
    pending = [metadata['resolve']['root']]
    visited = set()
    while pending:
        key = pending.pop()
        if key in visited:
            continue
        visited.add(key)
        pending.extend(d['pkg'] for d in nodes[key]['deps'] if any(k['kind'] != 'dev' for k in d['dep_kinds']))
    return [p for p in metadata['packages'] if p['id'] in visited and p['id'] != metadata['resolve']['root']]


def collect_notices(metadata, destination):
    inventory = []
    for package in sorted(dependencies(metadata), key=lambda p: (p['name'], p['version'])):
        source = Path(package['manifest_path']).parent
        names = ('license', 'copying', 'notice', 'copyright')
        files = [p for p in source.rglob('*') if p.is_file() and p.name.lower().startswith(names)]
        if package.get('license_file'):
            declared = source / package['license_file']
            if declared.is_file() and declared not in files:
                files.append(declared)
        override = ROOT / 'licenses/third-party' / f"{package['name']}-{package['version']}"
        if package['name'] in {'objc2-core-foundation', 'objc2-io-kit'} and package['version'] == '0.3.2':
            override = ROOT / 'licenses/third-party/objc2-0.3.2'
        if not files and override.is_dir():
            source = override
            files = [p for p in source.rglob('*') if p.is_file()]
        if not files:
            raise ValueError(f"Missing third-party license text: {package['name']} {package['version']}; add a reviewed upstream override")
        copied = []
        for file in sorted(files):
            if file.is_symlink():
                raise ValueError(f'License symlink not supported: {file}')
            relative = file.relative_to(source)
            target = destination / 'third-party' / f"{package['name']}-{package['version']}" / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(file, target)
            copied.append(str(target.relative_to(destination)))
        inventory.append({'name': package['name'], 'version': package['version'], 'license': package.get('license'), 'repository': package.get('repository'), 'selected_license': 'Apache-2.0' if package['name'] in {'objc2-core-foundation', 'objc2-io-kit'} and package['version'] == '0.3.2' else None, 'notices': copied})
    (destination / 'THIRD-PARTY.json').write_text(json.dumps(inventory, indent=2) + '\n')
    return inventory


def assemble(binary, destination, metadata, provenance):
    destination.mkdir()
    (destination / 'bin').mkdir()
    shutil.copyfile(binary, destination / 'bin/citta')
    (destination / 'bin/citta').chmod(0o755)
    for name in ('LICENSE.md', 'NOTICE', 'README.md', 'plan.md'):
        shutil.copyfile(ROOT / name, destination / name)
    readme = destination / 'README.md'
    readme.write_text(readme.read_text().replace('(docs/media/', f"(https://github.com/priyankark/citta/blob/{provenance['source_commit']}/docs/media/").replace('(docs/launch/', f"(https://github.com/priyankark/citta/blob/{provenance['source_commit']}/docs/launch/"))
    bundled = run('git', 'ls-files', '--cached', '--others', '--exclude-standard', '--', 'skills', 'docs', 'examples').splitlines()
    for relative in bundled:
        if relative.startswith(('docs/launch/', 'docs/media/')):
            continue
        source = ROOT / relative
        if source.is_symlink():
            raise ValueError(f'Packaging refuses symlinks: {relative}')
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    (destination / 'scripts').mkdir()
    for name in ('upgrade_device.py', 'render_service.py'):
        shutil.copyfile(ROOT / 'scripts' / name, destination / 'scripts' / name)
    shutil.copyfile(ROOT / 'scripts/install_release.py', destination / 'install.py')
    collect_notices(metadata, destination)
    files = {}
    for path in sorted(destination.rglob('*')):
        if path.is_symlink():
            raise ValueError(f'Packaging refuses symlinks: {path}')
        if path.is_file():
            files[str(path.relative_to(destination))] = digest(path)
    manifest = dict(provenance, schema_version=1, files=files)
    (destination / 'manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'local/release-candidates')
    parser.add_argument('--allow-dirty', action='store_true', help='Create a development candidate only, explicitly marked uncommitted')
    args = parser.parse_args()
    target = native_target()
    if target not in SUPPORTED:
        parser.error(f'No qualified packaging profile for {target}')
    dirty = bool(run('git', 'status', '--porcelain', '--untracked-files=normal'))
    if dirty and not args.allow_dirty:
        parser.error('Commit the reviewed source first, or use --allow-dirty for an unpublished development candidate.')
    source_commit = run('git', 'rev-parse', 'HEAD')
    metadata = json.loads(run('cargo', 'metadata', '--locked', '--format-version', '1', '--filter-platform', target))
    package = next(p for p in metadata['packages'] if p['id'] == metadata['resolve']['root'])
    target_dir = Path(metadata['target_directory'])
    environment = os.environ.copy()
    if target.endswith('apple-darwin'):
        environment['MACOSX_DEPLOYMENT_TARGET'] = '14.0'
    subprocess.run(['cargo', 'build', '--release', '--locked', '--target', target], cwd=ROOT, env=environment, check=True)
    binary = target_dir / target / 'release/citta'
    run(str(binary), '--version')
    if not dirty and (run('git', 'rev-parse', 'HEAD') != source_commit or run('git', 'status', '--porcelain', '--untracked-files=normal')):
        parser.error('Source changed during the build; no archive created. Rebuild from a reviewed commit.')
    provenance = {'version': package['version'], 'target': target, 'source_commit': source_commit, 'source_dirty': dirty,
                  'built_at': datetime.now(timezone.utc).isoformat(), 'rustc': run('rustc', '--version'),
                  'cargo_lock_sha256': digest(ROOT / 'Cargo.lock'), 'release_qualified': False,
                  'signing': 'not Developer ID signed or notarized' if target.endswith('apple-darwin') else 'not signed',
                  'minimum_os': 'macOS 14 deployment target; runtime acceptance required' if target.endswith('apple-darwin') else 'build-host glibc required; see glibc_version',
                  'build_host': run('uname', '-srm')}
    if target.endswith('apple-darwin'):
        provenance['minimum_os_version'] = '14.0'
    if target.endswith('linux-gnu'):
        provenance['glibc_version'] = run('getconf', 'GNU_LIBC_VERSION').split()[-1]
    args.output.mkdir(parents=True, exist_ok=True)
    name = f"citta-{package['version']}-{target}"
    archive = args.output / f'{name}.tar.gz'
    checksum = args.output / f'{name}.tar.gz.sha256'
    if archive.exists() or checksum.exists():
        parser.error(f'Refusing to overwrite an existing candidate: {archive}')
    source_time = int(run('git', 'show', '-s', '--format=%ct', source_commit))
    def archive_metadata(entry):
        entry.uid = entry.gid = 0
        entry.uname = entry.gname = ''
        entry.mtime = source_time
        return entry
    with tempfile.TemporaryDirectory(prefix='citta-package-') as tmp:
        bundle = Path(tmp) / name
        manifest = assemble(binary, bundle, metadata, provenance)
        if not dirty and (run('git', 'rev-parse', 'HEAD') != source_commit or run('git', 'status', '--porcelain', '--untracked-files=normal')):
            parser.error('Source changed during packaging; no archive created.')
        with tarfile.open(archive, 'x:gz') as tar:
            tar.add(bundle, arcname=name, filter=archive_metadata)
    checksum.write_text(f'{digest(archive)}  {archive.name}\n')
    (args.output / f'{name}.manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'archive': str(archive), 'checksum': str(checksum), 'source_commit': source_commit, 'source_dirty': dirty, 'release_qualified': False}, indent=2))


if __name__ == '__main__':
    main()
