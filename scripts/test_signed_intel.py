#!/usr/bin/env python3
"""Reject changed release bytes/identity before native execution."""
import copy
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import check_signed_intel as check


class SignedIntel(unittest.TestCase):
    def repository(self, private=False):
        return {'id': check.REPOSITORY_ID, 'full_name': check.REPOSITORY, 'private': private}

    def release(self):
        return {'id': check.RELEASE, 'draft': True, 'published_at': None, 'target_commitish': check.SOURCE,
                'assets': [{'id': value[0], 'name': name, 'digest': 'sha256:' + value[1], 'state': 'uploaded'}
                           for name, value in check.ASSETS.items()]}

    def test_repository_identity_and_exact_draft_are_required_regardless_of_visibility(self):
        release = self.release()
        for private in [True, False]:
            check.check_release(self.repository(private), release)
        for field, bad in [('id', check.REPOSITORY_ID + 1), ('full_name', 'another-owner/citta'), ('private', None)]:
            changed = self.repository()
            changed[field] = bad
            with self.subTest(repository_field=field), self.assertRaises(ValueError):
                check.check_release(changed, release)
        for field, bad in [('id', check.RELEASE + 1), ('draft', False), ('published_at', 'now'), ('target_commitish', 'another-commit')]:
            changed = copy.deepcopy(release)
            changed[field] = bad
            with self.subTest(release_field=field), self.assertRaises(ValueError):
                check.check_release(self.repository(), changed)
        for field, bad in [('id', -1), ('name', 'another.tar.gz'), ('state', 'new'), ('digest', 'sha256:' + '0' * 64)]:
            changed = copy.deepcopy(release)
            changed['assets'][0][field] = bad
            with self.subTest(asset_field=field), self.assertRaises(ValueError):
                check.check_release(self.repository(), changed)

    def test_download_uses_only_get_and_reports_actual_repository_visibility(self):
        data = b'exact test asset'
        assets = {check.NAME + '.tar.gz': (123, hashlib.sha256(data).hexdigest())}
        with tempfile.TemporaryDirectory() as temporary, patch.object(check, 'ASSETS', assets):
            release = self.release()
            endpoints = [f'repos/{check.REPOSITORY}', f'repos/{check.REPOSITORY}/releases/{check.RELEASE}']
            def api(args, **kwargs):
                self.assertEqual(args[:4], ['gh', 'api', '--method', 'GET'])
                self.assertIn(args[4], endpoints)
                return json.dumps(self.repository() if args[4] == endpoints[0] else release).encode()
            def asset(args, **kwargs):
                self.assertEqual(args, ['gh', 'api', '--method', 'GET', '-H', 'Accept: application/octet-stream',
                                        f'repos/{check.REPOSITORY}/releases/assets/123'])
                self.assertTrue(kwargs['check'])
                kwargs['stdout'].write(data)
            report = {}
            with patch.object(check.subprocess, 'check_output', side_effect=api), patch.object(check.subprocess, 'run', side_effect=asset):
                check.download(Path(temporary), report)
            self.assertIs(report['repository_private'], False)
            self.assertEqual(report['repository_id'], check.REPOSITORY_ID)
            self.assertEqual((Path(temporary) / next(iter(assets))).read_bytes(), data)

    def test_changed_repository_or_published_release_stops_before_asset_download(self):
        for repository, release in [(dict(self.repository(), id=-1), self.release()),
                                    (self.repository(), dict(self.release(), draft=False, published_at='now'))]:
            with tempfile.TemporaryDirectory() as temporary, patch.object(check.subprocess, 'check_output', side_effect=[json.dumps(repository).encode(), json.dumps(release).encode()]), patch.object(check.subprocess, 'run') as run:
                with self.assertRaises(ValueError):
                    check.download(Path(temporary), {})
                run.assert_not_called()
                self.assertEqual(list(Path(temporary).iterdir()), [])

    def test_tampered_download_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / next(iter(check.ASSETS))).write_bytes(b'wrong archive')
            with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                check.check_assets(directory)

    def test_signing_identity_runtime_and_timestamp_required(self):
        details = f'Identifier={check.IDENTIFIER}\nTeamIdentifier={check.TEAM}\nAuthority=Developer ID Application: Example\nflags=0x10000(runtime)\nTimestamp=Nov 11, 2026\n'
        requirement = f'identifier "{check.IDENTIFIER}" and anchor apple generic and certificate leaf[subject.OU] = "{check.TEAM}" and certificate leaf[field.1.2.840.113635.100.6.1.13] exists'
        check.check_signature_details(details, requirement)
        for valid, bad in [(check.TEAM, 'OTHERTEAM'), ('0x10000', '0x0'), ('Timestamp=Nov 11, 2026', 'Timestamp=none'), ('Developer ID Application:', 'Apple Development:')]:
            with self.assertRaises(ValueError):
                check.check_signature_details(details.replace(valid, bad), requirement)

    def test_wrong_architecture_stops_before_signature_or_execution(self):
        with patch('check_signed_intel.platform.system', return_value='Darwin'), patch('check_signed_intel.platform.machine', return_value='arm64'), patch('check_signed_intel.subprocess.run') as run:
            with self.assertRaisesRegex(ValueError, 'Native Intel'):
                check.verify(Path('/unused'), {})
            run.assert_not_called()

    def test_archive_escape_and_links_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for i, (name, kind) in enumerate([('../outside', tarfile.REGTYPE), (check.NAME + '/link', tarfile.SYMTYPE)]):
                archive = root / f'{i}.tar.gz'
                with tarfile.open(archive, 'w:gz') as tar:
                    member = tarfile.TarInfo(name)
                    member.type = kind
                    member.linkname = '/outside' if kind == tarfile.SYMTYPE else ''
                    tar.addfile(member, io.BytesIO())
                with self.assertRaisesRegex(ValueError, 'Unsafe archive'):
                    check.extract(archive, root / 'unpacked')
            self.assertFalse((root / 'unpacked').exists())


if __name__ == '__main__':
    unittest.main(verbosity=2)
