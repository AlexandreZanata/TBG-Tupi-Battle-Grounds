# Generated Augments art

This update changes presentation only. Rules, costs, eligibility and official effect text remain canonical. Originals and Lanczos-normalized masters are retained only as project copies under ignored `.local/art-masters/augments/`. Master normalization resizes the entire generated image without cropping.

Optional local tools: `{'vtracer': '0.6.15', 'Pillow': '12.2.0', 'numpy': '2.4.6', 'scipy': '1.18.0'}`. SVG rendering used librsvg via the bundled sharp runtime.

Run from the worktree root (tools live under `.local/art-tools/`):

```bash
ART_SHARP=/home/iiii/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp scripts/arcana-art.sh augments --workers 3
```

The shared implementation is `scripts/art_pipeline.py`; `scripts/art_catalog.py` owns visual sizes and budgets. All sets use the same VTracer 0.6.15 settings: color, stacked, spline, filter_speckle=2, color_precision=8, layer_difference=8, corner_threshold=60, length_threshold=4.0, splice_threshold=45, path_precision=3. A full-resolution trace is measured first. Size tuning tries progressively smaller trace inputs; SVG cleanup removes XML comments only and never simplifies path geometry.

SSIM is measured on RGB with a uniform 7×7 window (sample covariance, K1=.01, K2=.03), weighted by reference alpha. RGB MAE is on a 0–255 scale and weighted by alpha. Both full-size and direct display-size renders are checked. A vector must reach full SSIM ≥0.95, display SSIM ≥0.97 and alpha MAE ≤1 in both renders. Otherwise the UI selects a WebP fallback at twice the display size, quality/effort 90. Icons and tower portraits use lossless encoding; frames and opaque illustrations/banners use lossy quality 90 with alpha preserved where present. Failed vector scores below are reported honestly; they are never substituted for the detailed game artwork.

**Total art, including gzip SVG plus WebP: 4,067,598 B.** **BLOCKED: exceeds the 4,000,000 B budget by 67,598 B; see `../ART-ASSETS.md`.** Display comparison widths are 210 px for illustrations and 300 px for frames; the subject is also inspected at the requested 180 px scale. Only the four current options and their frames preload when an authoritative round opens. Names, rarities, texts and timing are checked against core by client Rust tests.

## Quality and sizes

