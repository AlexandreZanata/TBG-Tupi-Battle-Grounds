#!/usr/bin/env python3
"""Record one real P70 roster training engagement, preserving video, canonical events and provenance.

The client pauses only during startup. ffmpeg begins before ordinary combat resumes.
No build, simulation speed change, synthetic damage or effect is introduced here.
"""

import argparse
from collections import Counter
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import select
import signal
import subprocess
import sys
import time

sys.dont_write_bytecode = True
from capture_roster import ART_PATHS, MANIFEST, ROOT, git, sha256, write_new

FORMS = {'Base': (0, 0), 'A1': (1, 0), 'A2': (2, 0), 'B1': (1, 1), 'B2': (2, 1)}
DISTANCES = {'Trident': 1500, 'PyreWarden': 2000, 'OldFinn': 4300}


def wait_json(path, process, deadline):
    while time.monotonic() < deadline:
        try:
            return json.loads(path.read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            if process.poll() is not None:
                raise RuntimeError(f'Client exited ({process.returncode}) before {path.name}')
            time.sleep(0.05)
    raise RuntimeError(f'Timed out waiting for {path.name}; originals retained')


def recorder_ready(process, deadline):
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f'ffmpeg exited before recording ({process.returncode})')
        readable, _, _ = select.select([process.stdout], [], [], 0.1)
        if readable:
            line = process.stdout.readline().decode(errors='replace').strip()
            if line.startswith('frame=') and int(line.split('=', 1)[1]) > 0:
                return
    raise RuntimeError('ffmpeg did not produce its first frame')


def stop(process):
    if process is None or process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)


def environment(args):
    env = {key: value for key, value in os.environ.items()
           if not key.startswith('TBG_') and key not in ('WAYLAND_DISPLAY', 'WAYLAND_SOCKET')}
    tier, branch = FORMS[args.form]
    env.update({
        'WINIT_UNIX_BACKEND': 'x11', 'WGPU_BACKEND': 'vulkan', 'LP_NUM_THREADS': '2',
        'RUST_LOG': 'info', 'NO_COLOR': '1', 'TBG_DATA_DIR': str(args.output / 'private-data'),
        'TBG_CAPTURE_POLISH_LIVE': str(args.output), 'TBG_CAPTURE_SIZE': args.size,
        'TBG_UNIT': args.unit, 'TBG_TIER': str(tier), 'TBG_BRANCH': str(branch),
        'TBG_TRIAL_SEAT': args.seat, 'TBG_TRIAL_SEED': str(args.seed),
        'TBG_TRIAL_SECONDS': str(args.seconds),
        'TBG_QUALITY': args.quality, 'TBG_SERVER': 'ws://127.0.0.1:9/ws',
    })
    if args.distance is not None:
        env['TBG_TRIAL_DISTANCE'] = str(args.distance)
    return env


def record(args, provenance):
    args.output.mkdir(parents=True, exist_ok=False)
    write_new(args.output / 'run.json', provenance)
    env = environment(args)
    client = recorder = None
    deadline = time.monotonic() + args.timeout
    video = args.output / 'original.mkv'
    command = ['ffmpeg', '-hide_banner', '-loglevel', 'warning', '-nostats',
               '-f', 'x11grab', '-framerate', str(args.fps), '-video_size', args.size,
               '-i', os.environ['DISPLAY'], '-an', '-c:v', 'libx264', '-preset', 'ultrafast',
               '-crf', '18', '-pix_fmt', 'yuv420p', '-threads', '2',
               '-t', str(args.seconds + 5), '-progress', 'pipe:1', '-n', str(video)]
    with (args.output / 'renderer.log').open('x') as render_log, (args.output / 'ffmpeg.log').open('x') as encoder_log:
        try:
            client = subprocess.Popen([str(args.binary)], cwd=ROOT, env=env,
                                      stdout=render_log, stderr=subprocess.STDOUT)
            ready = wait_json(args.output / 'ready.json', client, deadline)
            if (ready['unit'], ready['shooter_seat'], ready['tier'], ready['branch']) != (
                    args.unit, args.seat, *FORMS[args.form]):
                raise ValueError('Client ready metadata differs from the requested scenario')
            if not isinstance(ready.get('subject_id'), int):
                raise ValueError('Subject identity missing from the actual reset training state')
            if ready['active_ticks'] != 0:
                raise ValueError('Recording must start from the reset canonical training state')
            if ready['viewport'] != list(map(int, args.size.split('x'))):
                raise ValueError('Actual viewport differs from requested recording size')
            recorder = subprocess.Popen(command, cwd=ROOT, env=env,
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=encoder_log)
            recorder_ready(recorder, min(deadline, time.monotonic() + 15))
            write_new(args.output / 'recording.start', {
                'recorder_first_frame_utc': datetime.now(timezone.utc).isoformat(),
                'command': command, 'requested_fps': args.fps,
                'encoding': 'Original X11 renderer-window recording; H.264 CRF 18, no crop or retiming.',
            })
            wait_json(args.output / 'started.json', client, deadline)
            finished = wait_json(args.output / 'finished.json', client, deadline)
            if recorder.poll() is not None:
                raise RuntimeError('Recorder stopped before the engagement finished')
            recorder.stdin.write(b'q\n')
            recorder.stdin.flush()
            if recorder.wait(timeout=10):
                raise RuntimeError('ffmpeg failed to finish the original recording')
            if client.wait(timeout=6):
                raise RuntimeError('Client reported a capture failure')
        finally:
            stop(recorder)
            stop(client)
    collect(args, provenance, ready, finished, video)


