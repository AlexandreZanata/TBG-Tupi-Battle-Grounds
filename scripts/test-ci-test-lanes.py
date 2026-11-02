#!/usr/bin/env python3
"""The split test lanes must keep running every member with the workspace's unified features."""
import importlib.util
from pathlib import Path
import sys
import tempfile
import textwrap
import unittest

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('test_lanes', Path(__file__).with_name('ci-test-lanes.py'))
LANES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LANES)

GOOD_WORKFLOW = textwrap.dedent('''\
    name: ci
    env:
      RUST_TOOLCHAIN: 1.99.0
    jobs:
      test:
        env:
          TEST_SCOPE: --workspace --exclude tbg-balance --features tbg-core/lab
        steps:
          - run: cargo --config .github/cargo-ci.toml test --locked --no-fail-fast $TEST_SCOPE --no-run
          - run: cargo --config .github/cargo-ci.toml test --locked --no-fail-fast $TEST_SCOPE
      balance:
        env:
          TEST_SCOPE: -p tbg-balance --features tbg-core/training
        steps:
          - run: cargo --config .github/cargo-ci.toml test --locked $TEST_SCOPE --test contract c5_every_tunable_number_is_registered -- --exact --test-threads=1
          - run: cargo --config .github/cargo-ci.toml test --locked --no-fail-fast $TEST_SCOPE
      wasm:
        steps:
          - run: cargo check --locked -p tbg-client -p tbg-core --target wasm32-unknown-unknown
    ''')
EXPECTED = {'test': '--workspace --exclude tbg-balance --features tbg-core/lab',
            'balance': '-p tbg-balance --features tbg-core/training'}


def workspace(manifests):
    """A throwaway workspace with crates/<name>/Cargo.toml for each manifest body."""
    temp = tempfile.TemporaryDirectory(prefix='tbg-lanes-')
    root = Path(temp.name)
    (root/'Cargo.toml').write_text('[workspace]\nresolver = "2"\nmembers = ["crates/*"]\n')
    for name, body in manifests.items():
        (root/'crates'/name).mkdir(parents=True)
        (root/'crates'/name/'Cargo.toml').write_text(f'[package]\nname = "{name}"\nversion = "0.1.0"\n' + body)
    return temp, root


REPO_SHAPE = {
    'tbg-core': '[features]\nlab = []\ntraining = []\n',
    'tbg-balance': '[dependencies]\ntbg-core = { path = "../tbg-core", features = ["lab"] }\n',
    'tbg-client': '[dependencies]\ntbg-core = { path = "../tbg-core", features = ["training"] }\nserde_json = "1"\n',
    'tbg-server': '[dependencies]\ntbg-core = { path = "../tbg-core" }\ntokio = "1"\n',
    'tbg-update': '[dependencies]\nserde = "1"\n',
}


def scopes(changes):
    temp, root = workspace({**REPO_SHAPE, **changes})
    with temp:
        return LANES.expected_scopes(LANES.members(root))


class LaneScopes(unittest.TestCase):
    def test_repository_shape_yields_the_documented_scopes(self):
        self.assertEqual(scopes({}), EXPECTED)

    def test_the_real_repository_and_workflow_agree(self):
        expected = LANES.expected_scopes(LANES.members(LANES.ROOT))
        self.assertEqual(LANES.problems(expected, LANES.WORKFLOW.read_text()), [])

    def test_a_new_member_feature_on_core_reaches_the_balance_lane(self):
        found = scopes({'tbg-server': '[dependencies]\ntbg-core = { path = "../tbg-core", features = ["bench-profile"] }\n'})
        self.assertEqual(found['balance'], '-p tbg-balance --features tbg-core/bench-profile,tbg-core/training')
        self.assertNotEqual(LANES.problems(found, GOOD_WORKFLOW), [])

    def test_target_specific_and_dev_dependencies_count(self):
        found = scopes({'tbg-client': "[target.'cfg(unix)'.dev-dependencies]\ntbg-core = { path = \"../tbg-core\", features = [\"training\"] }\n"})
        self.assertEqual(found, EXPECTED)

    def test_lab_without_core_features_runs_the_test_lane_without_feature_flags(self):
        found = scopes({'tbg-balance': '[dependencies]\ntbg-core = { path = "../tbg-core" }\n'})
        self.assertEqual(found['test'], '--workspace --exclude tbg-balance')


