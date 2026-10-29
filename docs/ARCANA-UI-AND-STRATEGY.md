# Arcana UI preview and competitive integration strategy

> Historical: this document describes the six-node preview. D-42/P40
> replaced the live tree with fifteen nodes and twelve points; GAME-DESIGN
> §9 and `docs/phases/P40-arcana-trees.md` are the current sources.

This branch supplies a desktop art/editor preview only. The badge reads
**Preview - not used in matches yet**. Changes are confined to the client,
art tooling, screenshots and this document. No selection crosses the HTML
`data-action` bridge; no protocol, simulation, server, queue or match changes.
All competitive values remain the Proposed hypotheses in GAME-DESIGN §9.

Base: `main` at `1ae5894`. Worktree: `../TBG-arcana-wt`; branch:
`feat/arcana-ui-preview`. Never push or merge this branch during execution.

## Preview behavior and BR-AR01–BR-AR10

- **BR-AR01:** every visitor sees every node and all six points immediately.
  There is no account requirement, purchase, progression or unlock.
- **BR-AR02:** the manifest owns the six-point budget, rank cap of two and
  serialization order: Guard, Assault, Piercing, Endurance, Resilience,
  Renewal. The complete default is `[2,0,2,0,2,0]`. Reset and partial drafts
  remain editable; a partial draft is explicitly incomplete. A rank can be
  added by click, Enter, Space or plus; right-click or minus removes it.
  The 141 complete selections are exercised exhaustively by a client test.
- **BR-AR03:** affected units are derived from base Archetype in the local
  roster manifest. Evolution does not change eligibility. War includes
  Rampart; Vitality Resilience includes Nester. Renewal reaches Bloomkeeper
  only: Nester has no existing heal. Sawbot/Siege Walker use their approved
  display names Sawknight/Ogre Archer.
- **BR-AR04:** descriptions state additive percentages of the base attribute.
  This preview computes no unit stats. The future core must add Evolution,
  Synergy and Arcana basis points using wide intermediates and truncate
  once, preserving the current HP ratio when max HP changes. Do not
  multiply already rounded display values or add a second scaling pass.
- **BR-AR05:** today the modal exists only over the idle desktop lobby. The
  existing lobby lifecycle closes it when the lobby disappears, including
  queue/room handoff, Practice and online matches. Future applied selections
  must freeze at server admission and survive reconnect/account takeover.
- **BR-AR06:** browser bounds protect editing only. Later the authoritative
  server must validate shape, ranks, budget, schema, RulesetVersion and
  session state. The seat comes from the connection. This branch sends no
  selection or bonus and cannot grant a gameplay effect.
- **BR-AR07:** later record both server-applied presets, Arcana schema and
  RulesetVersion with each result. Never reinterpret old outcomes or Rating
  when values change. The local preview save is not a match record.
- **BR-AR08:** preview storage is a bounded `{schema,ranks}` document under
  `tbg.arcana.preview`, guarded by try/catch. Corrupt, oversized or unknown
  versions recover the complete default with a visible explanation.
  Storage denial keeps a usable draft for the visit. No account or
  cross-device save is claimed; P37 still supplies those guarantees.
- **BR-AR09:** the preview changes neither Casual nor Ranked. Their eventual
  released Arcana Ruleset must be identical, with no identity, Elo, FIFO,
  Energy, Shop or Evolution changes. D-32 removed SlotCount/army caps;
  obsolete SlotCount wording in older Arcana specs needs reviewer cleanup.
- **BR-AR10:** the six descriptions cover max HP, normal damage and existing
  healing only. No new heal, status, cadence, range, movement, regeneration
  rate, splash radius or Stronghold bonus is implied.

The native modal supplies focus containment and Escape closure. Labels,
rank numbers and pips complement color; focus rings and reduced-motion
styles are included. Below 1280×720 or in portrait the existing menu is
retained. Images are requested only when the already interactive desktop
menu opens the preview, with asynchronous decoding and lazy loading.

## Competitive integration with P16-C

D-33 and P16-C are not present on this worktree's base `main`. Their
requested Ruleset/policy sections were read directly with `git show
phase/p16c-balance-lab:<path>` at `e5bff83`; no files in that agent's
worktree were read or modified. This strategy is a proposal, not a change
to the running P16-C work or its acceptance thresholds.

After P16-C is reviewed, extend its Ruleset data with the six Arcana
per-rank basis-point values and an explicit versioned release choice.
Keep structural rank layout/schema separate from balance knobs. Build
canonical values from the approved content source once. A preset is an
input to the deterministic core; the UI JSON must mirror approved data,
not become an alternative authority for combat.

