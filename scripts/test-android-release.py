#!/usr/bin/env python3
"""Security regressions for signed public APK promotion, compatibility and independent catalog entries."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'deploy/auto'))
import android_policy
import android_pull
import native_pull
from android_release import CERTIFICATE, REPOSITORY, configuration, filename, version


def module(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), ROOT/'scripts'/(name+'.py'))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


publisher = module('android-main-release')
packager = module('android-release-manifest')
SOURCE = 'a'*40
CONTRACT = 'w40-r16-f2e253b98ccb313a'
LIVE = {'contract': CONTRACT}


def candidate():
    value, config = version(), configuration()
    tag = 'android-v'+value
    name = filename(value)
    url = f'https://github.com/{REPOSITORY}/releases/download/{tag}/{name}'
    asset = {'kind': 'apk', 'name': name, 'url': url, 'sha256': 'f'*64, 'bytes': 12345}
    manifest = {'schema': 1, 'channel': 'beta', 'source_commit': SOURCE, 'version': value, 'tag': tag,
                'version_code': config['version_code'], 'package': config['package'], 'abi': 'arm64-v8a',
                'certificate_sha256': CERTIFICATE, 'debuggable': False, 'features': [],
                'min_sdk': 31, 'target_sdk': 36, 'protocol': 40, 'contract': CONTRACT, 'assets': [asset]}
    release = {'tag_name': tag, 'draft': False, 'prerelease': True,
               'assets': [{'name': name, 'browser_download_url': url, 'digest': 'sha256:'+'f'*64, 'size': 12345},
                          {'name': 'tbg-android-release.json'}]}
    return manifest, release


class AndroidPublication(unittest.TestCase):
    def test_only_the_reviewed_release_signature_abi_revision_and_live_contract_enable_downloads(self):
        meta, release = candidate()
        result = android_policy.catalog(meta, release, LIVE)['android']
        self.assertEqual((result['state'], result['abi'], result['certificate_sha256']),
                         ('available', 'arm64-v8a', CERTIFICATE))
        self.assertEqual(android_policy.CERTIFICATE, CERTIFICATE)
        for change in ['debug', 'internal-feature', 'certificate', 'abi', 'revision', 'contract', 'digest', 'size', 'url', 'missing', 'draft']:
            with self.subTest(change=change):
                meta, release = candidate()
                if change == 'debug': meta['debuggable'] = True
                elif change == 'internal-feature': meta['features'] = ['android-loopback']
                elif change == 'certificate': meta['certificate_sha256'] = '0'*64
                elif change == 'abi': meta['abi'] = 'x86_64'
                elif change == 'revision': meta['version_code'] -= 1
                elif change == 'contract': meta['contract'] = 'w38-r15-cc459351581b7763'
                elif change == 'digest': release['assets'][0]['digest'] = 'sha256:'+'0'*64
                elif change == 'size': release['assets'][0]['size'] = 1
                elif change == 'url': meta['assets'][0]['url'] = 'https://example.invalid/a.apk'
                elif change == 'missing': meta['assets'] = []
                elif change == 'draft': release['draft'] = True
                with self.assertRaises(ValueError): android_policy.catalog(meta, release, LIVE)

    def test_ci_cannot_be_reused_from_a_fork_pr_other_commit_or_failure(self):
        run = {'head_sha': SOURCE, 'conclusion': 'success', 'event': 'push', 'head_branch': 'main',
               'path': '.github/workflows/ci.yml', 'head_repository': {'full_name': REPOSITORY}}
        self.assertTrue(publisher.suite_passed([run], SOURCE))
        for key, value in [('head_sha', 'b'*40), ('event', 'pull_request'), ('conclusion', 'failure'),
                           ('head_branch', 'feature'), ('head_repository', {'full_name': 'foreign/repo'})]:
            with self.subTest(key=key):
                changed = copy.deepcopy(run)
                changed[key] = value
                self.assertFalse(publisher.suite_passed([changed], SOURCE))

    def test_manifest_refuses_changed_bytes_or_an_internal_inspection_report(self):
        # Synthetic metadata fixture; this is not an APK or runtime proof.
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            value, config = version(), configuration()
            binary = folder/filename(value)
            binary.write_bytes(b'synthetic fixture only')
            info = {'source_sha': SOURCE, 'channel': 'beta', 'abi': 'arm64-v8a',
                    'android_revision': config['version_code'], 'package': config['package'],
                    'cargo_lock_sha256': hashlib.sha256((ROOT/'Cargo.lock').read_bytes()).hexdigest(),
                    'protocol': 40, 'ruleset': {'version': 16, 'source_hash': 'f2e253b98ccb313a'}, 'assets_sha256': 'e'*64}
            report = {'build_info': info, 'debuggable': False, 'signing_certificate_sha256': CERTIFICATE,
                      'apk_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
                      'package_badging': f"versionCode='{config['version_code']}' versionName='{value}'"}
            path = folder/'android-inspection.json'
            path.write_text(json.dumps(report))
            self.assertEqual(packager.manifest(folder, SOURCE)['contract'], CONTRACT)
            binary.write_bytes(b'changed bytes')
            with self.assertRaisesRegex(ValueError, 'inspected bytes'): packager.manifest(folder, SOURCE)
            report['build_info']['channel'] = 'internal'
            path.write_text(json.dumps(report))
            with self.assertRaisesRegex(ValueError, 'public source'): packager.manifest(folder, SOURCE)

    def test_shared_catalog_writer_preserves_other_platforms_and_clears_a_changed_contract(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(native_pull, 'ROOT', Path(temporary)/'public'):
            meta, release = candidate()
            desktop = {'schema': 1, 'server': {'protocol': 40, 'contract': CONTRACT},
                       'linux': {'state': 'available'}, 'windows': {'state': 'available'}}
            native_pull.publish(desktop)
            native_pull.publish(android_policy.catalog(meta, release, LIVE))
            native_pull.pending(LIVE, ('linux', 'windows'))
            value = json.loads((native_pull.ROOT/'current.json').read_text())
            self.assertEqual(value['android']['state'], 'available')
            native_pull.publish(desktop)
            self.assertEqual(json.loads((native_pull.ROOT/'current.json').read_text())['android']['state'], 'available')
            native_pull.synchronize({'contract': 'w41-r16-f2e253b98ccb313a'})
            value = json.loads((native_pull.ROOT/'current.json').read_text())
            self.assertTrue(all(value[os]['state'] == 'pending' for os in ['linux', 'windows', 'android']))

    def test_revoked_cached_apk_disables_only_android_and_keeps_desktop_links(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            receipt = root/'receipt.json'
            meta, release = candidate()
            receipt.write_text(json.dumps(meta))
            release['assets'][0]['digest'] = 'sha256:'+'0'*64
            with patch.object(android_pull, 'RECEIPT', receipt), \
                 patch.object(android_pull, 'api', return_value={'id': 1285090346}), \
                 patch.object(native_pull, 'ROOT', root/'public'):
                native_pull.publish({'schema': 1, 'server': {'protocol': 40, 'contract': CONTRACT},
                                     'linux': {'state': 'available'}, 'windows': {'state': 'available'}})
                with self.assertRaises(ValueError): android_pull.run({}, LIVE, native_pull.publish, [release])
                value = json.loads((native_pull.ROOT/'current.json').read_text())
                self.assertEqual(value['android']['state'], 'pending')
                self.assertEqual(value['linux']['state'], 'available')
                self.assertEqual(value['windows']['state'], 'available')


if __name__ == '__main__':
    unittest.main()
