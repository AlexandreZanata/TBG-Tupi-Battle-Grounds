# Generated Augments art

This update changes presentation only. Rules, costs, eligibility and official effect text remain canonical. Originals and Lanczos-normalized masters are retained only as project copies under ignored `.local/art-masters/augments/`. Master normalization resizes the entire generated image without cropping.

Optional local tools: `{'vtracer': '0.6.15', 'Pillow': '12.2.0', 'numpy': '2.4.6', 'scipy': '1.18.0'}`. SVG rendering used librsvg via the bundled sharp runtime.

Run from the worktree root (tools live under `.local/art-tools/`):

```bash
ART_SHARP=/home/iiii/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp scripts/arcana-art.sh augments --workers 3
```

The shared implementation is `scripts/art_pipeline.py`; `scripts/art_catalog.py` owns visual sizes and budgets. All sets use the same VTracer 0.6.15 settings: color, stacked, spline, filter_speckle=2, color_precision=8, layer_difference=8, corner_threshold=60, length_threshold=4.0, splice_threshold=45, path_precision=3. A full-resolution trace is measured first. Size tuning tries progressively smaller trace inputs; SVG cleanup removes XML comments only and never simplifies path geometry.

SSIM is measured on RGB with a uniform 7×7 window (sample covariance, K1=.01, K2=.03), weighted by reference alpha. RGB MAE is on a 0–255 scale and weighted by alpha. Both full-size and direct display-size renders are checked. A vector must reach full SSIM ≥0.95, display SSIM ≥0.97 and alpha MAE ≤1 in both renders. Otherwise the UI selects a WebP fallback at twice the display size, quality/effort 90. Icons and tower portraits use lossless encoding; frames and opaque illustrations/banners use lossy quality 90 with alpha preserved where present. Failed vector scores below are reported honestly; they are never substituted for the detailed game artwork.

**Total art, including gzip SVG plus WebP: 3,829,406 B.** The budget is 4,000,000 B. Display comparison widths are 200 px for illustrations and 300 px for frames; the subject is also inspected at the requested 180 px scale. Only the four current options and their frames preload when an authoritative round opens. Names, rarities, texts and timing are checked against core by client Rust tests.

## Quality and sizes

