#!/usr/bin/env python3
"""Publish a native tag's verified files once: draft first, never replace, reconcile a partial retry."""
import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            value.update(chunk)
    return value.hexdigest()


def plan(existing, local):
    """Files still to upload; a foreign or different published asset is never replaced."""
    for name, value in existing.items():
        if name not in local:
            raise SystemExit(f'The release holds {name}, which this source did not produce; refusing to publish.')
        if value != 'sha256:' + local[name]:
            raise SystemExit(f'Published {name} differs from the promoted bytes; assets are never replaced.')
    return sorted(set(local) - set(existing))


def gh(*args):
    return subprocess.run(['gh', *args], check=True, text=True, capture_output=True).stdout


def find(repository, tag):
    # Drafts are invisible to /releases/tags/<tag>; list them with the write token.
    releases = json.loads(gh('api', f'repos/{repository}/releases?per_page=100'))
    return next((release for release in releases if release['tag_name'] == tag), None)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifacts', type=Path, required=True)
    parser.add_argument('--tag', required=True)
    parser.add_argument('--title', required=True)
    parser.add_argument('--notes', type=Path, required=True)
    args = parser.parse_args()
    files = {path.name: path for path in sorted(args.artifacts.iterdir()) if path.is_file()}
    local = {name: digest(path) for name, path in files.items()}
    repository = os.environ['GITHUB_REPOSITORY']
    release = find(repository, args.tag)
    if release is None:
        gh('release', 'create', args.tag, '--draft', '--prerelease', '--verify-tag',
           '--title', args.title, '--notes-file', str(args.notes))
        existing, draft = {}, True
    else:
        existing, draft = {asset['name']: asset.get('digest') for asset in release['assets']}, release['draft']
    upload = plan(existing, local)
    if upload:
        gh('release', 'upload', args.tag, *(str(files[name]) for name in upload))
    if draft:
        # Public only once every file is present, so no visitor sees a partial set.
        gh('release', 'edit', args.tag, '--draft=false')
    print(f'{args.tag}: uploaded {len(upload)} of {len(local)} files; '
          f"{'published' if draft else 'already public'}.")


if __name__ == '__main__':
    main()
