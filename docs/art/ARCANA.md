# Generated Arcana art

This update changes presentation only. Rules, costs, eligibility and official effect text remain canonical. Originals and Lanczos-normalized masters are retained only as project copies under ignored `.local/art-masters/arcana/`. Master normalization resizes the entire generated image without cropping.

Optional local tools: `{'vtracer': '0.6.15', 'Pillow': '12.2.0', 'numpy': '2.4.6', 'scipy': '1.18.0'}`. SVG rendering used librsvg via the bundled sharp runtime.

Run from the worktree root (tools live under `.local/art-tools/`):

```bash
ART_SHARP=/home/iiii/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp scripts/arcana-art.sh arcana --workers 3
```

The shared implementation is `scripts/art_pipeline.py`; `scripts/art_catalog.py` owns visual sizes and budgets. All sets use the same VTracer 0.6.15 settings: color, stacked, spline, filter_speckle=2, color_precision=8, layer_difference=8, corner_threshold=60, length_threshold=4.0, splice_threshold=45, path_precision=3. A full-resolution trace is measured first. Size tuning tries progressively smaller trace inputs; SVG cleanup removes XML comments only and never simplifies path geometry.

SSIM is measured on RGB with a uniform 7×7 window (sample covariance, K1=.01, K2=.03), weighted by reference alpha. RGB MAE is on a 0–255 scale and weighted by alpha. Both full-size and direct display-size renders are checked. A vector must reach full SSIM ≥0.95, display SSIM ≥0.97 and alpha MAE ≤1 in both renders. Otherwise the UI selects a WebP fallback at twice the display size, quality/effort 90. Icons and tower portraits use lossless encoding; frames and opaque illustrations/banners use lossy quality 90 with alpha preserved where present. Failed vector scores below are reported honestly; they are never substituted for the detailed game artwork.

**Total art, including gzip SVG plus WebP: 2,751,068 B.** The total includes all twelve previously approved assets and nine new icons. Approved existing WebPs are unchanged; unused older vectors were retuned to fit the expanded set within the same 3 MB budget. Original approved prompts remain in `../ARCANA-UI-AND-STRATEGY.md`.

## Quality and sizes

| Asset | Display px | Trace px | SSIM full/display | RGB MAE full/display | Alpha MAE full/display | SVG raw/gzip B | WebP B | UI source |
|---|---:|---:|---|---|---|---|---:|---|
| war | 160 | 160 | 0.307049 / 0.758738 | 18.483 / 10.611 | 13.291 / 14.137 | 214,122 / 61,746 | 40,866 | WebP fallback |
| precision | 160 | 160 | 0.314368 / 0.705097 | 21.234 / 14.068 | 13.784 / 14.613 | 221,769 / 62,236 | 41,738 | WebP fallback |
| vitality | 160 | 160 | 0.309003 / 0.708335 | 20.647 / 13.021 | 14.265 / 15.139 | 231,803 / 67,972 | 45,416 | WebP fallback |
| banner-war | 900 | 256 | 0.452969 / 0.483140 | 15.998 / 14.534 | 0.682 / 0.135 | 342,991 / 98,028 | 358,028 | WebP fallback |
| banner-precision | 900 | 256 | 0.414983 / 0.457252 | 20.088 / 18.194 | 1.033 / 0.304 | 341,703 / 101,746 | 359,492 | WebP fallback |
| banner-vitality | 900 | 256 | 0.348607 / 0.390579 | 20.541 / 18.880 | 0.743 / 0.147 | 374,068 / 109,565 | 419,114 | WebP fallback |
| guard | 72 | 128 | 0.333021 / 0.833671 | 19.661 / 9.862 | 15.582 / 18.718 | 135,827 / 38,545 | 26,736 | WebP fallback |
| assault | 72 | 128 | 0.282163 / 0.825768 | 20.580 / 9.410 | 17.687 / 19.938 | 131,624 / 36,888 | 27,012 | WebP fallback |
| piercing | 72 | 128 | 0.291206 / 0.830368 | 20.545 / 9.628 | 14.373 / 16.804 | 134,786 / 37,533 | 27,186 | WebP fallback |
| endurance | 72 | 128 | 0.286976 / 0.815213 | 22.005 / 10.363 | 17.190 / 19.522 | 133,512 / 37,355 | 27,760 | WebP fallback |
| resilience | 72 | 128 | 0.246604 / 0.782495 | 22.471 / 10.124 | 18.032 / 20.159 | 136,344 / 39,134 | 27,440 | WebP fallback |
| renewal | 72 | 128 | 0.296230 / 0.790616 | 19.953 / 9.527 | 15.944 / 18.668 | 140,993 / 38,920 | 28,296 | WebP fallback |
| shieldwall | 72 | 128 | 0.327868 / 0.820847 | 19.902 / 10.629 | 16.334 / 19.143 | 134,224 / 38,180 | 28,712 | WebP fallback |
| battle-rush | 72 | 128 | 0.290895 / 0.805590 | 22.740 / 10.892 | 15.094 / 17.695 | 128,847 / 36,846 | 26,918 | WebP fallback |
| combined-arms | 72 | 128 | 0.320298 / 0.835255 | 19.394 / 9.311 | 14.596 / 17.268 | 136,065 / 37,836 | 27,830 | WebP fallback |
| hunters-focus | 72 | 128 | 0.247402 / 0.788941 | 25.093 / 13.349 | 16.408 / 18.862 | 143,405 / 41,634 | 29,642 | WebP fallback |
| quickdraw | 72 | 128 | 0.286243 / 0.769725 | 23.054 / 12.016 | 16.153 / 18.220 | 123,032 / 35,323 | 27,086 | WebP fallback |
| deadeye | 72 | 128 | 0.267961 / 0.809010 | 23.861 / 13.020 | 16.150 / 18.468 | 128,862 / 35,762 | 26,396 | WebP fallback |
| unshaken | 72 | 112 | 0.282585 / 0.734818 | 23.597 / 11.973 | 19.398 / 21.408 | 113,301 / 31,871 | 30,498 | WebP fallback |
| emergency-care | 72 | 128 | 0.309701 / 0.800636 | 20.092 / 9.998 | 15.896 / 18.161 | 142,691 / 39,636 | 29,438 | WebP fallback |
| purification | 72 | 128 | 0.255983 / 0.802428 | 22.391 / 10.599 | 18.874 / 21.395 | 145,780 / 40,130 | 28,578 | WebP fallback |

