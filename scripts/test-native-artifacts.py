#!/usr/bin/env python3
"""Promotion boundaries: reject mixed-source/lock/features and changed or missing native assets."""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import tomllib
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('native_manifest', ROOT/'scripts/native-release-manifest.py')
promotion = importlib.util.module_from_spec(spec)
spec.loader.exec_module(promotion)
SOURCE = 'a' * 40
VERSION = tomllib.loads((ROOT/'crates/tbg-client/Cargo.toml').read_text())['package']['version']
STEM = 'tupi-battle-grounds-' + VERSION


def package(root, os, **changes):
    info = {'commit': SOURCE, 'version': VERSION, 'platform': os,
            'target': 'x86_64-pc-windows-msvc' if os == 'windows' else 'x86_64-unknown-linux-gnu',
            'profile': 'release', 'cargo_lock_sha256': hashlib.sha256((ROOT/'Cargo.lock').read_bytes()).hexdigest(),
            'protocol': 31, 'contract': 'w31-r6-097385fee9a10b0f', 'assets_sha256': 'b' * 64, 'features': ['ui-acceptance']}
    info.update(changes)
    body = json.dumps(info).encode()
    prefix = f'{STEM}-{os}-x86_64'
    if os == 'windows':
        archive = root/(prefix + '.zip')
        with zipfile.ZipFile(archive, 'w') as output: output.writestr(prefix + '/build-info.json', body)
    else:
        archive = root/(prefix + '.tar.gz')
        with tarfile.open(archive, 'w:gz') as output:
            member = tarfile.TarInfo(prefix + '/build-info.json')
            member.size = len(body)
            output.addfile(member, io.BytesIO(body))
    sidecar(archive)


def sidecar(path):
    path.with_name(path.name + '.sha256').write_text(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + path.name + '\n')


def fixtures(root):
    package(root, 'linux')
    package(root, 'windows')
    # These tiny test-only format stand-ins exercise metadata, never certify executable behavior.
    for name in [STEM + '-linux-x86_64.AppImage', STEM + '-windows-x86_64-setup.exe']:
        path = root/name
        path.write_bytes(b'test-only-not-an-executable')
        sidecar(path)


class NativeArtifacts(unittest.TestCase):
    def test_complete_matching_set_has_exact_original_urls_and_four_digests(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixtures(root)
            value = promotion.manifest(root, SOURCE, 'v' + VERSION)
            self.assertEqual(value['source_commit'], SOURCE)
            self.assertEqual(value['windows_publisher_signature'], 'unsigned')
            self.assertEqual(value['visual_acceptance'], 'pending')
            self.assertEqual(sum(len(p['assets']) for p in value['platforms'].values()), 4)

    def test_mixed_source_toolchain_dependency_lock_assets_or_features_are_refused(self):
        for changes in [{'commit': 'c' * 40}, {'target': 'x86_64-pc-windows-gnu'}, {'profile': 'debug'},
                        {'cargo_lock_sha256': '0' * 64}, {'assets_sha256': '0' * 64}, {'features': []}]:
            with self.subTest(changes=changes), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixtures(root)
                package(root, 'windows', **changes)
                with self.assertRaises(ValueError): promotion.manifest(root, SOURCE, 'v' + VERSION)

    def test_missing_or_modified_format_and_wrong_tag_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixtures(root)
            with self.assertRaises(ValueError): promotion.manifest(root, SOURCE, 'v999.0.0')
            installer = root/(STEM + '-windows-x86_64-setup.exe')
            installer.write_bytes(b'changed after checksum')
            with self.assertRaises(ValueError): promotion.manifest(root, SOURCE, 'v' + VERSION)
            installer.unlink()
            with self.assertRaises(ValueError): promotion.manifest(root, SOURCE, 'v' + VERSION)

    def test_archive_identity_is_bounded_and_unambiguous(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'invalid.zip'
            for members in [[('one/build-info.json', 'x' * 65537)],
                            [('one/build-info.json', '{}'), ('two/build-info.json', '{}')]]:
                with zipfile.ZipFile(path, 'w') as output:
                    for name, body in members: output.writestr(name, body)
                with self.assertRaises(ValueError): promotion.identity(path)


if __name__ == '__main__': unittest.main()
