#!/usr/bin/env python3
"""Scope automatic Android builds and promote their exact main-source bytes through immutable tags."""
import argparse
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import re
import subprocess

from android_release import REPOSITORY, WORKFLOW, configuration, relevant, version


def api(path, *args, optional=False):
    result = subprocess.run(['gh', 'api', 'repos/'+REPOSITORY+'/'+path, *args], text=True, capture_output=True)
    if result.returncode:
        if optional and 'HTTP 404' in result.stderr:
            return None
        raise RuntimeError(result.stderr.strip())
    return json.loads(result.stdout) if result.stdout.strip() else {}


def suite_passed(runs, source):
    return any(r.get('head_sha') == source and r.get('conclusion') == 'success'
               and r.get('event') == 'push' and r.get('head_branch') == 'main'
               and r.get('path') == '.github/workflows/ci.yml'
               and r.get('head_repository', {}).get('full_name') == REPOSITORY for r in runs)


def require_suite(source):
    runs = api(f'actions/workflows/ci.yml/runs?head_sha={source}&event=push&branch=main&per_page=20')
    if not suite_passed(runs.get('workflow_runs', []), source):
        raise ValueError('The exact Android source must pass the existing CI on main')


def output(values):
    if path := os.environ.get('GITHUB_OUTPUT'):
        with Path(path).open('a') as stream:
            stream.writelines(f'{key}={value}\n' for key, value in values.items())


def scope(source):
    if not re.fullmatch(r'[0-9a-f]{40}', source):
        raise ValueError('An exact main source SHA is required')
    if api('git/ref/heads/main')['object']['sha'] != source or os.environ.get('GITHUB_SHA', source) != source:
        output({'build': 'false'})
        print('Main advanced; its newer CI owns publication.')
        return
    require_suite(source)
    value, code = version(), configuration()['version_code']
    tag = 'android-v' + value
    existing = api('git/ref/tags/'+tag, optional=True)
    if existing:
        target = existing['object']
        if target['type'] == 'tag':
            target = api('git/tags/'+target['sha'])['object']
        changed = api('compare/'+target['sha']+'...'+source)
        paths = [p['filename'] for p in changed.get('files', [])]
        paths += [p['previous_filename'] for p in changed.get('files', []) if p.get('previous_filename')]
        if changed['status'] not in {'ahead', 'identical'} or len(changed.get('files', [])) >= 300 or relevant(paths):
            raise ValueError('Android inputs changed under an existing tag; increase version_code')
        if api('releases/tags/'+tag, optional=True) is None:
            raise ValueError('This Android tag is incomplete; promote its original build run, never rebuild/replace its bytes')
        output({'build': 'false'})
        print('The existing Android release covers these unchanged APK inputs.')
        return
    published = api('releases?per_page=100')
    codes = [int(m[1]) for r in published if not r.get('draft')
             and (m := re.fullmatch(r'android-v.+-android\.([0-9]+)', r.get('tag_name', '')))]
    if codes and code <= max(codes):
        raise ValueError('Android versionCode must increase beyond every public beta')
    output({'build': 'true', 'source': source, 'tag': tag, 'code': str(code)})


def promote(source, run_id):
    if not re.fullmatch(r'[1-9][0-9]{0,19}', run_id):
        raise ValueError('A completed exact-source Android run is required')
    run = api('actions/runs/'+run_id)
    started = datetime.fromisoformat(run['run_started_at'].replace('Z', '+00:00'))
    if not (run['head_sha'] == source and run['conclusion'] == 'success'
            and run['path'] == WORKFLOW and run['event'] == 'workflow_run' and run['head_branch'] == 'main'
            and run['repository']['full_name'] == REPOSITORY and run['head_repository']['full_name'] == REPOSITORY
            and datetime.now(timezone.utc) - started <= timedelta(days=13)):
        raise ValueError('Android promotion refuses a stale, foreign, PR or different-source build')
    require_suite(source)
    tag = 'android-v'+version()
    ref = api('git/ref/tags/'+tag)
    if ref['object']['sha'] != source or ref['object']['type'] != 'commit':
        raise ValueError('Android tag must name the exact tested commit directly')
    print('Verified exact-source Android promotion:', run_id)


def release_main(source):
    if api('compare/main...'+source)['status'] not in {'behind', 'identical'}:
        raise ValueError('Only source retained on main may name an Android beta')
    tag = 'android-v'+version()
    current = api('git/ref/tags/'+tag, optional=True)
    if current is None:
        api('git/refs', '-X', 'POST', '-f', 'ref=refs/tags/'+tag, '-f', 'sha='+source)
    elif current['object']['sha'] != source or current['object']['type'] != 'commit':
        raise ValueError('Android tag contention; never replace a tag')
    api('actions/workflows/android-release.yml/dispatches', '-X', 'POST', '-f', 'ref='+tag,
        '-f', 'inputs[publish]=true', '-f', 'inputs[promotion_run]='+os.environ['GITHUB_RUN_ID'])
    print('Dispatched immutable Android beta publication:', tag)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['scope', 'release-main', 'promote'])
    args = parser.parse_args()
    source = os.environ.get('SOURCE_SHA') or os.environ['GITHUB_SHA']
    if not re.fullmatch(r'[0-9a-f]{40}', source):
        raise SystemExit('Exact Android source SHA required')
    if args.command == 'scope':
        scope(source)
    elif args.command == 'promote':
        promote(source, os.environ['ANDROID_BUILD_RUN'])
    else:
        release_main(source)


if __name__ == '__main__':
    main()