| Asset | Display px | Trace px | SSIM full/display | RGB MAE full/display | Alpha MAE full/display | SVG raw/gzip B | WebP B | UI source |
|---|---:|---:|---|---|---|---|---:|---|
| frame-bronze | 300 | 112 | 0.179344 / 0.306055 | 29.051 / 25.628 | 40.108 / 40.701 | 115,589 / 33,227 | 150,538 | WebP fallback |
| frame-silver | 300 | 96 | 0.152231 / 0.268412 | 31.487 / 26.789 | 41.845 / 42.482 | 83,065 / 24,219 | 149,984 | WebP fallback |
| frame-gold | 300 | 96 | 0.154631 / 0.269651 | 33.032 / 28.635 | 47.072 / 47.387 | 101,900 / 29,379 | 172,124 | WebP fallback |
| frame-diamond | 300 | 112 | 0.148807 / 0.341810 | 38.111 / 32.887 | 41.800 / 42.262 | 121,809 / 34,687 | 188,792 | WebP fallback |
| forced-march | 200 | 64 | 0.383617 / 0.445905 | 21.972 / 18.393 | 0.000 / 0.000 | 57,713 / 16,729 | 64,880 | WebP fallback |
| field-kit | 200 | 64 | 0.447663 / 0.509392 | 18.104 / 15.133 | 0.000 / 0.000 | 50,214 / 14,796 | 54,064 | WebP fallback |
| sentry | 200 | 64 | 0.468150 / 0.503620 | 17.950 / 14.268 | 0.000 / 0.000 | 46,014 / 13,995 | 57,438 | WebP fallback |
| steady-aim | 200 | 64 | 0.548267 / 0.560267 | 13.374 / 11.069 | 0.000 / 0.000 | 51,472 / 14,615 | 50,524 | WebP fallback |
| reinforced-shields | 200 | 64 | 0.408295 / 0.505535 | 16.232 / 12.565 | 0.000 / 0.000 | 49,939 / 14,542 | 60,498 | WebP fallback |
| first-aid | 200 | 64 | 0.373801 / 0.441523 | 25.206 / 19.112 | 0.000 / 0.000 | 56,863 / 15,960 | 65,330 | WebP fallback |
| drilled-ranks | 200 | 64 | 0.377999 / 0.393007 | 23.911 / 20.636 | 0.000 / 0.000 | 50,110 / 15,308 | 62,086 | WebP fallback |
| frenzied-bones | 200 | 64 | 0.416712 / 0.487752 | 21.684 / 18.057 | 0.000 / 0.000 | 57,415 / 16,123 | 59,918 | WebP fallback |
| tuned-gears | 200 | 64 | 0.382790 / 0.447247 | 22.274 / 18.717 | 0.000 / 0.000 | 60,063 / 17,131 | 61,958 | WebP fallback |
| hardy-sap | 200 | 64 | 0.453485 / 0.474569 | 20.034 / 16.861 | 0.000 / 0.000 | 58,305 / 16,191 | 57,948 | WebP fallback |
| plunder | 200 | 64 | 0.429764 / 0.485407 | 20.189 / 16.325 | 0.000 / 0.000 | 54,297 / 15,676 | 60,968 | WebP fallback |
| stone-mend | 200 | 64 | 0.426223 / 0.475017 | 20.330 / 16.489 | 0.000 / 0.000 | 54,330 / 15,352 | 57,266 | WebP fallback |
| field-promotion | 200 | 64 | 0.488834 / 0.521024 | 17.479 / 14.157 | 0.000 / 0.000 | 52,515 / 15,368 | 52,804 | WebP fallback |
| veterans | 200 | 64 | 0.391162 / 0.485553 | 23.016 / 17.251 | 0.000 / 0.000 | 51,904 / 15,443 | 61,548 | WebP fallback |
| shield-wall | 200 | 64 | 0.293738 / 0.412446 | 22.804 / 17.681 | 0.000 / 0.000 | 52,445 / 15,026 | 69,010 | WebP fallback |
| siege-arrows | 200 | 64 | 0.382859 / 0.382331 | 19.539 / 16.301 | 0.000 / 0.000 | 51,000 / 14,645 | 63,398 | WebP fallback |
| armored-brood | 200 | 64 | 0.398930 / 0.479905 | 20.907 / 16.283 | 0.000 / 0.000 | 56,983 / 16,730 | 60,110 | WebP fallback |
| triage | 200 | 64 | 0.485355 / 0.543708 | 16.885 / 13.751 | 0.000 / 0.000 | 52,004 / 15,000 | 50,676 | WebP fallback |
| lingering-hex | 200 | 64 | 0.503538 / 0.514356 | 19.722 / 17.440 | 0.000 / 0.000 | 58,322 / 15,824 | 49,394 | WebP fallback |
| shield-step | 200 | 64 | 0.357860 / 0.434547 | 25.751 / 21.486 | 0.000 / 0.000 | 60,399 / 17,162 | 65,090 | WebP fallback |
| war-leech | 200 | 64 | 0.612909 / 0.619834 | 13.151 / 11.264 | 0.000 / 0.000 | 47,426 / 13,814 | 42,524 | WebP fallback |
| conquerors | 200 | 64 | 0.485603 / 0.534886 | 18.121 / 14.795 | 0.000 / 0.000 | 51,212 / 14,516 | 56,208 | WebP fallback |
| tower-fury | 200 | 64 | 0.463169 / 0.515665 | 17.169 / 14.319 | 0.000 / 0.000 | 51,599 / 15,094 | 57,164 | WebP fallback |
| entrenched | 200 | 64 | 0.336633 / 0.415227 | 23.305 / 18.593 | 0.000 / 0.000 | 53,896 / 15,930 | 67,568 | WebP fallback |
| grievous-wounds | 200 | 64 | 0.442819 / 0.519982 | 17.501 / 13.704 | 0.000 / 0.000 | 55,740 / 15,258 | 60,758 | WebP fallback |
| giant-slayer | 200 | 64 | 0.452020 / 0.500313 | 18.998 / 16.134 | 0.000 / 0.000 | 56,403 / 16,480 | 59,618 | WebP fallback |
| executioner | 200 | 64 | 0.527763 / 0.570599 | 15.013 / 12.193 | 0.000 / 0.000 | 46,081 / 13,360 | 52,476 | WebP fallback |
| hunter-s-instinct | 200 | 64 | 0.446233 / 0.490073 | 19.973 / 16.021 | 0.000 / 0.000 | 55,109 / 15,597 | 58,682 | WebP fallback |
| iron-vanguard | 200 | 64 | 0.474973 / 0.539491 | 19.615 / 15.975 | 0.000 / 0.000 | 53,909 / 15,339 | 57,878 | WebP fallback |
| spellweaver | 200 | 64 | 0.535270 / 0.538946 | 17.340 / 14.922 | 0.000 / 0.000 | 51,482 / 15,205 | 53,918 | WebP fallback |
| last-bastion | 200 | 64 | 0.360788 / 0.447712 | 24.196 / 20.576 | 0.000 / 0.000 | 52,134 / 14,436 | 62,602 | WebP fallback |
| desperate-assault | 200 | 64 | 0.395330 / 0.457292 | 22.982 / 19.339 | 0.000 / 0.000 | 54,153 / 16,185 | 64,780 | WebP fallback |
| legacy | 200 | 64 | 0.346423 / 0.456051 | 26.582 / 20.860 | 0.000 / 0.000 | 51,449 / 15,275 | 69,164 | WebP fallback |
| second-wind | 200 | 64 | 0.388706 / 0.453884 | 21.516 / 18.233 | 0.000 / 0.000 | 58,505 / 16,064 | 61,878 | WebP fallback |
| convergence | 200 | 64 | 0.360988 / 0.421359 | 24.112 / 20.647 | 0.000 / 0.000 | 54,225 / 14,831 | 66,522 | WebP fallback |
| royal-banner | 200 | 64 | 0.408659 / 0.452783 | 25.066 / 21.366 | 0.000 / 0.000 | 53,014 / 15,985 | 62,518 | WebP fallback |
| deathless-legion | 200 | 64 | 0.305466 / 0.406707 | 28.816 / 24.193 | 0.000 / 0.000 | 53,026 / 14,697 | 72,170 | WebP fallback |
| iron-legion | 200 | 64 | 0.342526 / 0.417849 | 27.807 / 23.455 | 0.000 / 0.000 | 55,821 / 15,838 | 70,004 | WebP fallback |
| bountiful-harvest | 200 | 64 | 0.261804 / 0.363693 | 31.265 / 25.537 | 0.000 / 0.000 | 57,614 / 16,769 | 78,078 | WebP fallback |
| hive-awakening | 200 | 64 | 0.347821 / 0.384775 | 26.876 / 23.915 | 0.000 / 0.000 | 57,445 / 15,865 | 68,884 | WebP fallback |

