"""Decide family relevance against the last published candidate, not only against HEAD^.

An outdated release run exits when main has advanced, trusting the newer run;
comparing only HEAD^..HEAD would then lose a relevant change followed by an
unrelated merge. The compare API avoids fetching the large history.
"""
import json
import os
import re
import subprocess
from pathlib import Path

# The compare API stops listing files here; a longer change is treated as relevant.
MAX_FILES = 300


def gh(path):
    return json.loads(subprocess.check_output(['gh', 'api', path], text=True))


def last_candidate(repository, prefix):
    releases = gh(f'repos/{repository}/releases?per_page=100')
    tags = [r for r in releases if not r.get('draft') and re.fullmatch(prefix + r'-[0-9a-f]{40}', r.get('tag_name', ''))]
    newest = max(tags, key=lambda r: r['published_at'], default=None)
    return newest['tag_name'].split('-', 1)[1] if newest else None


def changed_paths(repository, prefix, head):
    """Paths changed since the last published candidate, or None for 'treat as changed'."""
    base = last_candidate(repository, prefix)
    if base is None:
        return None
    if base == head:
        return []
    compare = gh(f'repos/{repository}/compare/{base}...{head}')
    files = compare.get('files', [])
    if compare.get('status') not in {'ahead', 'identical'} or len(files) >= MAX_FILES:
        return None
    return [f['filename'] for f in files] + [f['previous_filename'] for f in files if f.get('previous_filename')]


def decide(prefix, relevant, label):
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    paths = changed_paths(os.environ['GITHUB_REPOSITORY'], prefix, head)
    value = 'true' if paths is None or relevant(paths) else 'false'
    if output := os.environ.get('GITHUB_OUTPUT'):
        with Path(output).open('a') as stream:
            stream.write('deployable=' + value + '\n')
    basis = 'no comparable published candidate' if paths is None else f'{len(paths)} paths since the last candidate'
    print(f'{label} changed: {value} ({basis})')