| Asset | Display px | Trace px | SSIM full/display | RGB MAE full/display | Alpha MAE full/display | SVG raw/gzip B | WebP B | UI source |
|---|---:|---:|---|---|---|---|---:|---|
| frame-bronze | 300 | 112 | 0.179344 / 0.306055 | 29.051 / 25.628 | 40.108 / 40.701 | 115,589 / 33,227 | 150,538 | WebP fallback |
| frame-silver | 300 | 96 | 0.152231 / 0.268412 | 31.487 / 26.789 | 41.845 / 42.482 | 83,065 / 24,219 | 149,984 | WebP fallback |
| frame-gold | 300 | 96 | 0.154631 / 0.269651 | 33.032 / 28.635 | 47.072 / 47.387 | 101,900 / 29,379 | 172,124 | WebP fallback |
| frame-diamond | 300 | 112 | 0.148807 / 0.341810 | 38.111 / 32.887 | 41.800 / 42.262 | 121,809 / 34,687 | 188,792 | WebP fallback |
| forced-march | 210 | 64 | 0.383617 / 0.434837 | 21.972 / 18.588 | 0.000 / 0.000 | 57,713 / 16,729 | 72,070 | WebP fallback |
| field-kit | 210 | 64 | 0.447663 / 0.501106 | 18.104 / 15.261 | 0.000 / 0.000 | 50,214 / 14,796 | 58,908 | WebP fallback |
| sentry | 210 | 64 | 0.468150 / 0.497290 | 17.950 / 14.396 | 0.000 / 0.000 | 46,014 / 13,995 | 62,824 | WebP fallback |
| steady-aim | 210 | 64 | 0.548267 / 0.554666 | 13.374 / 11.182 | 0.000 / 0.000 | 51,472 / 14,615 | 54,690 | WebP fallback |
| reinforced-shields | 210 | 64 | 0.408295 / 0.497056 | 16.232 / 12.711 | 0.000 / 0.000 | 49,939 / 14,542 | 64,690 | WebP fallback |
| first-aid | 210 | 64 | 0.373801 / 0.432124 | 25.206 / 19.299 | 0.000 / 0.000 | 56,863 / 15,960 | 71,938 | WebP fallback |
| drilled-ranks | 210 | 64 | 0.377999 / 0.385927 | 23.911 / 20.818 | 0.000 / 0.000 | 50,110 / 15,308 | 66,942 | WebP fallback |
| frenzied-bones | 210 | 64 | 0.416712 / 0.477282 | 21.684 / 18.214 | 0.000 / 0.000 | 57,415 / 16,123 | 66,548 | WebP fallback |
| tuned-gears | 210 | 64 | 0.382790 / 0.437285 | 22.274 / 18.846 | 0.000 / 0.000 | 60,063 / 17,131 | 67,964 | WebP fallback |
| hardy-sap | 210 | 64 | 0.453485 / 0.466398 | 20.034 / 16.997 | 0.000 / 0.000 | 58,305 / 16,191 | 64,786 | WebP fallback |
| plunder | 210 | 64 | 0.429764 / 0.475449 | 20.189 / 16.455 | 0.000 / 0.000 | 54,297 / 15,676 | 67,466 | WebP fallback |
| stone-mend | 210 | 64 | 0.426223 / 0.466045 | 20.330 / 16.650 | 0.000 / 0.000 | 54,330 / 15,352 | 63,566 | WebP fallback |
| field-promotion | 210 | 64 | 0.488834 / 0.514106 | 17.479 / 14.262 | 0.000 / 0.000 | 52,515 / 15,368 | 57,170 | WebP fallback |
| veterans | 210 | 64 | 0.391162 / 0.474677 | 23.016 / 17.449 | 0.000 / 0.000 | 51,904 / 15,443 | 68,006 | WebP fallback |
| shield-wall | 210 | 64 | 0.293738 / 0.400708 | 22.804 / 17.877 | 0.000 / 0.000 | 52,445 / 15,026 | 76,250 | WebP fallback |
| siege-arrows | 210 | 64 | 0.382859 / 0.375570 | 19.539 / 16.468 | 0.000 / 0.000 | 51,000 / 14,645 | 70,336 | WebP fallback |
| armored-brood | 210 | 64 | 0.398930 / 0.470685 | 20.907 / 16.430 | 0.000 / 0.000 | 56,983 / 16,730 | 66,828 | WebP fallback |
| triage | 210 | 64 | 0.485355 / 0.535682 | 16.885 / 13.856 | 0.000 / 0.000 | 52,004 / 15,000 | 55,734 | WebP fallback |
| lingering-hex | 210 | 64 | 0.503538 / 0.506144 | 19.722 / 17.556 | 0.000 / 0.000 | 58,322 / 15,824 | 53,586 | WebP fallback |
| shield-step | 210 | 64 | 0.357860 / 0.422970 | 25.751 / 21.631 | 0.000 / 0.000 | 60,399 / 17,162 | 70,306 | WebP fallback |
| war-leech | 210 | 64 | 0.612909 / 0.614105 | 13.151 / 11.353 | 0.000 / 0.000 | 47,426 / 13,814 | 47,710 | WebP fallback |
| conquerors | 210 | 64 | 0.485603 / 0.525834 | 18.121 / 14.909 | 0.000 / 0.000 | 51,212 / 14,516 | 61,168 | WebP fallback |
| tower-fury | 210 | 64 | 0.463169 / 0.507083 | 17.169 / 14.457 | 0.000 / 0.000 | 51,599 / 15,094 | 63,072 | WebP fallback |
| entrenched | 210 | 64 | 0.336633 / 0.406581 | 23.305 / 18.798 | 0.000 / 0.000 | 53,896 / 15,930 | 74,366 | WebP fallback |
| grievous-wounds | 210 | 64 | 0.442819 / 0.510274 | 17.501 / 13.872 | 0.000 / 0.000 | 55,740 / 15,258 | 66,960 | WebP fallback |
| giant-slayer | 210 | 64 | 0.452020 / 0.490493 | 18.998 / 16.283 | 0.000 / 0.000 | 56,403 / 16,480 | 66,240 | WebP fallback |
| executioner | 210 | 64 | 0.527763 / 0.562503 | 15.013 / 12.306 | 0.000 / 0.000 | 46,081 / 13,360 | 58,458 | WebP fallback |
| hunter-s-instinct | 210 | 64 | 0.446233 / 0.483044 | 19.973 / 16.169 | 0.000 / 0.000 | 55,109 / 15,597 | 64,830 | WebP fallback |
| iron-vanguard | 210 | 64 | 0.474973 / 0.530497 | 19.615 / 16.103 | 0.000 / 0.000 | 53,909 / 15,339 | 63,940 | WebP fallback |
| spellweaver | 210 | 64 | 0.535270 / 0.532443 | 17.340 / 15.029 | 0.000 / 0.000 | 51,482 / 15,205 | 59,430 | WebP fallback |
| last-bastion | 210 | 64 | 0.360788 / 0.434276 | 24.196 / 20.717 | 0.000 / 0.000 | 52,134 / 14,436 | 67,092 | WebP fallback |
| desperate-assault | 210 | 64 | 0.395330 / 0.445461 | 22.982 / 19.503 | 0.000 / 0.000 | 54,153 / 16,185 | 71,386 | WebP fallback |
| legacy | 210 | 64 | 0.346423 / 0.442844 | 26.582 / 21.069 | 0.000 / 0.000 | 51,449 / 15,275 | 76,066 | WebP fallback |
| second-wind | 210 | 64 | 0.388706 / 0.442593 | 21.516 / 18.402 | 0.000 / 0.000 | 58,505 / 16,064 | 67,322 | WebP fallback |
| convergence | 210 | 64 | 0.360988 / 0.409932 | 24.112 / 20.844 | 0.000 / 0.000 | 54,225 / 14,831 | 72,574 | WebP fallback |
| royal-banner | 210 | 64 | 0.408659 / 0.443204 | 25.066 / 21.521 | 0.000 / 0.000 | 53,014 / 15,985 | 68,408 | WebP fallback |
| deathless-legion | 210 | 64 | 0.305466 / 0.393359 | 28.816 / 24.419 | 0.000 / 0.000 | 53,026 / 14,697 | 78,448 | WebP fallback |
| iron-legion | 210 | 64 | 0.342526 / 0.405413 | 27.807 / 23.650 | 0.000 / 0.000 | 55,821 / 15,838 | 77,144 | WebP fallback |
| bountiful-harvest | 210 | 64 | 0.261804 / 0.352126 | 31.265 / 25.786 | 0.000 / 0.000 | 57,614 / 16,769 | 86,120 | WebP fallback |
| hive-awakening | 210 | 64 | 0.347821 / 0.373821 | 26.876 / 24.042 | 0.000 / 0.000 | 57,445 / 15,865 | 75,152 | WebP fallback |

