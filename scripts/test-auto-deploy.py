#!/usr/bin/env python3
"""Regression guards for provenance gates, untrusted archives and durable backend rollback."""
import importlib.util
import io
import json
import os
import sqlite3
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'deploy/auto'))
import activate
import policy

SOURCE = 'b'*40
CONTRACT = 'w31-r6-097385fee9a10b0f'


def manifest():
    return {'schema': 1, 'source_commit': SOURCE, 'contract': CONTRACT,
            'target': 'x86_64-unknown-linux-gnu', 'database_schema': 10,
            'files': {name: __import__('hashlib').sha256(b'\x7fELFprobe').hexdigest() for name in policy.FILES}}


def archive(path, entries):
    with tarfile.open(path, 'w:gz') as tar:
        for name, data, kind in entries:
            member = tarfile.TarInfo(name)
            member.type = kind
            member.size = len(data)
            tar.addfile(member, io.BytesIO(data))


def entries(value=None):
    return [('manifest.json', json.dumps(value or manifest()).encode(), tarfile.REGTYPE)] + [
        (name, b'\x7fELFprobe', tarfile.REGTYPE) for name in sorted(policy.FILES)]


class DeploymentGuards(unittest.TestCase):
    def test_valid_package_is_strictly_bounded_and_hash_checked(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            archive(root/'a.tgz', entries())
            result = policy.unpack(root/'a.tgz', root/'verified', SOURCE)
            self.assertEqual(result['contract'], CONTRACT)
            self.assertEqual({p.name for p in (root/'verified').iterdir()}, policy.FILES)

    def test_traversal_symlink_duplicate_and_extra_members_are_refused(self):
        malicious = [
            entries()+[('../escape', b'x', tarfile.REGTYPE)],
            [*entries()[:-1], ('tbg-server', b'', tarfile.SYMTYPE)],
            [*entries()[:-1], entries()[1]],
            entries()+[('run-as-root.sh', b'bad', tarfile.REGTYPE)],
        ]
        for members in malicious:
            with self.subTest(members=[x[0] for x in members]), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                archive(root/'a.tgz', members)
                with self.assertRaises(ValueError):
                    policy.unpack(root/'a.tgz', root/'verified', SOURCE)
                self.assertFalse((root.parent/'escape').exists())

    def test_wrong_source_or_binary_hash_is_refused(self):
        for key in ['source_commit', 'files']:
            value = manifest()
            value[key] = 'c'*40 if key == 'source_commit' else {n: '0'*64 for n in policy.FILES}
            with self.subTest(key=key), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                archive(root/'a.tgz', entries(value))
                with self.assertRaises(ValueError):
                    policy.unpack(root/'a.tgz', root/'verified', SOURCE)

    def test_automatic_activation_refuses_contract_schema_slots_and_existing_drain(self):
        live = {'contract': CONTRACT, 'slot': None, 'shutdown': False}
        policy.compatible(manifest(), live, 10)
        cases = [({**live, 'contract': 'w30-r2-'+'0'*16}, 10), (live, 11),
                 ({**live, 'slot': 'a'}, 10), ({**live, 'shutdown': True}, 10)]
        for status, schema in cases:
            with self.subTest(status=status, schema=schema), self.assertRaises(ValueError):
                policy.compatible(manifest(), status, schema)

    def test_verifier_binds_repo_workflow_main_source_and_hosted_builder(self):
        import pull
        with patch.object(pull.subprocess, 'run') as run:
            pull.verify(Path('/tmp/package'), Path('/tmp/bundle'), SOURCE)
        command = run.call_args.args[0]
        for flag, expected in [('--repo', policy.REPOSITORY), ('--source-ref', 'refs/heads/main'),
                               ('--source-digest', SOURCE), ('--signer-workflow', policy.WORKFLOW)]:
            self.assertEqual(command[command.index(flag)+1], expected)
        self.assertIn('--deny-self-hosted-runners', command)

    def test_crash_recovery_restores_pointers_receipt_and_preserves_player_data(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            binaries, releases, state = [root/p for p in ['bin', 'releases', 'state']]
            for directory in [binaries, releases, state]:
                directory.mkdir()
            old = releases/'old'
            new = releases/SOURCE
            old.mkdir(); new.mkdir()
            for name in policy.FILES:
                (old/name).write_bytes(b'old')
                (new/name).write_bytes(b'new')
                (binaries/name).symlink_to(new/name)
            db = root/'players.sqlite'
            with sqlite3.connect(db) as connection:
                connection.execute('CREATE TABLE player_state(value TEXT)')
                connection.execute("INSERT INTO player_state VALUES ('must survive')")
            previous = {'source_commit': 'a'*40}
            activate.write_json(state/'current.json', manifest())
            activate.write_json(state/'transition.json', {'previous': {n: str(old/n) for n in policy.FILES},
                'contract': CONTRACT, 'public_health': 'https://api.example.com/health', 'previous_receipt': previous})
            with patch.multiple(activate, BIN=binaries, RELEASES=releases, STATE=state, JOURNAL=state/'transition.json'), \
                 patch.object(activate, 'service') as service, patch.object(activate, 'require_running_binary'), \
                 patch.object(activate, 'healthy', return_value=True):
                activate.recover()
            self.assertEqual([c.args[0] for c in service.call_args_list], ['stop', 'start'])
            self.assertTrue(all((binaries/n).resolve() == old/n for n in policy.FILES))
            self.assertEqual(json.loads((state/'current.json').read_text()), previous)
            with sqlite3.connect(db) as connection:
                self.assertEqual(connection.execute('SELECT value FROM player_state').fetchone()[0], 'must survive')
            self.assertFalse((state/'transition.json').exists())

    def test_package_round_trip_binds_the_checked_out_source_and_only_two_binaries(self):
        spec = importlib.util.spec_from_file_location('package_backend', ROOT/'scripts/package-backend.py')
        package = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(package)
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            binaries = root/'bin'
            binaries.mkdir()
            for name in policy.FILES:
                (binaries/name).write_bytes(b'\x7fELFprobe')
            identity = {k: v for k, v in manifest().items() if k not in ['files', 'schema']}
            with patch.object(sys, 'argv', ['package-backend.py', '--directory', str(binaries), '--output', str(root/'out')]), \
                 patch.object(package.subprocess, 'check_output', side_effect=[SOURCE, json.dumps(identity)]), \
                 __import__('contextlib').redirect_stdout(io.StringIO()):
                package.main()
            result = policy.unpack(root/'out'/('tbg-backend-'+SOURCE+'.tar.gz'), root/'verified', SOURCE)
            self.assertEqual(result['source_commit'], SOURCE)
            self.assertEqual(result['files'], manifest()['files'])

    def test_failed_health_rolls_back_binary_and_receipt_without_restoring_database(self):
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            binaries, releases, state, staged = [root/p for p in ['bin', 'releases', 'state', 'staged']]
            for directory in [binaries, releases, state, staged]:
                directory.mkdir()
            for name in policy.FILES:
                (binaries/name).write_bytes(('previous-'+name).encode())
                (staged/name).write_bytes(b'\x7fELFprobe')
            db = root/'players.sqlite'
            with sqlite3.connect(db) as connection:
                connection.execute('CREATE TABLE schema_version(version INTEGER)')
                connection.execute('INSERT INTO schema_version VALUES (10)')
                connection.execute('CREATE TABLE player_state(value TEXT)')
                connection.execute("INSERT INTO player_state VALUES ('new result survives rollback')")
            previous = {'source_commit': 'a'*40}
            activate.write_json(state/'current.json', previous)
            def command(args, **kwargs):
                return SimpleNamespace(stdout=json.dumps(manifest()).encode())
            live = {'contract': CONTRACT, 'slot': None, 'shutdown': False}
            with patch.multiple(activate, BIN=binaries, RELEASES=releases, STATE=state, DB=db, JOURNAL=state/'transition.json'), \
                 patch.object(activate, 'status', return_value=live), \
                 patch.object(activate.shutil, 'disk_usage', return_value=SimpleNamespace(free=1<<30)), \
                 patch.object(activate.subprocess, 'run', side_effect=command), \
                 patch.object(activate, 'service') as service, patch.object(activate, 'require_running_binary'), \
                 patch.object(activate, 'healthy', side_effect=[False, True]):
                previous_mask = os.umask(0o077)
                try:
                    with self.assertRaisesRegex(ValueError, 'candidate health failed'):
                        activate.activate(staged, manifest(), 'https://api.example.com/health')
                finally:
                    os.umask(previous_mask)
            self.assertEqual((releases/SOURCE).stat().st_mode & 0o777, 0o755)
            self.assertEqual([c.args[0] for c in service.call_args_list], ['stop', 'start', 'stop', 'start'])
            for name in policy.FILES:
                self.assertEqual((binaries/name).read_bytes(), ('previous-'+name).encode())
            self.assertEqual(json.loads((state/'current.json').read_text()), previous)
            with sqlite3.connect(db) as connection:
                self.assertEqual(connection.execute('SELECT value FROM player_state').fetchone()[0], 'new result survives rollback')
            self.assertFalse((state/'transition.json').exists())

    def test_unadopted_runtime_is_refused_before_database_or_service_changes(self):
        with patch.object(activate, 'require_running_binary', side_effect=ValueError('explicit adoption')), \
             patch.object(activate, 'service') as service, patch.object(activate.sqlite3, 'connect') as database:
            with self.assertRaisesRegex(ValueError, 'explicit adoption'):
                activate.activate(Path('/tmp/not-staged'), manifest(), 'https://api.example.com/health')
        service.assert_not_called()
        database.assert_not_called()

    def test_running_service_must_match_the_actual_kernel_executable_path(self):
        with patch.object(activate.subprocess, 'check_output', return_value=str(os.getpid())):
            activate.require_running_binary(Path(sys.executable))
            with tempfile.TemporaryDirectory() as directory:
                wrong = Path(directory)/'unused-tbg-server'
                wrong.write_bytes(b'not-running')
                with self.assertRaisesRegex(ValueError, 'explicit adoption'):
                    activate.require_running_binary(wrong)
        with patch.object(activate.subprocess, 'check_output', return_value='0'):
            with self.assertRaises(ValueError): activate.require_running_binary(Path(sys.executable))

    def test_frontend_only_merges_do_not_redeploy_backend(self):
        spec = importlib.util.spec_from_file_location('scope', ROOT/'scripts/backend-release-changes.py')
        scope = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(scope)
        self.assertFalse(scope.relevant(['web/landing/index.html', 'web/admin/app.js', 'docs/README.md']))
        for path in ['crates/tbg-core/src/types.rs', 'crates/tbg-server/src/main.rs',
                     'Cargo.lock', 'docs/patches/catalog.json']:
            self.assertTrue(scope.relevant([path]))

    def test_workflow_only_builds_successful_current_main_pushes(self):
        text = (ROOT/'.github/workflows/backend-release.yml').read_text()
        for guard in ["conclusion == 'success'", "event == 'push'", "head_branch == 'main'",
                      'head_repository.full_name == github.repository', 'head_sha == github.sha']:
            self.assertIn(guard, text)
        self.assertIn('persist-credentials: false', text)
        self.assertIn('attest-build-provenance@4d101475d8b20a2381f78447822ac1eab6504dd8', text)
        self.assertNotIn('secrets.', text)
        self.assertNotIn('self-hosted', text)
        self.assertNotIn('ssh ', text)



class ControllerSafety(unittest.TestCase):
    """Automation never restarts the only game process, loops on a failed rollback or spends quota blindly."""

    def setUp(self):
        import control
        import github
        import pull
        self.control, self.github, self.pull = control, github, pull
        self.temporary = tempfile.TemporaryDirectory()
        self.state = Path(self.temporary.name)
        self.patches = [patch.multiple(control, STATE=self.state, HOLD=self.state/'operator-hold.json'),
                        patch.multiple(pull, STATE=self.state, JOURNAL=self.state/'transition.json')]
        for item in self.patches:
            item.start()

    def tearDown(self):
        for item in reversed(self.patches):
            item.stop()
        self.temporary.cleanup()

    def release(self):
        prefix = 'https://github.com/' + policy.REPOSITORY + '/releases/download/server-' + SOURCE + '/'
        names = ['tbg-backend-' + SOURCE + '.tar.gz', 'provenance.jsonl']
        return {'tag_name': 'server-' + SOURCE, 'assets': [{'name': n, 'browser_download_url': prefix + n} for n in names]}

    def run_backend(self, window=False, **changes):
        options = dict(candidate=lambda *a: self.release(), api=lambda path: {'status': 'ahead'},
                       fetch=lambda url, limit: b'package', verify=lambda *a, **k: None,
                       unpack=lambda *a: manifest(), recover=lambda: None)
        options.update(changes)
        with patch.multiple(self.pull, **options):
            self.pull.run({'minimum_source': 'c' * 40, 'public_health': 'https://api.tupibg.com/health'},
                          True, False, window)

    def test_busy_lock_defers_and_the_runner_exits_zero(self):
        import fcntl
        with (self.state/'lock').open('w') as held:
            fcntl.flock(held, fcntl.LOCK_EX)
            with self.assertRaises(self.control.Deferred):
                with self.control.deployment_lock():
                    self.fail('a second promotion must not start')
            self.control.run_controller(lambda: (_ for _ in ()).throw(self.control.Deferred('busy')))
        with self.assertRaises(SystemExit) as failed:
            self.control.run_controller(lambda: (_ for _ in ()).throw(ValueError('forged')))
        self.assertEqual(failed.exception.code, 1)

    def test_timer_never_restarts_the_single_game_process(self):
        with patch.object(self.pull, 'activate') as activate_backend:
            with self.assertRaisesRegex(ValueError, 'maintenance-window'):
                self.run_backend()
            activate_backend.assert_not_called()
            self.assertFalse((self.state/'checked.json').exists(), 'a policy refusal is not a candidate outcome')

    def test_failed_rollback_holds_automation_until_an_operator_releases_it(self):
        def broken(*args):
            (self.state/'transition.json').write_text('{}')
            raise ValueError('rollback health failed; operator required')
        with patch.object(self.pull, 'activate', side_effect=broken):
            with self.assertRaises(ValueError):
                with patch.multiple(self.pull, candidate=lambda *a: self.release(), api=lambda p: {'status': 'ahead'},
                                    fetch=lambda u, m: b'p', verify=lambda *a, **k: None, unpack=lambda *a: manifest()):
                    self.pull.run({'minimum_source': 'c' * 40, 'public_health': 'x'}, True, False, True)
        self.assertTrue((self.state/'operator-hold.json').exists())
        recovered = []
        with patch.multiple(self.pull, recover=lambda: recovered.append(1), candidate=lambda *a: self.fail('no work')):
            with self.assertRaises(self.control.OperatorHold):
                self.pull.run({'minimum_source': 'c' * 40}, True, False)
        self.assertEqual(recovered, [], 'no further stop/start loop while held')
        self.control.release_hold()
        self.assertFalse((self.state/'operator-hold.json').exists())

    def test_forged_candidate_is_latched_but_network_failure_is_retried(self):
        import subprocess
        forged = subprocess.CalledProcessError(1, ['gh', 'attestation', 'verify'])
        with self.assertRaises(subprocess.CalledProcessError):
            self.run_backend(verify=lambda *a, **k: (_ for _ in ()).throw(forged), window=True)
        self.assertEqual(json.loads((self.state/'checked.json').read_text())['outcome'], 'rejected')
        (self.state/'checked.json').unlink()
        from urllib.error import URLError
        with self.assertRaises(URLError):
            self.run_backend(fetch=lambda url, limit: (_ for _ in ()).throw(URLError('offline')), window=True)
        self.assertFalse((self.state/'checked.json').exists())

    def test_unchanged_api_answers_are_conditional_and_quota_is_logged(self):
        calls = []
        def answer(url, maximum, headers=None):
            calls.append(headers or {})
            if len(calls) == 1:
                return 200, {'ETag': '"v1"', 'X-RateLimit-Remaining': '59'}, b'[{"tag_name": "web-x"}]'
            return 304, {'X-RateLimit-Remaining': '59'}, b''
        with patch.multiple(self.github, CACHE=self.state/'api-cache', request=answer):
            first = self.github.api('/releases?per_page=100')
            second = self.github.api('/releases?per_page=100')
        self.assertEqual(first, second)
        self.assertEqual(calls[1], {'If-None-Match': '"v1"'})


if __name__ == '__main__':
    unittest.main()
