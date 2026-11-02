#!/usr/bin/env python3
"""Verify and promote browser releases independently under the shared deployment lock."""
import argparse
import json
import re
import subprocess
import tempfile
from pathlib import Path
from urllib.error import URLError

import web_activate
import web_policy
from activate import STATE, write_json
from control import Deferred, deployment_lock, run_controller
from github import api, candidate, fetch
from policy import REPOSITORY, require
from pull import verify

WAITING = STATE/'web-waiting.json'


def live_server():
    try:
        live = web_activate.status()
    except (URLError, OSError) as error:
        raise Deferred(f'game server unreachable for the browser contract check: {error}') from None
    if live.get('shutdown'):
        raise Deferred('game server is draining; browser promotion waits')
    return live


def run(config, apply, recheck, adopt=False):
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
    live = live_server()
    waiting = json.loads(WAITING.read_text()) if WAITING.exists() else None
    if waiting and waiting.get('source_commit') == source and waiting.get('contract') != live['contract']:
        raise Deferred(f"browser {source} waits for server contract {waiting.get('contract')}")
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
        try:
            verify(package, bundle, source, workflow=web_policy.WORKFLOW)
            manifest = web_policy.unpack(package, root/'verified', source)
        except (subprocess.CalledProcessError, ValueError) as error:
            if apply:
                write_json(checked, {'source_commit': source, 'outcome': 'rejected', 'reason': str(error)[:500]})
            raise
        if manifest['contract'] != live['contract']:
            # A coordinated release brings the server first; then this applies on its own.
            if apply:
                write_json(WAITING, {'source_commit': source, 'contract': manifest['contract']})
            raise Deferred(f"browser {source} needs server contract {manifest['contract']}, live is {live['contract']}")
        if apply:
            WAITING.unlink(missing_ok=True)
        try:
            if apply:
                web_activate.activate(root/'verified', manifest, config.get('public_web', 'https://app.tupibg.com'), adopt)
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
    parser.add_argument('--adopt', action='store_true',
                        help='replace an entry pointer that changed outside this controller')
    args = parser.parse_args()

    def body():
        with deployment_lock():
            run(json.loads(args.config.read_text()), args.apply, args.recheck, args.adopt)
    run_controller(body)


if __name__ == '__main__':
    main()
