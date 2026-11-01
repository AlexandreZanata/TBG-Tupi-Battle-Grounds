#!/usr/bin/env python3
"""Publish changed public sites and their shared data/art without rebuilding the backend."""
import os
import subprocess
from pathlib import Path


def relevant(paths):
    exact = {'docs/patches/catalog.json', '.github/workflows/static-release.yml',
             'scripts/package-static-sites.py', 'scripts/static-release-changes.py'}
    prefixes = ('web/', 'crates/tbg-client/assets/branding/', 'crates/tbg-client/assets/fonts/',
                'crates/tbg-client/assets/arenas/', 'crates/tbg-client/assets/battle/portraits/',
                'crates/tbg-client/assets/i18n/', 'crates/tbg-core/src/content', 'crates/tbg-core/src/types')
    return any(path in exact or path.startswith(prefixes) for path in paths)


def main():
    paths = subprocess.check_output(['git', 'diff', '--name-only', 'HEAD^', 'HEAD'], text=True).splitlines()
    value = str(relevant(paths)).lower()
    with Path(os.environ['GITHUB_OUTPUT']).open('a') as output:
        output.write('deployable=' + value + '\n')
    print('Public static site input changed: ' + value)


if __name__ == '__main__':
    main()
