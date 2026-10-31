#!/usr/bin/env python3
"""Capture actual base-form models with a debug native client, Xvfb and ImageMagick."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--scratch', type=Path, default=ROOT / '.local/portraits')
    parser.add_argument('kinds', nargs='*')
    args = parser.parse_args()
    source = (ROOT / 'crates/tbg-core/src/types.rs').read_text()
    pool = re.search(r'ALL_KINDS: \[UnitKind; \d+\] = \[([\s\S]*?)\];', source)[1]
    kinds = args.kinds or re.findall(r'UnitKind::(\w+)', pool)
    assets = ROOT / 'crates/tbg-client/assets/battle/portraits'
    args.scratch.mkdir(parents=True, exist_ok=True)
    report = []
    for kind in kinds:
        if kind == 'Crate':
            continue
        png = (args.scratch / f'{kind}.png').resolve()
        env = {**os.environ, 'WINIT_UNIX_BACKEND': 'x11',
               'TBG_DATA_DIR': str((args.scratch / 'data').resolve()),
               'TBG_GALLERY': kind, 'TBG_PORTRAIT': '1', 'TBG_CAPTURE': str(png),
               'TBG_CAPTURE_SIZE': '1024x1024' if kind == 'Longaxe' else '400x400', 'TBG_CAPTURE_AFTER': '2'}
        with (args.scratch / f'{kind}.log').open('w') as log:
            subprocess.run(['xvfb-run', '-a', '-s', '-screen 0 1024x1024x24' if kind == 'Longaxe' else '-screen 0 400x400x24',
                            str(args.binary.resolve())], env=env, stdout=log,
                           stderr=log, timeout=60, check=True)
        dest = assets / (kind.lower() + '.webp')
        # Longaxe's unusually tall weapon would make the body unreadable in a small Shop card.
        crop = ['-crop', '308x384+358+601', '+repage'] if kind == 'Longaxe' else []
        subprocess.run(['convert', str(png), *crop, '-fuzz', '4%', '-trim', '+repage',
                        '-resize', '340x340', '-gravity', 'center', '-background',
                        '#14202a', '-extent', '400x400', '-quality', '85', str(dest)],
                       check=True)
        size = dest.stat().st_size
        if size > 40960:
            raise SystemExit(f'Portrait exceeds 40 KiB: {kind}: {size}')
        report.append({'kind': kind, 'bytes': size})
        print(f'{kind}: {size}', flush=True)
    (args.scratch / 'report.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
