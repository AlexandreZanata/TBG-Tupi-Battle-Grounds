#!/usr/bin/env python3
"""Normalize the illustrated lobby set and measure its served art fidelity."""
from concurrent.futures import ProcessPoolExecutor
import gzip
import hashlib
import importlib.metadata
import json
from pathlib import Path
import shutil

from PIL import Image, ImageOps
from art_pipeline import convert_one, compare

ROOT = Path(__file__).resolve().parents[1]
LOCAL = ROOT / '.local/art-masters/lobby-v2'
OUT = ROOT / 'crates/tbg-client/assets/lobby'


def convert(job):
    name, shown, measured = convert_one(job)
    master = Image.open(LOCAL / (name + '.png')).convert('RGBA')
    size = tuple(measured['display_size'])
    if measured['fallback'] and job['lossless']:
        dest = OUT / (name + '.webp')
        master.resize((size[0]*4, size[1]*4), Image.Resampling.LANCZOS).save(
            dest, 'WEBP', quality=90, method=6, exact=True, lossless=True)
        measured['webp_bytes'] = dest.stat().st_size
        measured['webp_display'] = compare(master.resize(size, Image.Resampling.LANCZOS),
            Image.open(dest).resize(size, Image.Resampling.LANCZOS))
    served = measured['webp_display'] if measured['fallback'] else measured['display']
    if served['ssim'] < .97 or served['alpha_mae'] > 1:
        raise ValueError(name + ': served fidelity gate failed: ' + str(served))
    measured['served_sha256'] = hashlib.sha256((OUT / shown).read_bytes()).hexdigest()
    measured['master_sha256'] = hashlib.sha256((LOCAL / (name + '.png')).read_bytes()).hexdigest()
    return name, shown, measured


def run():
    LOCAL.mkdir(parents=True, exist_ok=True)
    records = json.loads((ROOT / '.local/lobby-v2/generations.json').read_text())
    catalog = dict(schema=1, artBudgetBytes=3_000_000, assets=[])
    manifest = json.loads((OUT / 'art.json').read_text())
    jobs = []
    for r in records:
        name = r['name']
        original = LOCAL / (name + '-original.png')
        shutil.copy2(r['path'], original)
        image = Image.open(original).convert('RGBA')
        size = tuple(r['masterSize'])
        if name in ['logo-v2', 'crest-v2']:
            contained = ImageOps.contain(image, tuple(round(n*.96) for n in size), Image.Resampling.LANCZOS)
            master = Image.new('RGBA', size)
            master.alpha_composite(contained, ((size[0]-contained.width)//2, (size[1]-contained.height)//2))
        else:
            master = ImageOps.fit(image, size, Image.Resampling.LANCZOS)
        master.save(LOCAL / (name + '.png'))
        ref = dict(svg=name+'.svg')
        catalog['assets'].append(dict(id=name, masterSize=list(size), displayWidth=r['displayWidth'],
            lossless=r['lossless'], asset=ref))
        jobs.append(dict(name=name, local=str(LOCAL), out=str(OUT), ref=ref,
            display_width=r['displayWidth'], budget=250_000, target=40_000, lossless=r['lossless']))
    metrics = {}
    with ProcessPoolExecutor(max_workers=2) as pool:
        for name, shown, measured in pool.map(convert, jobs):
            next(a for a in catalog['assets'] if a['id']==name)['asset']['display']=shown
            key = {'crest-v2':'emblem'}.get(name, name.removesuffix('-v2'))
            manifest[key] = 'assets/lobby/'+shown
            metrics[name] = measured
            (LOCAL / 'quality.json').write_text(json.dumps(metrics, indent=2)+'\n')
            print(name, json.dumps(measured), flush=True)
    files = {v for a in catalog['assets'] for v in a['asset'].values()}
    total = sum(len(gzip.compress((OUT/f).read_bytes(), mtime=0)) if f.endswith('.svg') else (OUT/f).stat().st_size for f in files)
    if total > catalog['artBudgetBytes']:
        raise ValueError('Lobby art exceeds budget: '+str(total))
    (OUT / 'presentation.json').write_text(json.dumps(catalog, indent=2)+'\n')
    (OUT / 'art.json').write_text(json.dumps(manifest, indent=2)+'\n')
    (LOCAL / 'versions.json').write_text(json.dumps({n:importlib.metadata.version(n)
        for n in ['vtracer','Pillow','numpy','scipy']},indent=2)+'\n')
    print('TOTAL lobby v2 SVG gzip + WebP bytes:', total, 'budget:', catalog['artBudgetBytes'], flush=True)


if __name__ == '__main__':
    run()
