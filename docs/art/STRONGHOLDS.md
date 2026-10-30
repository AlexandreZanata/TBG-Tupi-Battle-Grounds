# Generated Strongholds art

This update changes presentation only. Rules, costs, eligibility and official effect text remain canonical. Originals and Lanczos-normalized masters are retained only as project copies under ignored `.local/art-masters/strongholds/`. Master normalization resizes the entire generated image without cropping.

Optional local tools: `{'vtracer': '0.6.15', 'Pillow': '12.2.0', 'numpy': '2.4.6', 'scipy': '1.18.0'}`. SVG rendering used librsvg via the bundled sharp runtime.

Run from the worktree root (tools live under `.local/art-tools/`):

```bash
ART_SHARP=/home/iiii/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp scripts/arcana-art.sh strongholds --workers 3
```

The shared implementation is `scripts/art_pipeline.py`; `scripts/art_catalog.py` owns visual sizes and budgets. All sets use the same VTracer 0.6.15 settings: color, stacked, spline, filter_speckle=2, color_precision=8, layer_difference=8, corner_threshold=60, length_threshold=4.0, splice_threshold=45, path_precision=3. A full-resolution trace is measured first. Size tuning tries progressively smaller trace inputs; SVG cleanup removes XML comments only and never simplifies path geometry.

SSIM is measured on RGB with a uniform 7×7 window (sample covariance, K1=.01, K2=.03), weighted by reference alpha. RGB MAE is on a 0–255 scale and weighted by alpha. Both full-size and direct display-size renders are checked. A vector must reach full SSIM ≥0.95, display SSIM ≥0.97 and alpha MAE ≤1 in both renders. Otherwise the UI selects a WebP fallback at twice the display size, quality/effort 90. Icons and tower portraits use lossless encoding; frames and opaque illustrations/banners use lossy quality 90 with alpha preserved where present. Failed vector scores below are reported honestly; they are never substituted for the detailed game artwork.

**Total art, including gzip SVG plus WebP: 1,733,333 B.** The budget is 2 MB, including the legacy compact HUD sprite. Display comparison widths are 160 px portraits, 450 px banners and 72 px branch icons. New images have no source URL until the picker opens. The original kind/tier/branch numbers are unchanged. Silhouettes were inspected first in `?gallery=strongholds`.

## Quality and sizes

| Asset | Display px | Trace px | SSIM full/display | RGB MAE full/display | Alpha MAE full/display | SVG raw/gzip B | WebP B | UI source |
|---|---:|---:|---|---|---|---|---:|---|
| treasury | 160 | 128 | 0.235803 / 0.645656 | 26.081 / 17.998 | 20.707 / 21.929 | 145,515 / 40,910 | 128,402 | WebP fallback |
| bastion | 160 | 128 | 0.184726 / 0.641035 | 28.218 / 17.665 | 21.595 / 22.689 | 129,012 / 36,584 | 116,162 | WebP fallback |
| artillery | 160 | 128 | 0.234952 / 0.608352 | 25.867 / 17.996 | 31.384 / 33.030 | 140,296 / 39,166 | 118,044 | WebP fallback |
| command | 160 | 128 | 0.231071 / 0.656791 | 25.664 / 17.136 | 19.014 / 20.221 | 95,614 / 27,349 | 89,810 | WebP fallback |
| banner-treasury | 450 | 192 | 0.347803 / 0.504207 | 21.327 / 17.104 | 0.234 / 0.080 | 199,129 / 59,855 | 130,382 | WebP fallback |
| banner-bastion | 450 | 192 | 0.330076 / 0.509473 | 20.858 / 16.422 | 0.208 / 0.091 | 189,728 / 55,188 | 122,550 | WebP fallback |
| banner-artillery | 450 | 192 | 0.401717 / 0.519311 | 19.199 / 15.803 | 0.229 / 0.080 | 188,682 / 53,666 | 122,018 | WebP fallback |
| banner-command | 450 | 192 | 0.371986 / 0.512851 | 20.192 / 16.383 | 0.100 / 0.077 | 188,343 / 54,877 | 124,708 | WebP fallback |
| armored-vault | 72 | 96 | 0.263830 / 0.711182 | 26.206 / 14.816 | 18.629 / 20.646 | 78,574 / 21,679 | 29,966 | WebP fallback |
| mint | 72 | 80 | 0.257509 / 0.692725 | 26.328 / 15.105 | 23.719 / 25.850 | 63,106 / 18,033 | 30,192 | WebP fallback |
| fortress | 72 | 96 | 0.254240 / 0.743292 | 23.397 / 12.837 | 20.446 / 22.808 | 71,581 / 21,073 | 24,428 | WebP fallback |
| sentinel | 72 | 96 | 0.204695 / 0.755762 | 28.341 / 14.695 | 18.990 / 21.212 | 88,687 / 24,765 | 29,176 | WebP fallback |
| heavy-cannon | 72 | 96 | 0.285771 / 0.796071 | 22.756 / 12.113 | 18.815 / 20.948 | 82,902 / 23,552 | 28,178 | WebP fallback |
| rapid-fire | 72 | 96 | 0.270397 / 0.758813 | 24.589 / 13.757 | 21.060 / 23.396 | 84,026 / 23,866 | 30,066 | WebP fallback |
| war-standard | 72 | 96 | 0.236386 / 0.680324 | 26.063 / 13.978 | 20.930 / 23.443 | 84,481 / 23,410 | 29,270 | WebP fallback |
| forward-banner | 72 | 96 | 0.222120 / 0.711423 | 27.962 / 15.503 | 18.365 / 19.860 | 79,863 / 23,210 | 30,436 | WebP fallback |