def collect(args, provenance, ready, finished, video):
    probe = json.loads(subprocess.check_output([
        'ffprobe', '-v', 'error', '-count_frames', '-select_streams', 'v:0',
        '-show_entries', 'stream=width,height,avg_frame_rate,nb_read_frames,duration',
        '-of', 'json', str(video)], text=True, timeout=30))
    streams = probe.get('streams', [])
    if len(streams) != 1 or int(streams[0].get('nb_read_frames', 0)) == 0:
        raise ValueError('Recording has no decodable video frames')
    records = [json.loads(line) for line in (args.output / 'events.jsonl').read_text().splitlines()]
    counts = Counter(name for record in records for event in record['events']
                     for name in (event if isinstance(event, dict) else [str(event)]))
    subject = Counter()
    for record in records:
        for event in record['events']:
            if not isinstance(event, dict):
                continue
            for name, payload in event.items():
                if not isinstance(payload, dict):
                    continue
                if name == 'AttackStart' and payload.get('from') == {'Unit': ready['subject_id']}:
                    subject[name] += 1
                if name in ('AbilityStarted', 'AbilityResolved', 'ProfaneFlame') and payload.get('unit') == ready['subject_id']:
                    subject[name] += 1
    if not counts['AttackStart'] or not counts['Hit']:
        raise ValueError('No actual AttackStart/Hit engagement observed; evidence is incomplete')
    if args.unit == 'PyreWarden' and not subject['ProfaneFlame']:
        raise ValueError('No actual ProfaneFlame observed; evidence is incomplete')
    log = (args.output / 'renderer.log').read_text(errors='replace')
    adapter = re.search(r'AdapterInfo\s*\{[^\n]*\}', log)
    if not adapter:
        raise ValueError('Renderer identity is missing from the real capture log')
    artifacts = {}
    for name in ['original.mkv', 'ready.json', 'started.json', 'finished.json',
                 'events.jsonl', 'renderer.log', 'ffmpeg.log', 'recording.start']:
        path = args.output / name
        artifacts[name] = {'path': str(path), 'sha256': sha256(path), 'bytes': path.stat().st_size}
    summary = {
        **provenance, 'kind': 'real_training_temporal_recording',
        'camera': ready['camera'], 'dpi': ready['dpi'], 'renderer': adapter.group(0),
        'video': streams[0], 'event_counts': dict(counts), 'subject_id': ready['subject_id'],
        'subject_source_event_counts': dict(subject), 'actual_initial_distance': ready['initial_distance'],
        'summary': finished, 'artifacts': artifacts,
        'simulation_seconds_observed': finished['observed_active_ticks'] / finished['tick_hz'],
        'limitations': ['Single configured engagement; all clips/transitions and P63 Victory remain separate.',
                        'Zero subject source events means no subject primary-action evidence, even if other units fought.',
                        'Conditional healing, passengers, summons and fusion need their own observed scenarios.',
                        'Event counts alone do not verify projectile readability, Echo counts or visual contact.',
                        'Rendering throughput and active simulation time must be compared with wall_seconds.',
                        'Seat B is a real opponent-side shooter viewed through the normal Seat A laboratory camera.'],
    }
    write_new(args.output / 'captures.json', [summary])
    print(f'{args.unit}/{args.form}/seat-{args.seat}: {counts["AttackStart"]} real attack starts; {video}')


