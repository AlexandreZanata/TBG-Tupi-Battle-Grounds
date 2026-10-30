#!/usr/bin/env python3
"""Shared normalization, vectorization and measured raster fallback for game art."""
import argparse
from concurrent.futures import ProcessPoolExecutor
import gzip
import importlib.metadata
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time

import numpy as np
from PIL import Image
from scipy.ndimage import uniform_filter
import vtracer
from art_catalog import entries, FOLDERS, MANIFESTS, TOTAL_BUDGETS

SETTINGS = dict(colormode='color', hierarchical='stacked', mode='spline',
                filter_speckle=2, color_precision=8, layer_difference=8,
                corner_threshold=60, length_threshold=4.0, splice_threshold=45,
                path_precision=3)
settings = SETTINGS

def render(svg, output, size):
    try:
        import cairosvg
    except ImportError:
        sharp = os.environ.get('ART_SHARP') or os.environ.get('ARCANA_SHARP')
        if not sharp:
            raise SystemExit('Install cairosvg locally or set ART_SHARP to a local sharp module.')
        density = 72 * size[0] / int(re.search(r'width="(\d+)"', svg.read_text()[:400]).group(1))
        js = 'require(process.argv[1])(process.argv[2],{limitInputPixels:false,density:+process.argv[6]}).resize(+process.argv[4],+process.argv[5]).png().toFile(process.argv[3])'
        subprocess.run(['node', '-e', js, sharp, str(svg), str(output), str(size[0]), str(size[1]), str(density)], check=True)
    else:
        cairosvg.svg2png(url=str(svg), write_to=str(output), output_width=size[0], output_height=size[1])


def compare(original, reconstructed):
    # RGB SSIM: standard 7x7 uniform window, K1=.01, K2=.03, sample covariance.
    # Average its three channel maps, weighted by reference alpha. Crop window border.
    a = np.asarray(original.convert('RGBA'), dtype=np.float64)
    b = np.asarray(reconstructed.convert('RGBA'), dtype=np.float64)
    x, y, weight = a[:, :, :3], b[:, :, :3], a[:, :, 3] / 255
    ux, uy = uniform_filter(x, (7, 7, 1)), uniform_filter(y, (7, 7, 1))
    covnorm = 49 / 48
    vx = covnorm * (uniform_filter(x*x, (7, 7, 1)) - ux*ux)
    vy = covnorm * (uniform_filter(y*y, (7, 7, 1)) - uy*uy)
    vxy = covnorm * (uniform_filter(x*y, (7, 7, 1)) - ux*uy)
    c1, c2 = (0.01 * 255)**2, (0.03 * 255)**2
    score = ((2*ux*uy+c1)*(2*vxy+c2)) / ((ux*ux+uy*uy+c1)*(vx+vy+c2))
    inner, w = score[3:-3, 3:-3].mean(axis=2), weight[3:-3, 3:-3]
    ssim = float((inner*w).sum() / w.sum())
    mae = float((np.abs(x-y).mean(axis=2)*weight).sum() / weight.sum())
    alpha_mae = float(np.abs(a[:, :, 3]-b[:, :, 3]).mean())
    return dict(ssim=round(ssim, 6), mae=round(mae, 6), alpha_mae=round(alpha_mae, 6))


def trace(master, name, width, label, local):
    resized = master.resize((width, round(master.height * width / master.width)), Image.Resampling.LANCZOS)
    source, svg = local / (name + '-' + label + '.png'), local / (name + '-' + label + '.svg')
    resized.save(source)
    vtracer.convert_image_to_svg_py(str(source), str(svg), **settings)
    # Lossless XML cleanup only; never run a path simplifier or round path data.
    body = re.sub(r'<!--.*?-->\s*', '', svg.read_text(), flags=re.S)
    body = body.replace(f'width="{resized.width}" height="{resized.height}"',
                        f'width="{master.width}" height="{master.height}" viewBox="0 0 {resized.width} {resized.height}"')
    svg.write_text(body)
    raw = svg.read_bytes()
    return svg, dict(trace_width=width, raw=len(raw), gzip=len(gzip.compress(raw, mtime=0)))