## Served fallback fidelity

| Asset | Encoding | Display SSIM | RGB MAE | Alpha MAE |
|---|---|---:|---:|---:|
| treasury | Lossless Q90 | 0.991358 | 0.556 | 0.149 |
| bastion | Lossless Q90 | 0.988940 | 0.594 | 0.172 |
| artillery | Lossless Q90 | 0.974942 | 0.607 | 0.209 |
| command | Lossless Q90 | 0.983516 | 0.628 | 0.139 |
| banner-treasury | Lossy Q90 | 0.985901 | 2.163 | 0.000 |
| banner-bastion | Lossy Q90 | 0.994281 | 1.525 | 0.000 |
| banner-artillery | Lossy Q90 | 0.990280 | 1.834 | 0.000 |
| banner-command | Lossy Q90 | 0.990197 | 1.875 | 0.000 |
| armored-vault | Lossless Q90 | 0.989170 | 0.602 | 0.252 |
| mint | Lossless Q90 | 0.990394 | 0.527 | 0.268 |
| fortress | Lossless Q90 | 0.982020 | 0.566 | 0.271 |
| sentinel | Lossless Q90 | 0.987506 | 0.588 | 0.271 |
| heavy-cannon | Lossless Q90 | 0.991130 | 0.520 | 0.268 |
| rapid-fire | Lossless Q90 | 0.974800 | 0.565 | 0.282 |
| war-standard | Lossless Q90 | 0.970929 | 0.533 | 0.285 |
| forward-banner | Lossless Q90 | 0.988453 | 0.554 | 0.251 |

## Exact selected generation prompts

### `treasury`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: whole Stronghold portrait, exactly 1024x1024 PNG with genuine transparent alpha background. Subject: a squat rectangular fortified stone trading post with a pitched brown roof topped by a small gilded dome, a large circular iron vault door in a bronze frame, a coin chest by the entrance. Consistent frontal camera slightly elevated, miniature fantasy architecture with realistic stone/bronze textures, full building fits inside central 86 percent of the canvas including every flag and cannon. Neutral team colors: bronze metal and parchment cloth. Upper-left light and soft warm rim light. No shield medallion around the building, no landscape, no ground plane, no clipping. Preserve this broad simple structural silhouette; no extra towers or giant palace. Original painterly interpretation of the described game structure.
```

### `banner-treasury`

Requested master: 2048×820, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: Stronghold family banner, exactly 2048x820 PNG, opaque wide landscape. A horizontal dark wooden frame bound with rope and iron corner plates, torn gold cloth on both sides. The complete Treasury Stronghold centered and large: a squat rectangular fortified stone trading post with a pitched brown roof topped by a small gilded dome, a large circular iron vault door in a bronze frame, a coin chest by the entrance. Neutral bronze and parchment team colors. Slightly elevated frontal camera and upper-left lighting. Blurred themed background: a medieval market at dusk. Keep the whole wooden rectangular frame in view with safe margins, and the building fully visible at center, same frame, layout, scale and light across all four banners.
```

