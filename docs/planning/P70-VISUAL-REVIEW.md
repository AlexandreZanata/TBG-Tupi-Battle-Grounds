# P70 visual execution review

**Status: all 46 champions implemented, compiled, rendered and individually
reviewed in selected samples; phase acceptance remains incomplete.**
Review covers five-form static samples, B2 motion samples, all Base
match-distance lineups and the actual Troops catalog. It is not exhaustive
continuous collision review or owner approval of final art.

Candidate source: `c4813691c560a1533e76867154cacd97798fa2ed`.
Execution branch: `phase/p70-champion-visual-polish`, isolated in
`.local/p70-champion-visual-polish`. Required base:
`55510c5363e3f1e555fa5133208a5faf20c5825b`; planning parent: `60416ae`.
Commit `3dc532b` preserves the newer owner validation policy. The owner's
original worktree, archive, active builds and other fronts were preserved.
No push, merge, deployment or packaging occurred.

## Review entry points

The [renderer review pack](p70-review/README.md) contains seven faction
before/after comparison sheets, individual observations and links for all
46 champions. The [manifest](P70-VISUAL-MANIFEST.json),
[dossiers](P70-DOSSIERS.json) and [evidence index](P70-VISUAL-EVIDENCE.json)
preserve the actual source/tooling revision of each capture. Earlier images
are never relabeled as the latest build. The fixed cohort is **46 champions /
230 forms**; auxiliaries do not inflate that count.

Decisions are detailed in the three pilot reports
([Trident](P70-TRIDENT.md), [Pyre Warden](P70-PYRE-WARDEN.md),
[Old Finn](P70-OLD-FINN.md)) and the continuation reports for
[Humans/Undead](P70-NEXT-HUMANS-UNDEAD.md),
[Machines/Plants](P70-NEXT-MACHINES-PLANTS.md) and
[Monsters/Celestials/Insects](P70-NEXT-MONSTERS-CELESTIALS-INSECTS.md).
Their original work-front validation sections are historical; this report
supersedes their pending-build and pending-render statuses.

## Changes confirmed during consolidation

Every champion has individual model/color and Idle, Walk, primary performance
and Victory authoring. Existing A/B equipment progression, shared IK and meshes
remain. Semantic palettes are documented as Proposed in ART-DIRECTION. No new
dependency, replacement animation framework or platform-specific generic
character was introduced.

- The [blue artifact investigation](P70-BLUE-ARTIFACT.md) reproduced an opaque
  hip-mounted tabard crossing independently animated chest geometry. Switching
  teams changed the patch from blue to red. Fitted chest heraldry and a split
  hem repair that cause while retaining team identification. Selected after
  images show the fitted panels; unrelated blue surfaces are separate cases.
- `e6b632f` repairs Longaxe and Bone Brute B2 two-hand reach and shaft/body
  clearance. `6311707` lowers Longaxe's attack tilt from 110 to 65 degrees,
  keeping sampled generated vertices above ground. Five Longaxe forms and
  Bone Brute B2 retain distinct grip anchors; maximum sampled wrist-to-haft
  axis error rounds to 0.000000.
- `6311707` gives Shatterback a connected custom head pivot, grounded life
  clips and forepaw IK. Its four detachable shell plates now appear in
  event-free gallery, preview and P63 presentation. The frozen before gallery
  omitted these plates, so its plate baseline is explicitly incomplete.
- `d4566f5` releases looping gallery Victory on Idle/Walk selection.
  `c6a7015` removes unused former common performances while preserving
  Beetling's helpers. `1d6e456` adds normal-rate motion capture and label-free
  lineups using the actual match camera.
- `c481369` repairs dark Troops models: hiding ArenaScene also hid its sun.
  A shadow-free key light now exists only while Troops is active. Default
  zoom and vertical framing fit long equipment. All 46 Base previews plus
  seven B2 previews were captured and inspected after this correction.

## Actual evidence coverage

