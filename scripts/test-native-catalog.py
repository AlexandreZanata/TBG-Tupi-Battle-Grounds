#!/usr/bin/env python3
"""Native publication contracts: exact packages, attested tag, live compatibility and revocation."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'deploy/auto'))
import native_policy as policy
import native_pull as worker
import pull

SOURCE = 'a' * 40
CONTRACT = 'w31-r6-097385fee9a10b0f'
LIVE = {'contract': CONTRACT}


def release_pair():
    version = '0.1.2-rc.1'
    meta = {'schema': 1, 'source_commit': SOURCE, 'tag': 'v' + version, 'version': version,
            'channel': 'beta', 'protocol': 31, 'contract': CONTRACT,
            'windows_publisher_signature': 'unsigned', 'visual_acceptance': 'pending', 'platforms': {}}
    release = {'tag_name': meta['tag'], 'draft': False, 'published_at': '2026-11-01T00:00:00Z', 'assets': []}
    for os, formats in [('linux', [('appimage', '.AppImage'), ('tarball', '.tar.gz')]),
                        ('windows', [('installer', '-setup.exe'), ('zip', '.zip')])]:
        assets = []
        for kind, suffix in formats:
            name = f'tupi-battle-grounds-{version}-{os}-x86_64{suffix}'
            url = f'https://github.com/{policy.REPOSITORY}/releases/download/{meta["tag"]}/{name}'
            assets.append({'kind': kind, 'name': name, 'url': url, 'bytes': 12345, 'sha256': 'f' * 64})
            release['assets'].append({'name': name, 'browser_download_url': url, 'size': 12345, 'digest': 'sha256:' + 'f' * 64})
        meta['platforms'][os] = {'assets': assets}
    release['assets'].append({'name': 'tbg-native-release.json'})
    return meta, release


class NativeCatalog(unittest.TestCase):
    def test_complete_original_packages_are_available_with_honest_visual_status(self):
        meta, release = release_pair()
        catalog = policy.catalog(meta, release, LIVE)
        for os in ['linux', 'windows']:
            self.assertEqual(catalog[os]['state'], 'available')
            self.assertEqual(catalog[os]['visual_acceptance'], 'pending')
            self.assertEqual(len(catalog[os]['assets']), 2)

    def test_incomplete_changed_foreign_or_incompatible_artifacts_are_refused(self):
        for change in ['missing-platform', 'duplicate-format', 'digest', 'size', 'url', 'filename', 'protocol', 'contract', 'tag', 'source']:
            with self.subTest(change=change):
                meta, release = release_pair()
                asset = meta['platforms']['windows']['assets'][0]
                if change == 'missing-platform': del meta['platforms']['windows']
                elif change == 'duplicate-format': meta['platforms']['windows']['assets'][1] = copy.deepcopy(asset)
                elif change == 'digest': release['assets'][2]['digest'] = 'sha256:' + '0' * 64
                elif change == 'size': release['assets'][2]['size'] = 12
                elif change == 'url': asset['url'] = 'https://evil.example/installer.exe'
                elif change == 'filename': asset['name'] = '../installer.exe'
                elif change == 'protocol': meta['protocol'] = 30
                elif change == 'contract': meta['contract'] = 'w31-r2-0000000000000000'
                elif change == 'tag': release['tag_name'] = 'v0.1.1'
                elif change == 'source': meta['source_commit'] = 'bad'
                with self.assertRaises(ValueError): policy.catalog(meta, release, LIVE)

    def test_semver_order_preserves_rc_progression_and_stable_release(self):
        values = ['0.1.1', '0.1.2-rc.1', '0.1.2-rc.2', '0.1.2-rc.10', '0.1.2', '0.1.3-rc.1']
        self.assertEqual(sorted(reversed(values), key=policy.version_key), values)
        for value in [None, 1, '../1.2.3', 'v1.2.3', '1.2']:
            with self.subTest(value=value), self.assertRaises(ValueError): policy.version_key(value)

    def test_verification_binds_native_workflow_to_exact_tag_source_and_hosted_builder(self):
        with patch.object(pull.subprocess, 'run') as run:
            pull.verify(Path('/tmp/native'), Path('/tmp/bundle'), SOURCE,
                        workflow=policy.WORKFLOW, source_ref='refs/tags/v0.1.2-rc.1')
        args = run.call_args.args[0]
        self.assertEqual(args[args.index('--signer-workflow') + 1], policy.WORKFLOW)
        self.assertEqual(args[args.index('--source-ref') + 1], 'refs/tags/v0.1.2-rc.1')
        self.assertEqual(args[args.index('--source-digest') + 1], SOURCE)
        self.assertIn('--deny-self-hosted-runners', args)

    def test_contract_change_and_revoked_cached_asset_replace_links_atomically(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            public, state = root/'public', root/'state'
            state.mkdir()
            receipt = state/'receipt.json'
            meta, release = release_pair()
            receipt.write_text(json.dumps(meta))
            api_results = [{'id': worker.REPOSITORY_ID}, [release]]
            with patch.multiple(worker, ROOT=public, STATE=state, RECEIPT=receipt, CHECKED=state/'checked.json'), \
                 patch.object(worker, 'status', return_value=LIVE), patch.object(worker, 'api', side_effect=api_results):
                worker.run({'minimum_source': 'b' * 40})
                current = public/'current.json'
                self.assertEqual(json.loads(current.read_text())['windows']['state'], 'available')
                release['assets'][2]['digest'] = 'sha256:' + '0' * 64
                with patch.object(worker, 'api', side_effect=[{'id': worker.REPOSITORY_ID}, [release]]):
                    with self.assertRaises(ValueError): worker.run({'minimum_source': 'b' * 40})
                self.assertEqual(json.loads(current.read_text())['windows']['state'], 'pending')
                worker.synchronize({'contract': 'w32-r6-097385fee9a10b0f'})
                value = json.loads(current.read_text())
                self.assertEqual(value['server']['protocol'], 32)
                self.assertEqual(value['linux']['state'], 'pending')
                self.assertFalse((public/'current.next').exists())
                self.assertEqual(current.stat().st_mode & 0o777, 0o644)
                self.assertEqual(json.loads(receipt.read_text()), meta)

    def test_no_release_keeps_both_downloads_pending_without_binary_or_game_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.multiple(worker, ROOT=root/'public', RECEIPT=root/'receipt.json'), \
                 patch.object(worker, 'status', return_value=LIVE), \
                 patch.object(worker, 'api', side_effect=[{'id': worker.REPOSITORY_ID}, []]), \
                 patch.object(worker, 'fetch') as fetch:
                worker.run({'minimum_source': SOURCE})
                fetch.assert_not_called()
                self.assertEqual(json.loads((root/'public/current.json').read_text())['linux']['state'], 'pending')
                self.assertFalse((root/'receipt.json').exists())


    def test_active_tag_beyond_the_first_page_is_revalidated_and_a_deleted_one_fails_closed(self):
        # Every main merge adds server/web/static releases; the active native tag
        # must not silently fall out of view and keep stale links enabled.
        from urllib.error import HTTPError
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            public, state = root/'public', root/'state'
            state.mkdir()
            receipt = state/'receipt.json'
            meta, release = release_pair()
            receipt.write_text(json.dumps(meta))
            newer = [{'tag_name': 'web-' + 'c' * 40, 'draft': False, 'assets': []}]
            with patch.multiple(worker, ROOT=public, STATE=state, RECEIPT=receipt, CHECKED=state/'checked.json'), \
                 patch.object(worker, 'status', return_value=LIVE), \
                 patch.object(worker, 'api', side_effect=[{'id': worker.REPOSITORY_ID}, newer, release]) as api:
                worker.run({'minimum_source': 'b' * 40})
                self.assertEqual(api.call_args_list[2].args[0], '/releases/tags/v0.1.2-rc.1')
                self.assertEqual(json.loads((public/'current.json').read_text())['linux']['state'], 'available')
                gone = HTTPError('https://api.github.com', 404, 'Not Found', {}, None)
                with patch.object(worker, 'api', side_effect=[{'id': worker.REPOSITORY_ID}, newer, gone]):
                    with self.assertRaisesRegex(ValueError, 'no longer published'):
                        worker.run({'minimum_source': 'b' * 40})
                value = json.loads((public/'current.json').read_text())
                self.assertEqual((value['linux']['state'], value['windows']['state']), ('pending', 'pending'))


if __name__ == '__main__': unittest.main()
