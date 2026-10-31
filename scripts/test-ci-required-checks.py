#!/usr/bin/env python3
"""Prevent a failed, skipped, cancelled or absent CI lane from passing the merge gate."""
import importlib.util
from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('ci_gate', Path(__file__).with_name('ci-required-checks.py'))
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


class RequiredChecks(unittest.TestCase):
    def test_all_three_successful_lanes_are_required(self):
        self.assertEqual(GATE.failures({name: {'result': 'success'} for name in GATE.REQUIRED}), [])

    def test_failure_cancellation_skips_and_missing_results_fail_closed(self):
        for name in GATE.REQUIRED:
            for state in ['failure', 'cancelled', 'skipped', None, '']:
                with self.subTest(name=name, state=state):
                    needs = {job: {'result': 'success'} for job in GATE.REQUIRED}
                    needs[name] = {'result': state}
                    self.assertTrue(GATE.failures(needs))

    def test_missing_unexpected_and_malformed_lanes_fail_closed(self):
        for needs in [{}, [], None, {'quality': {'result': 'success'}},
                      {**{name: {'result': 'success'} for name in GATE.REQUIRED}, 'unreviewed': {'result': 'success'}},
                      {**{name: {'result': 'success'} for name in GATE.REQUIRED}, 'native': 'success'}]:
            with self.subTest(needs=needs):
                self.assertTrue(GATE.failures(needs))


if __name__ == '__main__':
    unittest.main()