Adapt P33's neutral constructors, preset validation and additive Stats
calculation to `MatchState::with_rules` rather than creating a second
modifier or lab framework. Public server/client entry points continue to
use only CANON. Non-canonical candidates stay confined to native lab runs.
The preview's JSON contract test catches documentation drift while the
core does not yet expose an Arcana export.

Evaluate **all 141 complete presets**, first under the same policy and
then against several independently useful opponent policies. Reuse P16-C's
Bot, Heavy, Cheap, Evolver, Saver, each Faction and each Mono unit policy;
Random is a separate sanity floor. Preserve legal-view information and
reaction schedules. Include heavy/Rampart spam, ranged concentration,
Nester populations and Support/Control compositions as targeted stress
cases. Do not select balance values against only the current heuristic bot.

Use participant-paired seeds with a seat swap, carrying each participant's
initial Shop RNG/offers and policy RNG to the opposite seat. The pair,
not each individual game, is the sampling unit. Adaptive decisions can
change later Shop exposure; record it instead of promising identical
future draws. Keep diagnostic neutral presets out of the competitive
selection pool and report their controls separately.

Reuse deterministic pair bootstrap 95% confidence intervals and P16-C's
separate discovery, screening and untouched confirmation seed blocks.
Freeze policies, preset set and stopping rules before confirmation. Record
HEAD, Ruleset fingerprint/version, Arcana schema, policies, reactions,
seeds, thread count and commands. Thread count must not change results.
Screen the full selection space; confirm the best, worst, counter and
non-dominated candidates on held-out pairs. If exhaustive all-policy
141×141 confirmation is costly, report the coverage honestly and keep
unconfirmed cells out of release claims.

Apply GAME-DESIGN §9's hypotheses: investigate aggregate scores **above
55%** or any matchup **above 65%**; require at least **three competitive,
non-dominated presets**, and a default within **3 percentage points** of
the best generalist reference. A 50% aggregate can conceal oppressive
matchups. Confidence intervals and sample counts accompany every score;
intervals crossing a gate mean insufficient evidence, not a pass. Reuse
P16-C meta-game/exploitability analysis and the existing §6 fun gates.
An unresolved counterless advantage blocks release. Human validation still
requires eight participants, 24 completed matches and at least 80%
understanding their preset and a relevant reason for defeat.

## Open army and summon eligibility

D-32 makes percentage gains scale with the purchased army: a small
per-unit HP or damage gain can accumulate across many durable units and
extend the lifetime of a growing formation. Heavy spam may amplify this
through survivors, tanking and sustained damage, even without compounding
percentage formulas. Observe purchase distribution, time alive, damage,
healing, max army size, queueing around Rampart and held-out counterplay.
No new unit cap, cost or effect is authorized here.

War covers Tank+DPS: Shieldbearer, Bone Brute, Rampart and Sawknight.
Precision covers Ranged: Archer and Ogre Archer. Vitality Resilience covers
Support+Control: Bloomkeeper, Nester, Hexer and Rootcaller. Renewal requires
an existing heal and therefore reaches Bloomkeeper alone, including its
attacking Evolution forms that retain healing priority.

**Proposed:** summons receive no Arcana bonuses, consistent with BR-U08's
Synergy exclusion. Beetling remains listed as a summon and explicitly
unaffected in the preview, with the proposal label visible. BR-AR03 alone
would otherwise classify its base DPS Archetype as eligible for War; that
ambiguity must be decided by the owner and entered into canon before
implementation. Test direct stat paths and inherited Nester Evolution
separately so an exclusion cannot be bypassed at hatch.

## Remaining integration sequence after P16-C

Respect D-25's post-P31 gameplay gate and the P32 combat audit. This
owner-requested visual preview does not activate or complete P33–P37.

1. **P33:** rebase the deterministic experiment on the reviewed P16-C
   Ruleset; implement bounded presets, exact additive integer calculations,
   purchase/Evolution/Synergy refresh integration and contract tests. Include
   Nester/Rampart and the owner-decided summon rule. Existing preview
   descriptions and rank interaction are reusable; no combat work is done.
2. **P34:** reuse the P16-C runner, policies, paired-seed statistics,
   candidate registry and reports for the full preset screen and held-out
   confirmation. Review balance and human evidence before online work.
3. **P35:** implement authoritative selection/ack/rejection and versioned
   handshake/MatchRules, queue freezes, reconnect and account takeover
   correctness, applied-preset records and browser protocol regression.
4. **P36:** connect the reviewed UI to the draft/accepted/frozen lifecycle,
   server ack before queue admission and core-calculated integer previews.
   Add read-only match/result selections, Practice parity and phone UI.
   This preview already supplies desktop art, families/nodes, effect copy,
   eligibility, point editing, local preview storage and accessibility.
   Replace the preview badge/storage only when those contracts are real.
