#!/usr/bin/env python3
"""Keep frontend-only and documentation merges from restarting the game backend."""
import os
import subprocess
from pathlib import Path


def relevant(paths):
    exact = {'Cargo.toml', 'Cargo.lock', 'docs/patches/catalog.json',
             '.github/workflows/backend-release.yml', 'scripts/package-backend.py',
             'scripts/backend-release-changes.py'}
    return any(p in exact or p.startswith(('crates/tbg-core/', 'crates/tbg-server/')) for p in paths)


def main():
    paths = subprocess.check_output(['git', 'diff', '--name-only', 'HEAD^', 'HEAD'], text=True).splitlines()
    value = str(relevant(paths)).lower()
    if output := os.environ.get('GITHUB_OUTPUT'):
        with Path(output).open('a') as stream:
            stream.write('deployable='+value+'\n')
    print('Backend source changed: '+value)


if __name__ == '__main__':
    main()
