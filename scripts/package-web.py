#!/usr/bin/env python3
"""Package the browser game under immutable source URLs; exclude other frontends."""
import argparse
import hashlib
import json
import re
import subprocess
import tarfile
import tempfile
from pathlib import Path


def package(dist, output, source, contract):
    if not re.fullmatch(r'[0-9a-f]{40}', source) or not re.fullmatch(r'w\d+-r\d+-[0-9a-f]{16}', contract):
        raise ValueError('exact source and canonical match contract required')
    prefix = '/releases/' + source + '/'
    files = sorted(p for p in dist.rglob('*') if p.is_file())
    if any(p.is_symlink() for p in dist.rglob('*')) or not files:
        raise ValueError('empty bundle or links in distribution')
    html = (dist/'index.html').read_text()
    if '<base' in html.lower() or prefix not in html:
        raise ValueError('build Trunk with the exact immutable public URL and no existing base')
    html = html.replace('<head>', '<head>\n<base href="' + prefix + '">', 1)
    if '<base href="' + prefix + '">' not in html:
        raise ValueError('missing HTML head')
    if any(p.relative_to(dist).parts[0] in {'admin', 'landing', 'mobile', 'android', 'ios'} for p in files):
        raise ValueError('out-of-scope frontend in game bundle')
    if not any(p.suffix == '.wasm' for p in files):
        raise ValueError('compiled browser game missing')
    output.mkdir(parents=True, exist_ok=True)
    archive = output/('tbg-web-' + source + '.tar.gz')
    if archive.exists():
        raise ValueError('refusing to replace an immutable web package')
    with tempfile.TemporaryDirectory() as temporary:
        stage = Path(temporary)
        index = stage/'index.html'
        index.write_text(html)
        manifest = {'schema': 1, 'source_commit': source, 'contract': contract,
                    'public_url': prefix, 'files': {}}
        paths = {p.relative_to(dist).as_posix(): p for p in files}
        paths['index.html'] = index
        for name, path in paths.items():
            manifest['files'][name] = hashlib.sha256(path.read_bytes()).hexdigest()
        meta = stage/'manifest.json'
        meta.write_text(json.dumps(manifest, sort_keys=True) + '\n')
        with tarfile.open(archive, 'x:gz', format=tarfile.PAX_FORMAT) as tar:
            tar.add(meta, arcname='manifest.json')
            for name, path in paths.items():
                tar.add(path, arcname='web/' + name, recursive=False)
    return archive


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dist', type=Path, default=Path('crates/tbg-client/dist'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--contract', required=True)
    args = parser.parse_args()
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    print(package(args.dist, args.output, source, args.contract))


if __name__ == '__main__':
    main()
