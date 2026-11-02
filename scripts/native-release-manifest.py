#!/usr/bin/env python3
"""Bind complete Linux/Windows download assets to one verified source and match contract."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = 'AlexandreZanata/TBG-Tupi-Battle-Grounds'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def identity(archive):
    if archive.suffix == '.zip':
        with zipfile.ZipFile(archive) as package:
            names = package.namelist()
            indexes = [name for name in names if name.count('/') == 1 and name.endswith('/build-info.json')]
            require(len(indexes) == 1 and package.getinfo(indexes[0]).file_size <= 65536, 'one bounded package identity required')
            return json.loads(package.read(indexes[0]))
    with tarfile.open(archive, 'r:gz') as package:
        indexes = [m for m in package if m.isfile() and m.name.count('/') == 1 and m.name.endswith('/build-info.json')]
        require(len(indexes) == 1 and indexes[0].size <= 65536, 'one bounded package identity required')
        return json.load(package.extractfile(indexes[0]))


def manifest(artifacts, source, tag):
    version = tomllib.loads((ROOT/'crates/tbg-client/Cargo.toml').read_text())['package']['version']
    require(tag == 'v' + version and re.fullmatch(r'[0-9a-f]{40}', source), 'exact tag/source required')
    stem = 'tupi-battle-grounds-' + version
    linux = identity(artifacts/(stem+'-linux-x86_64.tar.gz'))
    windows = identity(artifacts/(stem+'-windows-x86_64.zip'))
    for info, platform, target in [(linux, 'linux', 'x86_64-unknown-linux-gnu'), (windows, 'windows', 'x86_64-pc-windows-msvc')]:
        require(info['commit'] == source and info['version'] == version, 'mixed source/version artifacts')
        require(info['platform'] == platform and info['target'] == target and info['profile'] == 'release', 'wrong platform/toolchain/profile')
        require(info['cargo_lock_sha256'] == hashlib.sha256((ROOT/'Cargo.lock').read_bytes()).hexdigest(), 'dependency lock differs')
    for key in ['protocol', 'contract', 'assets_sha256', 'cargo_lock_sha256', 'features']:
        require(linux[key] == windows[key], 'platform identity differs: ' + key)
    result = {'schema': 1, 'source_commit': source, 'tag': tag, 'version': version,
              'channel': 'beta', 'protocol': linux['protocol'], 'contract': linux['contract'],
              'assets_sha256': linux['assets_sha256'], 'features': linux['features'],
              'windows_publisher_signature': 'unsigned', 'visual_acceptance': 'pending', 'platforms': {}}
    for platform, assets, minimum in [
        ('linux', [('tarball', '-linux-x86_64.tar.gz'), ('appimage', '-linux-x86_64.AppImage')], 'x86_64, glibc 2.35+, Vulkan'),
        ('windows', [('zip', '-windows-x86_64.zip'), ('installer', '-windows-x86_64-setup.exe')], 'Windows 11 x64, DirectX 12')]:
        rows = []
        for kind, suffix in assets:
            path = artifacts/(stem+suffix)
            require(path.is_file() and not path.is_symlink(), 'missing native asset: ' + path.name)
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            require((artifacts/(path.name+'.sha256')).read_text().split() == [digest, path.name], 'native checksum sidecar differs')
            rows.append({'kind': kind, 'name': path.name, 'bytes': path.stat().st_size, 'sha256': digest,
                         'url': f'https://github.com/{REPOSITORY}/releases/download/{tag}/{path.name}'})
        result['platforms'][platform] = {'minimum': minimum, 'assets': rows}
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifacts', type=Path, required=True)
    parser.add_argument('--tag', required=True)
    args = parser.parse_args()
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    value = manifest(args.artifacts, source, args.tag)
    target = args.artifacts/'tbg-native-release.json'
    require(not target.exists(), 'immutable native manifest already exists')
    target.write_text(json.dumps(value, indent=2) + '\n')
    print(target)