5. **P37:** implement one durable account preset, guest/account transitions,
   schema/ruleset migration, database and async ownership/race tests, and
   ranked history metadata. Local preview storage is never silently adopted
   as a server-applied or cloud-saved selection.

P38 remains the public release gate: owner look/fun approval, performance,
security and real-server/multiplayer checks still apply. Update stale
P36 references to retired P19/P20/P21 and obsolete eight-unit examples in
P33 only in a later reviewed spec change.

## Proposed decision (D-number intentionally unassigned)

**D-__ · Arcana desktop art/editor preview before gameplay integration**

**Decision.** Allow an explicitly labeled, browser-local desktop Arcana
workshop over the D-27 lobby. Use three consistent fantasy art families,
six stable-order nodes and exactly the Proposed GAME-DESIGN §9 values.
Ship vector assets where quality/size allow and alpha-preserving WebP
fallbacks elsewhere. No preview selection affects a match or crosses the
network. Authoritative gameplay remains gated by D-25, P16-C and P32–P38.

**Why.** Art and interaction can be reviewed independently while balance
work proceeds in an isolated worktree. The small editor makes six-point
tradeoffs and Archetype eligibility inspectable without claiming balance.

**Guard rails.** Free complete default; no progression, paid power or
account restriction; canonical node-table drift test; local storage only;
no server or core modifications; lazy art requests; raster quality takes
precedence over vector format. Summon exclusion needs a separate explicit
canon decision before integration.

**Reopen when.** The owner rejects the appearance, lab/human evidence
rejects the six-node candidate, or approved values/schema change.

## Owner questions for review

- Approve the art direction and the mind-map interaction at both desktop sizes?
- Approve excluding Beetling and future summons from Arcana bonuses?
- Does the lab find at least three distinct competitive choices, or should
  percentage Arcanas be revised/removed before investing in online support?
- After P16-C/P31/P32, should remaining phases reuse the reviewed lab and
  this desktop shell while keeping P36 phone work and P37 persistence?

These questions do not block a browser-only preview and authorize no
competitive rule or release change.

## Art generation, conversion and measurements

Generation uses the built-in imagegen tool, one original image per asset.
The six owner references were inspected locally for style only. They are
never copied, traced or committed. Project copies of full tool outputs and normalized master
PNGs are retained under ignored `.local/art-masters/arcana/`.

The tool emitted 1254×1254 square originals and 1983×793 banners despite
requested dimensions. Originals are retained with `-original.png` names;
Lanczos normalization produces the requested 1024×1024 and 2048×820 masters
without cropping or using reference pixels. Fourteen generation calls
produced twelve selected assets: War was regenerated for scale and then
for its correct red glow. Rejected outputs stay in ignored local storage.

### Exact selected generation prompts

#### `war`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: red. Central symbol: two crossed bronze swords, broad blades pointing upward. Asset type: family emblem, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft red glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the other Arcana emblems with red light for War as a coordinated set.
```

#### `precision`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: blue. Central symbol: arrow piercing the center of a circular target. Asset type: family emblem, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft blue glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coordinated set.
```

#### `vitality`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: green. Central symbol: bronze vines and leaves encircling a glowing green drop. Asset type: family emblem, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft green glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coordinated set.
```

#### `guard`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: red. Central symbol: one simple bronze heraldic shield. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft red glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coordinated set.
```

#### `assault`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: red. Central symbol: one simple bronze upright blade. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft red glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coordinated set.
```

#### `piercing`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: blue. Central symbol: one simple bronze arrowhead pointing upward. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft blue glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coordinated set.
```

#### `endurance`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: blue. Central symbol: one bronze heart mounted on a curved quiver strap. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft blue glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coordinated set.
```

#### `resilience`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: green. Central symbol: one simple bronze bark-textured shield. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft green glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coordinated set.
```

#### `renewal`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: green. Central symbol: one bronze leaf holding a glowing green drop. Asset type: node icon with simple readable central symbol, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft green glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the prior War emblem as a coordinated set.
```

#### `banner-war`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: red. Central symbol: two crossed bronze swords. Asset type: family banner, exactly 2048x820 PNG, opaque landscape. Horizontal dark wooden frame with rope bindings and iron corner plates. Torn red cloth hangs on both sides of the central round shield emblem. Blurred fortified battlefield at sunset behind it. Emblem occupies center 36 percent of width; entire wooden rectangular frame is visible, 4 percent safe margin, same camera, lighting, composition and frame dimensions across all three banners.
```

