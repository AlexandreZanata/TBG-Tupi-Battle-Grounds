#!/usr/bin/env python3
"""Refresh public native download links from verified GitHub releases; never install executables."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import re
import tempfile

from activate import STATE, status, sync_directory, write_json
from native_policy import WORKFLOW, catalog, version_key
from policy import REPOSITORY, require
from pull import REPOSITORY_ID, api, fetch, verify

ROOT = Path('/srv/tbg/downloads')
RECEIPT = STATE/'native-current.json'
CHECKED = STATE/'native-checked.json'


def publish(value):
    require(not ROOT.is_symlink(), 'native catalog root requires explicit adoption')
    ROOT.mkdir(mode=0o755, parents=True, exist_ok=True)
    ROOT.chmod(0o755)
    encoded = (json.dumps(value, sort_keys=True) + '\n').encode()
    require(len(encoded) <= 16384, 'public native catalog exceeds its budget')
    temporary = ROOT/'current.next'
    with temporary.open('wb') as stream:
        stream.write(encoded)
        stream.flush()
        os.fsync(stream.fileno())
    temporary.chmod(0o644)
    os.replace(temporary, ROOT/'current.json')
    sync_directory(ROOT)


def pending(live):
    protocol = int(re.match(r'w([0-9]+)-', live['contract'])[1])
    publish({'schema': 1, 'server': {'protocol': protocol, 'contract': live['contract']},
             'linux': {'state': 'pending'}, 'windows': {'state': 'pending'}})


def synchronize(live):
    current = ROOT/'current.json'
    if current.exists() and json.loads(current.read_text()).get('server', {}).get('contract') == live['contract']:
        return
    pending(live)


def run(config):
    live = status()
    synchronize(live)
    require(api('')['id'] == REPOSITORY_ID, 'repository identity changed')
    releases = api('/releases?per_page=30')
    candidates = [r for r in releases if not r.get('draft') and re.fullmatch(r'v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?', r.get('tag_name', ''))
                  and any(a['name'] == 'tbg-native-release.json' for a in r.get('assets', []))]
    if not candidates:
        print('Waiting for the first complete verified native release.')
        return
    release = max(candidates, key=lambda r: (version_key(r['tag_name'][1:]), r['published_at']))
    old = json.loads(RECEIPT.read_text()) if RECEIPT.exists() else None
    if old and old['tag'] == release['tag_name']:
        try:
            publish(catalog(old, release, live))
        except ValueError:
            # Revoked/changed assets must not leave a previously enabled broken link.
            pending(live)
            raise
        print('Verified native catalog remains current.')
        return
    if old:
        require(version_key(release['tag_name'][1:]) > version_key(old['version']), 'native version rollback')
    with tempfile.TemporaryDirectory(prefix='native-catalog-', dir=STATE) as temporary:
        folder = Path(temporary)
        meta, bundle = folder/'native.json', folder/'provenance.jsonl'
        prefix = f'https://github.com/{REPOSITORY}/releases/download/{release["tag_name"]}/'
        for name, path, maximum in [('tbg-native-release.json', meta, 65536), ('native-provenance.jsonl', bundle, 8 * 1024 * 1024)]:
            asset = next((a for a in release['assets'] if a['name'] == name), None)
            require(asset is not None and asset['browser_download_url'] == prefix + name, 'native manifest/provenance not published')
            path.write_bytes(fetch(prefix + name, maximum))
        manifest = json.loads(meta.read_bytes())
        source = manifest.get('source_commit', '')
        require(re.fullmatch(r'[0-9a-f]{40}', source), 'native source SHA required')
        verify(meta, bundle, source, workflow=WORKFLOW, source_ref='refs/tags/' + release['tag_name'])
        floor = old['source_commit'] if old else config['minimum_source']
        require(re.fullmatch(r'[0-9a-f]{40}', floor) and api('/compare/' + floor + '...' + source)['status'] in {'ahead', 'identical'}, 'native source regression')
        public = catalog(manifest, release, live)
        publish(public)
        write_json(RECEIPT, manifest)
        write_json(CHECKED, {'source_commit': source, 'tag': release['tag_name'], 'outcome': 'healthy'})
        print('Published compatible native download catalog:', release['tag_name'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=Path, default=Path('/etc/tbg/auto-deploy.json'))
    args = parser.parse_args()
    STATE.mkdir(mode=0o700, exist_ok=True)
    with (STATE/'lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        run(json.loads(args.config.read_text()))


if __name__ == '__main__':
    main()
