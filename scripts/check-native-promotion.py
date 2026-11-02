#!/usr/bin/env python3
"""Only reuse successful exact-source desktop artifacts from this repository's build workflow."""
import json
import os
import re
import subprocess
from datetime import datetime, timedelta, timezone
from pathlib import Path

BUILD = '.github/workflows/native-release.yml'
SUITE = '.github/workflows/ci.yml'
# Build artifacts are retained 14 days; leave a day of margin for the publication.
FRESH = timedelta(days=13)


def problem(run, repository, sha, now):
    """Why this run's artifacts may not become a release, or None."""
    if not (run['head_sha'] == sha and run['conclusion'] == 'success' and run['path'] == BUILD
            and run['repository']['full_name'] == repository and run['head_repository']['full_name'] == repository):
        return 'Native promotion source/run/workflow mismatch'
    # A verified main push build or an explicit exact-source dispatch; never a
    # pull request merge ref, another branch's push or a fork.
    if not (run['event'] == 'workflow_dispatch' or (run['event'] == 'push' and run['head_branch'] == 'main')):
        return f"Native promotion refuses {run['event']} runs on {run['head_branch']}"
    started = datetime.fromisoformat(run['run_started_at'].replace('Z', '+00:00'))
    if now - started > FRESH:
        return 'Native build artifacts are about to expire; rebuild instead of promoting'
    return None


def suite_passed(runs, repository, sha):
    """Exact evidence that the required suite already passed on this source as a main push."""
    return any(r['head_sha'] == sha and r['conclusion'] == 'success' and r['event'] == 'push'
               and r['head_branch'] == 'main' and r['path'] == SUITE
               and r['head_repository']['full_name'] == repository for r in runs)


def gh(path):
    return json.loads(subprocess.check_output(['gh', 'api', path], text=True))


def main():
    run_id = os.environ['NATIVE_BUILD_RUN']
    if not re.fullmatch(r'[1-9][0-9]{0,19}', run_id):
        raise SystemExit('A numeric completed native build run is required')
    repository, sha = os.environ['GITHUB_REPOSITORY'], os.environ['GITHUB_SHA']
    error = problem(gh(f'repos/{repository}/actions/runs/{run_id}'), repository, sha, datetime.now(timezone.utc))
    if error:
        raise SystemExit(error)
    runs = gh(f'repos/{repository}/actions/workflows/ci.yml/runs?head_sha={sha}&event=push&branch=main&per_page=20')
    proven = suite_passed(runs.get('workflow_runs', []), repository, sha)
    if output := os.environ.get('GITHUB_OUTPUT'):
        with Path(output).open('a') as stream:
            stream.write(f'ci_proven={str(proven).lower()}\n')
    print('Verified completed exact-source native build:', run_id, '(suite already passed on main)' if proven else '')


if __name__ == '__main__':
    main()