## Served fallback fidelity

| Asset | Encoding | Display SSIM | RGB MAE | Alpha MAE |
|---|---|---:|---:|---:|
| war | Lossy Q90 | 0.971452 | 2.692 | 0.128 |
| precision | Lossy Q90 | 0.971661 | 3.099 | 0.128 |
| vitality | Lossy Q90 | 0.974776 | 2.443 | 0.135 |
| banner-war | Lossy Q90 | 0.985281 | 1.669 | 0.000 |
| banner-precision | Lossy Q90 | 0.989231 | 1.672 | 0.000 |
| banner-vitality | Lossy Q90 | 0.989710 | 1.695 | 0.000 |
| guard | Lossless Q90 | 0.985436 | 0.511 | 0.249 |
| assault | Lossless Q90 | 0.983357 | 0.492 | 0.283 |
| piercing | Lossless Q90 | 0.978889 | 0.527 | 0.242 |
| endurance | Lossless Q90 | 0.983297 | 0.560 | 0.287 |
| resilience | Lossless Q90 | 0.975955 | 0.508 | 0.278 |
| renewal | Lossless Q90 | 0.983094 | 0.485 | 0.259 |
| shieldwall | Lossless Q90 | 0.983143 | 0.506 | 0.265 |
| battle-rush | Lossless Q90 | 0.977981 | 0.531 | 0.268 |
| combined-arms | Lossless Q90 | 0.987473 | 0.462 | 0.264 |
| hunters-focus | Lossless Q90 | 0.976557 | 0.553 | 0.279 |
| quickdraw | Lossless Q90 | 0.982372 | 0.561 | 0.238 |
| deadeye | Lossless Q90 | 0.976943 | 0.559 | 0.266 |
| unshaken | Lossless Q90 | 0.977222 | 0.531 | 0.269 |
| emergency-care | Lossless Q90 | 0.977641 | 0.495 | 0.266 |
| purification | Lossless Q90 | 0.975129 | 0.525 | 0.289 |

## Exact selected generation prompts

### `shieldwall`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: red. Central symbol: three overlapping bronze tower shields in a row. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft red glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `battle-rush`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: red. Central symbol: a bronze winged boot with a short speed trail. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft red glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `combined-arms`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: red. Central symbol: a bronze shield crossed with a sword, both whole and readable. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft red glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set. Supreme node: a slightly richer bronze crown motif on the top stud, same frame size.
```

### `hunters-focus`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: blue. Central symbol: a bronze arrowhead over a coiled chain representing slowed or rooted prey. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft blue glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `quickdraw`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: blue. Central symbol: two short bronze arrows fanned from a drawn bow grip. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft blue glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `deadeye`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: blue. Central symbol: a bronze eye inside a targeting reticle with one arrow. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft blue glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set. Supreme node: a slightly richer bronze crown motif on the top stud, same frame size.
```

### `unshaken`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: green. Central symbol: a bronze oak root splitting a broken shackle. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft green glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `emergency-care`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: green. Central symbol: a bronze hand cupping a glowing green drop below a half heart. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft green glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `purification`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: green. Central symbol: a glowing green flame rising from a bronze chalice. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft green glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set. Supreme node: a slightly richer bronze crown motif on the top stud, same frame size.
```

## Rejected generations

None. Every selected image was inspected in local contact sheets before integration.