#### `banner-precision`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: blue. Central symbol: bronze arrow piercing a circular target. Asset type: family banner, exactly 2048x820 PNG, opaque landscape. Horizontal dark wooden frame with rope bindings and iron corner plates. Torn blue cloth hangs on both sides of the central round shield emblem. Blurred castle and archery grounds under a blue sky behind it. Emblem occupies center 36 percent of width; entire wooden rectangular frame is visible, 4 percent safe margin, same camera, lighting, composition and frame dimensions across all three banners.
```

#### `banner-vitality`

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: green. Central symbol: bronze vines and leaves around a glowing green drop. Asset type: family banner, exactly 2048x820 PNG, opaque landscape. Horizontal dark wooden frame with rope bindings and iron corner plates. Torn green cloth hangs on both sides of the central round shield emblem. Blurred overgrown woodland ruins behind it. Emblem occupies center 36 percent of width; entire wooden rectangular frame is visible, 4 percent safe margin, same camera, lighting, composition and frame dimensions across all three banners.
```

### Rejected War prompts

Attempt 1: First frame exceeded the common safe margin.

```text
Use case: stylized-concept. Create an original fantasy strategy game family emblem for War, 1024x1024 PNG with genuine transparent background. Straight front camera, centered round dark-iron shield with a thick weathered bronze rim and exactly four bronze diamond studs at top, bottom, left and right. Frame occupies 86 percent of the canvas, all studs visible with safe margin. Torn crimson-red cloth drapes across the upper shield and down its sides. One central sculpted bronze symbol: two crossed swords, broad blades pointing upward. Painterly high-detail premium medieval game icon, tactile brushed bronze and rough woven cloth, soft red family glow confined close to the shield. Light from upper left, warm highlights, deep cool iron shadows. Consistent frame and lighting for a twelve-asset Arcana set. No text, letters, watermark, extra studs, scenery or background. Design a new original object; do not copy or trace any reference.
```

Attempt 2: Regeneration inherited a blue glow; corrected to red.

```text
Use case: stylized-concept. Create one new original premium painterly medieval fantasy game asset. Straight-front orthographic camera, sculpted weathered bronze with highlights from upper left, cool dark hammered iron, rich rough woven cloth, high detail without lettering. Match a coherent Arcana set: a round dark-iron shield, thick bronze rim, exactly four bronze diamond studs at cardinal points, torn cloth in the family color draped over the upper rim and along both sides. No watermark, no letters. Do not paste, trace or crop any existing reference. Family color: red. Central symbol: two crossed bronze swords, broad blades pointing upward. Asset type: family emblem, exactly 1024x1024 PNG with genuine transparent alpha background. Shield including all cloth and studs fits inside central 86 percent of canvas; generous clear transparent margins, no clipping. All four studs visible. Soft blue glow close to the symbol, no scenery, no opaque background. Frame, stud positions and light must match the other Arcana emblems as a coordinated set.
```

### Reproducible conversion and quality comparison

Run from the worktree root. All packages are optional local tools; none
was added to Cargo.toml, Cargo.lock or a project dependency manifest.

```bash
# Optional isolated installation; using existing local Python packages also works.
uv venv --system-site-packages .local/art-tools/venv
uv pip install --python .local/art-tools/venv/bin/python vtracer==0.6.15 pillow numpy scipy cairosvg
ARCANA_PYTHON=.local/art-tools/venv/bin/python scripts/arcana-art.sh

# Exact renderer selection used for the reported measurements:
ARCANA_SHARP=/home/iiii/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp scripts/arcana-art.sh
# Recheck existing vectors and rebuild WebP without repeating vectorization:
ARCANA_SHARP=/home/iiii/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp scripts/arcana-art.sh --measure-only
```

Execution was split into per-asset invocations while generation completed:
`precision`; `vitality banner-war banner-precision banner-vitality guard
assault piercing endurance resilience renewal`; `war`; then
`--measure-only` for the complete set and the final lossless node encoding.
The first partial runs are in `.local/arcana-art.log`; the final output
and measurements are in `.local/arcana-art-final.log` and
`.local/art-masters/arcana/quality.json`. Those local files are not shipped.

The exact VTracer Python call for every image is:

```python
vtracer.convert_image_to_svg_py(
    source_png, output_svg,
    colormode="color", hierarchical="stacked", mode="spline",
    filter_speckle=2, color_precision=8, layer_difference=8,
    corner_threshold=60, length_threshold=4.0,
    splice_threshold=45, path_precision=3,
)
```