### `armored-vault`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: gold. Central symbol: a plated bronze vault door. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft gold glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `mint`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: gold. Central symbol: a bronze coin die stamping a coin. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft gold glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `bastion`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: whole Stronghold portrait, exactly 1024x1024 PNG with genuine transparent alpha background. Subject: a low wide very thick stone fortification with two broad crenellated walls, kite shields on its flanks, a central courtyard, a short squat square keep and front stone steps. Consistent frontal camera slightly elevated, miniature fantasy architecture with realistic stone/bronze textures, full building fits inside central 86 percent of the canvas including every flag and cannon. Neutral team colors: bronze metal and parchment cloth. Upper-left light and soft warm rim light. No shield medallion around the building, no landscape, no ground plane, no clipping. Preserve this broad simple structural silhouette; no extra towers or giant palace. Original painterly interpretation of the described game structure. Critical silhouette correction: NO cannons, NO guns, NO artillery, NO ballistas anywhere on this building. Only stone, wood, shields, railings and parchment banners. Keep a 7 percent fully clear transparent margin on every side, including the top of every flagpole. The complete building must be visible.
```

### `banner-bastion`

Requested master: 2048×820, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: Stronghold family banner, exactly 2048x820 PNG, opaque wide landscape. A horizontal dark wooden frame bound with rope and iron corner plates, torn steel-gray cloth on both sides. The complete Bastion Stronghold centered and large: a low wide very thick stone fortification with two broad crenellated walls, kite shields on its flanks, a central courtyard, a short squat square keep and front stone steps. Neutral bronze and parchment team colors. Slightly elevated frontal camera and upper-left lighting. Blurred themed background: castle walls under an overcast sky. Keep the whole wooden rectangular frame in view with safe margins, and the building fully visible at center, same frame, layout, scale and light across all four banners.
```

### `fortress`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: steel-gray. Central symbol: a massive bronze stone buttress. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft steel-gray glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `sentinel`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: steel-gray. Central symbol: a bronze watchtower. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft steel-gray glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `artillery`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: whole Stronghold portrait, exactly 1024x1024 PNG with genuine transparent alpha background. Subject: an artillery platform on four thick stone pillars, a wooden square deck, one huge long iron cannon pointing slightly toward the viewer's right so its full barrel is visible, exposed brass gearing and ammunition by the cannon. Consistent frontal camera slightly elevated, miniature fantasy architecture with realistic stone/bronze textures, full building fits inside central 86 percent of the canvas including every flag and cannon. Neutral team colors: bronze metal and parchment cloth. Upper-left light and soft warm rim light. No shield medallion around the building, no landscape, no ground plane, no clipping. Preserve this broad simple structural silhouette; no extra towers or giant palace. Original painterly interpretation of the described game structure.
```

### `banner-artillery`

Requested master: 2048×820, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: Stronghold family banner, exactly 2048x820 PNG, opaque wide landscape. A horizontal dark wooden frame bound with rope and iron corner plates, torn rust-orange cloth on both sides. The complete Artillery Stronghold centered and large: an artillery platform on four thick stone pillars, a wooden square deck, one huge long iron cannon pointing slightly toward the viewer's right so its full barrel is visible, exposed brass gearing and ammunition by the cannon. Neutral bronze and parchment team colors. Slightly elevated frontal camera and upper-left lighting. Blurred themed background: a smoky siege field. Keep the whole wooden rectangular frame in view with safe margins, and the building fully visible at center, same frame, layout, scale and light across all four banners.
```

