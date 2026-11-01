#!/usr/bin/env python3
"""Static release contracts: no public operator preview and no gameplay/proxy side effects."""
import hashlib
import importlib.util
import io
import json
import os
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'deploy/auto'))
import static_activate as activation
import static_policy as policy
import static_pull
import pull

SOURCE, OLD = 'a' * 40, 'b' * 40


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT/path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


packager = load('package_sites', 'scripts/package-static-sites.py')
scope = load('static_scope', 'scripts/static-release-changes.py')


def bundle(root, script=False, active=False):
    payload = {'landing/index.html': b'<h1>Landing</h1>', 'admin/index.html': b'<h1>In preparation</h1>'}
    if script:
        payload['admin/app.js'] = b'console.log("not public")'
    if active:
        payload['admin/index.html'] = b'<script src="app.js"></script>'
    meta = {'schema': 1, 'source_commit': SOURCE, 'sites': {site: {} for site in policy.SITES}}
    for name, body in payload.items():
        site, path = name.split('/', 1)
        meta['sites'][site][path] = hashlib.sha256(body).hexdigest()
    with tarfile.open(root/'sites.tgz', 'w:gz') as tar:
        for name, body in {'manifest.json': json.dumps(meta).encode(), **payload}.items():
            member = tarfile.TarInfo(name)
            member.size = len(body)
            tar.addfile(member, io.BytesIO(body))
    return root/'sites.tgz'


class StaticRelease(unittest.TestCase):
    def test_actual_site_manifest_exports_public_placeholder_and_landing_only(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = packager.package(root, SOURCE)
            meta = policy.unpack(archive, root/'verified', SOURCE)
            self.assertEqual(set(meta['sites']), {'landing', 'admin'})
            self.assertIn('index.html', meta['sites']['landing'])
            self.assertIn('config/release.json', meta['sites']['landing'])
            self.assertIn('index.html', meta['sites']['admin'])
            self.assertNotIn('assets/js/main.js', meta['sites']['admin'])
            self.assertNotIn('app/assets/js/main.js', meta['sites']['admin'])
            with self.assertRaises(ValueError):
                packager.package(root, SOURCE)

    def test_active_admin_and_preview_javascript_are_refused_even_with_valid_hashes(self):
        for script, active in [(True, False), (False, True)]:
            with self.subTest(script=script, active=active), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                with self.assertRaises(ValueError):
                    policy.unpack(bundle(root, script, active), root/'verified', SOURCE)

    def test_wrong_source_and_too_small_expansion_budget_are_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = bundle(root)
            with self.assertRaises(ValueError):
                policy.unpack(archive, root/'wrong', OLD)
            with patch.object(policy, 'MAX_CONTENT', 1), self.assertRaises(ValueError):
                policy.unpack(archive, root/'large', SOURCE)

    def test_public_static_promotion_keeps_old_assets_and_game_directory(self):
        self.promotion(False)

    def test_second_site_health_failure_recovers_both_entries_without_game_changes(self):
        self.promotion(True)

    def promotion(self, fail):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = policy.unpack(bundle(root), root/'staged', SOURCE)
            state = root/'state'
            state.mkdir()
            game = root/'game'
            game.mkdir()
            (game/'player-data').write_text('preserve')
            for site in policy.SITES:
                old = root/site/'releases'/OLD
                old.mkdir(parents=True)
                (old/'index.html').write_text('old ' + site)
                (root/site/'current').symlink_to('releases/' + OLD)
            receipt = state/'current.json'
            receipt.write_text(json.dumps({'source_commit': OLD}))
            with patch.multiple(activation, ROOT=root, STATE=state, JOURNAL=state/'journal.json', RECEIPT=receipt), \
                 patch.object(activation, 'probe', side_effect=[None, ValueError('public failed')] if fail else None):
                mask = os.umask(0o077)
                try:
                    if fail:
                        with self.assertRaisesRegex(ValueError, 'public failed'):
                            activation.activate(root/'staged', manifest)
                    else:
                        activation.activate(root/'staged', manifest)
                finally:
                    os.umask(mask)
            for site in policy.SITES:
                self.assertEqual((root/site/'current').resolve().name, OLD if fail else SOURCE)
                self.assertEqual((root/site/'releases'/OLD/'index.html').read_text(), 'old ' + site)
                self.assertEqual((root/site/'releases'/SOURCE).stat().st_mode & 0o777, 0o755)
            self.assertEqual((game/'player-data').read_text(), 'preserve')
            self.assertEqual(json.loads(receipt.read_text())['source_commit'], OLD if fail else SOURCE)
            self.assertFalse((state/'journal.json').exists())

    def test_immutable_payload_collision_never_changes_live_pointer(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = policy.unpack(bundle(root), root/'staged', SOURCE)
            candidate = root/'landing/releases'/SOURCE
            candidate.mkdir(parents=True)
            (candidate/'index.html').write_text('different')
            with patch.object(activation, 'ROOT', root), self.assertRaises(ValueError):
                activation.stage(root/'staged', manifest)
            self.assertFalse((root/'landing/current').exists())

    def test_static_scope_ignores_server_only_and_mobile_changes(self):
        self.assertTrue(scope.relevant(['web/landing/index.html']))
        self.assertTrue(scope.relevant(['web/admin/public/admin-public.css']))
        self.assertTrue(scope.relevant(['docs/patches/catalog.json']))
        self.assertFalse(scope.relevant(['crates/tbg-server/src/auth.rs', 'mobile/app.rs']))

    def test_static_verifier_is_bound_to_static_workflow_and_same_main_source(self):
        with patch.object(pull.subprocess, 'run') as run:
            pull.verify(Path('/tmp/package'), Path('/tmp/bundle'), SOURCE, workflow=policy.WORKFLOW)
        args = run.call_args.args[0]
        self.assertEqual(args[args.index('--signer-workflow') + 1], policy.WORKFLOW)
        self.assertIn('--deny-self-hosted-runners', args)
        text = (ROOT/'.github/workflows/static-release.yml').read_text()
        self.assertIn("conclusion == 'success'", text)
        self.assertIn("event == 'push'", text)
        self.assertIn('head_sha == github.sha', text)
        self.assertNotIn('secrets.', text)
        self.assertNotIn('ssh ', text)
        self.assertIn('python3 web/scripts/check.py', text)

    def test_pull_uses_github_browser_download_url_and_the_static_family(self):
        text = (ROOT/'deploy/auto/static_pull.py').read_text()
        self.assertIn("candidate('static')", text)
        self.assertIn("asset['browser_download_url']", text)
        self.assertNotIn('static_download_url', text)
        self.assertNotIn('service(', text)


if __name__ == '__main__':
    unittest.main()
