#!/usr/bin/env python3
"""Archive integrity and fresh-install failure checks; no live setup or services."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import install_release as installer
import package_release as packaging


class Release(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='citta-release-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bundle = self.root / 'bundle'
        self.bundle.mkdir()
        self.prefix = self.root / 'installed'
        for name, content in {'bin/citta': '#!/bin/sh\necho fixture\n', 'LICENSE.md': 'license', 'NOTICE': 'notice', 'install.py': 'installer', 'THIRD-PARTY.json': '[]', 'skills/citta-setup/SKILL.md': 'skill'}.items():
            file = self.bundle / name
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_text(content)
        self.manifest = {'schema_version': 1, 'version': '0.1.0', 'target': installer.native_target(), 'source_commit': 'a' * 40, 'files': {str(p.relative_to(self.bundle)): installer.digest(p) for p in self.bundle.rglob('*') if p.is_file()}}
        self.save_manifest()
        for patcher in [patch('install_release.Path.home', return_value=self.root / 'home'), patch.dict(os.environ, {'XDG_CONFIG_HOME': str(self.root / 'config')}), patch('install_release.shutil.which', return_value=None), patch('install_release.os.geteuid', return_value=501)]:
            patcher.start()
            self.addCleanup(patcher.stop)

    def save_manifest(self):
        (self.bundle / 'manifest.json').write_text(json.dumps(self.manifest))

    def test_verify_then_install_and_refuse_overwrite(self):
        self.assertEqual(installer.install(self.bundle, self.prefix)['state'], 'verified')
        self.assertFalse(self.prefix.exists())
        result = installer.install(self.bundle, self.prefix, True)
        self.assertEqual(result['state'], 'installed')
        self.assertEqual((self.prefix / 'bin/citta').read_bytes(), (self.bundle / 'bin/citta').read_bytes())
        self.assertEqual((self.prefix / 'bin/citta').stat().st_mode & 0o777, 0o755)
        self.assertTrue((self.prefix / 'share/citta/0.1.0/NOTICE').is_file())
        self.assertTrue((self.prefix / 'share/citta/0.1.0/skills/citta-setup/SKILL.md').is_file())
        with self.assertRaisesRegex(ValueError, 'already exists'):
            installer.install(self.bundle, self.prefix, True)
        self.assertFalse((self.prefix / 'share/citta/.install-lock').exists())

    def test_tampered_binary_does_not_create_destination(self):
        (self.bundle / 'bin/citta').write_text('tampered')
        with self.assertRaisesRegex(ValueError, 'integrity'):
            installer.install(self.bundle, self.prefix, True)
        self.assertFalse(self.prefix.exists())

    def test_wrong_architecture_and_extra_file_are_rejected(self):
        self.manifest['target'] = 'not-this-machine'
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, 'does not match'):
            installer.verify(self.bundle)
        self.manifest['target'] = installer.native_target()
        self.save_manifest()
        (self.bundle / 'unexpected').write_text('unlisted')
        with self.assertRaisesRegex(ValueError, 'not listed'):
            installer.verify(self.bundle)

    def test_minimum_os_and_glibc_checks(self):
        self.manifest['minimum_os_version'] = '14.0'
        self.save_manifest()
        with patch('install_release.native_target', return_value=self.manifest['target']), patch('install_release.platform.system', return_value='Darwin'), patch('install_release.platform.mac_ver', return_value=('13.6', (), '')):
            with self.assertRaisesRegex(ValueError, 'requires macOS'):
                installer.verify(self.bundle)
        self.manifest['glibc_version'] = '2.39'
        self.save_manifest()
        with patch('install_release.native_target', return_value=self.manifest['target']), patch('install_release.platform.system', return_value='Linux'), patch('install_release.platform.libc_ver', return_value=('glibc', '2.35')):
            with self.assertRaisesRegex(ValueError, 'requires glibc'):
                installer.verify(self.bundle)

    def test_path_escape_and_symlink_rejected(self):
        self.manifest['files']['../outside'] = '0' * 64
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, 'Unsafe archive path'):
            installer.verify(self.bundle)
        del self.manifest['files']['../outside']
        self.save_manifest()
        original = (self.bundle / 'NOTICE').read_bytes()
        (self.bundle / 'NOTICE').unlink()
        (self.root / 'outside').write_bytes(original)
        (self.bundle / 'NOTICE').symlink_to(self.root / 'outside')
        with self.assertRaisesRegex(ValueError, 'symlink'):
            installer.verify(self.bundle)

    def test_existing_config_or_other_binary_preserved(self):
        config = self.root / 'config/citta/config.toml'
        config.parent.mkdir(parents=True)
        config.write_text('private-existing-state')
        with self.assertRaisesRegex(ValueError, 'Existing Citta configuration'):
            installer.install(self.bundle, self.prefix, True)
        self.assertEqual(config.read_text(), 'private-existing-state')
        self.assertFalse(self.prefix.exists())
        config.unlink()
        with patch('install_release.shutil.which', return_value='/old/citta'):
            with self.assertRaisesRegex(ValueError, 'already on PATH'):
                installer.install(self.bundle, self.prefix, True)
        self.assertFalse(self.prefix.exists())

    def test_actual_default_config_is_preserved_when_xdg_points_elsewhere(self):
        config = self.root / 'home/.config/citta/config.toml'
        config.parent.mkdir(parents=True)
        config.write_text('actual-citta-default')
        with self.assertRaisesRegex(ValueError, 'Existing Citta configuration'):
            installer.install(self.bundle, self.prefix, True)
        self.assertEqual(config.read_text(), 'actual-citta-default')
        self.assertFalse(self.prefix.exists())

    def test_failed_link_leaves_existing_binary_and_removes_partial_assets(self):
        with patch('install_release.os.link', side_effect=OSError('simulated disk failure')):
            with self.assertRaisesRegex(OSError, 'disk failure'):
                installer.install(self.bundle, self.prefix, True)
        self.assertFalse((self.prefix / 'bin/citta').exists())
        self.assertFalse((self.prefix / 'share/citta/0.1.0').exists())
        self.assertFalse((self.prefix / 'share/citta/.install-lock').exists())

    def test_dependency_closure_excludes_dev_only_and_requires_notices(self):
        crate = self.root / 'crate'
        crate.mkdir()
        (crate / 'Cargo.toml').write_text('[package]\nname="example"')
        metadata = {'resolve': {'root': 'root', 'nodes': [{'id': 'root', 'deps': [{'pkg': 'normal', 'dep_kinds': [{'kind': None}]}, {'pkg': 'dev', 'dep_kinds': [{'kind': 'dev'}]}]}, {'id': 'normal', 'deps': []}]}, 'packages': [{'id': 'normal', 'name': 'example', 'version': '1', 'manifest_path': str(crate / 'Cargo.toml'), 'license': 'MIT'}, {'id': 'dev', 'name': 'dev-only'}]}
        self.assertEqual([p['name'] for p in packaging.dependencies(metadata)], ['example'])
        with self.assertRaisesRegex(ValueError, 'Missing third-party'):
            packaging.collect_notices(metadata, self.root / 'notices')
        (crate / 'LICENSE').write_text('upstream legal text')
        inventory = packaging.collect_notices(metadata, self.root / 'notices')
        self.assertEqual(inventory[0]['name'], 'example')
        self.assertEqual((self.root / 'notices/third-party/example-1/LICENSE').read_text(), 'upstream legal text')

    def test_packaged_readme_media_uses_pinned_raw_urls(self):
        source = self.root / 'source'
        readme = ('[![Demo](docs/media/demo-thumbnail.png)](https://youtu.be/example)\n'
                  '[Download](docs/media/citta-demo.mp4) [Captions](docs/media/citta-demo.srt)\n'
                  '[Raw download](https://raw.githubusercontent.com/priyankark/citta/main/docs/media/citta-demo.mp4)\n'
                  '[Launch](docs/launch/story.md) [Setup](docs/setup.md)\n')
        files = {'README.md': readme, 'LICENSE.md': 'license', 'NOTICE': 'notice',
                 'plan.md': 'plan', 'docs/setup.md': 'setup',
                 'docs/media/demo-thumbnail.png': 'image', 'docs/media/citta-demo.mp4': 'video',
                 'docs/media/citta-demo.srt': 'captions', 'docs/launch/story.md': 'story',
                 'scripts/upgrade_device.py': 'upgrade', 'scripts/render_service.py': 'service',
                 'scripts/install_release.py': 'installer'}
        for name, content in files.items():
            file = source / name
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_text(content)
        metadata = {'resolve': {'root': 'root', 'nodes': [{'id': 'root', 'deps': []}]}, 'packages': []}
        commit = 'a' * 40
        destination = self.root / 'packaged'
        tracked = '\n'.join(name for name in files if name.startswith('docs/'))
        with patch.object(packaging, 'ROOT', source), patch.object(packaging, 'run', return_value=tracked):
            manifest = packaging.assemble(self.bundle / 'bin/citta', destination, metadata,
                                          {'source_commit': commit})
        expected = (f'[![Demo](https://raw.githubusercontent.com/priyankark/citta/{commit}/docs/media/demo-thumbnail.png)](https://youtu.be/example)\n'
                    f'[Download](https://raw.githubusercontent.com/priyankark/citta/{commit}/docs/media/citta-demo.mp4) '
                    f'[Captions](https://raw.githubusercontent.com/priyankark/citta/{commit}/docs/media/citta-demo.srt)\n'
                    f'[Raw download](https://raw.githubusercontent.com/priyankark/citta/{commit}/docs/media/citta-demo.mp4)\n'
                    f'[Launch](https://github.com/priyankark/citta/blob/{commit}/docs/launch/story.md) [Setup](docs/setup.md)\n')
        self.assertEqual((destination / 'README.md').read_text(), expected)
        self.assertFalse((destination / 'docs/media').exists())
        self.assertFalse((destination / 'docs/launch').exists())
        self.assertEqual((destination / 'docs/setup.md').read_text(), 'setup')
        self.assertEqual(manifest['files']['README.md'], packaging.digest(destination / 'README.md'))


if __name__ == '__main__':
    unittest.main(verbosity=2)
