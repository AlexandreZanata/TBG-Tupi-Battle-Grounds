"""Refresh only the Android entry through the shared catalog lock and atomic writer."""
import json
from pathlib import Path
import re
import tempfile
from urllib.error import HTTPError
from urllib.parse import quote

from activate import STATE, write_json
from android_policy import WORKFLOW, catalog
from github import REPOSITORY_ID, api, fetch
from policy import REPOSITORY, require
from pull import verify

RECEIPT = STATE/'android-current.json'
CHECKED = STATE/'android-checked.json'


def android(release):
    return (not release.get('draft')
            and re.fullmatch(r'android-v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?-android\.[0-9]+', release.get('tag_name', ''))
            and any(a['name'] == 'tbg-android-release.json' for a in release.get('assets', [])))


def run(config, live, publish, listing):
    require(api('')['id'] == REPOSITORY_ID, 'repository identity changed')
    old = json.loads(RECEIPT.read_text()) if RECEIPT.exists() else None
    candidates = [r for r in listing if android(r)]
    pending = {'schema': 1, 'server': {'protocol': int(re.match(r'w([0-9]+)-', live['contract'])[1]),
                                     'contract': live['contract']}, 'android': {'state': 'pending'}}
    if old and not any(r['tag_name'] == old['tag'] for r in candidates):
        try:
            active = api('/releases/tags/'+quote(old['tag'], safe=''))
        except HTTPError as error:
            if error.code != 404:
                raise
            active = None
        if active is None or not android(active):
            publish(pending)
            raise ValueError('The active Android beta is no longer published')
        candidates.append(active)
    if not candidates:
        publish(pending)
        print('Waiting for the first verified Android beta.')
        return
    release = max(candidates, key=lambda r: int(r['tag_name'].rsplit('.', 1)[1]))
    if old and old['tag'] == release['tag_name']:
        try:
            publish(catalog(old, release, live))
        except ValueError:
            publish(pending)
            raise
        print('Verified Android catalog remains current.')
        return
    if old:
        require(int(release['tag_name'].rsplit('.', 1)[1]) > old['version_code'], 'Android version rollback')
    with tempfile.TemporaryDirectory(prefix='android-catalog-', dir=STATE) as temporary:
        folder = Path(temporary)
        meta, bundle = folder/'android.json', folder/'provenance.jsonl'
        prefix = f'https://github.com/{REPOSITORY}/releases/download/{release["tag_name"]}/'
        for name, path, maximum in [('tbg-android-release.json', meta, 65536), ('android-provenance.jsonl', bundle, 8*1024*1024)]:
            asset = next((a for a in release['assets'] if a['name'] == name), None)
            require(asset is not None and asset['browser_download_url'] == prefix+name, 'Android manifest/provenance missing')
            path.write_bytes(fetch(prefix+name, maximum))
        manifest = json.loads(meta.read_bytes())
        source = manifest.get('source_commit', '')
        require(re.fullmatch(r'[0-9a-f]{40}', source), 'Android source SHA required')
        verify(meta, bundle, source, workflow=WORKFLOW, source_ref='refs/tags/'+release['tag_name'])
        floor = old['source_commit'] if old else config['minimum_source']
        require(re.fullmatch(r'[0-9a-f]{40}', floor)
                and api('/compare/'+floor+'...'+source)['status'] in {'ahead', 'identical'}, 'Android source regression')
        try:
            value = catalog(manifest, release, live)
        except ValueError:
            publish(pending)
            raise
        publish(value)
        write_json(RECEIPT, manifest)
        write_json(CHECKED, {'source_commit': source, 'tag': release['tag_name'], 'outcome': 'healthy'})
        print('Published compatible Android beta catalog:', release['tag_name'])
