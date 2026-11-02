#!/usr/bin/env python3
"""Capture the scoped AB-01/OG-02 native Training witnesses using real game actions."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cases', nargs='+', choices=['barrel-allies', 'barrel-intercept', 'abyssal-crowd', 'barrel-rear', 'foam-rear'])
    args = parser.parse_args()
    binary, out = args.binary.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    report = {'build': json.loads(subprocess.check_output([str(binary), '--build-info'], text=True)),
              'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'scenes': [],
              'parity': 'Linux only; Windows and desktop game web pending'}
    cases = [('barrel-allies', 'GustDrummer', {'TBG_CAPTURE_BARREL':'1'}, 'Native rolling-barrel witness passed:'),
             ('barrel-intercept', 'GustDrummer', {'TBG_CAPTURE_BARREL':'1', 'TBG_CAPTURE_BARREL_INTERCEPT':'1'}, 'Native rolling-barrel witness passed:'),
             ('abyssal-crowd', 'Trident', {'TBG_CAPTURE_ABYSSAL_CROWD':'1'}, 'Training journey passed:'),
             ('barrel-rear', 'GustDrummer', {'TBG_CAPTURE_SUPPORT_REAR':'1'}, 'Support rear-heal visual regression passed:'),
             ('foam-rear', 'FoamWeaver', {'TBG_CAPTURE_SUPPORT_REAR':'1'}, 'Support rear-heal visual regression passed:')]
    for name, kind, flags, witness in cases:
        if args.cases and name not in args.cases:
            continue
        png, log = out/f'{name}.png', out/f'{name}.log'
        env = {k:v for k,v in os.environ.items() if not k.startswith('TBG_')}
        env.pop('WAYLAND_DISPLAY', None)
        env.pop('WAYLAND_SOCKET', None)
        env.update(WINIT_UNIX_BACKEND='x11', TBG_CAPTURE=str(png), TBG_CAPTURE_SIZE='1600x1000',
                   TBG_CAPTURE_PANEL='training', TBG_CHAMPION=kind, TBG_CAPTURE_AFTER='8', TBG_LANG='pt-BR',
                   TBG_DATA_DIR=str(out/'data'), LP_NUM_THREADS='2', OMP_NUM_THREADS='2', **flags)
        cpus = ','.join(str(c) for c in sorted(os.sched_getaffinity(0))[:2])
        command = ['taskset', '-c', cpus, 'xvfb-run', '-a', '-s', '-screen 0 1600x1000x24', str(binary)]
        with log.open('w') as handle:
            process = subprocess.Popen(command, env=env, stdout=handle, stderr=handle, start_new_session=True)
            try:
                code = process.wait(timeout=120)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)
                raise SystemExit(f'Timed out; owned capture process group closed: {log}')
            if code:
                raise SystemExit(f'Capture exited {code}: {log}')
        text = log.read_text()
        for required in [witness, 'Training journey passed:', 'boot_ready=true, scene_waiting=false, pending_images=0, pending_texts=0, failed_images=0']:
            if required not in text:
                raise SystemExit(f'Missing witness {required}: {log}')
        if name == 'barrel-intercept' and 'intercepted=true' not in text:
            raise SystemExit(f'Enemy contact was not observed: {log}')
        size = subprocess.check_output(['identify', '-format', '%w %h %[fx:standard_deviation]', str(png)], text=True).split()
        if size[:2] != ['1600','1000'] or float(size[2]) < .001:
            raise SystemExit(f'Blank or malformed capture: {png}')
        report['scenes'].append({'kind':kind, 'case':name, 'image':png.name, 'log':log.name,
                                 'sha256':hashlib.sha256(png.read_bytes()).hexdigest(),
                                 'witness':[line for line in text.splitlines() if 'passed:' in line]})
        (out/'linux-training.json').write_text(json.dumps(report, indent=2)+'\n')
        print(name+': passed', flush=True)


if __name__ == '__main__':
    main()
