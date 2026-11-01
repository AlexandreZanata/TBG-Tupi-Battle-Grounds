#!/usr/bin/env python3
"""Select game-browser releases without publishing admin, landing or mobile frontends."""
import os
import subprocess
from pathlib import Path


def relevant(paths):
    exact = {'Cargo.toml', 'Cargo.lock', '.github/workflows/web-release.yml',
             'scripts/package-web.py', 'scripts/web-release-changes.py', 'scripts/install-web-build-tools.sh'}
    return any(p in exact or p.startswith(('crates/tbg-client/', 'crates/tbg-core/')) for p in paths)


def main():
    paths = subprocess.check_output(['git', 'diff', '--name-only', 'HEAD^', 'HEAD'], text=True).splitlines()
    value = str(relevant(paths)).lower()
    with Path(os.environ['GITHUB_OUTPUT']).open('a') as stream:
        stream.write('deployable=' + value + '\n')
    print('Game browser source changed: ' + value)


if __name__ == '__main__':
    main()