Installed binding: **VTracer 0.6.15**. Its generated XML comment reports
upstream generator 0.6.12; the installed package version above is the tool
version. Other execution packages: Pillow 12.2.0, NumPy 2.4.6, SciPy 1.18.0.
SVG rasterization used **librsvg 2.63.2 via sharp 0.35.5 / libvips 8.18.7**.
The script also supports CairoSVG when selected through a local Python.

Each image was first traced at full master resolution with the settings
above. Those outputs were already far above the per-image budgets and
failed the fidelity gate. The script then tries decreasing trace input
widths, retaining the highest width below 56% of its per-image ceiling to
reserve transport space for the required raster fallback. Emblem/node
candidates are 384, 320, 256, 224, 192, 160, 128 px; banners try 640, 512,
448, 384, 320, 256, 224, 192, 160 px. Aspect ratio is retained; the SVG
viewBox scales to full master dimensions. Final input widths are recorded
below. This is conversion-budget tuning, not a claim of lossless tracing.

After tracing, only XML comments are removed and the viewport is declared.
Path data is never simplified or re-rounded. Gzip uses level 9 and a zero
mtime (`gzip.compress(..., mtime=0)`). Vector silhouettes retain transparent
surroundings, but VTracer does not reproduce the soft alpha fringe
accurately enough; the UI uses alpha-preserving WebP instead.

SSIM uses RGB channels with a standard 7×7 uniform window, K1=.01, K2=.03,
255 range and sample covariance. Average the three channel maps weighted
by the reference alpha, excluding the three-pixel window border. RGB MAE
is weighted by reference alpha over the full image, on a 0–255 scale.
Transparent backgrounds cannot inflate the scores. Alpha MAE is also
recorded separately in the local quality JSON. SVGs are rendered both at
master dimensions and directly at display dimensions (160 px emblems,
72 px nodes, 900 px wide banners), using density scaled to the target size.
A separate scikit-image 0.26.0 check reproduced the Guard WebP SSIM
0.9571731238537599 for the initial lossy candidate, validating the metric
implementation before its lossless replacement.

**Fidelity result:** none of the SVGs meets both SSIM gates. The UI
therefore uses WebP for all twelve assets, as explicitly allowed by the
fallback requirement. The size-limited SVGs are conversion artifacts;
they are never presented as high-fidelity game art. Full-resolution trace
attempts remain local. The game receives the detailed original appearance
through the WebPs. Emblems and banners use lossy quality 90, method 6;
nodes use lossless WebP with quality/effort 90, method 6, because this
improves their 72 px appearance while remaining inside the global budget.
All are exactly twice the display dimensions (320×320, 144×144,
1800×720). Display-size WebP SSIM is reported separately, including any
remaining resampling/compression difference from the master.

### Final SVG quality and sizes

SSIM targets are ≥0.95 at full resolution and ≥0.97 at display size.
All rows intentionally select the WebP fallback after failing those targets.
MAE pairs below are full/display, on the 0–255 RGB scale.

| Asset | Trace width | Full SSIM | Display SSIM | RGB MAE full/display | SVG raw B | SVG gzip B | WebP fallback |
|---|---:|---:|---:|---:|---:|---:|---|
| war | 224 | 0.348131 | 0.811697 | 17.062 / 9.059 | 439,960 | 125,041 | Yes |
| precision | 224 | 0.365764 | 0.790861 | 18.896 / 11.593 | 430,479 | 124,482 | Yes |
| vitality | 224 | 0.360984 | 0.792135 | 18.585 / 10.926 | 446,266 | 130,754 | Yes |
| guard | 192 | 0.379956 | 0.882603 | 17.319 / 8.135 | 293,600 | 82,764 | Yes |
| assault | 160 | 0.307816 | 0.866799 | 19.032 / 7.860 | 214,060 | 59,910 | Yes |
| piercing | 160 | 0.309634 | 0.855092 | 19.669 / 8.682 | 206,559 | 59,223 | Yes |
| endurance | 160 | 0.311898 | 0.833128 | 20.733 / 9.421 | 207,229 | 58,312 | Yes |
| resilience | 160 | 0.273578 | 0.825424 | 20.886 / 8.777 | 217,178 | 63,168 | Yes |
| renewal | 160 | 0.322007 | 0.846891 | 18.771 / 8.291 | 226,825 | 65,810 | Yes |
| banner-war | 320 | 0.480102 | 0.537736 | 14.489 / 13.150 | 536,327 | 156,098 | Yes |
| banner-precision | 320 | 0.459736 | 0.537427 | 17.297 / 15.846 | 538,113 | 159,999 | Yes |
| banner-vitality | 320 | 0.388302 | 0.473121 | 18.536 / 16.956 | 576,345 | 166,035 | Yes |

