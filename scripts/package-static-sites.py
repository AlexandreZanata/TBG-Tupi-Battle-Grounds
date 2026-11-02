#!/usr/bin/env python3
"""Seal the landing and safe public admin only; the operator preview is never exported."""
import argparse
import hashlib
import io
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'web/scripts'))
from sites import SITES, files
from asset_versions import rendered


def package(output, source):
    mappings = {name: files(SITES[site]) for name, site in [('landing', 'landing'), ('admin', 'admin-public')]}
    payloads = {}
    manifest = {'schema': 1, 'source_commit': source, 'sites': {}}
    for name, mapping in mappings.items():
        if any(path.is_symlink() for path in mapping.values()):
            raise ValueError('static source links are not publishable')
        payloads[name] = rendered(mapping)
        manifest['sites'][name] = {path: hashlib.sha256(body).hexdigest() for path, body in payloads[name].items()}
    output.mkdir(parents=True, exist_ok=True)
    archive = output/('tbg-static-' + source + '.tar.gz')
    if archive.exists():
        raise ValueError('immutable static package already exists')
    with tempfile.TemporaryDirectory() as temporary:
        meta = Path(temporary)/'manifest.json'
        meta.write_text(json.dumps(manifest, sort_keys=True) + '\n')
        with tarfile.open(archive, 'x:gz') as tar:
            tar.add(meta, arcname='manifest.json')
            for name, payload in payloads.items():
                for path, body in sorted(payload.items()):
                    member = tarfile.TarInfo(name + '/' + path)
                    member.size, member.mode = len(body), 0o644
                    tar.addfile(member, io.BytesIO(body))
    return archive


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(package(args.output, subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()))
