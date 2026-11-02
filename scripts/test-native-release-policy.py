#!/usr/bin/env python3
"""Desktop release policy: versioned inputs, one tag per version, exact-byte promotion and safe retries."""
import importlib.util
import sys
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'scripts'))
import native_release_policy as policy


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT/path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


gate = load('check_native_version', 'scripts/check-native-version.py')
main_release = load('native_main_release', 'scripts/native-main-release.py')
promotion = load('check_native_promotion', 'scripts/check-native-promotion.py')
publication = load('native_publication', 'scripts/native-publication.py')
REPO, SHA, OTHER = 'owner/repo', 'a' * 40, 'b' * 40


class Inputs(unittest.TestCase):
    def test_shipped_inputs_need_a_version_but_workflow_tests_and_server_do_not(self):
        for path in ['crates/tbg-client/src/main.rs', 'crates/tbg-core/src/combat.rs', 'Cargo.lock',
                     'deploy/native/RELEASE-NOTES.md', 'docs/patches/catalog.json', 'scripts/package-native.py',
                     'scripts/windows-env.ps1', 'LICENSE', '.gitattributes', 'rust-toolchain.toml']:
            with self.subTest(path=path):
                self.assertTrue(policy.release_relevant([path]))
        for path in ['crates/tbg-core/tests/contract_protocol.rs', 'crates/tbg-client/tests/fixtures/x.json',
                     '.github/workflows/native-release.yml', 'scripts/test-native-package.py',
                     'crates/tbg-server/src/main.rs', 'web/landing/index.html', 'docs/RELEASES.md']:
            with self.subTest(path=path):
                self.assertFalse(policy.release_relevant([path]))

    def test_reviewed_sources_name_the_same_version(self):
        self.assertEqual(policy.notes_version(), policy.client_version())


class PullRequestGate(unittest.TestCase):
    def test_reused_published_version_and_stale_notes_fail_early(self):
        client = ['crates/tbg-client/src/ui/mod.rs']
        self.assertIsNone(gate.problem(['docs/README.md'], '0.1.2-rc.1', '0.1.2-rc.1', True))
        self.assertIn('Bump the version', gate.problem(client, '0.1.2-rc.1', '0.1.2-rc.1', True))
        self.assertIn('write the notes for 0.1.2-rc.2', gate.problem(client, '0.1.2-rc.2', '0.1.2-rc.1', False))
        self.assertIsNone(gate.problem(client, '0.1.2-rc.2', '0.1.2-rc.2', False))


class FakeGitHub:
    def __init__(self, tag=None, on_main=True, published=False, changed=None, race=None):
        self.tag, self.main, self.done, self.paths, self.race = tag, on_main, published, changed, race
        self.created, self.dispatched = [], []

    def tag_target(self, tag):
        return self.tag

    def on_main(self, sha):
        return self.main

    def changed(self, base, head):
        return self.paths

    def create_tag(self, tag, sha):
        if self.race:
            self.tag = self.race
            raise FileExistsError('Reference already exists')
        self.created.append((tag, sha))

    def published(self, tag):
        return self.done

    def dispatch(self, tag, run_id):
        self.dispatched.append((tag, run_id))


class MainRelease(unittest.TestCase):
    def test_new_version_on_main_is_tagged_once_and_its_exact_build_dispatched(self):
        github = FakeGitHub()
        main_release.release(github, SHA, '0.1.2-rc.2', '42')
        self.assertEqual(github.created, [('v0.1.2-rc.2', SHA)])
        self.assertEqual(github.dispatched, [('v0.1.2-rc.2', '42')])

    def test_commit_that_left_main_is_never_named(self):
        github = FakeGitHub(on_main=False)
        with self.assertRaises(SystemExit):
            main_release.release(github, SHA, '0.1.2-rc.2', '42')
        self.assertEqual((github.created, github.dispatched), ([], []))

    def test_concurrent_tag_for_the_same_source_continues_and_another_source_is_contention(self):
        same = FakeGitHub(race=SHA)
        main_release.release(same, SHA, '0.1.2-rc.2', '42')
        self.assertEqual(same.dispatched, [('v0.1.2-rc.2', '42')])
        other = FakeGitHub(race=OTHER)
        with self.assertRaisesRegex(SystemExit, 'contention'):
            main_release.release(other, SHA, '0.1.2-rc.2', '42')
        self.assertEqual(other.dispatched, [])

    def test_retry_redispatches_only_an_incomplete_publication(self):
        complete = FakeGitHub(tag=SHA, published=True)
        main_release.release(complete, SHA, '0.1.2-rc.2', '43')
        self.assertEqual(complete.dispatched, [])
        partial = FakeGitHub(tag=SHA, published=False)
        main_release.release(partial, SHA, '0.1.2-rc.2', '43')
        self.assertEqual(partial.dispatched, [('v0.1.2-rc.2', '43')])

    def test_main_advancing_without_shipped_changes_is_quiet_and_with_them_is_contention(self):
        quiet = FakeGitHub(tag=OTHER, changed=['docs/RELEASES.md', '.github/workflows/native-release.yml'])
        main_release.release(quiet, SHA, '0.1.2-rc.2', '44')
        self.assertEqual((quiet.created, quiet.dispatched), ([], []))
        for paths in [['crates/tbg-client/src/main.rs'], None]:
            with self.subTest(paths=paths), self.assertRaisesRegex(SystemExit, 'contention'):
                main_release.release(FakeGitHub(tag=OTHER, changed=paths), SHA, '0.1.2-rc.2', '44')


