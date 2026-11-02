#!/usr/bin/env python3
"""Verify and promote static sites independently under the shared deployment lock."""
import argparse
import json
import re
import subprocess
import tempfile
from pathlib import Path
from urllib.error import URLError

import static_activate
import static_policy
from activate import STATE, status, write_json
from control import Deferred, deployment_lock, run_controller
from github import api, candidate, fetch
from policy import REPOSITORY, require
from pull import verify

WAITING = STATE/'static-waiting.json'


def landing_contract(verified):
    """The server contract the landing advertises downloads for."""
    config = json.loads((Path(verified)/'landing/config/release.json').read_text())
    contract = config.get('server', {}).get('contract')
    require(isinstance(contract, str) and contract, 'landing release config names no server contract')
    return contract


def live_contract():
    try:
        live = status()
    except (URLError, OSError) as error:
        raise Deferred(f'game server unreachable for the landing contract check: {error}') from None
    return live['contract']


def run(config, apply, recheck, adopt=False):
    if apply:
        static_activate.recover()
    release = candidate('static')
    if release is None:
        print('Waiting for the first CI-approved static candidate.')
        return
    source = release['tag_name'][7:]
    checked = STATE/'static-checked.json'
    if not recheck and checked.exists() and json.loads(checked.read_text()).get('source_commit') == source:
        print('Static candidate already handled; inspect static-checked.json.')
        return
    live = live_contract()
    waiting = json.loads(WAITING.read_text()) if WAITING.exists() else None
    if waiting and waiting.get('source_commit') == source and waiting.get('contract') != live:
        raise Deferred(f"static {source} waits for server contract {waiting.get('contract')}")
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
        try:
            verify(package, bundle, source, workflow=static_policy.WORKFLOW)
            manifest = static_policy.unpack(package, root/'verified', source)
            advertised = landing_contract(root/'verified')
        except (subprocess.CalledProcessError, ValueError, OSError) as error:
            if apply:
                write_json(checked, {'source_commit': source, 'outcome': 'rejected', 'reason': str(error)[:500]})
            raise
        if advertised != live:
            # Downloads must never be advertised for a server that is not live yet.
            if apply:
                write_json(WAITING, {'source_commit': source, 'contract': advertised})
            raise Deferred(f'static {source} advertises server contract {advertised}, live is {live}')
        if apply:
            WAITING.unlink(missing_ok=True)
            try:
                static_activate.activate(root/'verified', manifest, adopt)
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
    parser.add_argument('--adopt', action='store_true',
                        help='replace site pointers that changed outside this controller')
    args = parser.parse_args()

    def body():
        with deployment_lock():
            run(json.loads(args.config.read_text()), args.apply, args.recheck, args.adopt)
    run_controller(body)


if __name__ == '__main__':
    main()