def frozen_provenance(args, parser):
    args.content_sha = git('rev-parse', args.content_sha + '^{commit}')
    tooling = git('rev-parse', 'HEAD')
    if git('diff', args.content_sha, '--', *ART_PATHS):
        parser.error('Art/combat content differs from --content-sha')
    if git('diff', 'HEAD', '--', 'crates/tbg-client/src', 'scripts/p70'):
        parser.error('Commit tooling before recording')
    if git('ls-files', '--others', '--exclude-standard', '--', *ART_PATHS,
           'crates/tbg-client/src', 'scripts/p70'):
        parser.error('Untracked source/tooling cannot identify the frozen recording')
    receipt = json.loads(args.build_receipt.read_text())
    digest = sha256(args.binary)
    if receipt['binary_sha256'] != digest or receipt['tooling_sha'] != tooling:
        parser.error('Binary/build receipt does not match frozen tooling')
    return {'phase': args.phase, 'content_sha': args.content_sha, 'tooling_sha': tooling,
            'binary': str(args.binary), 'binary_sha256': digest,
            'build_receipt': str(args.build_receipt.resolve()), 'unit': args.unit,
            'form': args.form, 'seat': args.seat, 'quality': args.quality, 'seed': args.seed,
            'initial_distance': args.distance, 'seconds': args.seconds, 'fps': args.fps,
            'size': args.size, 'artifact_derivation': 'original-renderer-window-recording',
            'platform': 'linux', 'scenario': 'P70 canonical laboratory engagement v2'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--build-receipt', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--phase', choices=['before', 'after'], required=True)
    parser.add_argument('--content-sha', required=True)
    cohort = [unit['id'] for unit in json.loads(MANIFEST.read_text())['champions']]
    parser.add_argument('--unit', choices=cohort, required=True)
    parser.add_argument('--form', choices=list(FORMS), default='Base')
    parser.add_argument('--seat', choices=['A', 'B'], default='A')
    parser.add_argument('--quality', choices=['standard', 'low'], default='standard')
    parser.add_argument('--seed', type=int, default=70)
    parser.add_argument('--distance', type=int)
    parser.add_argument('--seconds', type=int, choices=range(12, 19), default=16)
    parser.add_argument('--fps', type=int, choices=[30, 60], default=60)
    parser.add_argument('--size', default='1920x1080')
    parser.add_argument('--timeout', type=int, default=120)
    parser.add_argument('--plan-only', action='store_true')
    parser.add_argument('--existing-display', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    args.binary, args.output = args.binary.resolve(), args.output.resolve()
    args.distance = args.distance if args.distance is not None else DISTANCES.get(args.unit)
    if (args.distance is not None and not 500 <= args.distance <= 12000) or not 0 <= args.seed < 2**64:
        parser.error('Distance must be 500..12000; seed must fit u64')
    if not re.fullmatch(r'[1-9]\d{2,3}x[1-9]\d{2,3}', args.size):
        parser.error('Size must be WIDTHxHEIGHT')
    width, height = map(int, args.size.split('x'))
    if width < 1280 or height < 720 or width <= height or width % 2 or height % 2:
        parser.error('Use an even-pixel desktop landscape viewport of at least 1280x720')
    provenance = frozen_provenance(args, parser)
    if args.plan_only:
        print(json.dumps(provenance, indent=2))
        return
    if args.output.exists():
        parser.error('Output already exists; recordings are immutable, choose a new directory')
    if args.existing_display:
        record(args, provenance)
        return
    command = ['xvfb-run', '-a', '-s', f'-screen 0 {args.size}x24',
               sys.executable, str(Path(__file__).resolve()), *sys.argv[1:], '--existing-display']
    process = subprocess.Popen(command, cwd=ROOT, start_new_session=True)
    try:
        code = process.wait(timeout=args.timeout + 45)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=5)
        raise RuntimeError('Capture group timed out; originals/logs retained')
    if code:
        raise RuntimeError(f'Trial recording failed ({code}); originals/logs retained')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
