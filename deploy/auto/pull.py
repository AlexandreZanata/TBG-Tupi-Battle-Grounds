#!/usr/bin/env python3
"""Fetch public main candidates over HTTPS; no inbound runner, SSH key or GitHub token."""
import argparse
import fcntl
import json
import re
import subprocess
import tempfile
from pathlib import Path
from urllib.error import HTTPError
from urllib.parse import urlparse
from urllib.request import HTTPRedirectHandler, Request, build_opener

from activate import STATE, activate, check_compatibility, recover, write_json
from policy import MAX_ARCHIVE, REPOSITORY, WORKFLOW, require, unpack

API = 'https://api.github.com/repos/'+REPOSITORY
REPOSITORY_ID = 1285090346


class HttpsRedirects(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        parsed = urlparse(newurl)
        require(parsed.scheme == 'https' and parsed.hostname in {
            'api.github.com', 'github.com', 'release-assets.githubusercontent.com', 'objects.githubusercontent.com',
        }, 'unexpected download redirect')
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def fetch(url, maximum):
    require(url.startswith('https://'), 'HTTPS required')
    request = Request(url, headers={'User-Agent': 'TBG-main-deploy', 'Accept': 'application/vnd.github+json'})
    with build_opener(HttpsRedirects()).open(request, timeout=30) as response:
        data = response.read(maximum+1)
    require(len(data) <= maximum, 'response exceeds size limit')
    return data


def api(path):
    return json.loads(fetch(API+path, 4 * 1024 * 1024))


def verify(package, bundle, source, workflow=WORKFLOW):
    subprocess.run([
        'gh', 'attestation', 'verify', str(package), '--bundle', str(bundle),
        '--repo', REPOSITORY, '--signer-workflow', workflow,
        '--source-ref', 'refs/heads/main', '--source-digest', source,
        '--deny-self-hosted-runners',
    ], check=True, stdout=subprocess.DEVNULL, timeout=60)


def candidate(prefix='server'):
    require(prefix in {'server', 'web'}, 'unknown release family')
    require(api('')['id'] == REPOSITORY_ID, 'repository identity changed')
    releases = api('/releases?per_page=30')
    options = [r for r in releases if not r.get('draft') and re.fullmatch(prefix + r'-[0-9a-f]{40}', r.get('tag_name', ''))]
    return max(options, key=lambda r: r['published_at'], default=None)


def run(config, apply, recheck):
    if apply:
        recover()
    release = candidate()
    if release is None:
        print('Waiting for the first CI-approved main backend candidate.')
        return
    source = release['tag_name'][7:]
    checked = STATE/'checked.json'
    if not recheck and checked.exists() and json.loads(checked.read_text()).get('source_commit') == source:
        print('Candidate already handled; inspect checked.json for its outcome.')
        return
    current = STATE/'current.json'
    floor = json.loads(current.read_text())['source_commit'] if current.exists() else config['minimum_source']
    require(re.fullmatch(r'[0-9a-f]{40}', floor) is not None, 'verified deployment floor required')
    compare = api('/compare/'+floor+'...'+source)
    require(compare['status'] in ['ahead', 'identical'], 'candidate does not descend from deployment floor')
    if current.exists() and json.loads(current.read_text()).get('source_commit') == source:
        print('Candidate is the current verified deployment.')
        return
    download_and_handle(release, source, config, apply)


def download_and_handle(release, source, config, apply):
    checked = STATE/'checked.json'
    with tempfile.TemporaryDirectory(prefix='candidate-', dir=STATE) as temporary:
        folder = Path(temporary)
        package = folder/'backend.tar.gz'
        bundle = folder/'provenance.jsonl'
        prefix = 'https://github.com/'+REPOSITORY+'/releases/download/'+release['tag_name']+'/'
        names = {'tbg-backend-'+source+'.tar.gz': package, 'provenance.jsonl': bundle}
        for name, destination in names.items():
            asset = next((a for a in release['assets'] if a['name'] == name), None)
            require(asset is not None and asset['browser_download_url'] == prefix+name, 'missing or unexpected asset URL')
            destination.write_bytes(fetch(prefix+name, MAX_ARCHIVE if destination == package else 8*1024*1024))
        verify(package, bundle, source)
        manifest = unpack(package, folder/'verified', source)
        if not apply:
            check_compatibility(manifest)
            print(json.dumps({'verified_source': source, 'contract': manifest['contract'], 'database_schema': manifest['database_schema'], 'activated': False}))
            return
        try:
            activate(folder/'verified', manifest, config['public_health'])
        except Exception as error:
            write_json(checked, {'source_commit': source, 'outcome': 'operator_required', 'reason': str(error)[:500]})
            raise
        write_json(checked, {'source_commit': source, 'outcome': 'healthy'})
        print('Activated CI-approved compatible source '+source)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=Path, default=Path('/etc/tbg/auto-deploy.json'))
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--recheck', action='store_true')
    args = parser.parse_args()
    config = json.loads(args.config.read_text())
    STATE.mkdir(mode=0o700, exist_ok=True)
    with (STATE/'lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        run(config, args.apply, args.recheck)


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, subprocess.SubprocessError, HTTPError) as error:
        raise SystemExit(str(error)) from None
