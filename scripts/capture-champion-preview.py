#!/usr/bin/env python3
"""Capture actual CC17 native models once; encode Shop portraits without changing their pixels."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import time


ROOT = Path(__file__).resolve().parents[1]


def capture(binary, scratch, kind, lineup, delay):
    catalog = json.loads((ROOT / 'docs/champion-completion/catalog.json').read_text())['champions']
    slug = next((c['id'] for c in catalog if c['kind'] == kind), kind.lower())
    width, height = (1600, 1000) if lineup else (400, 400)
    png = scratch / f'{slug}{"-lineup" if lineup else "-portrait"}.png'
    env = {k:v for k,v in os.environ.items() if not k.startswith("TBG_")}
    env.update(WINIT_UNIX_BACKEND='x11', TBG_GALLERY=kind,
               TBG_CAPTURE=str(png), TBG_CAPTURE_SIZE=f'{width}x{height}',
               TBG_CAPTURE_AFTER=str(delay), TBG_DATA_DIR=str(scratch / 'data'),
               LP_NUM_THREADS='2', OMP_NUM_THREADS='2')
    for key in ['WAYLAND_DISPLAY', 'WAYLAND_SOCKET', 'TBG_PORTRAIT', 'TBG_LINEUP']:
        env.pop(key, None)
    env['TBG_LINEUP' if lineup else 'TBG_PORTRAIT'] = '1'
    command = ['xvfb-run', '-a', '-s', f'-screen 0 {width}x{height}x24', str(binary)]
    if shutil.which('taskset') and hasattr(os, 'sched_getaffinity'):
        cpus = ','.join(str(c) for c in sorted(os.sched_getaffinity(0))[:2])
        command = ['taskset', '-c', cpus, *command]
    log = png.with_suffix('.log')
    started = time.monotonic()
    with log.open('w') as out:
        subprocess.run(command, env=env, stdout=out, stderr=out, timeout=90, check=True)
    if not ready(log):
        raise SystemExit(f'Capture raced loading; preserve the evidence and increase --capture-after: {log}')
    dimensions = subprocess.check_output(['identify','-format','%w %h %[fx:standard_deviation]',str(png)], text=True).split()
    if tuple(map(int,dimensions[:2])) != (width,height) or float(dimensions[2]) < 0.001:
        raise SystemExit(f'Blank or malformed native capture: {png}')
    if not lineup:
        dest = ROOT / 'crates/tbg-client/assets/battle/portraits' / f'{kind.lower()}.webp'
        # Lossless format encoding only: composition and pixels originate in the engine.
        subprocess.run(['convert',str(png),'-define','webp:lossless=true',str(dest)],check=True)
        subprocess.run(['compare','-metric','AE',str(png),str(dest),'null:'],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,check=True)
        if dest.stat().st_size > 40960:
            raise SystemExit(f'Portrait exceeds the established 40 KiB budget: {dest}')
    return {'kind': kind, 'forms': ['base','A1','A2','B1','B2'] if lineup else ['base'],
            'image': png.name, 'sha256': hashlib.sha256(png.read_bytes()).hexdigest(),
            'size': [width,height], 'seconds': round(time.monotonic()-started,2),
            'mode': 'lineup: idle/walk/attack' if lineup else 'base portrait', 'log': log.name}


def ready(log):
    text = log.read_text()
    return ('boot_ready=true, scene_waiting=false, pending_images=0, pending_texts=0, failed_images=0' in text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--kinds', nargs='*')
    parser.add_argument('--capture-after', type=float, default=8.0)
    parser.add_argument('--seed-manifest', type=Path, help='Reuse ready, hash-verified captures with their original build provenance')
    parser.add_argument('--invalidate-kind', nargs='*', default=[])
    parser.add_argument('--all-portraits', action='store_true', help='Refresh every registered non-Crate portrait, plus the seventeen new lineups')
    args = parser.parse_args()
    binary = args.binary.resolve()
    scratch = args.output.resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    info = json.loads(subprocess.check_output([str(binary),'--build-info'], text=True))
    catalog = json.loads((ROOT / 'docs/champion-completion/catalog.json').read_text())['champions']
    new_kinds = [c['kind'] for c in catalog]
    expanded = json.loads((ROOT / 'docs/ogre-faction/catalog.json').read_text())['champions']
    abyssals = json.loads((ROOT / 'docs/abyssal-faction/catalog.json').read_text())['champions']
    lineup_kinds = [*new_kinds, 'SiegeWalker', 'Trident', *(c['kind'] for c in expanded), *(c['kind'] for c in abyssals)]
    if args.all_portraits:
        source = (ROOT / 'crates/tbg-core/src/types.rs').read_text()
        pool = re.search(r'ALL_KINDS: \[UnitKind; \d+\] = \[([\s\S]*?)\];', source)[1]
        kinds = [k for k in re.findall(r'UnitKind::(\w+)', pool) if k != 'Crate']
    else:
        kinds = args.kinds or new_kinds
    report = {'build': info, 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'os': platform.platform(), 'surface': 'Linux X11/Xvfb; actual Bevy native render',
              'parity': 'Windows/desktop web unverified', 'captures': []}
    existing=scratch / 'capture-manifest.json'
    if existing.exists():
        previous=json.loads(existing.read_text())
        if previous['binary_sha256'] != report['binary_sha256']:
            raise SystemExit('Choose a fresh output directory for a different executable')
        report=previous
    elif args.seed_manifest:
        seed_path = args.seed_manifest.resolve()
        seed = json.loads(seed_path.read_text())
        for record in seed['captures']:
            if record['kind'] in args.invalidate_kind:
                continue
            png, log = seed_path.parent/record['image'], seed_path.parent/record['log']
            if not ready(log):
                report.setdefault('rejected_loading', []).append({**record, 'origin': str(seed_path)})
                continue
            if hashlib.sha256(png.read_bytes()).hexdigest() != record['sha256']:
                raise SystemExit(f'Seed capture hash mismatch: {png}')
            for source in [png, log]:
                shutil.copy2(source, scratch/source.name)
            report['captures'].append({**record, 'build': seed['build'], 'binary_sha256': seed['binary_sha256']})
        existing.write_text(json.dumps(report, indent=2)+'\n')
    for kind in kinds:
        for lineup in ([False, True] if kind in lineup_kinds else [False]):
            mode='lineup: idle/walk/attack' if lineup else 'base portrait'
            cached=next((c for c in report['captures'] if c['kind']==kind and c['mode']==mode),None)
            if cached and (scratch/cached['image']).exists() and hashlib.sha256((scratch/cached['image']).read_bytes()).hexdigest()==cached['sha256'] and ready(scratch/cached['log']):
                print(f'{kind}: reused verified {mode}',flush=True)
                continue
            if cached:
                rejected = scratch / 'rejected-loading'
                rejected.mkdir(exist_ok=True)
                for name in [cached['image'], cached['log']]:
                    shutil.copy2(scratch/name, rejected/name)
                report.setdefault('rejected_loading', []).append(cached)
                report['captures'].remove(cached)
            record = capture(binary, scratch, kind, lineup, args.capture_after)
            record.update(build=info, binary_sha256=report['binary_sha256'])
            report['captures'].append(record)
            (scratch / 'capture-manifest.json').write_text(json.dumps(report, indent=2)+'\n')
            print(f'{kind}: {record["mode"]} captured ({record["seconds"]} s)', flush=True)


if __name__ == '__main__':
    main()
