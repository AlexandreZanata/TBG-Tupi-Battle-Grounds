#!/usr/bin/env python3
"""Release regressions: immutable browser assets, rejected bundles and recoverable entry changes."""
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
import web_activate as activation
import web_policy as policy
import pull

SOURCE, OLD = 'a' * 40, 'b' * 40
CONTRACT = 'w31-r6-097385fee9a10b0f'


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT/path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


packager = module('package_web', 'scripts/package-web.py')
scope = module('web_scope', 'scripts/web-release-changes.py')


def fixture(folder):
    dist = folder/'dist'
    dist.mkdir()
    (dist/'index.html').write_text('<html><head></head><script src="/releases/' + SOURCE + '/app.js"></script></html>')
    (dist/'app.js').write_text('fetch("assets/model.bin")')
    (dist/'app.wasm').write_bytes(b'\0asm')
    (dist/'assets').mkdir()
    (dist/'assets/model.bin').write_bytes(b'new model')
    return packager.package(dist, folder/'out', SOURCE, CONTRACT)


def rewrite(archive, output, change):
    with tarfile.open(archive) as source, tarfile.open(output, 'w:gz') as target:
        for member in source:
            body = source.extractfile(member).read()
            member, body = change(member, body)
            target.addfile(member, io.BytesIO(body))