## Served fallback fidelity

| Asset | Encoding | Display SSIM | RGB MAE | Alpha MAE |
|---|---|---:|---:|---:|
| frame-bronze | Lossy Q90 | 0.981045 | 2.683 | 0.081 |
| frame-silver | Lossy Q90 | 0.978935 | 2.579 | 0.069 |
| frame-gold | Lossy Q90 | 0.957439 | 3.238 | 0.077 |
| frame-diamond | Lossy Q90 | 0.985113 | 2.888 | 0.078 |
| forced-march | Lossy Q90 | 0.980008 | 2.276 | 0.000 |
| field-kit | Lossy Q90 | 0.981768 | 1.810 | 0.000 |
| sentry | Lossy Q90 | 0.969498 | 2.274 | 0.000 |
| steady-aim | Lossy Q90 | 0.968501 | 2.147 | 0.000 |
| reinforced-shields | Lossy Q90 | 0.973788 | 2.128 | 0.000 |
| first-aid | Lossy Q90 | 0.986853 | 2.013 | 0.000 |
| drilled-ranks | Lossy Q90 | 0.985978 | 2.310 | 0.000 |
| frenzied-bones | Lossy Q90 | 0.983110 | 2.185 | 0.000 |
| tuned-gears | Lossy Q90 | 0.984771 | 2.217 | 0.000 |
| hardy-sap | Lossy Q90 | 0.977723 | 2.474 | 0.000 |
| plunder | Lossy Q90 | 0.982305 | 2.138 | 0.000 |
| stone-mend | Lossy Q90 | 0.988612 | 1.705 | 0.000 |
| field-promotion | Lossy Q90 | 0.984014 | 1.887 | 0.000 |
| veterans | Lossy Q90 | 0.986783 | 2.000 | 0.000 |
| shield-wall | Lossy Q90 | 0.987070 | 2.077 | 0.000 |
| siege-arrows | Lossy Q90 | 0.973062 | 2.586 | 0.000 |
| armored-brood | Lossy Q90 | 0.984673 | 2.038 | 0.000 |
| triage | Lossy Q90 | 0.983924 | 1.790 | 0.000 |
| lingering-hex | Lossy Q90 | 0.987535 | 2.106 | 0.000 |
| shield-step | Lossy Q90 | 0.990519 | 2.080 | 0.000 |
| war-leech | Lossy Q90 | 0.965562 | 1.880 | 0.000 |
| conquerors | Lossy Q90 | 0.973982 | 2.066 | 0.000 |
| tower-fury | Lossy Q90 | 0.968302 | 2.071 | 0.000 |
| entrenched | Lossy Q90 | 0.982532 | 2.062 | 0.000 |
| grievous-wounds | Lossy Q90 | 0.971911 | 1.908 | 0.000 |
| giant-slayer | Lossy Q90 | 0.969748 | 2.129 | 0.000 |
| executioner | Lossy Q90 | 0.966924 | 1.970 | 0.000 |
| hunter-s-instinct | Lossy Q90 | 0.956839 | 2.336 | 0.000 |
| iron-vanguard | Lossy Q90 | 0.960594 | 2.148 | 0.000 |
| spellweaver | Lossy Q90 | 0.965718 | 2.167 | 0.000 |
| last-bastion | Lossy Q90 | 0.990238 | 2.093 | 0.000 |
| desperate-assault | Lossy Q90 | 0.985219 | 2.081 | 0.000 |
| legacy | Lossy Q90 | 0.984068 | 2.467 | 0.000 |
| second-wind | Lossy Q90 | 0.989362 | 1.926 | 0.000 |
| convergence | Lossy Q90 | 0.986633 | 2.223 | 0.000 |
| royal-banner | Lossy Q90 | 0.983481 | 2.475 | 0.000 |
| deathless-legion | Lossy Q90 | 0.991068 | 2.122 | 0.000 |
| iron-legion | Lossy Q90 | 0.987633 | 2.261 | 0.000 |
| bountiful-harvest | Lossy Q90 | 0.984617 | 2.735 | 0.000 |
| hive-awakening | Lossy Q90 | 0.981628 | 2.699 | 0.000 |