def run(**changes):
    value = {'head_sha': SHA, 'conclusion': 'success', 'path': promotion.BUILD, 'event': 'push',
             'head_branch': 'main', 'repository': {'full_name': REPO}, 'head_repository': {'full_name': REPO},
             'run_started_at': '2026-11-02T00:00:00Z'}
    value.update(changes)
    return value


class Promotion(unittest.TestCase):
    NOW = datetime(2026, 11, 2, 6, tzinfo=timezone.utc)

    def test_only_verified_main_pushes_and_exact_dispatches_are_promotable(self):
        self.assertIsNone(promotion.problem(run(), REPO, SHA, self.NOW))
        self.assertIsNone(promotion.problem(run(event='workflow_dispatch', head_branch='codex/x'), REPO, SHA, self.NOW))
        for changes in [{'event': 'pull_request'}, {'head_branch': 'codex/feature'}, {'conclusion': 'failure'},
                        {'path': '.github/workflows/ci.yml'}, {'head_repository': {'full_name': 'fork/repo'}},
                        {'head_sha': OTHER}, {'run_started_at': '2026-10-19T00:00:00Z'}]:
            with self.subTest(changes=changes):
                self.assertIsNotNone(promotion.problem(run(**changes), REPO, SHA, self.NOW))

    def test_suite_reuse_needs_the_same_commit_passing_as_a_main_push(self):
        suite = dict(run(path=promotion.SUITE))
        self.assertTrue(promotion.suite_passed([suite], REPO, SHA))
        for changes in [{'head_sha': OTHER}, {'event': 'pull_request'}, {'conclusion': 'failure'},
                        {'head_branch': 'codex/x'}, {'path': promotion.BUILD}]:
            with self.subTest(changes=changes):
                self.assertFalse(promotion.suite_passed([dict(suite, **changes)], REPO, SHA))


class Publication(unittest.TestCase):
    LOCAL = {'a.tar.gz': '1' * 64, 'a.zip': '2' * 64, 'tbg-native-release.json': '3' * 64}

    def test_partial_retry_uploads_only_missing_files_and_never_replaces(self):
        self.assertEqual(publication.plan({}, self.LOCAL), sorted(self.LOCAL))
        partial = {'a.tar.gz': 'sha256:' + '1' * 64}
        self.assertEqual(publication.plan(partial, self.LOCAL), ['a.zip', 'tbg-native-release.json'])
        complete = {name: 'sha256:' + value for name, value in self.LOCAL.items()}
        self.assertEqual(publication.plan(complete, self.LOCAL), [])
        for existing in [{'a.tar.gz': 'sha256:' + '9' * 64}, {'foreign.exe': 'sha256:' + '1' * 64},
                         {'a.tar.gz': None}]:
            with self.subTest(existing=existing), self.assertRaises(SystemExit):
                publication.plan(existing, self.LOCAL)


class Workflow(unittest.TestCase):
    def test_main_release_tags_only_after_both_builds_and_publication_needs_suite_evidence(self):
        text = (ROOT/'.github/workflows/native-release.yml').read_text()
        job = text[text.index('  release-main:'):]
        self.assertIn('needs: [linux, windows]', job)
        self.assertIn("if: github.event_name == 'push' && github.ref == 'refs/heads/main'", job)
        self.assertIn('contents: write', job)
        self.assertIn('actions: write', job)
        self.assertIn("needs.promotion-source.outputs.ci_proven == 'true'", text)
        self.assertIn("needs.validate.result == 'success'", text)
        self.assertIn('scripts/native-publication.py', text)
        self.assertNotIn('secrets.', text)
        self.assertNotIn('gh release create "$GITHUB_REF_NAME" release/*', text)


if __name__ == '__main__':
    unittest.main()