class BrowserRelease(unittest.TestCase):
    def test_public_probe_uses_product_agent_and_still_rejects_changed_content(self):
        # The official Cloudflare edge returns 1010 for Python's generic agent.
        class Response:
            status = 200
            def __enter__(self):
                return self
            def __exit__(self, *args):
                return False
            def read(self, size):
                return b'approved entry'
        def edge(request, *, timeout):
            self.assertEqual(request.full_url, 'https://app.tupibg.com/')
            self.assertEqual(request.get_header('User-agent'), 'TBG-release-health/1.0')
            return Response()
        with patch.object(activation, 'urlopen', side_effect=edge):
            activation.probe('https://app.tupibg.com/', hashlib.sha256(b'approved entry').hexdigest())
            with self.assertRaises(ValueError):
                activation.probe('https://app.tupibg.com/', '0' * 64)

    def test_round_trip_seals_source_hashes_and_relative_asset_base(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = policy.unpack(fixture(root), root/'verified', SOURCE)
            self.assertEqual(manifest['contract'], CONTRACT)
            html = (root/'verified/index.html').read_text()
            self.assertIn('<base href="/releases/' + SOURCE + '/">', html)
            from urllib.parse import urljoin
            self.assertEqual(urljoin('https://app.tupibg.com' + manifest['public_url'], 'assets/model.bin'),
                             'https://app.tupibg.com/releases/' + SOURCE + '/assets/model.bin')
            self.assertEqual(set(manifest['files']), {'index.html', 'app.js', 'app.wasm', 'assets/model.bin'})

    def test_rejects_traversal_links_and_duplicate_archive_members(self):
        for kind in ['traversal', 'symlink', 'duplicate']:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                archive = fixture(root)
                def change(member, body):
                    if member.name == 'web/app.js':
                        if kind == 'traversal':
                            member.name = 'web/../../escape'
                        elif kind == 'symlink':
                            member.type, member.linkname, member.size, body = tarfile.SYMTYPE, '/etc/passwd', 0, b''
                        else:
                            member.name = 'web/index.html'
                    return member, body
                rewrite(archive, root/'bad.tar.gz', change)
                with self.assertRaises(ValueError):
                    policy.unpack(root/'bad.tar.gz', root/'verified', SOURCE)
                self.assertFalse((root/'escape').exists())

    def test_changed_source_hash_and_expansion_budget_fail_closed(self):
        for kind in ['source', 'hash', 'budget']:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                archive = fixture(root)
                def change(member, body):
                    if kind == 'hash' and member.name == 'web/app.js':
                        body = b'changed payload'
                        member.size = len(body)
                    return member, body
                rewrite(archive, root/'changed.tar.gz', change)
                with patch.object(policy, 'MAX_CONTENT', 1 if kind == 'budget' else policy.MAX_CONTENT):
                    with self.assertRaises(ValueError):
                        policy.unpack(root/'changed.tar.gz', root/'verified', OLD if kind == 'source' else SOURCE)

    def test_immutable_package_refuses_replacement_and_excluded_frontends(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = fixture(root).read_bytes()
            with self.assertRaises(ValueError):
                packager.package(root/'dist', root/'out', SOURCE, CONTRACT)
            self.assertEqual((root/'out'/('tbg-web-' + SOURCE + '.tar.gz')).read_bytes(), original)
            (root/'dist/admin').mkdir()
            (root/'dist/admin/index.html').write_text('excluded')
            with self.assertRaises(ValueError):
                packager.package(root/'dist', root/'other', SOURCE, CONTRACT)

    def test_web_promotion_preserves_legacy_assets_and_old_tabs_with_private_umask(self):
        self.promotion(False)

    def test_failed_public_probe_restores_previous_entry_and_receipt(self):
        self.promotion(True)

    def promotion(self, fail):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            staged = root/'staged'
            manifest = policy.unpack(fixture(root), staged, SOURCE)
            web, state = root/'web', root/'state'
            state.mkdir()
            old = web/'releases'/OLD
            old.mkdir(parents=True)
            (old/'model.bin').write_bytes(b'old tab must keep this')
            (web/'index.html').write_bytes(b'legacy entry')
            (web/'current').symlink_to('releases/' + OLD)
            receipt = state/'web-current.json'
            receipt.write_text(json.dumps({'source_commit': OLD}))
            with patch.multiple(activation, ROOT=web, JOURNAL=state/'journal.json', RECEIPT=receipt, STATE=state), \
                 patch.object(activation, 'status', return_value={'contract': CONTRACT, 'shutdown': False}), \
                 patch.object(activation, 'probe', side_effect=[None, ValueError('public failed')] if fail else None) as probe:
                mask = os.umask(0o077)
                try:
                    if fail:
                        with self.assertRaisesRegex(ValueError, 'public failed'):
                            activation.activate(staged, manifest, 'https://app.tupibg.com')
                    else:
                        activation.activate(staged, manifest, 'https://app.tupibg.com')
                finally:
                    os.umask(mask)
            self.assertEqual((web/'current').resolve(), web/'releases'/(OLD if fail else SOURCE))
            self.assertEqual((old/'model.bin').read_bytes(), b'old tab must keep this')
            self.assertEqual((web/'index.html').read_bytes(), b'legacy entry')
            self.assertEqual(json.loads(receipt.read_text())['source_commit'], OLD if fail else SOURCE)
            self.assertFalse((state/'journal.json').exists())
            self.assertEqual((web/'releases').stat().st_mode & 0o777, 0o755)
            self.assertEqual((web/'releases'/SOURCE).stat().st_mode & 0o777, 0o755)
            self.assertEqual(probe.call_args_list[0].args[0], 'https://app.tupibg.com/releases/' + SOURCE + '/index.html')

    def test_contract_mismatch_refuses_before_writing_a_browser_entry(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = policy.unpack(fixture(root), root/'staged', SOURCE)
            with patch.object(activation, 'status', return_value={'contract': 'old', 'shutdown': False}), \
                 patch.object(activation, 'ROOT', root/'web'):
                with self.assertRaises(ValueError):
                    activation.activate(root/'staged', manifest, 'https://app.tupibg.com')
            self.assertFalse((root/'web').exists())

    def test_interrupted_first_promotion_recovers_legacy_entry_without_deleting_assets(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            release = root/'releases'/SOURCE
            release.mkdir(parents=True)
            (release/'index.html').write_text('new')
            (root/'index.html').write_text('legacy')
            (root/'current').symlink_to('releases/' + SOURCE)
            journal = root/'journal.json'
            journal.write_text(json.dumps({'previous': None, 'receipt': None}))
            with patch.multiple(activation, ROOT=root, STATE=root, JOURNAL=journal, RECEIPT=root/'receipt.json'):
                activation.recover()
            self.assertFalse((root/'current').exists())
            self.assertEqual((root/'index.html').read_text(), 'legacy')
            self.assertEqual((release/'index.html').read_text(), 'new')

    def test_source_scope_excludes_deferred_frontends_and_keeps_game_dependencies(self):
        self.assertFalse(scope.relevant(['web/admin/app.js', 'web/landing/index.html', 'mobile/app.rs']))
        self.assertTrue(scope.relevant(['crates/tbg-client/assets/arenas/forest.webp']))
        self.assertTrue(scope.relevant(['crates/tbg-core/src/protocol.rs']))

    def test_browser_verifier_binds_the_browser_workflow(self):
        with patch.object(pull.subprocess, 'run') as command:
            pull.verify(Path('/tmp/web'), Path('/tmp/bundle'), SOURCE, workflow=policy.WORKFLOW)
        args = command.call_args.args[0]
        self.assertEqual(args[args.index('--signer-workflow') + 1], policy.WORKFLOW)
        self.assertIn('--deny-self-hosted-runners', args)

    def test_pipeline_retains_ci_source_gates_and_no_origin_credential(self):
        text = (ROOT/'.github/workflows/web-release.yml').read_text()
        for guard in ["conclusion == 'success'", "event == 'push'", "head_branch == 'main'",
                      'head_repository.full_name == github.repository', 'head_sha == github.sha']:
            self.assertIn(guard, text)
        self.assertNotIn('secrets.', text)
        self.assertNotIn('ssh ', text)
        self.assertIn('--public-url "/releases/$TBG_SOURCE_COMMIT/"', text)
        self.assertIn('attest-build-provenance@4d101475d8b20a2381f78447822ac1eab6504dd8', text)


    def test_entry_changed_outside_the_controller_needs_explicit_adoption(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            staged = root/'staged'
            manifest = policy.unpack(fixture(root), staged, SOURCE)
            web, state = root/'web', root/'state'
            state.mkdir()
            for name in [OLD, 'c' * 40]:
                (web/'releases'/name).mkdir(parents=True)
            (web/'current').symlink_to('releases/' + 'c' * 40)
            receipt = state/'web-current.json'
            receipt.write_text(json.dumps({'source_commit': OLD}))
            with patch.multiple(activation, ROOT=web, JOURNAL=state/'journal.json', RECEIPT=receipt, STATE=state), \
                 patch.object(activation, 'status', return_value={'contract': CONTRACT, 'shutdown': False}), \
                 patch.object(activation, 'probe'):
                with self.assertRaisesRegex(ValueError, 'changed outside the controller'):
                    activation.activate(staged, manifest, 'https://app.tupibg.com')
                self.assertEqual((web/'current').resolve().name, 'c' * 40)
                activation.activate(staged, manifest, 'https://app.tupibg.com', adopt=True)
            self.assertEqual((web/'current').resolve().name, SOURCE)

    def test_new_server_contract_waits_and_draining_server_defers_without_latch(self):
        import web_pull
        from control import Deferred
        prefix = 'https://github.com/' + pull.REPOSITORY + '/releases/download/web-' + SOURCE + '/'
        names = ['tbg-web-' + SOURCE + '.tar.gz', 'provenance.jsonl']
        release = {'tag_name': 'web-' + SOURCE, 'assets': [{'name': n, 'browser_download_url': prefix + n} for n in names]}
        with tempfile.TemporaryDirectory() as temporary:
            state = Path(temporary)
            receipt = state/'web-current.json'
            receipt.write_text(json.dumps({'source_commit': OLD}))
            live = {'contract': CONTRACT, 'shutdown': False}
            with patch.multiple(web_pull, STATE=state, WAITING=state/'web-waiting.json',
                                candidate=lambda family: release, api=lambda path: {'status': 'ahead'},
                                verify=lambda *a, **k: None), \
                 patch.object(web_pull, 'fetch', return_value=b'x') as fetch, \
                 patch.object(web_pull.web_policy, 'unpack', return_value={'contract': 'w32-r6-097385fee9a10b0f'}), \
                 patch.object(web_pull.web_activate, 'RECEIPT', receipt), \
                 patch.object(web_pull.web_activate, 'status', side_effect=lambda: live), \
                 patch.object(web_pull.web_activate, 'activate') as promote:
                with self.assertRaisesRegex(Deferred, 'needs server contract w32'):
                    web_pull.run({'minimum_source': OLD}, True, False)
                fetch.reset_mock()
                with self.assertRaisesRegex(Deferred, 'waits for server contract'):
                    web_pull.run({'minimum_source': OLD}, True, False)
                fetch.assert_not_called()
                live['shutdown'] = True
                with self.assertRaisesRegex(Deferred, 'draining'):
                    web_pull.run({'minimum_source': OLD}, True, False)
                promote.assert_not_called()
            self.assertFalse((state/'web-checked.json').exists(), 'transient states are not outcomes')


if __name__ == '__main__':
    unittest.main()
