#!/usr/bin/env python3
"""Package only backend executables and their offline build identity for CI provenance."""
import argparse
import hashlib
import json
import re
import subprocess
import tarfile
import tempfile
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, default=Path('target/release'))
    parser.add_argument('--output', type=Path, default=Path('target/backend'))
    args = parser.parse_args()
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    identity = json.loads(subprocess.check_output([str((args.directory/'tbg-server').resolve()), '--build-info'], text=True))
    if not re.fullmatch(r'[0-9a-f]{40}', source) or identity['source_commit'] != source:
        raise SystemExit('Backend must embed the exact checked-out source commit')
    identity['schema'] = 1
    identity['files'] = {}
    for name in ['tbg-server', 'tbg-release-admin']:
        with (args.directory/name).open('rb') as stream:
            identity['files'][name] = hashlib.file_digest(stream, 'sha256').hexdigest()
    args.output.mkdir(parents=True, exist_ok=True)
    archive = args.output/f'tbg-backend-{source}.tar.gz'
    with tempfile.TemporaryDirectory() as temporary:
        manifest = Path(temporary)/'manifest.json'
        manifest.write_text(json.dumps(identity, sort_keys=True, indent=2)+'\n')
        with tarfile.open(archive, 'w:gz', format=tarfile.USTAR_FORMAT) as tar:
            tar.add(manifest, arcname='manifest.json')
            for name in identity['files']:
                tar.add(args.directory/name, arcname=name)
    print(archive)


if __name__ == '__main__':
    main()