Before content is frozen at the required base, tooling
`27f98acd98b51fa69eb09f4703bd2ca59a33ed41`, executable SHA-256
`1883b601100297d8cf890d9a1a90658fee55713d485e64ef7678aff07675fc52`.
There are **630 immutable before PNGs**: 460 champion sheets, 120 enemy/view
diagnostics for Archer/Bone Brute/Trident and 50 auxiliary sheets.

Selected after evidence includes:

- **460 primary sheets**, ten clip IDs per champion, five forms per sheet.
  Shatterback/Longaxe use repaired `1d6e456` originals. Unchanged kit captures
  preserve their `8319c12`/`d4566f5` revision.
- **12 original normal-rate motion videos**, all 230 champion/form combinations,
  ten clips and Idle recovery. Selected B2 sequences were inspected for all
  46. This is sampled frame review, not continuous playback acceptance of
  every form. Inapplicable clips remain diagnostic samples only.
- **12 label-free normal-match-camera lineups**, all 46 Base champions,
  inspected. Five-form lineup comparison is not complete.
- **53 actual Troops previews**, all 46 Base plus B2 Trident, Shatterback,
  Longaxe, Bone Brute, Siege Walker, Pyre Warden and Old Finn. All inspected
  after final lighting/framing. These are Idle previews, not every clip.
- **Two actual P63 Result scenes**, both seats, inspected. Each fixture records
  46 purchases, includes dead participants and displays 46 mixed-form figures.
  Assertions: `count=46; range=0..46; figures=46; cycles=0; entities=1749;
  game_metrics=0; reduced=false`. This is not a 230-form or lifecycle proof.
- Pilot live recordings use ordinary training, seed 70, authoritative event/HP
  journals, a normal match camera and unchanged combat timing. Subject events
  are counted separately from other combatants. Latest solo Trident/Pyre
  trials remove the optional friendly blocker, without changing enemy rules.

The matrix contains 2,300 clip/form slots per revision: 1,835 applicable and
465 explicitly inapplicable. All applicable static slots are recorded before
and after. The final metadata validator finds **1,343 logical artifact
references locally available**, including shared-video/P63 references; this
is not the number of unique physical files: there are 1,185 indexed originals,
including 28 videos (12 motion and 16 live trials). It reports 542 pending acceptance
items, no bookkeeping errors and `phase_complete=false`.

Captures use Linux/Vulkan, NVIDIA GeForce RTX 4060 Laptop GPU, driver 595.104.02.
Primary sheets: 1920 × 2160; actual Troops/live frames: 1920 × 1080. Motion
viewport, camera, quality and schedule are stored per recording. Videos request
60 fps; renderer throughput is roughly 26–30 updates/s in these fixtures.
Animator rate remains 1 and combat is not slowed to stage effects. Exact video
PTS is recorded; derivative clip labels are approximate, not event timestamps.

Original PNG/video decoding, dimensions and SHA-256 were checked at capture
time. Consolidation reuses those checks rather than redecoding all originals.
Compact WebP sheets are labeled crops/resizes with provenance; no generated or
painted image substitutes for evidence. Some inspection crops cut tall props
or include an adjacent row; use originals and corrected motion group `03-fixed`
for complete Longaxe equipment. Bulk originals/binaries are ignored local
files, not portable until explicitly transferred.

## Mandatory pilots

### Trident

The shark has a fitted jaw, gills, dorsal fin, slate/ivory regions, larger
legs/torso and a longer three-tine weapon. Both wrists follow one rigid
chest-relative spear frame. The stroke loads, extends at the unchanged
**0.30 s Strike** and withdraws; it is not an inherited overhead chop.

Generated-mesh measurements, in world units:

- Base: height 2.235000; weapon length 2.626560; scale 1.728000.
- A1/B1: height 2.413800; weapon length 2.836685; scale 1.866240.
- A2: height 2.987043; weapon length 3.046810; scale 2.004480.
- B2: height 2.592600; weapon length 3.046810; scale 2.004480.

