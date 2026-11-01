#!/usr/bin/env python3
"""Capture immutable P70 sheets with one real-renderer process per champion/view.

This never builds the game, edits portraits, or claims live combat/VFX coverage.
Run against the frozen tooling commit and an explicitly built development binary.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / 'docs/planning/P70-VISUAL-MANIFEST.json'
ART_PATHS = ['crates/tbg-client/src/art', 'crates/tbg-client/src/vfx',
             'crates/tbg-client/src/vfx.rs', 'crates/tbg-client/src/visuals',
             'crates/tbg-client/src/visuals.rs', 'crates/tbg-client/src/sync',
             'crates/tbg-core/src']


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def sha256(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def write_new(path, value):
    with path.open('x') as dest:
        json.dump(value, dest, indent=2)
        dest.write('\n')


def png_size(path):
    subprocess.run(['convert', '-regard-warnings', str(path), 'null:'],
                   check=True, capture_output=True, timeout=30)
    with path.open('rb') as source:
        header = source.read(24)
    if len(header) != 24 or header[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError(f'Invalid PNG: {path}')
    return list(struct.unpack('>II', header[16:24]))


def collect(directory, unit, args, provenance):
    log = (directory / 'renderer.log').read_text(errors='replace')
    adapter = re.search(r'AdapterInfo\s*\{[^\n]*\}', log)
    if not adapter:
        raise ValueError(f'Renderer identity missing: {directory}')
    captures = []
    for page, clip in enumerate(provenance['clip_ids']):
        png = directory / f'{page:02}.png'
        raw = json.loads((directory / f'{page:02}.json').read_text())
        if raw['unit'] != unit or raw['clip'] != clip:
            raise ValueError(f'Unexpected unit/clip metadata: {directory}/{page:02}')
        expected_forms = [(f['tier'], f['branch']) for f in provenance['forms']]
        if [(f['tier'], f['branch']) for f in raw['forms']] != expected_forms:
            raise ValueError(f'Unexpected form coverage: {directory}/{page:02}')
        if (raw['team'] != args.team or raw['quality'] != args.quality
                or directory.name != f"{args.team}-{raw['view']}"):
            raise ValueError(f'Unexpected view/team/quality: {directory}/{page:02}')
        samples = []
        for form in raw['forms']:
            name = next(f['name'] for f in provenance['forms']
                        if (f['tier'], f['branch']) == (form['tier'], form['branch']))
            for time in form['samples_s']:
                samples.append({'form': name, 'clip': clip, 'sample_time': time,
                                'duration': form['duration_s'],
                                'sample_label': f'{time:.5f}s'})
        width, height = png_size(png)
        if [width, height] != raw['viewport'] or f'{width}x{height}' != args.size:
            raise ValueError(f'Unexpected viewport: {directory}/{page:02}')
        captures.append({
            'id': f'{args.phase}-{unit}-{directory.name}-{args.quality}-{clip}',
            'unit_id': unit, 'forms': [f['name'] for f in provenance['forms']],
            'clip_or_ability': clip, 'phase_before_or_after': args.phase,
            'samples': samples, 'kind': 'raw_pose_sheet',
            'content_sha': args.content_sha, 'tooling_sha': provenance['tooling_sha'],
            'camera': raw.get('camera', {'preset': 'polish-fixed', 'view': raw['view']}),
            'aim': raw['aim'], 'viewport': raw['viewport'], 'dpi': raw['dpi'],
            'quality': raw['quality'], 'platform': 'linux',
            'team': raw['team'], 'view': raw['view'],
            'renderer': {'adapter_log': adapter.group(0), 'log': str(directory / 'renderer.log')},
            'scenario_seed': 0, 'path_or_uri': str(png), 'sha256': sha256(png),
            'width': width, 'height': height, 'metadata_path': str(directory / f'{page:02}.json'),
            'metadata_sha256': sha256(directory / f'{page:02}.json'),
            'secondary_motion': raw.get('secondary_motion', 'see raw metadata'),
            'limitation': 'Event-free pose samples; no transition or live-effect claim.',
        })
    return captures


def capture(directory, unit, view, args):
    directory.mkdir(parents=True, exist_ok=False)
    env = {key: value for key, value in os.environ.items()
           if not key.startswith('TBG_') and key not in ('WAYLAND_DISPLAY', 'WAYLAND_SOCKET')}
    env.update({
        'WINIT_UNIX_BACKEND': 'x11', 'WGPU_BACKEND': 'vulkan',
        'LP_NUM_THREADS': '2', 'RUST_LOG': 'info', 'NO_COLOR': '1',
        'TBG_DATA_DIR': str(directory / 'private-data'),
        'TBG_GALLERY': 'polish', 'TBG_UNIT': unit, 'TBG_VIEW': view,
        'TBG_REVIEW_TEAM': args.team,
        'TBG_CAPTURE_POLISH': str(directory), 'TBG_CAPTURE_SIZE': args.size,
        'TBG_QUALITY': args.quality, 'TBG_SERVER': 'ws://127.0.0.1:9/ws',
    })
    command = ['xvfb-run', '-a', '-s', f'-screen 0 {args.size}x24', str(args.binary)]
    # A dedicated process group lets timeout clean up only this capture's Xvfb/client.
    with (directory / 'renderer.log').open('x') as log:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=log,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = process.wait(timeout=args.timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
            raise RuntimeError(f'Capture timed out; originals/log retained: {directory}')
    if code:
        raise RuntimeError(f'Capture failed ({code}); log retained: {directory}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--build-receipt', type=Path, required=True,
                        help='JSON with tooling_sha and binary_sha256 recorded after the scoped build')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--phase', choices=['before', 'after'], required=True)
    parser.add_argument('--content-sha', required=True)
    parser.add_argument('--views', nargs='+', choices=['threequarter', 'front', 'profile', 'back'],
                        default=['threequarter'])
    parser.add_argument('--quality', choices=['standard', 'low'], default='standard')
    parser.add_argument('--team', choices=['ally', 'enemy'], default='ally')
    parser.add_argument('--size', default='1920x2160')
    parser.add_argument('--timeout', type=int, default=180)
    parser.add_argument('--resume', action='store_true')
    parser.add_argument('--plan-only', action='store_true')
    parser.add_argument('units', nargs='*')
    args = parser.parse_args()
    if not re.fullmatch(r'[1-9]\d{2,3}x[1-9]\d{2,3}', args.size):
        parser.error('--size must be WIDTHxHEIGHT')
    manifest = json.loads(MANIFEST.read_text())
    cohort = [unit['id'] for unit in manifest['champions']]
    units = args.units or cohort
    if len(set(units)) != len(units) or set(units) - set(cohort):
        parser.error('Select unique champion IDs from the fixed 46-champion cohort')
    args.content_sha = git('rev-parse', args.content_sha + '^{commit}')
    tooling = git('rev-parse', 'HEAD')
    if git('diff', args.content_sha, '--', *ART_PATHS):
        parser.error('Art/combat content differs from --content-sha; freeze the correct revision')
    if git('diff', 'HEAD', '--', 'crates/tbg-client/src', 'scripts/p70'):
        parser.error('Commit capture tooling before recording evidence')
    if git('ls-files', '--others', '--exclude-standard', '--', *ART_PATHS,
           'crates/tbg-client/src', 'scripts/p70'):
        parser.error('Untracked source/tooling cannot be attributed to the frozen revision')
    args.binary = args.binary.resolve()
    args.output = args.output.resolve()
    receipt = json.loads(args.build_receipt.read_text())
    digest = sha256(args.binary)
    if receipt['binary_sha256'] != digest or receipt['tooling_sha'] != tooling:
        parser.error('Binary/build receipt does not match the frozen tooling revision')
    plan = {'phase': args.phase, 'content_sha': args.content_sha, 'tooling_sha': tooling,
            'units': units, 'views': args.views, 'forms': manifest['forms'],
            'clip_ids': manifest['clip_ids'], 'size': args.size, 'quality': args.quality,
            'team': args.team, 'binary': str(args.binary), 'binary_sha256': digest,
            'build_receipt': str(args.build_receipt.resolve()),
            'evidence_kind': 'event-free-pose-sheets', 'artifact_derivation': 'renderer-original'}
    if args.plan_only:
        print(json.dumps(plan, indent=2))
        return
    args.output.mkdir(parents=True, exist_ok=True)
    run = args.output / 'run.json'
    if run.exists():
        if not args.resume or json.loads(run.read_text()) != plan:
            parser.error('Existing run differs or --resume missing; originals are immutable')
    else:
        write_new(run, plan)
    captures = []
    for unit in units:
        for view in args.views:
            directory = args.output / unit / f'{args.team}-{view}'
            saved = directory / 'captures.json'
            if saved.exists() and args.resume:
                records = json.loads(saved.read_text())
                if any(sha256(Path(r['path_or_uri'])) != r['sha256'] for r in records):
                    raise ValueError(f'Original hash changed: {saved}')
                for record in records:
                    if png_size(Path(record['path_or_uri'])) != [record['width'], record['height']]:
                        raise ValueError(f'Original dimensions changed: {saved}')
            else:
                if directory.exists():
                    raise ValueError(f'Incomplete originals retained: {directory}; use a new run directory')
                if sha256(args.binary) != digest:
                    raise ValueError('Capture binary changed during the campaign; freeze a private copy')
                capture(directory, unit, view, args)
                records = collect(directory, unit, args, plan)
                write_new(saved, records)
            captures.extend(records)
            print(f'{unit}/{view}: {len(records)} original sheets recorded', flush=True)
    result = args.output / 'captures.json'
    if not result.exists():
        write_new(result, captures)
    print(f'{len(captures)} original pose sheets: {result}')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
