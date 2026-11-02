#!/usr/bin/env python3
"""Prevent a failed, skipped, cancelled, absent or ungated CI lane from passing the merge gate."""
import importlib.util
from pathlib import Path
import re
import sys
import tomllib
import unittest

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('ci_gate', Path(__file__).with_name('ci-required-checks.py'))
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)
ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT/'.github/workflows'
PINNED = re.compile(r'^\s*(?:-\s+)?uses:\s+(?:\./\.github/workflows/[\w.-]+\.ya?ml|[\w.-]+/[\w.-]+(?:/[\w./-]+)?@[0-9a-f]{40}\s+#\s*\S.*)$')


def workflow_texts():
    return {path.name: path.read_text() for path in sorted(WORKFLOWS.glob('*.y*ml'))}


class RequiredChecks(unittest.TestCase):
    def test_every_successful_lane_is_required(self):
        self.assertEqual(GATE.failures({name: {'result': 'success'} for name in GATE.REQUIRED}), [])

    def test_failure_cancellation_skips_and_missing_results_fail_closed(self):
        for name in GATE.REQUIRED:
            for state in ['failure', 'cancelled', 'skipped', None, '']:
                with self.subTest(name=name, state=state):
                    needs = {job: {'result': 'success'} for job in GATE.REQUIRED}
                    needs[name] = {'result': state}
                    self.assertTrue(GATE.failures(needs))

    def test_missing_unexpected_and_malformed_lanes_fail_closed(self):
        everything = {name: {'result': 'success'} for name in GATE.REQUIRED}
        for needs in [{}, [], None, {'quality': {'result': 'success'}},
                      {**everything, 'unreviewed': {'result': 'success'}},
                      {**everything, 'test': 'success'},
                      {key: value for key, value in everything.items() if key != 'balance'}]:
            with self.subTest(needs=needs):
                self.assertTrue(GATE.failures(needs))


class WorkflowGate(unittest.TestCase):
    """The workflow file and the gate must name exactly the same lanes."""

    def setUp(self):
        self.jobs = GATE.workflow_jobs((WORKFLOWS/'ci.yml').read_text())

    def test_every_lane_is_aggregated_by_check(self):
        lanes = set(self.jobs) - {'check'}
        self.assertEqual(lanes, set(GATE.REQUIRED))
        self.assertEqual(GATE.aggregator_needs(self.jobs.get('check')), set(GATE.REQUIRED))

    def test_check_keeps_its_required_name_and_always_runs(self):
        body = self.jobs['check']
        self.assertRegex(body, r'(?m)^    if: \$\{\{ always\(\) \}\}$')
        self.assertNotRegex(body, r'(?m)^    name:')

    def test_parsers_fail_closed_on_unexpected_shapes(self):
        self.assertEqual(GATE.workflow_jobs('name: ci\non:\n  push:\n'), {})
        self.assertIsNone(GATE.aggregator_needs('    needs:\n      - quality\n'))
        self.assertIsNone(GATE.aggregator_needs(None))
        jobs = GATE.workflow_jobs('jobs:\n  a:\n    runs-on: x\n# note\n  check:\n    needs: [a]\nenv:\n  b:\n')
        self.assertEqual(set(jobs), {'a', 'check'})
        self.assertEqual(GATE.aggregator_needs(jobs['check']), {'a'})


class WorkflowPolicy(unittest.TestCase):
    """Owner rules: no masked failures, pinned actions, cancel only superseded PR runs."""

    def test_no_workflow_continues_on_error(self):
        for name, text in workflow_texts().items():
            with self.subTest(workflow=name):
                self.assertNotIn('continue-on-error', text)

    def test_every_action_is_pinned_to_a_commit_with_its_version(self):
        for name, text in workflow_texts().items():
            for line in text.splitlines():
                if re.match(r'^\s*(?:-\s+)?uses:', line):
                    with self.subTest(workflow=name, line=line.strip()):
                        self.assertRegex(line, PINNED)

    def test_toolchain_file_pins_the_workflow_compiler(self):
        # rust-toolchain.toml overrides the action's toolchain input on Linux.
        channel = tomllib.loads((ROOT/'rust-toolchain.toml').read_text())['toolchain']['channel']
        for name, text in workflow_texts().items():
            for value in re.findall(r'(?m)^\s*RUST_TOOLCHAIN:\s*(\S+)\s*$', text):
                with self.subTest(workflow=name):
                    self.assertEqual(value, channel)

    def test_only_pull_request_runs_are_ever_cancelled(self):
        texts = workflow_texts()
        for name in ['ci.yml', 'native-release.yml']:
            with self.subTest(declared_in=name):
                self.assertRegex(texts.get(name, ''), r'(?m)^\s*cancel-in-progress:')
        for name, text in texts.items():
            for value in re.findall(r'(?m)^\s*cancel-in-progress:\s*(.+?)\s*$', text):
                with self.subTest(workflow=name, value=value):
                    self.assertEqual(value, "${{ github.event_name == 'pull_request' }}")


if __name__ == '__main__':
    unittest.main()