Nominal construction length 1.52 versus 1.21 is +25.6%; this is source
arithmetic, not measured before standing height. Sampled geometry reports
maximum grip error 0.007858 and minimum shaft/body triangle clearance 0.080054.
These finite sampled bounds are not a continuous collision guarantee.

`8afb13f` changes actual authoritative range **1100 → 2000**, inherited by
all five forms, with GAME-DESIGN Proposed documentation and ruleset revision
**3**, fingerprint `e529a73974c816e3`. Calibration sampled
1100/1600/1800/2000/2200. At 2000, tip depth beyond target center is
0.1391 Base, 0.2330 A1/B1, 0.3684 A2/B2. This is a visual heuristic,
not balance evidence or a changed hitbox.

Both-seat/form boundary, Guard priority and Stronghold regressions pass.
Damage, cadence, HP, movement, cost, Bloody Rend radius and bleed are unchanged.
Existing special acquisition uses effective basic range, as the canonical
relationship already specifies; it consequently searches a wider interval.
That effect is explicitly documented and belongs in D-33. **2000 remains a
candidate pending D-33 and owner balance approval.**

Live Base/seat A and B2/seat B are linked, including the final range. The enemy
closes during windup and obscures weapon contact; these frames do not establish
clean contact for every form and spacing. The B2 close-range samples especially
need further contact/axis inspection before visual acceptance.

### Pyre Warden

The open nozzle feeds a thicker bound spectral jet. `a946237` improves volume;
`cffe46d` keeps green/gold tongues upright in world Y instead of flattening
them with jet rotation. Standard/Low use eight/four puffs, existing 0.20 s life,
unchanged ProfaneFlame trigger, exact ground segment, endpoint and friendly
fire. The endpoint regression passes.

Live Base/seat A/standard and B2/seat B/Low recordings contain actual flame
events and rendered pieces in view. Crowding still occludes the short jet in
some samples. Solo samples expose the nozzle-to-target effect more clearly.
Recognizable flame at ordinary combat distance remains a visual acceptance
item; event counts alone do not pass it.

### Old Finn

Oilskin, fishing equipment and the forked-tail fish distinguish the kit.
Fish growth remains 1.0/1.35/1.70 for Base/tier1/tier2 on both branches;
release 0.22 s, flight 0.08 s, impact 0.30 s and Echo behavior are preserved.
Focused fish regressions pass. A2/seat A and B2/seat B live sequences were
inspected: B2 shows the blue fish/tail in flight; A2 is partly hidden by the
friendly frontline. Full five-form growth and actual Echo-event readability
remain unaccepted.

## Validation actually completed

Level: **targeted consolidation and the owner-requested screenshot/motion
campaign**. No full suite, workspace Clippy/test, release, packaging, balance
gate or performance campaign ran. Every compiling command used the same
nonblocking wrapper, two jobs and existing cache:

```sh
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
  cargo <scoped command below>
```

Successful commands and real results:

- `build --offline --locked -p tbg-client --bin tbg-client`: final `c481369`,
  exit 0, 6.60 s. Frozen executable SHA-256:
  `205906542c2cd0057fe879165119d9fa9d8973419fb8472e0e67b543cc54d476`.
- `check --offline --locked -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`:
  final `c481369`, exit 0, 0.75 s. Both outputs are in
  `.local/p70-evidence/preview-light-build.log`.
- `test --offline --locked -p tbg-client --bin tbg-client <FILTER> -- --nocapture --test-threads=1`:
  initial `8319c12`, filters `p70_`, `fish`, `pyre_jet`,
  `kits_keep_the_simulation_event_times`, `art::kits::archer::tests`,
  `art::celebration::tests`, `custom_ring_follows_its_later_weapon_parent`:
  16 selected tests passed, zero failures. Results are in the execution
  transcript; no nonexistent consolidated log is claimed.
- Same command with `p70_pole_` after `6311707`: one passed, zero failed,
  155 filtered, 0.32 s; `refinement-check-fixed.log`.
