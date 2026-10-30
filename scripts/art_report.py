#!/usr/bin/env python3
"""Write generation provenance and measured art quality for one completed set."""
import argparse
import gzip
import json
from pathlib import Path


def report(name):
    root = Path(__file__).resolve().parents[1]
    jobs = json.loads((root / '.local/art-jobs.json').read_text())
    jobs = [j for j in jobs if j['set'] == name]
    local = root / '.local/art-masters' / name
    quality = json.loads((local / 'quality.json').read_text())
    rejected_path = root / '.local/art-rejected.json'
    rejected = json.loads(rejected_path.read_text()) if rejected_path.exists() else []
    rejected = [r for r in rejected if r['set'] == name]
    versions = json.loads((local / 'versions.json').read_text())
    output_folder = 'stronghold' if name == 'strongholds' else name
    out = root / 'crates/tbg-client/assets' / output_folder
    total = sum(len(gzip.compress(p.read_bytes(), mtime=0)) if p.suffix == '.svg' else p.stat().st_size
                for p in out.iterdir() if p.suffix in ['.svg', '.webp'])
    text = f'# Generated {name.title()} art\n\n'
    text += ('This update changes presentation only. Rules, costs, eligibility and official effect text remain canonical. '
             'Originals and Lanczos-normalized masters are retained only as project copies under ignored '
             f'`.local/art-masters/{name}/`. Master normalization resizes the entire generated image without cropping.\n\n')
    text += f'Optional local tools: `{versions}`. SVG rendering used librsvg via the bundled sharp runtime.\n\n'
    text += ('Run from the worktree root (tools live under `.local/art-tools/`):\n\n'
             f'```bash\nART_SHARP=/home/iiii/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp scripts/arcana-art.sh {name} --workers 3\n```\n\n'
             'The shared implementation is `scripts/art_pipeline.py`; `scripts/art_catalog.py` owns visual sizes and budgets. '
             'All sets use the same VTracer 0.6.15 settings: color, stacked, spline, filter_speckle=2, color_precision=8, '
             'layer_difference=8, corner_threshold=60, length_threshold=4.0, splice_threshold=45, path_precision=3. '
             'A full-resolution trace is measured first. Size tuning tries progressively smaller trace inputs; '
             'SVG cleanup removes XML comments only and never simplifies path geometry.\n\n'
             'SSIM is measured on RGB with a uniform 7×7 window (sample covariance, K1=.01, K2=.03), '
             'weighted by reference alpha. RGB MAE is on a 0–255 scale and weighted by alpha. '
             'Both full-size and direct display-size renders are checked. A vector must reach full SSIM ≥0.95, '
             'display SSIM ≥0.97 and alpha MAE ≤1 in both renders. Otherwise the UI selects a WebP fallback '
             'at twice the display size, quality/effort 90. Icons and tower portraits use lossless encoding; '
             'frames and opaque illustrations/banners use lossy quality 90 with alpha preserved where present. Failed vector scores below are reported honestly; '
             'they are never substituted for the detailed game artwork.\n\n')
    text += f'**Total art, including gzip SVG plus WebP: {total:,} B.** '
    if name == 'arcana':
        text += ('The total includes all twelve previously approved assets and nine new icons. Approved existing WebPs are unchanged; '
                 'unused older vectors were retuned to fit the expanded set within the same 3 MB budget. '
                 'Original approved prompts remain in `../ARCANA-UI-AND-STRATEGY.md`.\n\n')
    elif name == 'augments':
        text += ('The budget is 4,000,000 B. Display comparison widths are 200 px for illustrations and 300 px for frames; '
                 'the subject is also inspected at the requested 180 px scale. Only the four current options and their frames preload '
                 'when an authoritative round opens. Names, rarities, texts and timing are checked against core by client Rust tests.\n\n')
    else:
        text += ('The budget is 2 MB, including the legacy compact HUD sprite. Display comparison widths are 160 px portraits, '
                 '450 px banners and 72 px branch icons. New images have no source URL until the picker opens. '
                 'The original kind/tier/branch numbers are unchanged. Silhouettes were inspected first in `?gallery=strongholds`.\n\n')
    text += '## Quality and sizes\n\n| Asset | Display px | Trace px | SSIM full/display | RGB MAE full/display | Alpha MAE full/display | SVG raw/gzip B | WebP B | UI source |\n|---|---:|---:|---|---|---|---|---:|---|\n'
    for asset, q in quality.items():
        text += (f"| {asset} | {q['display_size'][0]} | {q['svg']['trace_width']} | {q['full']['ssim']:.6f} / {q['display']['ssim']:.6f} | "
                 f"{q['full']['mae']:.3f} / {q['display']['mae']:.3f} | {q['full']['alpha_mae']:.3f} / {q['display']['alpha_mae']:.3f} | "
                 f"{q['svg']['raw']:,} / {q['svg']['gzip']:,} | {q['webp_bytes']:,} | {'WebP fallback' if q['fallback'] else 'SVG'} |\n")
    text += '\n## Served fallback fidelity\n\n| Asset | Encoding | Display SSIM | RGB MAE | Alpha MAE |\n|---|---|---:|---:|---:|\n'
    for asset, q in quality.items():
        if q['webp_display']:
            m = q['webp_display']
            text += f"| {asset} | {'Lossless' if q['webp_lossless'] else 'Lossy'} Q90 | {m['ssim']:.6f} | {m['mae']:.3f} | {m['alpha_mae']:.3f} |\n"
    text += '\n## Exact selected generation prompts\n\n'
    for job in jobs:
        text += f"### `{job['name']}`\n\nRequested master: {job['masterSize'][0]}×{job['masterSize'][1]}, {'transparent' if job['transparent'] else 'opaque'}.\n\n```text\n{job['prompt']}\n```\n\n"
    text += '## Rejected generations\n\n'
    if not rejected:
        text += 'None. Every selected image was inspected in local contact sheets before integration.\n'
    for job in rejected:
        text += f"### `{job['name']}`\n\nReason: {job['reason']}\n\n```text\n{job['prompt']}\n```\n\n"
    dest = root / 'docs/art' / (name.upper() + '.md')
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(text.rstrip()+'\n')
    print(dest, total)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('set', choices=['arcana','augments','strongholds'])
    report(parser.parse_args().set)