class UnsafeSplitsFailClosed(unittest.TestCase):
    def test_lab_with_any_other_dependency_is_rejected(self):
        for extra in ['serde = "1"\n', '[dev-dependencies]\nproptest = "1"\n',
                      '[build-dependencies]\ncc = "1"\n', "[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"\n"]:
            body = REPO_SHAPE['tbg-balance'] + extra
            with self.subTest(extra=extra), self.assertRaisesRegex(ValueError, 'must depend on tbg-core only'):
                scopes({'tbg-balance': body})

    def test_lab_with_a_non_plain_core_dependency_is_rejected(self):
        for spec in ['{ path = "../tbg-core", default-features = false }', '{ path = "../tbg-core", optional = true }',
                     '{ git = "https://example.invalid/tbg" }', '"0.1"']:
            with self.subTest(spec=spec), self.assertRaisesRegex(ValueError, 'plain path dependency'):
                scopes({'tbg-balance': f'[dependencies]\ntbg-core = {spec}\n'})

    def test_missing_lab_or_core_is_rejected(self):
        for name in ['tbg-balance', 'tbg-core']:
            shape = {key: body for key, body in REPO_SHAPE.items() if key != name}
            temp, root = workspace(shape)
            with temp, self.subTest(missing=name), self.assertRaisesRegex(ValueError, 'workspace members'):
                LANES.expected_scopes(LANES.members(root))


class WorkflowDrift(unittest.TestCase):
    def test_matching_workflow_passes(self):
        self.assertEqual(LANES.problems(EXPECTED, GOOD_WORKFLOW), [])

    def test_changed_missing_or_extra_scopes_fail(self):
        cases = {
            'changed': GOOD_WORKFLOW.replace('--features tbg-core/lab', ''),
            'missing': GOOD_WORKFLOW.replace('      TEST_SCOPE: -p tbg-balance --features tbg-core/training\n', ''),
            'extra': GOOD_WORKFLOW + '  more:\n    env:\n      TEST_SCOPE: -p tbg-core\n',
        }
        for name, text in cases.items():
            with self.subTest(case=name):
                self.assertNotEqual(LANES.problems(EXPECTED, text), [])

    def test_a_cargo_test_bypassing_the_checked_scope_fails(self):
        text = GOOD_WORKFLOW + '          - run: cargo test --locked -p tbg-server\n'
        self.assertTrue(any('without the checked scope' in issue for issue in LANES.problems(EXPECTED, text)))

    def test_a_workflow_without_cargo_test_fails(self):
        text = '\n'.join(line for line in GOOD_WORKFLOW.splitlines() if 'cargo --config' not in line)
        self.assertIn('no cargo test invocation found', LANES.problems(EXPECTED, text))

    def test_missing_or_late_balance_preflight_fails(self):
        preflight = next(line for line in GOOD_WORKFLOW.splitlines() if 'c5_every_tunable' in line)
        full = GOOD_WORKFLOW.splitlines()[GOOD_WORKFLOW.splitlines().index(preflight) + 1]
        for changed in [GOOD_WORKFLOW.replace(preflight + '\n', ''),
                        GOOD_WORKFLOW.replace(preflight + '\n' + full, full + '\n' + preflight)]:
            self.assertTrue(any('exact C5 preflight' in issue for issue in LANES.problems(EXPECTED, changed)))

    def test_preflight_cannot_omit_exact_selection_or_bypass_feature_scope(self):
        for changed in [GOOD_WORKFLOW.replace(' --exact', ''),
                        GOOD_WORKFLOW.replace('$TEST_SCOPE --test contract', '-p tbg-balance --test contract')]:
            self.assertNotEqual(LANES.problems(EXPECTED, changed), [])

    def test_full_balance_lane_cannot_skip_the_contract_after_preflight(self):
        changed = GOOD_WORKFLOW.replace('--no-fail-fast $TEST_SCOPE\n',
                                         '--no-fail-fast $TEST_SCOPE -- --skip c7_planted_imbalances\n')
        self.assertTrue(any('unfiltered test lane' in issue for issue in LANES.problems(EXPECTED, changed)))

    def test_scopes_outside_jobs_are_ignored(self):
        text = 'env:\n  TEST_SCOPE: --workspace\n' + GOOD_WORKFLOW
        self.assertEqual(LANES.problems(EXPECTED, text), [])


if __name__ == '__main__':
    unittest.main()
