#!/usr/bin/env python3
"""Turn a successful main desktop build into its version's single tag and exact-byte publication.

Runs in native-release.yml after the Linux and Windows main builds. The tag is
created once from a commit still on main; the publication is a tag-ref run that
promotes this run's artifacts, so the published bytes are the tested bytes.
"""
import json
import os
import subprocess

from native_release_policy import client_version, release_relevant

WORKFLOW = 'native-release.yml'


class GitHub:
    def __init__(self, repository):
        self.repository = repository

    def api(self, path, *args):
        result = subprocess.run(['gh', 'api', f'repos/{self.repository}/{path}', *args],
                                text=True, capture_output=True)
        if result.returncode != 0:
            if 'HTTP 404' in result.stderr:
                return None
            if 'HTTP 422' in result.stderr:
                raise FileExistsError(result.stderr.strip())
            raise RuntimeError(result.stderr.strip())
        return json.loads(result.stdout) if result.stdout.strip() else {}

    def tag_target(self, tag):
        ref = self.api(f'git/ref/tags/{tag}')
        if ref is None:
            return None
        target = ref['object']
        if target['type'] == 'tag':
            target = self.api(f"git/tags/{target['sha']}")['object']
        return target['sha']

    def on_main(self, sha):
        compare = self.api(f'compare/main...{sha}')
        return compare is not None and compare['status'] in {'behind', 'identical'}

    def changed(self, base, head):
        compare = self.api(f'compare/{base}...{head}')
        if compare is None or compare['status'] not in {'ahead', 'identical'} or len(compare.get('files', [])) >= 300:
            return None
        return [f['filename'] for f in compare['files']] + [f['previous_filename'] for f in compare['files'] if f.get('previous_filename')]

    def create_tag(self, tag, sha):
        self.api('git/refs', '-X', 'POST', '-f', f'ref=refs/tags/{tag}', '-f', f'sha={sha}')

    def published(self, tag):
        release = self.api(f'releases/tags/{tag}')
        names = {asset['name'] for asset in (release or {}).get('assets', [])}
        return bool(release) and not release.get('draft') and {'tbg-native-release.json', 'native-provenance.jsonl'} <= names

    def dispatch(self, tag, run_id):
        # workflow_dispatch is the one event GITHUB_TOKEN may start (actions: write).
        self.api(f'actions/workflows/{WORKFLOW}/dispatches', '-X', 'POST', '-f', f'ref={tag}',
                 '-f', 'inputs[publish]=true', '-f', f'inputs[promotion_run]={run_id}')


def release(github, sha, version, run_id):
    tag = 'v' + version
    target = github.tag_target(tag)
    if target is None:
        if not github.on_main(sha):
            raise SystemExit(f'{sha} is no longer on main; refusing to name {tag} after it.')
        try:
            github.create_tag(tag, sha)
        except FileExistsError:
            target = github.tag_target(tag)
            if target != sha:
                raise SystemExit(f'Version contention: {tag} was just created for {target}; bump the client version.')
        github.dispatch(tag, run_id)
        return f'tagged {tag} at {sha} and dispatched its publication'
    if target == sha:
        if github.published(tag):
            return f'{tag} is already published from this source'
        github.dispatch(tag, run_id)
        return f'{tag} names this source but is not fully published; dispatched again'
    paths = github.changed(target, sha)
    if paths is not None and not release_relevant(paths):
        return f'{tag} already covers the shipped inputs of {sha}; nothing to publish'
    raise SystemExit(f'Version contention: {tag} already names {target} and shipped inputs changed since; '
                     'bump crates/tbg-client/Cargo.toml in a reviewed pull request.')


def main():
    github = GitHub(os.environ['GITHUB_REPOSITORY'])
    print(release(github, os.environ['SOURCE_SHA'], client_version(), os.environ['NATIVE_BUILD_RUN']))


if __name__ == '__main__':
    main()
