#!/usr/bin/env python3
"""Focused checks for additive static publication, with no private keys or service changes."""
import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('native_channel',Path(__file__).with_name('publish-native-channel.py'))
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)

class Publication(unittest.TestCase):
    def test_targets_precede_metadata_and_timestamp_is_last(self):
        plan=module.deployment_plan(Path('/tmp/public-repo'),'tupibg-vps','/srv/tbg/updates')
        self.assertIn('/tmp/public-repo/targets/',plan[1])
        self.assertIn('--exclude=timestamp.json',plan[2])
        self.assertIn('/tmp/public-repo/metadata/timestamp.json',plan[-1])
        text=str(plan)
        for forbidden in ['--delete','.pk8','restart','reload','systemctl']:
            self.assertNotIn(forbidden,text)
    def test_shell_fragments_and_parent_paths_are_rejected(self):
        for host,remote in [('host;false','/srv/updates'),('host','/srv/../keys'),('host','/srv/updates;false')]:
            with self.assertRaises(ValueError):module.deployment_plan(Path('/tmp/public'),host,remote)

if __name__=='__main__':unittest.main()
