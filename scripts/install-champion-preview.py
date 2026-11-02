#!/usr/bin/env python3
"""Install the committed CC17 Linux preview without replacing the ordinary client or its data."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--prefix', type=Path, default=Path.home() / '.local')
    args = parser.parse_args()
    binary = args.binary.resolve()
    subprocess.run(['git', 'diff', '--quiet', 'HEAD', '--', 'crates', 'Cargo.toml',
                    'Cargo.lock', 'scripts/install-champion-preview.py'], cwd=ROOT, check=True)
    subprocess.run(['git', 'merge-base', '--is-ancestor',
                    '9fb7be01ac08cbed3bce1aa466d7243418490957', 'HEAD'], cwd=ROOT, check=True)
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    info = json.loads(subprocess.check_output([str(binary), '--build-info'], text=True))
    if (info['source_commit'] != commit or info['version'] != '0.1.2-cc.2'
            or info['protocol'] != 33 or info['target'] != 'x86_64-unknown-linux-gnu'):
        raise SystemExit('Build the committed CC17 Linux source with TBG_SOURCE_COMMIT before installation')
    prefix = args.prefix.expanduser().resolve()
    slot = prefix / 'opt/tupi-battle-grounds-champion-preview' / f"{info['version']}-{commit[:12]}"
    data = prefix / 'share/tupi-battle-grounds-champion-preview'
    files = [{'path': p.relative_to(ROOT / 'crates/tbg-client/assets').as_posix(),
              'sha256': sha(p)} for p in sorted((ROOT / 'crates/tbg-client/assets').rglob('*')) if p.is_file()]
    manifest = {'schema': 1, **info, 'binary_sha256': sha(binary),
                'cargo_lock_sha256': sha(ROOT / 'Cargo.lock'), 'assets': files,
                'scope': 'CC17 local Practice/Training preview; online roster is not activated',
                'laboratory_baseline': 'PR44 / 9fb7be01ac08cbed3bce1aa466d7243418490957',
                'data_directory': str(data)}
    if slot.exists():
        existing = slot / 'preview-build.json'
        if not existing.exists() or json.loads(existing.read_text()) != manifest:
            raise SystemExit(f'Refusing to replace different contents at {slot}')
    else:
        slot.mkdir(parents=True)
        shutil.copy2(binary, slot / 'tbg-client')
        shutil.copytree(ROOT / 'crates/tbg-client/assets', slot / 'assets')
        for name in ['THIRD_PARTY_NOTICES.md', 'LICENSE']:
            if (ROOT / name).exists():
                shutil.copy2(ROOT / name, slot / name)
        shutil.copytree(ROOT / 'deploy/native/licenses', slot / 'licenses')
        (slot / 'preview-build.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (slot / 'tbg-client').chmod(0o755)
    launcher = prefix / 'bin/tbg-champions'
    launcher.parent.mkdir(parents=True, exist_ok=True)
    content = ('#!/bin/sh\n'
               f'export TBG_DATA_DIR={shlex.quote(str(data))}\n'
               f'exec {shlex.quote(str(slot / "tbg-client"))} --panel troops '
               '--champion Gravecutter --server ws://127.0.0.1:8080/ws "$@"\n')
    if launcher.exists() and not launcher.read_text().startswith('#!/bin/sh\nexport TBG_DATA_DIR='):
        raise SystemExit(f'Refusing to replace an unrelated launcher at {launcher}')
    launcher.write_text(content)
    launcher.chmod(0o755)
    applications = Path(os.environ.get('XDG_DATA_HOME', str(prefix / 'share'))) / 'applications'
    applications.mkdir(parents=True, exist_ok=True)
    quoted = str(launcher).replace('\\', '\\\\').replace('"', '\\"').replace('`', '\\`').replace('$', '\\$')
    desktop = applications / 'tbg-champions-preview.desktop'
    desktop.write_text('[Desktop Entry]\nType=Application\nName=TBG · 17 Champions Preview\n'
                       'Name[pt_BR]=TBG · Prévia dos 17 Campeões\n'
                       'Comment=Local champion and Evolution validation\n'
                       f'Exec="{quoted}"\nIcon={slot}/assets/branding/tbg-icon-512.png\n'
                       'Terminal=false\nCategories=Game;StrategyGame;\n')
    print(json.dumps({'installed': str(slot), 'launcher': str(launcher),
                      'desktop': str(desktop), 'version': info['version'],
                      'source_commit': commit, 'binary_sha256': manifest['binary_sha256']}, indent=2))


if __name__ == '__main__':
    main()