### `heavy-cannon`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: rust-orange. Central symbol: a thick bronze heavy cannon barrel. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft rust-orange glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `rapid-fire`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: rust-orange. Central symbol: a bronze rotating ammunition drum. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft rust-orange glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `command`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: whole Stronghold portrait, exactly 1024x1024 PNG with genuine transparent alpha background. Subject: a round stone tower carrying a broad circular wooden observation platform and railing, a bronze royal crest on the front and one large parchment-colored standard raised over the platform. Consistent frontal camera slightly elevated, miniature fantasy architecture with realistic stone/bronze textures, full building fits inside central 86 percent of the canvas including every flag and cannon. Neutral team colors: bronze metal and parchment cloth. Upper-left light and soft warm rim light. No shield medallion around the building, no landscape, no ground plane, no clipping. Preserve this broad simple structural silhouette; no extra towers or giant palace. Original painterly interpretation of the described game structure. Critical silhouette correction: NO cannons, NO guns, NO artillery, NO ballistas anywhere on this building. Only stone, wood, shields, railings and parchment banners. Keep a 7 percent fully clear transparent margin on every side, including the top of every flagpole. The complete building must be visible.
```

### `banner-command`

Requested master: 2048×820, opaque.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: Stronghold family banner, exactly 2048x820 PNG, opaque wide landscape. A horizontal dark wooden frame bound with rope and iron corner plates, torn royal-purple cloth on both sides. The complete Command Stronghold centered and large: a round stone tower carrying a broad circular wooden observation platform and railing, a bronze royal crest on the front and one large parchment-colored standard raised over the platform. Neutral bronze and parchment team colors. Slightly elevated frontal camera and upper-left lighting. Blurred themed background: a military camp with flags. Keep the whole wooden rectangular frame in view with safe margins, and the building fully visible at center, same frame, layout, scale and light across all four banners.
```

### `war-standard`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: royal-purple. Central symbol: a bronze standard surrounded by laurels. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft royal-purple glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

### `forward-banner`

Requested master: 1024×1024, transparent.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: royal-purple. Central symbol: a bronze forward flag planted over an unfurled map. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft royal-purple glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coherent set.
```

## Rejected generations

### `bastion`

Reason: Added cannon emplacements conflict with the non-artillery game silhouette.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: whole Stronghold portrait, exactly 1024x1024 PNG with genuine transparent alpha background. Subject: a low wide very thick stone fortification with two broad crenellated walls, kite shields on its flanks, a central courtyard, a short squat square keep and front stone steps. Consistent frontal camera slightly elevated, miniature fantasy architecture with realistic stone/bronze textures, full building fits inside central 86 percent of the canvas including every flag and cannon. Neutral team colors: bronze metal and parchment cloth. Upper-left light and soft warm rim light. No shield medallion around the building, no landscape, no ground plane, no clipping. Preserve this broad simple structural silhouette; no extra towers or giant palace. Original painterly interpretation of the described game structure.
```

### `command`

Reason: Added cannon emplacements conflict with the non-artillery game silhouette.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. No watermark, no letters. Do not paste, trace or crop any existing reference. Asset type: whole Stronghold portrait, exactly 1024x1024 PNG with genuine transparent alpha background. Subject: a round stone tower carrying a broad circular wooden observation platform and railing, a bronze royal crest on the front and one large parchment-colored standard raised over the platform. Consistent frontal camera slightly elevated, miniature fantasy architecture with realistic stone/bronze textures, full building fits inside central 86 percent of the canvas including every flag and cannon. Neutral team colors: bronze metal and parchment cloth. Upper-left light and soft warm rim light. No shield medallion around the building, no landscape, no ground plane, no clipping. Preserve this broad simple structural silhouette; no extra towers or giant palace. Original painterly interpretation of the described game structure.
```