### Full-resolution starting attempts (kept local)

| Asset | Full trace SSIM | Raw B | Gzip B |
|---|---:|---:|---:|
| war | 0.739350 | 9,756,272 | 2,870,653 |
| precision | 0.779972 | 9,197,597 | 2,725,274 |
| vitality | 0.772323 | 9,989,449 | 3,020,259 |
| guard | 0.776344 | 8,794,568 | 2,592,873 |
| assault | 0.775927 | 9,357,342 | 2,767,400 |
| piercing | 0.765201 | 9,738,618 | 2,886,829 |
| endurance | 0.767508 | 9,044,175 | 2,678,012 |
| resilience | 0.752221 | 9,277,152 | 2,761,786 |
| renewal | 0.775228 | 9,460,544 | 2,828,061 |
| banner-war | 0.860003 | 18,474,367 | 5,905,729 |
| banner-precision | 0.860095 | 19,577,513 | 6,212,315 |
| banner-vitality | 0.850391 | 21,134,480 | 6,621,815 |

### WebP assets actually used by the editor

| Asset | WebP B | Encoding at quality 90 | Display SSIM | RGB MAE |
|---|---:|---|---:|---:|
| war | 40,866 | Lossy | 0.971452 | 2.692 |
| precision | 41,738 | Lossy | 0.971661 | 3.099 |
| vitality | 45,416 | Lossy | 0.974776 | 2.443 |
| guard | 26,736 | Lossless | 0.985436 | 0.511 |
| assault | 27,012 | Lossless | 0.983357 | 0.492 |
| piercing | 27,186 | Lossless | 0.978889 | 0.527 |
| endurance | 27,760 | Lossless | 0.983297 | 0.560 |
| resilience | 27,440 | Lossless | 0.975955 | 0.508 |
| renewal | 28,296 | Lossless | 0.983094 | 0.485 |
| banner-war | 358,028 | Lossy | 0.985281 | 1.669 |
| banner-precision | 359,492 | Lossy | 0.989231 | 1.672 |
| banner-vitality | 419,114 | Lossy | 0.989710 | 1.695 |

SVG gzip total: **1,251,596 B**. WebP total actually requested by the
editor: **1,429,084 B**. Combined: **2,680,680 B
(2.681 MB)**, below the 3,000,000 B ceiling. Every SVG
also meets its own gzip size ceiling (250,000 B emblems, 150,000 B nodes,
400,000 B banners). No art request is made before the menu opens the editor.

## Execution report — 2026-10-29

Built twelve original Arcana assets and their budgeted SVG/WebP pairs,
a desktop mind-map editor over the live arena, a canonical JSON manifest,
client contracts, reproducible art/browser scripts and the strategy above.
The tab and card both open the preview. No gameplay, protocol, server,
core, balance crate, roadmap, decision log or canon file was changed.

### Verification commands and actual output

All final commands below exited 0. Full local logs stay in ignored `.local/`.
The test output excerpt retains the actual binary names and result lines;
110 workspace tests passed, with no failures or ignored tests.

```text
$ cargo fmt --all -- --check
(no output; exit 0)

$ cargo clippy --workspace -- -D warnings
    Checking tbg-core v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-core)
    Checking tbg-server v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-server)
    Checking tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 38s

$ cargo clippy -p tbg-client --target wasm32-unknown-unknown -- -D warnings
    Checking tbg-core v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-core)
    Checking tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.01s

$ cargo test --workspace
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.54s
     Running unittests src/main.rs (target/debug/deps/tbg_client-5f7f2e8e212888a4)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running unittests src/lib.rs (target/debug/deps/tbg_core-f8297775b32adaea)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accounts.rs (target/debug/deps/accounts-955af726fa3564d1)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/contract_constants.rs (target/debug/deps/contract_constants-274e84a05ccccee1)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/contract_determinism.rs (target/debug/deps/contract_determinism-0c0694f52134d996)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
     Running tests/contract_protocol.rs (target/debug/deps/contract_protocol-102087de9bacece1)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/contract_protocol_server.rs (target/debug/deps/contract_protocol_server-224d1a876067c296)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/contract_roster.rs (target/debug/deps/contract_roster-8d6072e7a404df6c)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/open_army.rs (target/debug/deps/open_army-f909ddf293f256c7)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.83s
     Running tests/rating.rs (target/debug/deps/rating-b2ece2568729bc17)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/rules.rs (target/debug/deps/rules-a1918cdbb59202c9)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running unittests src/lib.rs (target/debug/deps/tbg_server-5dab9106a7d3e6c8)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/bin/tbg-bots.rs (target/debug/deps/tbg_bots-cb9586640711bbe1)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/main.rs (target/debug/deps/tbg_server-07e43d10df46fef6)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accounts.rs (target/debug/deps/accounts-fcda8bca02b8df74)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.12s
     Running tests/contract.rs (target/debug/deps/contract-2c20b06bf31e1d31)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
     Running tests/leaderboard.rs (target/debug/deps/leaderboard-9f8d90b13f171dbd)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.51s
     Running tests/online.rs (target/debug/deps/online-1b0b8bb63eab3bf1)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.05s
     Running tests/ranked.rs (target/debug/deps/ranked-bf97380c403dd2d2)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.03s
   Doc-tests tbg_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests tbg_server
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
    Checking tbg-core v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-core)
    Checking tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.52s

$ cd crates/tbg-client && trunk build --release
   Compiling tbg-core v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-core)
   Compiling tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-arcana-wt/crates/tbg-client)
    Finished `release` profile [optimized] target(s) in 1m 32s

$ trunk build --release --config crates/tbg-client/Trunk.toml
    Finished `release` profile [optimized] target(s) in 0.31s
```

