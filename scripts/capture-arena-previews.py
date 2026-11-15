#!/usr/bin/env python3
"""Refresh shared Arena thumbnails from ready, identified native Blender scenes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--capture-after', type=float, default=8)
    args = parser.parse_args()
    binary, out = args.binary.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    info = json.loads(subprocess.check_output([str(binary), '--build-info'], text=True))
    version = tomllib.loads((ROOT/'crates/tbg-client/Cargo.toml').read_text())['package']['version']
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if info['version'] != version or info['source_commit'] != source:
        raise SystemExit('Capture binary must match the current client version and commit')
    if info['profile'] != 'debug' and 'ui-acceptance' not in info['features']:
        raise SystemExit('Capture binary requires debug mode or the ui-acceptance feature')
    arenas = json.loads((ROOT/'crates/tbg-client/assets/arenas/arenas.json').read_text())['arenas']
    report = {'build': info, 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'surface': 'Linux X11/Xvfb; actual Bevy native Blender scenery',
              'parity': 'Windows and desktop web observations pending', 'captures': []}
    for arena in sorted(arenas, key=lambda a: a['id']):
        slug = arena['slug']
        png, log = out/f'{slug}.png', out/f'{slug}.log'
        env = {k: v for k, v in os.environ.items() if not k.startswith('TBG_')}
        for key in ['WAYLAND_DISPLAY', 'WAYLAND_SOCKET', 'DBUS_SESSION_BUS_ADDRESS']:
            env.pop(key, None)
        env.update(WINIT_UNIX_BACKEND='x11', TBG_ARENA=slug,
                   TBG_CAPTURE=str(png), TBG_CAPTURE_SIZE='1536x764',
                   TBG_CAPTURE_AFTER=str(args.capture_after), TBG_DATA_DIR=str(out/'data'),
                   TBG_SERVER='ws://127.0.0.1:9/ws', LP_NUM_THREADS='2', OMP_NUM_THREADS='2')
        with log.open('w') as handle:
            subprocess.run(['xvfb-run', '-a', '-s', '-screen 0 1536x764x24', str(binary)],
                           env=env, stdout=handle, stderr=handle, timeout=120, check=True)
        witness = 'boot_ready=true, scene_waiting=false, pending_images=0, pending_texts=0, failed_images=0'
        if witness not in log.read_text():
            raise SystemExit(f'Arena capture was not ready: {log}')
        if 'Blender scenery pack unavailable' in log.read_text():
            raise SystemExit(f'Arena capture used fallback scenery: {log}')
        measured = subprocess.check_output(
            ['identify', '-format', '%w %h %[fx:standard_deviation]', str(png)], text=True).split()
        if measured[:2] != ['1536', '764'] or float(measured[2]) < .001:
            raise SystemExit(f'Blank or malformed Arena capture: {png}')
        dest = ROOT/'crates/tbg-client/assets/arenas'/arena['image']
        subprocess.run(['convert', str(png), '-resize', '768x382', '-quality', '85',
                        '-define', 'webp:method=6', str(dest)], check=True)
        report['captures'].append({'arena': slug, 'image': png.name, 'log': log.name,
                                   'sha256': hashlib.sha256(png.read_bytes()).hexdigest(),
                                   'thumbnail_sha256': hashlib.sha256(dest.read_bytes()).hexdigest()})
        (out/'capture-manifest.json').write_text(json.dumps(report, indent=2)+'\n')
        print(f'{slug}: ready Blender scene captured', flush=True)


if __name__ == '__main__':
    main()
