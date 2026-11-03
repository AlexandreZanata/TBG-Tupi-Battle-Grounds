"""Verify static site archives without permitting the local admin console or private routes."""
import json
import re
import tarfile
from pathlib import Path

from policy import MAX_ARCHIVE, MAX_CONTENT, SHA, digest, require
from web_policy import MAX_FILES, safe_name

WORKFLOW = 'AlexandreZanata/TBG-Tupi-Battle-Grounds/.github/workflows/static-release.yml'
CONSOLE_CSP = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' blob:; connect-src 'self'; font-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"
SITES = {'landing': 'https://tupibg.com', 'admin': 'https://admin.tupibg.com'}


def unpack(archive, destination, source, *, admin_console_csp=None):
    require(bool(SHA.fullmatch(source)) and Path(archive).stat().st_size <= MAX_ARCHIVE, 'invalid static package')
    with tarfile.open(archive, 'r:gz') as tar:
        members, size = [], 0
        for member in tar:
            size += member.size
            require(member.isfile() and member.size >= 0 and safe_name(member.name), 'unsafe static member')
            require(size <= MAX_CONTENT and len(members) < MAX_FILES + 1, 'static content limit')
            members.append(member)
        names = [m.name for m in members]
        require(len(names) == len(set(names)) and 'manifest.json' in names, 'invalid static members')
        require(tar.getmember('manifest.json').size <= 1024 * 1024, 'static manifest too large')
        meta = json.load(tar.extractfile('manifest.json'))
        require(meta.get('schema') == 1 and meta.get('source_commit') == source, 'static source mismatch')
        require(isinstance(meta.get('sites'), dict) and set(meta['sites']) == set(SITES), 'unexpected static site')
        expected = {'manifest.json'}
        for site, files in meta['sites'].items():
            require(isinstance(files, dict) and 'index.html' in files, 'static entry missing')
            require(site + '/index.html' in names, 'static index missing')
            require(tar.getmember(site + '/index.html').size <= 2 * 1024 * 1024, 'static index too large')
            for name in files:
                require(safe_name(name), 'invalid static path')
                expected.add(site + '/' + name)
        require(set(names) == expected, 'static manifest mismatch')
        root = Path(destination)
        root.mkdir()
        for site, files in meta['sites'].items():
            for name, expected_hash in files.items():
                require(isinstance(expected_hash, str) and re.fullmatch(r'[0-9a-f]{64}', expected_hash), 'invalid static hash')
                console = site == 'admin' and meta.get('admin_mode') == 'app'
                if console:
                    require(admin_console_csp == CONSOLE_CSP, 'console requires attended CSP activation')
                elif site == 'admin':
                    require(Path(name).suffix.lower() not in {'.js', '.mjs', '.json'}, 'admin preview/code is not publishable')
                path = root/site/name
                path.parent.mkdir(parents=True, exist_ok=True)
                with path.open('xb') as out:
                    stream = tar.extractfile(site + '/' + name)
                    while chunk := stream.read(1024 * 1024):
                        out.write(chunk)
                require(digest(path) == expected_hash, 'static hash mismatch')
            if site == 'admin' and meta.get('admin_mode') != 'app':
                html = (root/site/'index.html').read_text().lower()
                require(not any(tag in html for tag in ['<script', '<form', '<input', '<iframe']), 'active admin content refused')
        return meta
