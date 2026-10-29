#!/usr/bin/env bash
# Rebuild original-image vectors, alpha-preserving WebP fallbacks and quality evidence.
# Optional local tools only (never Cargo/project dependencies):
#   uv venv --system-site-packages .local/art-tools/venv
#   uv pip install --python .local/art-tools/venv/bin/python vtracer==0.6.15 pillow numpy scipy cairosvg
# Alternatively set ARCANA_SHARP to a locally installed sharp module for SVG rendering.
#   ARCANA_PYTHON=.local/art-tools/venv/bin/python scripts/arcana-art.sh [asset-name ...]
# Add --measure-only to recheck existing SVGs and rebuild only WebP fallbacks.
# Masters: .local/art-masters/arcana/{name}.png; output: client/assets/arcana/.
set -euo pipefail
cd "$(dirname "$0")/.."
"${ARCANA_PYTHON:-python3}" - "$@" <<'PY'
import gzip
import importlib.metadata
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

import numpy as np
from PIL import Image
from scipy.ndimage import uniform_filter
import vtracer

ROOT = Path.cwd()
LOCAL = ROOT / '.local/art-masters/arcana'
OUT = ROOT / 'crates/tbg-client/assets/arcana'
MANIFEST = OUT / 'arcana.json'
data = json.loads(MANIFEST.read_text())
assets = [(f['id'], f['emblem'], 160, 250_000) for f in data['families']]
assets += [('banner-' + f['id'], f['banner'], 900, 400_000) for f in data['families']]
assets += [(n['id'], n['asset'], 72, 150_000) for n in data['nodes']]
measure_only = '--measure-only' in sys.argv[1:]
requested = set(sys.argv[1:]) - {'--measure-only'}
if requested - {a[0] for a in assets}:
    raise SystemExit('Unknown asset: ' + ', '.join(sorted(requested - {a[0] for a in assets})))
settings = dict(colormode='color', hierarchical='stacked', mode='spline',
                filter_speckle=2, color_precision=8, layer_difference=8,
                corner_threshold=60, length_threshold=4.0, splice_threshold=45,
                path_precision=3)


def render(svg, output, size):
    try:
        import cairosvg
    except ImportError:
        sharp = os.environ.get('ARCANA_SHARP')
        if not sharp:
            raise SystemExit('Install cairosvg locally or set ARCANA_SHARP to a local sharp module.')
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


def trace(master, name, width, label):
    resized = master.resize((width, round(master.height * width / master.width)), Image.Resampling.LANCZOS)
    source, svg = LOCAL / (name + '-' + label + '.png'), LOCAL / (name + '-' + label + '.svg')
    resized.save(source)
    vtracer.convert_image_to_svg_py(str(source), str(svg), **settings)
    # Lossless XML cleanup only; never run a path simplifier or round path data.
    body = re.sub(r'<!--.*?-->\s*', '', svg.read_text(), flags=re.S)
    body = body.replace(f'width="{resized.width}" height="{resized.height}"',
                        f'width="{master.width}" height="{master.height}" viewBox="0 0 {resized.width} {resized.height}"')
    svg.write_text(body)
    raw = svg.read_bytes()
    return svg, dict(trace_width=width, raw=len(raw), gzip=len(gzip.compress(raw, mtime=0)))


records_path = LOCAL / 'quality.json'
records = json.loads(records_path.read_text()) if records_path.exists() else {}
for name, asset, display_width, budget in assets:
    if requested and name not in requested:
        continue
    started = time.monotonic()
    master = Image.open(LOCAL / (name + '.png')).convert('RGBA')
    display_size = (display_width, round(master.height * display_width / master.width))
    final_svg = OUT / asset['svg']
    rendered = LOCAL / (name + '-full-render.png')
    if measure_only:
        measured = records[name]['svg']
        initial = records[name]['baseline']
    else:
        # Full-resolution reference attempt precedes budget tuning for every image.
        baseline, initial = trace(master, name, master.width, 'full-trace')
        rendered = LOCAL / (name + '-full-render.png')
        render(baseline, rendered, master.size)
        initial.update(compare(master, Image.open(rendered)))
        initial['settings'] = settings.copy()
        candidates = []
        # Reserve space for WebP fallbacks inside the global 3 MB transport budget.
        target = int(budget * 0.56)
        if initial['gzip'] <= target:
            candidates.append((baseline, initial))
        else:
            widths = [640, 512, 448, 384, 320, 256, 224, 192, 160] if name.startswith('banner-') else [384, 320, 256, 224, 192, 160, 128]
            for width in widths:
                svg, measured = trace(master, name, width, 'w' + str(width))
                candidates.append((svg, measured))
                if measured['gzip'] <= target:
                    break
        svg, measured = next(((p, m) for p, m in candidates if m['gzip'] <= target), candidates[-1])
        shutil.copy2(svg, final_svg)
    render(final_svg, rendered, master.size)
    reconstructed = Image.open(rendered).convert('RGBA')
    full = compare(master, reconstructed)
    display_render = LOCAL / (name + '-display-render.png')
    render(final_svg, display_render, display_size)
    display = compare(master.resize(display_size, Image.Resampling.LANCZOS), Image.open(display_render))
    fallback = full['ssim'] < .95 or display['ssim'] < .97 or measured['gzip'] > budget or full['alpha_mae'] > 1
    webp_metrics = None
    if fallback:
        webp = OUT / (name + '.webp')
        master.resize((display_size[0]*2, display_size[1]*2), Image.Resampling.LANCZOS).save(webp, 'WEBP', quality=90, method=6, exact=True, lossless=(display_width == 72))
        asset['display'] = webp.name
        webp_metrics = compare(master.resize(display_size, Image.Resampling.LANCZOS), Image.open(webp).resize(display_size, Image.Resampling.LANCZOS))
    else:
        asset['display'] = asset['svg']
    records[name] = dict(full=full, display=display, baseline=initial, svg=measured,
                         budget=budget, fallback=fallback, webp_bytes=(OUT / asset['display']).stat().st_size if fallback else 0,
                         webp_display=webp_metrics, webp_lossless=(fallback and display_width == 72), settings=settings.copy(), seconds=round(time.monotonic()-started, 2))
    MANIFEST.write_text(json.dumps(data, indent=2)+'\n')
    records_path.write_text(json.dumps(records, indent=2)+'\n')
    print(name, json.dumps(records[name]), flush=True)
versions = {name:importlib.metadata.version(name) for name in ['vtracer','Pillow','numpy','scipy']}
(LOCAL / 'versions.json').write_text(json.dumps(versions, indent=2)+'\n')
if len(records) == len(assets):
    total = sum(r['svg']['gzip'] + r['webp_bytes'] for r in records.values())
    print('TOTAL gzip SVG + WebP bytes:', total, flush=True)
    if total > 3_000_000:
        raise SystemExit('Arcana art exceeds the 3 MB total budget')
    if any(r['svg']['gzip'] > r['budget'] for r in records.values()):
        raise SystemExit('An SVG exceeds its per-asset budget')
PY