def convert_one(job):
    started = time.monotonic()
    local, out = Path(job['local']), Path(job['out'])
    name, display_width = job['name'], job['display_width']
    master = Image.open(local / (name + '.png')).convert('RGBA')
    display_size = (display_width, round(master.height * display_width / master.width))
    svg = out / job['ref']['svg']
    full_render = local / (name + '-full-render.png')
    if job.get('measure_only'):
        initial = job['previous']['baseline']
        measured = job['previous']['svg']
    else:
        baseline, initial = trace(master, name, master.width, 'full-trace', local)
        render(baseline, full_render, master.size)
        initial.update(compare(master, Image.open(full_render)))
        selected, measured = baseline, initial
        if initial['gzip'] > job['target']:
            widths = [640, 512, 448, 384, 320, 256, 224, 192, 160, 128, 112, 96, 80, 64]
            for width in widths:
                selected, measured = trace(master, name, width, 'w' + str(width), local)
                if measured['gzip'] <= job['target']:
                    break
        shutil.copy2(selected, svg)
    render(svg, full_render, master.size)
    full = compare(master, Image.open(full_render))
    display_render = local / (name + '-display-render.png')
    render(svg, display_render, display_size)
    display = compare(master.resize(display_size, Image.Resampling.LANCZOS), Image.open(display_render))
    fallback = (full['ssim'] < .95 or display['ssim'] < .97 or measured['gzip'] > job['budget']
                or full['alpha_mae'] > 1 or display['alpha_mae'] > 1)
    webp_bytes, webp_metrics = 0, None
    shown = svg.name
    if fallback:
        webp = out / (name + '.webp')
        master.resize((display_size[0]*2, display_size[1]*2), Image.Resampling.LANCZOS).save(
            webp, 'WEBP', quality=90, method=6, exact=True, lossless=job['lossless'])
        shown = webp.name
        webp_bytes = webp.stat().st_size
        webp_metrics = compare(master.resize(display_size, Image.Resampling.LANCZOS),
                               Image.open(webp).resize(display_size, Image.Resampling.LANCZOS))
    return name, shown, dict(full=full, display=display, baseline=initial, svg=measured,
        budget=job['budget'], display_size=display_size, fallback=fallback,
        webp_bytes=webp_bytes, webp_display=webp_metrics, webp_lossless=job['lossless'],
        settings=SETTINGS, seconds=round(time.monotonic()-started, 2))


def normalize(root):
    records_path = root / '.local/art-generations.json'
    records = json.loads(records_path.read_text())
    for job in records:
        if not job.get('path'):
            continue
        local = root / '.local/art-masters' / job['set']
        local.mkdir(parents=True, exist_ok=True)
        source = Path(job['path'])
        original = local / (job['name'] + '-original.png')
        master = local / (job['name'] + '.png')
        if not master.exists() or job.get('replace'):
            shutil.copy2(source, original)
            image = Image.open(original).convert('RGBA')
            job['generatedSize'] = list(image.size)
            image.resize(tuple(job['masterSize']), Image.Resampling.LANCZOS).save(master)
            job.pop('replace', None)
    records_path.write_text(json.dumps(records, indent=2)+'\n')


def run(args):
    root = Path(__file__).resolve().parents[1]
    if args.normalize:
        normalize(root)
        return
    out = root / 'crates/tbg-client/assets' / FOLDERS[args.art_set]
    manifest = out / MANIFESTS[args.art_set]
    data = json.loads(manifest.read_text())
    data['artBudgetBytes'] = TOTAL_BUDGETS[args.art_set]
    local = root / '.local/art-masters' / args.art_set
    local.mkdir(parents=True, exist_ok=True)
    all_entries = list(entries(data, args.art_set))
    requested = set(args.names)
    unknown = requested - {a['name'] for a in all_entries}
    if unknown:
        raise SystemExit('Unknown art asset: ' + ', '.join(sorted(unknown)))
    jobs = [dict(a, local=str(local), out=str(out), measure_only=args.measure_only) for a in all_entries if not requested or a['name'] in requested]
    records_path = local / 'quality.json'
    records = json.loads(records_path.read_text()) if records_path.exists() else {}
    if args.measure_only:
        for job in jobs:
            job['previous'] = records[job['name']]
    with ProcessPoolExecutor(max_workers=args.workers) as pool:
        for name, shown, measured in pool.map(convert_one, jobs):
            next(a for a in all_entries if a['name'] == name)['ref']['display'] = shown
            records[name] = measured
            records_path.write_text(json.dumps(records, indent=2)+'\n')
            manifest.write_text(json.dumps(data, indent=2, ensure_ascii=False)+'\n')
            print(name, json.dumps(measured), flush=True)
    # Count every shipped picture, including the existing compact Stronghold sprite.
    total = sum(len(gzip.compress(p.read_bytes(), mtime=0)) if p.suffix == '.svg' else p.stat().st_size
                for p in out.iterdir() if p.suffix in ['.svg', '.webp'])
    print('TOTAL', args.art_set, 'gzip SVG + WebP bytes:', total, 'budget:', TOTAL_BUDGETS[args.art_set], flush=True)
    versions = {n:importlib.metadata.version(n) for n in ['vtracer','Pillow','numpy','scipy']}
    (local / 'versions.json').write_text(json.dumps(versions, indent=2)+'\n')
    if total > TOTAL_BUDGETS[args.art_set]:
        raise SystemExit('Art set exceeds its total budget')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('art_set', choices=TOTAL_BUDGETS, nargs='?', default='arcana')
    parser.add_argument('names', nargs='*')
    parser.add_argument('--workers', type=int, choices=range(1, 9), default=1)
    parser.add_argument('--normalize', action='store_true')
    parser.add_argument('--measure-only', action='store_true')
    run(parser.parse_args())
