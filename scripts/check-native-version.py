#!/usr/bin/env python3
"""A pull request that changes shipped desktop inputs must carry a new, documented client version."""
import argparse
import subprocess

from native_release_policy import client_version, notes_version, release_relevant


def problem(paths, version, notes, published):
    if not release_relevant(paths):
        return None
    if published:
        return (f'Desktop inputs changed but v{version} is already published. Bump the version in '
                'crates/tbg-client/Cargo.toml (and Cargo.lock) and describe it in deploy/native/RELEASE-NOTES.md.')
    if notes != version:
        return f'deploy/native/RELEASE-NOTES.md describes {notes}; write the notes for {version}.'
    return None


def published(tag):
    # A public tag lookup needs no token and no fetched history.
    return bool(subprocess.check_output(['git', 'ls-remote', '--tags', 'origin', 'refs/tags/' + tag], text=True).strip())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', required=True)
    args = parser.parse_args()
    paths = subprocess.check_output(['git', 'diff', '--name-only', args.base, 'HEAD'], text=True).splitlines()
    version = client_version()
    error = problem(paths, version, notes_version(), published('v' + version))
    if error:
        raise SystemExit(error)
    print(f'Native version policy satisfied for {version}.')


if __name__ == '__main__':
    main()