The final Trunk invocation repackaged the complete assets and compact-layout fix.

```text
$ node --check crates/tbg-client/assets/arcana/editor.js
(no output; exit 0)

$ node --check crates/tbg-client/assets/arcana/preset.js
(no output; exit 0)

$ node --check crates/tbg-client/tests/arcana.test.js
(no output; exit 0)

$ node --check scripts/arcana-browser.cjs
(no output; exit 0)

$ bash -n scripts/arcana-art.sh
(no output; exit 0)

$ node --test crates/tbg-client/tests/arcana.test.js
✔ BR-AR02/03/10: six nodes match the canon table, serialization order and values (5.169409ms)
✔ BR-AR02: exactly six points, bounded ranks, valid default and 141 complete presets (11.221728ms)
✔ every manifest art asset exists locally and has a safe filename (1.111361ms)
✔ BR-AR03/10 preview: roster includes Nester, Rampart; Renewal only reaches an existing healer (0.328385ms)
ℹ tests 4
ℹ suites 0
ℹ pass 4
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 166.291375

$ bash -n scripts/arcana-art.sh
(no output; exit 0)

$ scripts/arcana-art.sh --measure-only  # ARCANA_SHARP set as documented above
TOTAL gzip SVG + WebP bytes: 2680680
```

The server was run against this worktree's dist and a temporary local database:

```bash
TBG_ADDR=127.0.0.1:18747 TBG_DB=.local/arcana-preview.sqlite3 TBG_STATIC_DIR=crates/tbg-client/dist cargo run -p tbg-server --bin tbg-server
```

Browser automation used local Chromium 1228 / SwiftShader, not a mock page.
To keep software rendering manageable, the final bounded runs used the
existing `?shadows=0` rendering option; gallery captures used
`?gallery&shadows=0`. No simulation setting was changed. These observations
are not frame-rate measurements or physical Android/iPhone approval.

```bash
export ARCANA_PLAYWRIGHT=/home/iiii/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright
export ARCANA_CHROMIUM=/home/iiii/.cache/ms-playwright/chromium-1228/chrome-linux64/chrome
node scripts/arcana-browser.cjs desktop-1920
node scripts/arcana-browser.cjs desktop-1280
node scripts/arcana-browser.cjs art
node scripts/arcana-browser.cjs gallery-1920
node scripts/arcana-browser.cjs gallery-phone
```

Every bounded invocation exited 0. Actual browser check output:

```json
{
  "desktop-1920": [
    {
      "viewport": "1920x1080",
      "lazyArt": true,
      "entries": 2,
      "emptyDefaultPartial": true,
      "keyboard": true,
      "rightClick": true,
      "budgetCap": true,
      "persistence": true,
      "invalidStorage": true,
      "storageDenied": true,
      "focusReturn": true,
      "phoneCloses": true,
      "noEditRequests": true,
      "panel": {
        "content": 816,
        "visible": 816
      }
    }
  ],
  "desktop-1280": [
    {
      "viewport": "1280x800",
      "lazyArt": true,
      "emptyDefaultPartial": true,
      "panel": {
        "content": 620,
        "visible": 620
      }
    }
  ]
}
```

Both entry points worked; empty/default/partial presets, right-click,
Enter/Space and +/- input, rank/budget caps, reload persistence, malformed
storage recovery, denied storage, focus containment/return and closing
when resized to phone were exercised. Editing made no HTTP requests and
left the Rust action bridge empty. Desktop runs recorded zero uncaught
JavaScript errors and zero failed HTTP responses. Gallery and close-up
screenshots were visually inspected. Reduced-motion mode was enabled in
the desktop runs; focus rings and text/pips complement family colors.

