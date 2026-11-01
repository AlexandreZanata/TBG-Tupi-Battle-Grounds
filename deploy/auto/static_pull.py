#!/usr/bin/env python3
"""Verify and promote static sites independently under the shared deployment lock."""
import argparse
import fcntl
import json
import re
import tempfile
from pathlib import Path

import static_activate
import static_policy
from activate import STATE, write_json
from policy import REPOSITORY, require
from pull import api, candidate, fetch, verify


def run(config, apply, recheck):
    if apply:
        static_activate.recover()
    release = candidate('static')
    if release is None:
        print('Waiting for the first CI-approved static candidate.')
        return
    source = release['tag_name'][7:]
    checked = STATE/'static-checked.json'
    if not recheck and checked.exists() and json.loads(checked.read_text()).get('source_commit') == source:
        print('Static candidate already handled; inspect web-checked.json.')
        return
    current = static_activate.RECEIPT
    floor = json.loads(current.read_text())['source_commit'] if current.exists() else config['minimum_source']
    require(re.fullmatch(r'[0-9a-f]{40}', floor) is not None, 'verified static source floor required')
    require(api('/compare/' + floor + '...' + source)['status'] in {'ahead', 'identical'}, 'static source regression')
    if current.exists() and floor == source:
        print('Static candidate is already current.')
        return
    with tempfile.TemporaryDirectory(prefix='static-candidate-', dir=STATE) as temporary:
        root = Path(temporary)
        package, bundle = root/'web.tar.gz', root/'provenance.jsonl'
        prefix = 'https://github.com/' + REPOSITORY + '/releases/download/' + release['tag_name'] + '/'
        for name, path, limit in [('tbg-static-' + source + '.tar.gz', package, static_policy.MAX_ARCHIVE),
                                  ('provenance.jsonl', bundle, 8 * 1024 * 1024)]:
            asset = next((a for a in release['assets'] if a['name'] == name), None)
            require(asset is not None and asset['browser_download_url'] == prefix + name, 'unexpected static asset URL')
            path.write_bytes(fetch(prefix + name, limit))
        verify(package, bundle, source, workflow=static_policy.WORKFLOW)
        manifest = static_policy.unpack(package, root/'verified', source)
        if apply:
            try:
                static_activate.activate(root/'verified', manifest)
            except Exception as error:
                write_json(checked, {'source_commit': source, 'outcome': 'operator_required', 'reason': str(error)[:500]})
                raise
            write_json(checked, {'source_commit': source, 'outcome': 'healthy'})
        print(json.dumps({'verified_static_source': source, 'sites': sorted(manifest['sites']), 'activated': apply}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=Path, default=Path('/etc/tbg/auto-deploy.json'))
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--recheck', action='store_true')
    args = parser.parse_args()
    STATE.mkdir(mode=0o700, exist_ok=True)
    with (STATE/'lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        run(json.loads(args.config.read_text()), args.apply, args.recheck)


if __name__ == '__main__':
    main()