- Same command with `p70_trident_calibrates_candidate_world_distances_at_contact`
  and `pyre_jet`: one passed each, zero failed; `pilot-calibration.log`.
  `pyre_jet` reran after upright flame changes: one passed, zero failed,
  156 filtered; `flame-troops-build-fixed.log`.
- `test --offline --locked -p tbg-core --features lab --test p70_trident_range -- --nocapture --test-threads=1`:
  two passed, zero failed.
- `test --offline --locked -p tbg-core --features lab --lib trident::tests -- --nocapture --test-threads=1`:
  three passed, zero failed.
- `test --offline --locked -p tbg-core --test contract_ruleset_version -- --nocapture --test-threads=1`:
  two passed, zero failed. These three outputs are in
  `.local/p70-evidence/trident-range-check.log` at `8afb13f`.
- Affected package formatting, Python AST/CLI inspection, changed Markdown
  local links and `git diff --check`: passed. Final index command:
  `python3 -B scripts/p70/validate_evidence.py --metadata-only --json`;
  `.local/p70-evidence/index-check-final.json` records the result above.

Known warnings: test-only unused `Layout.clear`, WASM capture-only
`Motion.units`/`PAGE_COUNT`. No warning-free or `-D warnings` claim is made.
Native and WASM compilation do not establish browser/Windows runtime behavior.

## Failures, recovery and deviations

> blocked (historical, subsequently resumed): two early build/consolidation
> attempts returned 75 before Cargo because the shared validation slot was
> busy. Compilation stopped then; independent art continued. The later owner
> continuation resumed consolidation successfully. No polling, bypass, cache
> deletion, new target or terminated owner build was used.

> blocked (capture batching): two long roster sessions ended 143 without a
> renderer error; later Troops batches also ended 143 after completed images.
> Host termination is suspected, not proven. Long batches stopped. Completed
> PNGs were decoded and checked against renderer logs; missing entries finished
> in bounded invocations. Originals were retained, never overwritten.

> blocked (resolved environment): two restricted Trident Xvfb attempts failed
> with `XNotSupported(XOpenDisplayFailed)`; an Xvfb probe reported `Cannot
> establish any listening sockets`. Those attempts stopped. Approved local
> display execution subsequently produced the linked originals.

Early pole diagnostics exposed Bone Brute chest intersection and then
coincident grip anchors. Geometry was corrected and assertions passed; they
were not removed. A private `Motion` visibility compile error and a
`Single<Transform>` debug-format compile error were fixed before successful
rechecks. Troops inspection exposed the hidden-light defect, leading to the
final source correction and affected recaptures.

Sequence deviation: remaining source candidates were developed while the
initial pilot runtime gate was blocked. That gate is not retroactively passed.
Reports preserve missing acceptance rather than fabricate before/live evidence.

## Deferred acceptance and exact scope

- Inspect remaining five-form continuous clips, transitions and front/profile/
  back clearance. Reuse existing recordings first. Before motion/live/Troops/
  P63 counterparts require unchanged frozen art plus reviewed capture-only
  tooling; after recordings cannot stand in for before.
- Conditional scenarios remain for other champion specials and parent-linked
  auxiliaries: Nester/Beetling, Smuggler/Crate, Cargo Crab passengers and
  Drillnest production/fusion. Generic training does not trigger each condition.
  Old Finn five-form fish/Echo and clear Trident/Pyre contact/effect readability
  also remain scoped above.
- Desktop browser: inspect final WASM Troops, pilot/lineup scenes and P63
  entry/exit. Windows: the same shared-kit previews and pilots. These runtime
  campaigns were not completed in this Linux round; mobile stays outside scope.
- D-33 baseline/candidate and held-out Trident evidence remain pending.
  The phase's lab-equivalence and `cargo run --release -p tbg-balance -- gate --pairs 1000`
  require a separately authorized heavy round under the shared lock.
- Owner look/fun acceptance of all 46 identities and pilot effects, and balance
  acceptance of range 2000, remain pending against the concrete review pack.

All source and compact review artifacts remain local and reviewable.
**P70 is not complete while applicable acceptance items remain open.**