## Served fallback fidelity

| Asset | Encoding | Display SSIM | RGB MAE | Alpha MAE |
|---|---|---:|---:|---:|
| frame-bronze | Lossy Q90 | 0.981045 | 2.683 | 0.081 |
| frame-silver | Lossy Q90 | 0.978935 | 2.579 | 0.069 |
| frame-gold | Lossy Q90 | 0.957439 | 3.238 | 0.077 |
| frame-diamond | Lossy Q90 | 0.985113 | 2.888 | 0.078 |
| forced-march | Lossy Q90 | 0.978920 | 2.346 | 0.000 |
| field-kit | Lossy Q90 | 0.982271 | 1.802 | 0.000 |
| sentry | Lossy Q90 | 0.970185 | 2.263 | 0.000 |
| steady-aim | Lossy Q90 | 0.968544 | 2.162 | 0.000 |
| reinforced-shields | Lossy Q90 | 0.975867 | 2.055 | 0.000 |
| first-aid | Lossy Q90 | 0.987212 | 2.001 | 0.000 |
| drilled-ranks | Lossy Q90 | 0.986438 | 2.299 | 0.000 |
| frenzied-bones | Lossy Q90 | 0.983074 | 2.205 | 0.000 |
| tuned-gears | Lossy Q90 | 0.984527 | 2.254 | 0.000 |
| hardy-sap | Lossy Q90 | 0.977512 | 2.529 | 0.000 |
| plunder | Lossy Q90 | 0.982498 | 2.156 | 0.000 |
| stone-mend | Lossy Q90 | 0.988689 | 1.718 | 0.000 |
| field-promotion | Lossy Q90 | 0.984309 | 1.878 | 0.000 |
| veterans | Lossy Q90 | 0.986910 | 2.014 | 0.000 |
| shield-wall | Lossy Q90 | 0.986799 | 2.110 | 0.000 |
| siege-arrows | Lossy Q90 | 0.972549 | 2.639 | 0.000 |
| armored-brood | Lossy Q90 | 0.984603 | 2.057 | 0.000 |
| triage | Lossy Q90 | 0.984150 | 1.801 | 0.000 |
| lingering-hex | Lossy Q90 | 0.987846 | 2.124 | 0.000 |
| shield-step | Lossy Q90 | 0.991054 | 2.069 | 0.000 |
| war-leech | Lossy Q90 | 0.964009 | 1.929 | 0.000 |
| conquerors | Lossy Q90 | 0.975208 | 2.045 | 0.000 |
| tower-fury | Lossy Q90 | 0.968462 | 2.079 | 0.000 |
| entrenched | Lossy Q90 | 0.983098 | 2.065 | 0.000 |
| grievous-wounds | Lossy Q90 | 0.971650 | 1.921 | 0.000 |
| giant-slayer | Lossy Q90 | 0.969032 | 2.153 | 0.000 |
| executioner | Lossy Q90 | 0.966740 | 1.992 | 0.000 |
| hunter-s-instinct | Lossy Q90 | 0.955362 | 2.403 | 0.000 |
| iron-vanguard | Lossy Q90 | 0.961641 | 2.151 | 0.000 |
| spellweaver | Lossy Q90 | 0.965406 | 2.190 | 0.000 |
| last-bastion | Lossy Q90 | 0.990497 | 2.090 | 0.000 |
| desperate-assault | Lossy Q90 | 0.985077 | 2.129 | 0.000 |
| legacy | Lossy Q90 | 0.984266 | 2.478 | 0.000 |
| second-wind | Lossy Q90 | 0.989279 | 1.942 | 0.000 |
| convergence | Lossy Q90 | 0.986831 | 2.245 | 0.000 |
| royal-banner | Lossy Q90 | 0.983717 | 2.485 | 0.000 |
| deathless-legion | Lossy Q90 | 0.991476 | 2.104 | 0.000 |
| iron-legion | Lossy Q90 | 0.987724 | 2.281 | 0.000 |
| bountiful-harvest | Lossy Q90 | 0.985079 | 2.744 | 0.000 |
| hive-awakening | Lossy Q90 | 0.982064 | 2.701 | 0.000 |

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