### Screenshot evidence and file sizes

All JPEGs are below 300,000 B. Close-ups show the shipped game assets,
not the references or master comparisons; those remain local only.

| Screenshot | Dimensions | Bytes |
|---|---|---:|
| [editor-1280x800-default.jpg](media/arcana-ui/editor-1280x800-default.jpg) | 1280×800 | 152,460 |
| [editor-1280x800-empty.jpg](media/arcana-ui/editor-1280x800-empty.jpg) | 1280×800 | 146,104 |
| [editor-1280x800-partial.jpg](media/arcana-ui/editor-1280x800-partial.jpg) | 1280×800 | 154,195 |
| [editor-1920x1080-default.jpg](media/arcana-ui/editor-1920x1080-default.jpg) | 1920×1080 | 209,315 |
| [editor-1920x1080-empty.jpg](media/arcana-ui/editor-1920x1080-empty.jpg) | 1920×1080 | 201,266 |
| [editor-1920x1080-partial.jpg](media/arcana-ui/editor-1920x1080-partial.jpg) | 1920×1080 | 211,281 |
| [emblems-close-up.jpg](media/arcana-ui/emblems-close-up.jpg) | 1200×520 | 74,731 |
| [gallery-1920x1080.jpg](media/arcana-ui/gallery-1920x1080.jpg) | 1920×1080 | 257,940 |
| [gallery-390x844.jpg](media/arcana-ui/gallery-390x844.jpg) | 390×844 | 33,545 |
| [nodes-close-up.jpg](media/arcana-ui/nodes-close-up.jpg) | 1200×660 | 63,018 |
| [phone-390x844-menu.jpg](media/arcana-ui/phone-390x844-menu.jpg) | 390×844 | 46,983 |

Client runtime additions (uncompressed):

- `arcana.json`: 5,814 B.
- `editor.css`: 8,455 B.
- `editor.js`: 9,387 B.
- `preset.js`: 600 B.

Art sizes and both vector/served-raster quality tables are recorded above.
No master PNG, reference file, build output, temporary database or local
comparison is committed.

### Deviations, resolved findings and remaining review

- D-33/P16-C were absent from base main; the requested spec excerpts were
  consulted through the other branch's Git objects, without touching its
  checkout. Only this worktree was changed.
- Tool outputs did not match the requested dimensions exactly; original
  files were retained and normalized to the required master sizes.
- The supplied full-size vector settings failed fidelity and size gates.
  Budgeted SVGs also fail the SSIM gates, so every visible asset uses the
  authorized WebP fallback. Lossless quality/effort-90 node WebPs improved
  small-icon fidelity; all twelve served assets exceed 0.97 display SSIM.
- The first local browser probe preceded server startup and returned
  `page.goto: net::ERR_CONNECTION_REFUSED at http://127.0.0.1:18747/`.
  The compiled server subsequently started and served the complete build.
- An initial interaction probe found `element is not enabled` on the main
  card marked `aria-disabled="true"` at rank 2. The card must still support
  decrement input; only increment controls are now disabled, while the
  main card remains focusable with explicit rank labels.
- A combined long browser command terminated with exit 143 before writing
  its final report. Checks were split into bounded invocations that save
  results independently; the final invocations all completed.
- The first close-up capture from `about:blank` returned
  `page.waitForFunction: Timeout 30000ms exceeded.` Loading the capture
  document from the same local origin resolved image loading.
- Compact-layout acceptance initially returned
  `AssertionError [ERR_ASSERTION]: All six nodes and eligibility text fit the desktop panel`.
  The saved-status line reduced available height. Compact header/footer
  padding was reduced; the final test shows the whole map and both
  eligibility lines fit with saved/partial status visible.
- The requested preview mind-map and `tbg.arcana.preview` storage are
  deliberately separate from P36's later applied-preset lifecycle and
  `tbg_arcanas_v1` key. No queued edit, server ack, real combat-stat preview,
  account persistence, balance result or online Arcana match is claimed.

Owner review remains open for appearance/fun, summon exclusion and the
later integration sequence. The four owner questions above are proposals,
not unrecorded canon changes. No current task step is blocked.

Implementation commits:

```text
7756d7d fix(client): fit saved Arcana drafts in compact desktop panel
017f9cc feat(client): add browser-local Arcana editor preview
4cc9032 feat(client): add original Arcana art and reproducible conversion
```

The final documentation commit contains this report, the browser harness and
the screenshots. No push or merge was performed.