## Exact selected generation prompts

### `frame-bronze`

Requested master: 820×1180, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card frame, in the same art direction as the Arcana shields: sculpted weathered bronze with upper-left highlights, cool dark hammered iron, torn woven cloth and high detail. Exactly 820x1180 PNG, portrait, with genuine transparent alpha. Rarity Bronze: red ruby gemstone and crimson-copper rim glow. One thin rectangular ornate frame, front view, 4 percent safe margin, all edges visible. Two completely empty transparent cutout windows: square art window x=10%-90%, y=5%-49%, and text window x=10%-90%, y=60%-88%. The windows contain no painting, no paper, no opaque fill, no checkerboard. Put the gemstone centered on the horizontal divider at y=51%; an empty narrow cloth rarity ribbon at y=55%; a small empty metal cost plaque at y=93%. Both windows and outside the card must be genuinely transparent. No text, letters, numbers or watermark. Same layout, outline, dimensions and lighting across all four rarities. Do not paste, trace or crop any reference.
```

### `frame-silver`

Requested master: 820×1180, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card frame, in the same art direction as the Arcana shields: sculpted weathered bronze with upper-left highlights, cool dark hammered iron, torn woven cloth and high detail. Exactly 820x1180 PNG, portrait, with genuine transparent alpha. Rarity Silver: blue sapphire gemstone and steel-blue rim glow. One thin rectangular ornate frame, front view, 4 percent safe margin, all edges visible. Two completely empty transparent cutout windows: square art window x=10%-90%, y=5%-49%, and text window x=10%-90%, y=60%-88%. The windows contain no painting, no paper, no opaque fill, no checkerboard. Put the gemstone centered on the horizontal divider at y=51%; an empty narrow cloth rarity ribbon at y=55%; a small empty metal cost plaque at y=93%. Both windows and outside the card must be genuinely transparent. No text, letters, numbers or watermark. Same layout, outline, dimensions and lighting across all four rarities. Do not paste, trace or crop any reference.
```

### `frame-gold`

