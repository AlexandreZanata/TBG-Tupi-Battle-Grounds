#!/usr/bin/env python3
"""Verify and promote browser releases independently under the shared deployment lock."""
import argparse
import fcntl
import json
import re
import tempfile
from pathlib import Path

import web_activate
import web_policy
from activate import STATE, write_json
from policy import REPOSITORY, require
from pull import api, candidate, fetch, verify


def run(config, apply, recheck):
    if apply:
        web_activate.recover()
    release = candidate('web')
    if release is None:
        print('Waiting for the first CI-approved browser candidate.')
        return
    source = release['tag_name'][4:]
    checked = STATE/'web-checked.json'
    if not recheck and checked.exists() and json.loads(checked.read_text()).get('source_commit') == source:
        print('Browser candidate already handled; inspect web-checked.json.')
        return
    current = web_activate.RECEIPT
    floor = json.loads(current.read_text())['source_commit'] if current.exists() else config['minimum_source']
    require(re.fullmatch(r'[0-9a-f]{40}', floor) is not None, 'verified browser source floor required')
    require(api('/compare/' + floor + '...' + source)['status'] in {'ahead', 'identical'}, 'browser source regression')
    if current.exists() and floor == source:
        print('Browser candidate is already current.')
        return
    with tempfile.TemporaryDirectory(prefix='web-candidate-', dir=STATE) as temporary:
        root = Path(temporary)
        package, bundle = root/'web.tar.gz', root/'provenance.jsonl'
        prefix = 'https://github.com/' + REPOSITORY + '/releases/download/' + release['tag_name'] + '/'
        for name, path, limit in [('tbg-web-' + source + '.tar.gz', package, web_policy.MAX_ARCHIVE),
                                  ('provenance.jsonl', bundle, 8 * 1024 * 1024)]:
            asset = next((a for a in release['assets'] if a['name'] == name), None)
            require(asset is not None and asset['browser_download_url'] == prefix + name, 'unexpected web asset URL')
            path.write_bytes(fetch(prefix + name, limit))
        verify(package, bundle, source, workflow=web_policy.WORKFLOW)
        manifest = web_policy.unpack(package, root/'verified', source)
        try:
            live = web_activate.status()
            require(live['contract'] == manifest['contract'] and not live.get('shutdown'), 'incompatible browser candidate')
            if apply:
                web_activate.activate(root/'verified', manifest, config.get('public_web', 'https://app.tupibg.com'))
                write_json(checked, {'source_commit': source, 'outcome': 'healthy'})
        except Exception as error:
            if apply:
                write_json(checked, {'source_commit': source, 'outcome': 'operator_required', 'reason': str(error)[:500]})
            raise
        print(json.dumps({'verified_web_source': source, 'contract': manifest['contract'], 'activated': apply}))


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
