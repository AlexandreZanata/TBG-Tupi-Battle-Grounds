#!/usr/bin/env python3
"""Keep the existing required check fail-closed across all parallel CI lanes."""
import json
import os
import sys

REQUIRED = frozenset({'quality', 'native', 'wasm'})


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
    print('All required quality, native test and WASM checks passed.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
