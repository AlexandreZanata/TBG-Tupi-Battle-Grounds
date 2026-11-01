#!/usr/bin/env python3
"""Only reuse successful exact-source desktop artifacts from this repository's build workflow."""
import json
import os
import re
import subprocess

run_id = os.environ['NATIVE_BUILD_RUN']
if not re.fullmatch(r'[1-9][0-9]{0,19}', run_id):
    raise SystemExit('A numeric completed native build run is required')
repository = os.environ['GITHUB_REPOSITORY']
run = json.loads(subprocess.check_output(['gh', 'api', f'repos/{repository}/actions/runs/{run_id}'], text=True))
if not (run['head_sha'] == os.environ['GITHUB_SHA'] and run['conclusion'] == 'success'
        and run['event'] == 'workflow_dispatch' and run['path'] == '.github/workflows/native-release.yml'
        and run['repository']['full_name'] == repository and run['head_repository']['full_name'] == repository):
    raise SystemExit('Native promotion source/run/workflow mismatch')
print('Verified completed exact-source native build:', run_id)
