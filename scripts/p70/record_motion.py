#!/usr/bin/env python3
"""Record an immutable normal-speed gallery sequence with four champions and five forms."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
sys.dont_write_bytecode = True
from capture_roster import ROOT, MANIFEST, ART_PATHS, git, sha256, write_new
from record_trial import wait_json, recorder_ready, stop


def record(args, provenance):
    args.output.mkdir(parents=True, exist_ok=False)
    write_new(args.output / 'run.json', provenance)
    env = {k:v for k,v in os.environ.items() if not k.startswith('TBG_') and k not in ('WAYLAND_DISPLAY','WAYLAND_SOCKET')}
    env.update(WINIT_UNIX_BACKEND='x11', WGPU_BACKEND='vulkan', LP_NUM_THREADS='2', RUST_LOG='info', NO_COLOR='1',
               TBG_GALLERY='motion', TBG_REVIEW_UNITS=','.join(args.units), TBG_NAMELESS='1',
               TBG_CAPTURE_MOTION=str(args.output), TBG_CAPTURE_SIZE='1920x1440', TBG_QUALITY='standard',
               TBG_DATA_DIR=str(args.output/'private-data'), TBG_SERVER='ws://127.0.0.1:9/ws')
    client = recorder = None
    video = args.output/'original.mkv'
    with (args.output/'renderer.log').open('x') as log, (args.output/'ffmpeg.log').open('x') as encoder:
        try:
            client = subprocess.Popen([str(args.binary)],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
            ready = wait_json(args.output/'ready.json',client,time.monotonic()+40)
            if ready['units'] != args.units or ready['viewport'] != [1920,1440]:
                raise ValueError('Rendered units/viewport differ from request')
            seconds = sum(s['seconds'] for s in ready['schedule'])
            command = ['ffmpeg','-hide_banner','-loglevel','warning','-nostats','-f','x11grab','-framerate','60',
                       '-video_size','1920x1440','-i',os.environ['DISPLAY'],'-an','-c:v','libx264','-preset','ultrafast',
                       '-crf','18','-pix_fmt','yuv420p','-threads','2','-t',str(seconds+8),'-progress','pipe:1','-n',str(video)]
            recorder = subprocess.Popen(command,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=encoder)
            recorder_ready(recorder,time.monotonic()+15)
            write_new(args.output/'recording.start',{'utc':datetime.now(timezone.utc).isoformat(),'command':command})
            wait_json(args.output/'started.json',client,time.monotonic()+10)
            finished = wait_json(args.output/'finished.json',client,time.monotonic()+seconds+20)
            if recorder.poll() is not None: raise RuntimeError('Recording ended before motion completed')
            recorder.stdin.write(b'q\n'); recorder.stdin.flush()
            if recorder.wait(timeout=10) or client.wait(timeout=10): raise RuntimeError('Recorder/client failed')
        finally:
            stop(recorder); stop(client)
    probe = json.loads(subprocess.check_output(['ffprobe','-v','error','-count_frames','-select_streams','v:0',
                  '-show_entries','stream=width,height,avg_frame_rate,nb_read_frames','-of','json',str(video)],text=True))
    if int(probe['streams'][0]['nb_read_frames']) < seconds*30: raise ValueError('Incomplete video')
    log = (args.output/'renderer.log').read_text()
    if len(re.findall(r'P70_MOTION step=',log)) != len(ready['schedule']): raise ValueError('Incomplete clip schedule')
    adapter = re.search(r'AdapterInfo\s*\{[^\n]*\}', log)
    if not adapter: raise ValueError('Missing renderer identity')
    evidence = {**provenance,'kind':'real_renderer_motion_sequence','ready':ready,'finished':finished,
                'video':probe['streams'][0],'renderer':adapter.group(0),'review_status':'recorded_not_reviewed',
                'artifacts':{name:{'path':str(args.output/name),'sha256':sha256(args.output/name)}
                for name in ['original.mkv','renderer.log','ready.json','started.json','finished.json','recording.start']}}
    write_new(args.output/'captures.json',evidence)
    print(f'{args.units}: {len(ready["schedule"])} actual clip/recovery steps; {video}')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--build-receipt',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--content-sha',required=True)
    parser.add_argument('--units',nargs='+',required=True)
    parser.add_argument('--existing-display',action='store_true',help=argparse.SUPPRESS)
    args=parser.parse_args()
    ids={u['id'] for u in json.loads(MANIFEST.read_text())['champions']}
    if not 1 <= len(args.units) <= 4 or len(set(args.units)) != len(args.units) or not set(args.units)<=ids:
        parser.error('Choose one to four distinct champions')
    args.binary=args.binary.resolve(); args.output=args.output.resolve()
    args.content_sha=git('rev-parse',args.content_sha)
    head=git('rev-parse','HEAD')
    if git('diff',args.content_sha,'--',*ART_PATHS) or git('diff','HEAD','--','crates/tbg-client/src','scripts/p70'):
        parser.error('Freeze source/tooling before capture')
    if git('ls-files','--others','--exclude-standard','--',*ART_PATHS,'crates/tbg-client/src','scripts/p70'):
        parser.error('Untracked source/tooling cannot identify the recording')
    receipt=json.loads(args.build_receipt.read_text()); digest=sha256(args.binary)
    if receipt['tooling_sha']!=head or receipt['binary_sha256']!=digest: parser.error('Binary/build receipt mismatch')
    provenance={'content_sha':args.content_sha,'tooling_sha':head,'binary_sha256':digest,'units':args.units,
                'phase':'after','forms':['Base','A1','A2','B1','B2'],'speed':1,'quality':'standard',
                'build_receipt':str(args.build_receipt.resolve()),'platform':'linux','derivation':'original renderer recording'}
    if args.output.exists(): parser.error('Immutable output already exists')
    if args.existing_display: return record(args,provenance)
    subprocess.run(['xvfb-run','-a','-s','-screen 0 1920x1440x24',sys.executable,str(Path(__file__).resolve()),
                    *sys.argv[1:],'--existing-display'],cwd=ROOT,check=True,timeout=100)

if __name__=='__main__':
    try: main()
    except (OSError,ValueError,RuntimeError,subprocess.SubprocessError) as error:
        print(error,file=sys.stderr);sys.exit(1)
