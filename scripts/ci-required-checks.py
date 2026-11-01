#!/usr/bin/env python3
"""Keep the existing required check fail-closed across all parallel CI lanes."""
import json
import os
import re
import sys

REQUIRED = frozenset({'quality', 'clippy', 'test', 'balance', 'wasm', 'desktop-ui'})


def failures(needs):
    if not isinstance(needs, dict) or set(needs) != REQUIRED:
        return ['Required CI lane set is missing or unexpected']
    failed = []
    for name in sorted(REQUIRED):
        job = needs[name]
        state = job.get('result') if isinstance(job, dict) else None
        if state != 'success':
            failed.append(f'{name}: {state or "missing result"}')
    return failed


def workflow_jobs(text):
    """Job id -> raw body under the top-level `jobs:` key (no YAML dependency)."""
    jobs, current, in_jobs = {}, None, False
    for line in text.splitlines():
        if re.match(r'^\s*#', line):
            continue
        if re.match(r'^\S', line):
            in_jobs, current = line.startswith('jobs:'), None
            continue
        heading = re.match(r'^  ([A-Za-z0-9_-]+):\s*$', line)
        if in_jobs and heading:
            current = heading.group(1)
            jobs[current] = []
        elif in_jobs and current:
            jobs[current].append(line)
    return {name: '\n'.join(body) for name, body in jobs.items()}


def aggregator_needs(body):
    """The `needs: [...]` list of one job, or None when it is not written inline."""
    found = re.search(r'^    needs:\s*\[([^\]]*)\]\s*$', body or '', re.M)
    return {name.strip() for name in found.group(1).split(',') if name.strip()} if found else None


def main():
    try:
        needs = json.loads(os.environ.get('NEEDS_JSON', ''))
    except (ValueError, TypeError):
        print('Invalid required CI results', file=sys.stderr)
        return 1
    rejected = failures(needs)
    if rejected:
        print('\n'.join(rejected), file=sys.stderr)
        return 1
    print('All required quality, Clippy, test, balance-lab, WASM and desktop UI checks passed.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
