#!/usr/bin/env python3
"""Normalize original ranking rasters and measure their served WebP fidelity."""
from pathlib import Path
import hashlib
import importlib.metadata
import json
import re

from PIL import Image
from art_pipeline import normalize, compare

ROOT = Path(__file__).resolve().parents[1]


def run():
    normalize(ROOT)
    folder = ROOT / 'crates/tbg-client/assets/ranking'
    manifest = folder / 'ranking.json'
    data = json.loads(manifest.read_text())
    local = ROOT / '.local/art-masters/ranking'
    metrics = {}
    for item in data['assets']:
        name = item['id']
        if not re.fullmatch(r'[a-z0-9-]+', name):
            raise ValueError('Unsafe asset name')
        image = Image.open(local / (name + '.png')).convert('RGBA')
        width = item['displayWidth']
        size = (width, round(image.height * width / image.width))
        dest = folder / (name + '.webp')
        # Painterly ranking art uses the same Q90/alpha raster path as the approved sets.
        scale = 4 if item['lossless'] else 2
        image.resize((size[0]*scale, size[1]*scale), Image.Resampling.LANCZOS).save(
            dest, 'WEBP', quality=90, method=6, exact=True, lossless=item['lossless'])
        measured = compare(image.resize(size, Image.Resampling.LANCZOS),
                           Image.open(dest).resize(size, Image.Resampling.LANCZOS))
        if measured['ssim'] < .97 or measured['alpha_mae'] > 1:
            raise ValueError(name + ': served fidelity gate failed: ' + str(measured))
        item['asset'] = {'display': dest.name}
        metrics[name] = dict(display_size=size, master_size=image.size,
                             bytes=dest.stat().st_size, lossless=item['lossless'],
                             sha256=hashlib.sha256(dest.read_bytes()).hexdigest(), **measured)
        print(name, json.dumps(metrics[name]), flush=True)
    total = sum(m['bytes'] for m in metrics.values())
    if total > data['artBudgetBytes']:
        raise ValueError('Ranking art exceeds its transport budget')
    manifest.write_text(json.dumps(data, indent=2)+'\n')
    (local/'quality.json').write_text(json.dumps(metrics, indent=2)+'\n')
    (local/'versions.json').write_text(json.dumps({n:importlib.metadata.version(n) for n in ['Pillow','numpy','scipy']}, indent=2)+'\n')
    print('TOTAL ranking WebP bytes:', total, 'budget:', data['artBudgetBytes'])


if __name__ == '__main__':
    run()
