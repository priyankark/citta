#!/usr/bin/env python3
"""Reject changed release bytes/identity before native execution."""
import copy
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import check_signed_intel as check


class SignedIntel(unittest.TestCase):
    def test_release_must_remain_private_draft_and_exact_source(self):
        release = {'id': check.RELEASE, 'draft': True, 'published_at': None, 'target_commitish': check.SOURCE,
                   'assets': [{'id': value[0], 'name': name, 'digest': 'sha256:' + value[1], 'state': 'uploaded'}
                              for name, value in check.ASSETS.items()]}
        check.check_release({'private': True}, release)
        for field, bad in [('draft', False), ('published_at', 'now'), ('target_commitish', 'another-commit')]:
            changed = copy.deepcopy(release)
            changed[field] = bad
            with self.assertRaises(ValueError):
                check.check_release({'private': True}, changed)
        with self.assertRaises(ValueError):
            check.check_release({'private': False}, release)
        release['assets'][0]['digest'] = 'sha256:' + '0' * 64
        with self.assertRaisesRegex(ValueError, 'digest changed'):
            check.check_release({'private': True}, release)

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