Requested master: 820×1180, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card frame, in the same art direction as the Arcana shields: sculpted weathered bronze with upper-left highlights, cool dark hammered iron, torn woven cloth and high detail. Exactly 820x1180 PNG, portrait, with genuine transparent alpha. Rarity Gold: golden amber gemstone and amber-gold rim glow. One thin rectangular ornate frame, front view, 4 percent safe margin, all edges visible. Two completely empty transparent cutout windows: square art window x=10%-90%, y=5%-49%, and text window x=10%-90%, y=60%-88%. The windows contain no painting, no paper, no opaque fill, no checkerboard. Put the gemstone centered on the horizontal divider at y=51%; an empty narrow cloth rarity ribbon at y=55%; a small empty metal cost plaque at y=93%. Both windows and outside the card must be genuinely transparent. No text, letters, numbers or watermark. Same layout, outline, dimensions and lighting across all four rarities. Do not paste, trace or crop any reference.
```

### `frame-diamond`

Requested master: 820×1180, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card frame, in the same art direction as the Arcana shields: sculpted weathered bronze with upper-left highlights, cool dark hammered iron, torn woven cloth and high detail. Exactly 820x1180 PNG, portrait, with genuine transparent alpha. Rarity Diamond: prismatic diamond gemstone and white-gold prismatic rim glow. Diamond uses richer gilding and white-gold light. One thin rectangular ornate frame, front view, 4 percent safe margin, all edges visible. Two completely empty transparent cutout windows: square art window x=10%-90%, y=5%-49%, and text window x=10%-90%, y=60%-88%. The windows contain no painting, no paper, no opaque fill, no checkerboard. Put the gemstone centered on the horizontal divider at y=51%; an empty narrow cloth rarity ribbon at y=55%; a small empty metal cost plaque at y=93%. Both windows and outside the card must be genuinely transparent. No text, letters, numbers or watermark. Same layout, outline, dimensions and lighting across all four rarities. Do not paste, trace or crop any reference.
```

### `forced-march`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: crimson-copper. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: marching boots with dust trail.
```

### `field-kit`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: crimson-copper. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: leather satchel with bandages.
```

### `sentry`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: crimson-copper. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: watchtower crossbow.
```

### `steady-aim`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: crimson-copper. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: longbow at full draw.
```

### `reinforced-shields`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: crimson-copper. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: iron-banded shield.
```

### `first-aid`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: healing herb poultice.
```

### `drilled-ranks`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: row of pikes, human banner.
```

### `frenzied-bones`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: skull with spectral fury.
```

### `tuned-gears`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: interlocking brass gears with sparks.
```

### `hardy-sap`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: amber sap dripping from living bark.
```

### `plunder`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: spilled coin pouch.
```

### `stone-mend`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: mortar trowel sealing cracked stone.
```

### `field-promotion`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: bronze chevron medal.
```

### `veterans`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: scarred helmet with laurel.
```

### `shield-wall`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: wall of overlapping shields.
```

### `siege-arrows`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: fire arrows hitting a tower wall.
```

### `armored-brood`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: armored beetle shell.
```

### `triage`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: field stretcher with green glow.
```

### `lingering-hex`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: violet rune chains.
```

### `shield-step`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: steel-blue. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: advancing shield with motion.
```

### `war-leech`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: fanged blade dripping crimson energy.
```

### `conquerors`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: banner planted on a broken battlement.
```

### `tower-fury`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: tower cannon with three flame marks.
```

### `entrenched`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: trench with spiked palisade.
```

### `grievous-wounds`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: serrated dagger with black rot.
```

### `giant-slayer`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: spear piercing a giant's gauntlet.
```

### `executioner`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: headsman's axe.
```

### `hunter-s-instinct`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: wolf eye over crossed daggers.
```

### `iron-vanguard`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: charging iron helm.
```

### `spellweaver`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: amber-gold. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: staff weaving a glowing knot.
```

### `last-bastion`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: golden fortress with a radiant shield dome.
```

### `desperate-assault`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: cracked sword still glowing.
```

### `legacy`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: ancestral crowned helm.
```

### `second-wind`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: rising spirit breath over a fallen sword.
```

### `convergence`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: two crests fusing into one light.
```

### `royal-banner`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: gilded royal standard.
```

### `deathless-legion`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: skeletal legion with a crown of bones.
```

### `iron-legion`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: marching war machines.
```

### `bountiful-harvest`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: overflowing magical harvest basket.
```

### `hive-awakening`

Requested master: 1024×1024, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game card illustration. Same art direction as the Arcana set: sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail, no lettering, no watermark. One bold central object, readable at 180 px, dramatic rim light, painterly particles, dark vignette background tinted with the rarity glow color: white-gold prismatic. Exactly 1024x1024 PNG, opaque. Do not paste, trace or crop any existing reference. Central subject: glowing beetle hive queen.
```

## Rejected generations

None. Every selected image was inspected in local contact sheets before integration.
