#!/usr/bin/env python3
"""Enforce common desktop routing and compare a complete same-source packaged UI matrix."""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT=Path(__file__).resolve().parents[1]
CONTRACT=ROOT/'docs/ui/desktop-ui-contract.json'

def source_errors(root=ROOT):
    errors=[]
    sources=list((root/'crates/tbg-client/src/ui').rglob('*.rs'))
    sources += [root/'crates/tbg-client/src'/name for name in ['main.rs','lib.rs','app.rs','sync.rs','sync/messages.rs','local.rs','training/preview.rs','visuals/assets.rs','capture.rs']]
    pattern=re.compile(r'\bcfg(?:!|_attr)?\s*\([^)]*(?:target_os\s*=\s*"(?:linux|windows)"|\bunix\b|\bwindows\b)',re.S)
    for path in sources:
        raw=path.read_text();lines=raw.splitlines()
        for match in pattern.finditer(raw):
            line=raw.count('\n',0,match.start())+1
            if path.name=='main.rs' and 'windows_subsystem' in lines[line-1]:continue
            errors.append(f'{path.relative_to(root)}:{line}: OS-specific presentation condition')
    build=(root/'crates/tbg-client/build.rs').read_text()
    for required in ['CARGO_CFG_TARGET_OS','cargo:rustc-check-cfg=cfg(tbg_desktop)','cargo:rustc-cfg=tbg_desktop','let native_ui = desktop ||', 'native_art_manifest(native_ui)']:
        if required not in build:errors.append(f'build.rs is missing {required}')
    return errors

def key(capture):
    return (capture['screen'],capture['language'],tuple(capture['viewport']),capture['effective_ui_scale'],capture['frame'])

def expected(contract):
    return {(screen,lang,tuple(scene['viewport']),scene['effective_ui_scale'],frame)
            for screen in contract['screens'] for lang in contract['languages'] for scene in contract['scenarios']
            for frame in (contract['motion_frames'] if screen in contract['motion_screens'] else [0])}

def contained(directory,name):
    path=(directory/name).resolve()
    if not path.is_relative_to(directory.resolve()) or not path.is_file():raise ValueError('capture path is missing or escapes its directory')
    return path

def compare(left,right,contract):
    from PIL import Image,ImageChops
    errors=[];results=[]
    indexes=[json.loads((d/'index.json').read_text()) for d in (left,right)]
    for index,platform in zip(indexes,['linux','windows']):
        if index.get('schema')!=1 or index.get('platform')!=platform:raise ValueError('incorrect capture schema/platform')
        for field in ['source_commit','cargo_lock_sha256','assets_sha256','binary_sha256','package_sha256']:
            width=40 if field=='source_commit' else 64
            if not re.fullmatch('[0-9a-f]{'+str(width)+'}',index.get(field,'')):raise ValueError(f'invalid {field}')
        if any(not index.get('environment',{}).get(field) for field in ['os','gpu','driver','backend']):raise ValueError('capture environment is incomplete')
    for field in contract['matching_identity_fields']:
        if field not in indexes[0] or indexes[0].get(field)!=indexes[1].get(field):errors.append(f'mismatched identity/settings: {field}')
    actual=[]
    for index in indexes:
        mapping={key(c):c for c in index['captures']}
        if len(mapping)!=len(index['captures']):errors.append('duplicate capture scenario')
        if set(mapping)!=expected(contract):errors.append('missing or unexpected required matrix scenarios')
        actual.append(mapping)
    if errors:return {'passed':False,'errors':errors,'captures':results}
    for scenario in sorted(actual[0]):
        a,b=[Image.open(contained(directory,mapping[scenario]['file'])).convert('RGBA') for directory,mapping in zip((left,right),actual)]
        if a.size!=scenario[2] or b.size!=scenario[2]:errors.append(f'wrong framebuffer dimensions: {scenario}');continue
        pixels_a,pixels_b=a.tobytes(),b.tobytes()
        changed=0 if pixels_a==pixels_b else sum(pixels_a[i:i+4]!=pixels_b[i:i+4] for i in range(0,len(pixels_a),4))
        results.append({'scenario':scenario,'different_pixels':changed})
        if changed:
            errors.append(f'visible mismatch: {scenario} ({changed} pixels)')
            # Output is diagnostic only; no masking or rewritten baseline is accepted.
            name=hashlib.sha256(repr(scenario).encode()).hexdigest()[:16]+'.png'
            diff_dir=right/'comparison-diffs';diff_dir.mkdir(exist_ok=True)
            ImageChops.difference(a,b).convert('RGB').save(diff_dir/name)
    return {'passed':not errors,'errors':errors,'captures':results}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compare',nargs=2,type=Path,metavar=('LINUX','WINDOWS'))
    parser.add_argument('--output',type=Path)
    args=parser.parse_args();errors=source_errors()
    result={'passed':not errors,'errors':errors,'level':'source routing only; packaged visual equivalence unverified'}
    if args.compare and not errors:
        try:result=compare(*args.compare,json.loads(CONTRACT.read_text()))
        except (ValueError,KeyError,OSError,ImportError) as error:result={'passed':False,'errors':[str(error)]}
    if args.output:args.output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2));raise SystemExit(0 if result['passed'] else 1)

if __name__=='__main__':main()
