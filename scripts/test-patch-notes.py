#!/usr/bin/env python3
"""Regression coverage for the mandatory gameplay patch-note release gate."""
import copy
import sys
sys.dont_write_bytecode = True
import importlib.util
import json
import re
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('patch_gate', Path(__file__).with_name('check-patch-notes.py'))
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


class PatchGateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='tbg-patch-gate-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        source = Path(__file__).resolve().parents[1]
        for path in ['docs/patches/catalog.json', 'crates/tbg-core/ruleset-sources.txt', 'crates/tbg-core/src/ruleset_version.rs']:
            target = self.root/path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((source/path).read_bytes())
        self.catalog = json.loads((self.root/'docs/patches/catalog.json').read_text())
        for note in self.catalog['notes']:
            for path in [note['document'], note['evidence']['report']]:
                target = self.root/path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text('Release evidence fixture\n')
        self.root_patch = patch.object(GATE, 'ROOT', self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)
        self.fingerprint = self.catalog['notes'][0]['ruleset_hash']
        self.version = int(re.search(r'RULESET_VERSION: u32 = (\d+)', (self.root/'crates/tbg-core/src/ruleset_version.rs').read_text())[1])

    def run_gate(self, changes='', previous=None):
        previous = self.version-1 if previous is None else previous
        (self.root/'docs/patches/catalog.json').write_text(json.dumps(self.catalog))
        base = subprocess.CompletedProcess([], 0, f'pub const RULESET_VERSION: u32 = {previous};', '')
        with patch.object(GATE, 'source_hash', return_value=(self.fingerprint, ['content.rs'])), \
             patch.object(GATE, 'git', return_value=changes), \
             patch.object(GATE.subprocess, 'run', return_value=base), \
             patch('sys.argv', ['check-patch-notes.py', '--base', 'test-base']):
            GATE.main()

    def test_valid_gameplay_revision_has_a_matching_note(self):
        self.run_gate('crates/tbg-core/src/content.rs')

    def test_gameplay_without_a_version_bump_is_rejected(self):
        with self.assertRaisesRegex(SystemExit, 'new RulesetVersion'):
            self.run_gate('crates/tbg-core/src/content.rs', previous=self.version)

    def test_missing_current_note_is_rejected(self):
        self.fingerprint = '0123456789abcdef'
        with self.assertRaisesRegex(SystemExit, 'No patch note'):
            self.run_gate()

    def test_numeric_balance_without_accepted_evidence_is_rejected(self):
        self.catalog['notes'][0]['kind'] = 'balance'
        self.catalog['notes'][0]['evidence']['accepted'] = False
        with self.assertRaisesRegex(SystemExit, 'held-out evidence'):
            self.run_gate()

    def test_duplicate_ids_and_private_document_paths_are_rejected(self):
        original = copy.deepcopy(self.catalog)
        self.catalog['notes'][1]['id'] = self.catalog['notes'][0]['id']
        with self.assertRaisesRegex(SystemExit, 'Duplicate'):
            self.run_gate()
        self.catalog = original
        self.catalog['notes'][0]['document'] = 'crates/tbg-core/src/ruleset_version.rs'
        with self.assertRaisesRegex(SystemExit, 'Missing patch document'):
            self.run_gate()


if __name__ == '__main__':
    unittest.main()
