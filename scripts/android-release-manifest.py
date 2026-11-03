#!/usr/bin/env python3
"""Bind one non-debuggable signed ARM64 beta to its exact source and public download."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from android_release import CERTIFICATE, ROOT, configuration, filename, version


def manifest(folder, source):
    config, value = configuration(), version()
    report = json.loads((folder/'android-inspection.json').read_text())
    info = report['build_info']
    if not (re.fullmatch(r'[0-9a-f]{40}', source) and info['source_sha'] == source
            and info.get('channel') == 'beta' and info['abi'] == config['abi']
            and info['android_revision'] == config['version_code']
            and info['package'] == config['package'] and report.get('debuggable') is False
            and report['signing_certificate_sha256'].lower() == CERTIFICATE
            and info['cargo_lock_sha256'] == hashlib.sha256((ROOT/'Cargo.lock').read_bytes()).hexdigest()):
        raise ValueError('Android publication requires the exact public source/ABI/revision/signature')
    if not (f"versionCode='{config['version_code']}'" in report['package_badging']
            and f"versionName='{value}'" in report['package_badging']):
        raise ValueError('APK version does not match its immutable release')
    apk = folder/filename(value)
    digest = hashlib.sha256(apk.read_bytes()).hexdigest()
    if report['apk_sha256'] != digest:
        raise ValueError('APK differs from the inspected bytes')
    tag = 'android-v' + value
    asset = {'kind': 'apk', 'name': apk.name, 'bytes': apk.stat().st_size, 'sha256': digest,
             'url': f'https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/download/{tag}/{apk.name}'}
    return {'schema': 1, 'channel': 'beta', 'source_commit': source, 'tag': tag,
            'version': value, 'version_code': config['version_code'], 'package': config['package'],
            'abi': config['abi'], 'min_sdk': 31, 'target_sdk': 36,
            'certificate_sha256': CERTIFICATE, 'debuggable': False, 'features': [],
            'protocol': info['protocol'],
            'contract': f"w{info['protocol']}-r{info['ruleset']['version']}-{info['ruleset']['source_hash']}",
            'assets_sha256': info['assets_sha256'], 'visual_acceptance': 'pending', 'assets': [asset]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifacts', type=Path, required=True)
    args = parser.parse_args()
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    result = manifest(args.artifacts, source)
    path = args.artifacts/'tbg-android-release.json'
    path.write_text(json.dumps(result, indent=2) + '\n')
    for file in [args.artifacts/result['assets'][0]['name'], path, args.artifacts/'android-inspection.json']:
        (args.artifacts/(file.name+'.sha256')).write_text(hashlib.sha256(file.read_bytes()).hexdigest()+'  '+file.name+'\n')
    print('Verified immutable Android beta manifest:', result['tag'])


if __name__ == '__main__':
    main()
