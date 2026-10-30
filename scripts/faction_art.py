#!/usr/bin/env python3
"""Normalize Faction molds and emblems and measure vector/raster fidelity."""
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
LOCAL = ROOT / '.local/art-masters/factions'
OUT = ROOT / 'crates/tbg-client/assets/factions'


def convert(job):
    name, shown, measured = convert_one(job)
    master = Image.open(LOCAL / (name + '.png')).convert('RGBA')
    size = tuple(measured['display_size'])
    if measured['fallback'] and job['lossless']:
        # Four samples per display pixel preserve the silhouette of future small emblems.
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
    OUT.mkdir(parents=True, exist_ok=True)
    records = json.loads((ROOT / '.local/faction-art/generations.json').read_text())
    designs = json.loads((ROOT / '.local/faction-art/designs.json').read_text())
    data = dict(schema=1, artBudgetBytes=2_000_000, factions=[])
    for d in designs:
        data['factions'].append(dict(name=d['faction'], identity=d['identity'], color=d['color'],
            secondary=d['secondary'], frame=dict(svg='frame-'+d['slug']+'.svg'),
            emblem=dict(svg='emblem-'+d['slug']+'.svg'),
            layout=dict(portraitTop=17, portraitHeight=47, nameY=71.5 if d['faction']=='Machines' else 74,
                roleY=77 if d['faction']=='Machines' else 80,
                costY={'Humans':85.6,'Undead':84.3,'Machines':82.9,'Plants':85.1}[d['faction']])))
    jobs = []
    for record in records:
        name = record['name']
        original = LOCAL / (name + '-original.png')
        shutil.copy2(record['path'], original)
        image = Image.open(original).convert('RGBA')
        size = tuple(record['masterSize'])
        # Contain, never stretch or crop. A shared 2% margin protects each silhouette.
        inner = tuple(round(n * .96) for n in size)
        contained = ImageOps.contain(image, inner, Image.Resampling.LANCZOS)
        master = Image.new('RGBA', size)
        master.alpha_composite(contained, ((size[0]-contained.width)//2, (size[1]-contained.height)//2))
        master.save(LOCAL / (name + '.png'))
        row = next(f for f in data['factions'] if f['name'] == record['faction'])
        jobs.append(dict(name=name, local=str(LOCAL), out=str(OUT), ref=row[record['kind']],
            display_width=record['displayWidth'], budget=record['budget'], target=record['target'],
            lossless=record['lossless']))
    metrics = {}
    with ProcessPoolExecutor(max_workers=2) as pool:
        for name, shown, measured in pool.map(convert, jobs):
            record = next(r for r in records if r['name'] == name)
            next(f for f in data['factions'] if f['name'] == record['faction'])[record['kind']]['display'] = shown
            metrics[name] = measured
            (LOCAL / 'quality.json').write_text(json.dumps(metrics, indent=2)+'\n')
            print(name, json.dumps(measured), flush=True)
    total = sum(len(gzip.compress(p.read_bytes(), mtime=0)) if p.suffix == '.svg' else p.stat().st_size
        for p in OUT.iterdir() if p.suffix in ('.svg', '.webp'))
    if total > data['artBudgetBytes']:
        raise ValueError('Faction art exceeds budget: '+str(total))
    (OUT / 'factions.json').write_text(json.dumps(data, indent=2)+'\n')
    (LOCAL / 'versions.json').write_text(json.dumps({n:importlib.metadata.version(n)
        for n in ['vtracer', 'Pillow', 'numpy', 'scipy']}, indent=2)+'\n')
    print('TOTAL Faction SVG gzip + WebP bytes:', total, 'budget:', data['artBudgetBytes'], flush=True)


if __name__ == '__main__':
    run()
