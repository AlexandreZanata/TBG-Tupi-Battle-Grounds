# P70 — Champion identity, bespoke animation and 3D polish

**Status:** all 46 champion art source candidates implemented locally; final
compilation/evidence, Trident range change and owner approval pending.
See [the execution review](../planning/P70-VISUAL-REVIEW.md).
**Owner requests:** 2026-10-31, including the follow-up requiring unique
characters, stronger individual colors, personalized animation and the
Trident weapon/body intersection fix. The subsequent owner instruction
also authorizes a taller Trident, a longer trident, spear-like thrusts and
an increase to its real basic attack range; see the narrow exception below.
**Planning branch:** `codex/p70-champion-visual-polish-plan`.
**Execution branch:** `phase/p70-champion-visual-polish`.
**Reference:** `codex/consolidated-linux-release` at
`55510c5363e3f1e555fa5133208a5faf20c5825b`: **46 champions**, never the older
25-champion roster. Freeze the actual execution revision before capture.
**Delivery:** local reviewable commits and evidence; no push, merge or deploy
under the current owner-supplied AGENTS.md. A later explicit publication
instruction may authorize a PR targeting `main`.

## 1. Player outcome

A player recognizes each champion by its body, equipment, color blocking
and characteristic movement at normal match distance, without reading its
name. All 46 champions receive individual visual review and polish. They
must not look like one generic body recolored or one shared animation
with a different weapon.

Remove the reported blue square/hole artifact wherever it occurs. Make
attacks communicate what the champion actually does: Pyre Warden visibly
emits fire; Old Finn throws an identifiable fish which grows with Evolution;
Trident is taller, holds a longer trident outside its shark body and attacks
with a frontal spear-like thrust instead of an axe-like chop. Increase its
real basic attack range through the separately validated balance change
described in S2. Preserve the
game's stylized medieval look rather than introducing photorealism.

Every champion has genuine before/after evidence, including its animations.
Screenshots prove appearance; temporal frame sequences or short recordings
prove motion, release, travel, impact and transitions.

## 2. Read first

- `AGENTS.md`; apply the owner's 2026-10-31 minimal-validation policy stated
  in section 10 below even if the selected base contains an older broad
  verification block. Preserve newer owner guidance during integration.
- This spec and [the executor prompt](../planning/P70-EXECUTION-PROMPT.md).
- [The frozen roster and proposed individual briefs](../planning/P70-VISUAL-MANIFEST.json).
- `docs/ART-DIRECTION.md`: style, lighting, palette, rig/motion, Evolution
  and character refinement sections. Its older phone and full-build
  checklists do not broaden this desktop phase's scope.
- `docs/art/UNIT-ART-KITS.md` for existing kit conventions.
- Client: `art/kits/mod.rs`, `art/figure.rs`, `art/rig.rs`, `art/anim.rs`,
  `art/clips.rs`, `art/palette.rs`, `art/kits/common/{body,moves}.rs`,
  `visuals/attacks.rs`, `vfx.rs`, `vfx/shots.rs`, `sync/actions.rs`.
- Capture: `gallery/settings.rs`, `gallery/figures.rs`, `gallery/range.rs`,
  `capture.rs`, `config.rs`, `scripts/native-portraits.py`.
- For the current champion only: its `art/kits/<kit>/` files and relevant
  existing cue/effect modules. Read core definitions/abilities read-only.
- For P63 integration only: existing celebration presentation and animation
  ownership. Preserve P62 Troops/training preview and P63 result behavior.

Client paths above are relative to `crates/tbg-client/src/`. Use local
Bevy 0.20 examples for changed APIs; do not copy older API assumptions.

## 3. Findings from source inspection

These are code findings, not claims that the game was visually inspected
or that an artifact's root cause has already been proven.

1. `FULL_POOL` has 46 champions; the shared form list is Base, A1, A2, B1,
   B2: **230 champion/form combinations**. The 5 auxiliary entities
   Beetling, Crate, Microdrill, Duodrill and Megadrill are separate checks,
   not additional champions.
2. Kits already own figure/pose/gait/attack data and can supply custom
   anatomy. Extend these existing boundaries; no replacement engine,
   external animation framework or generic content platform is needed.
3. `common/body.rs::tabard` places an ALLY-colored rectangular piece on a
   spring. Other models contain their own team patches, and the scene has
   other blue surfaces. This is an **investigation lead**, not a confirmed
   explanation of the reported square/hole. Check mesh closure, normals,
   culling, overlap, transforms and depth artifacts as well.
4. Old Finn already selects `ShotKind::Fish`. The mesh already has a body,
   tail, eye and fin, but `look(Fish)` uses a fixed body scale of 1.0 and
   the kit's shot selector ignores tier/branch. Improve recognizability
   and propagate visual form information without changing combat.
5. Pyre Warden already has a nozzle model and authoritative
   `Event::ProfaneFlame { unit, start, end, tick }`. The current cue uses
   start/end and renders a tapered green shape plus a danger strip. Add
   readable flame motion from the actual nozzle; retain the exact
   authoritative ground segment and the existing friendly-fire semantics.
6. Trident has custom anatomy, a two-handed `grip` and a three-tine weapon
   attached to `WeaponR`. Inspect model dimensions, grip offsets, parent
   transforms and sweeping poses together. Moving the weapon in one idle
   frame is not a complete collision fix.
7. The existing frame gallery uses `(tier, 0)` and shared `Clip::duration()`
   in `frames()`, so requesting branch B does not establish branch-B frame
   coverage and per-kit timing may be sampled incorrectly. Fix or narrowly
   extend review capture before trusting its matrix.
8. Raw figure previews do not prove live VFX or authoritative ability
   events. The range gallery helps with basic shots; special-event cases
   also need the real cue path in a controlled training/replay scenario.
9. Existing portrait tooling writes Shop portraits and uses its own pool
   selection. Do not run it blindly as the P70 evidence driver or overwrite
   shipping portraits during the baseline capture campaign.

## 4. Scope and invariants

- Polish all 46 individually: model proportions, face/head, materials,
  silhouette, equipment, joints, movement, attacks, abilities and effects.
- Personalize each champion's Idle, locomotion, primary action and Victory
  with authored poses/layers. Also review Spawn, Hit, Death and Evolve and
  give them a character-appropriate performance. Attack variants and
  special abilities follow the actual kit; do not invent unused attacks.
- Shared IK, easing, rig helpers, event timing and motion primitives remain
  shared. A common helper is acceptable; visibly identical performances
  for unrelated characters are not an acceptable final result.
- Reuse the existing roster, classes, factions, lore and abilities. Do not
  invent new champions, gameplay roles, mechanics, power, damage, range,
  hitboxes, target eligibility, attack cadence or extra simulation hits.
  **Only exception:** the owner explicitly requests increased Trident
  basic attack range. That narrow core/content change follows S2 and D-33;
  it is not permission to rebalance other champions or Trident's other stats.
- Preserve authority and protocol. Never make flame particles deal damage,
  fish size enlarge an area, or an animation decide an online outcome.
- Reuse shared meshes/materials and procedural kits. No per-frame mesh or
  material allocation, no per-unit custom shader pipeline, and no new
  dependency unless a concrete missing need is recorded under AGENTS.md.
- Use shared rendering assets on Linux, Windows and web. Do not introduce
  OS-specific artistic identities or degrade another platform to a generic
  figure. This phase validates Linux native and desktop web; Windows
  runtime confirmation belongs to the coordinated P68 front if unavailable.
- No Android/iOS layouts, packaging, auth, backend, store, balance tuning,
  new result-scene behavior or rewrite of the P63 celebration flow.

## 5. Individual identity contract

For **each** of the 46 manifest entries, complete a short dossier before
editing that champion. The manifest's artistic directions are **Proposed**
starting briefs, not new lore, locked colors or owner approval.

Record: existing identity; unique silhouette anchor; recognizable face/head;
large equipment shape; main/support/accent colors and their purpose;
signature Idle, locomotion, attack/ability and Victory beats; differences
between Base/A1/A2/B1/B2; potential clipping and closest visual look-alikes.

### Color and recognizability

- Give each champion deliberate, stronger individual color blocking. Use
  large readable regions, not tiny accents that vanish at match distance.
  Distinguish neighbors from the same faction as well as other factions.
- Keep faction materials as a foundation while using individual cloth,
  skin/chitin, armor, weapon and accent colors to express the character.
  Existing palette values are not a reason to leave all bodies generic.
- If a needed color is missing, add a semantic named color to the central
  palette and document its character/faction role in ART-DIRECTION in the
  same commit. The owner's explicit request authorizes this visual palette
  expansion; it does not authorize gameplay changes or arbitrary restyling
  of the arena and lighting.
- Avoid global saturation/post-processing changes, uniform team-colored
  torsos, emissive glow on every surface or random rainbow decoration.
  Keep team markers and character colors distinct and readable.
- Inspect a nameless lineup at match camera scale, a grayscale/silhouette
  lineup and small portraits. Identity must not depend solely on hue.
  Ask the owner for final subjective recognition/beauty approval after
  producing the complete reviewable evidence, not before routine edits.

### Animation and anatomy

- Each dossier names the specific anticipation, key pose, contact/release,
  recoil/follow-through and recovery that makes this champion recognizable.
  Merely changing speed or scale of the shared class clip is insufficient.
- Respect anatomy: quadrupeds stay quadrupedal; segmented insects use their
  limbs; machines hinge and piston; floating bodies hover intentionally;
  the shark's jaw, fins, weight and two-handed thrust read as Trident.
- Adjust sockets, weapon rests, hand targets and secondary attachments
  together. Hands hold props; blades/shafts avoid the torso, face, fins and
  major armor surfaces throughout the movement and transitions.
- Honor current per-kit durations and the established event contracts:
  melee contact 0.30 s, ranged release 0.18 s plus 0.12 s flight, cast/Finn
  release 0.22 s plus 0.08 s flight, where those existing paths apply.
  Special abilities follow their actual authoritative events, not a new
  generic timing. Do not extend flight or delay the hit for visual drama.
- Keep feet planted and prop length stable; avoid skating, teleporting,
  detached weapons, instant pose snaps and interpenetration during blends.
- P63 must play each champion's personalized Victory through its existing
  ownership. Reuse existing hooks and finish the per-champion layer; do
  not replace the whole roster's dance with another common routine.

## 6. Execution sequence

### S0 — Freeze, prepare the capture tool and save the baseline

1. Start from the 46-champion reference or a reviewed descendant containing
   it. Do not downgrade to main's older roster. Record base SHA, working
   diff, GPU/driver, renderer, platform, viewport, DPI, quality tier,
   lighting, arena and camera preset. Keep other worktrees/services intact.
2. Validate all manifest IDs against `FULL_POOL`, all kit mappings and the
   230 form combinations. Snapshot the five auxiliary entities separately.
   If another front adds a champion, report the delta; keep the requested
   46 as the phase's fixed comparison cohort.
3. Extend existing capture/gallery modules only where missing. Suggested
   responsibility boundaries: `capture/polish.rs` for review scenes,
   `scripts/p70/capture_roster.py` for sequential orchestration, and
   `scripts/p70/validate_evidence.py` for coverage/file checks. These are
   proposed files, not claims that they exist today. No new capture engine.
4. Capture tooling must select form **and branch**, use the kit's actual
   duration, control pose time, camera, quality and aim, and wait for assets
   and scene readiness. Save metadata and logs. Keep pose sampling
   independent of combat clocks; side-effect-free previews must not spawn
   combat effects. Use the real release/cue path for effect sequences.
5. Use a tooling-only commit with identical model/pose/effect content to
   the baseline. Record both content and tooling SHA so its captures can
   be reproduced. Fixing gallery sampling is not character polish.
6. Capture **all 46 before shared visual edits**. Baselines are immutable.
   If any baseline is missing, reconstruct from the frozen content commit
   using the same capture tooling, never label an already-polished image
   as before. Do not stash/reset an active worktree to run old code.

### S1 — Diagnose and remove the blue square/hole

- Reproduce at least a humanoid, an undead and an unusual body on both
  teams, front/side/back and animated. Record affected/unaffected entries.
- Isolate the responsible part/material/transform with a local debug scene
  or temporary visibility toggles. Record the root cause and affected
  helper/models before selecting a shared or per-kit fix.
- Restore a visually solid, coherent body surface or properly fitted
  clothing. Do not hide the artifact with camera framing, change the sky,
  disable culling globally, darken the entire character or remove all team
  identification. Preserve intentional skeletal/anatomical openings.
- After a shared fix, inspect all 46 for regressions through the coverage
  matrix; do not assume that every blue patch is the same defect.

### S2 — Three mandatory pilots

**Trident: taller shark, longer spear-like trident and real increased reach.**
This is a mandatory owner requirement, not an optional artistic variant:

- Increase the champion's visible standing height relative to its current
  base model, and lengthen its trident. Tune shoulders, arms, legs, head,
  dorsal fin, stance and camera fit coherently; do not simply stretch the
  body along one axis or make only the weapon taller. Keep the existing
  Evolution progression relative to the new base and preserve a distinctly
  recognizable shark silhouette in all five forms.
- Replace the axe-like basic motion with a **two-handed frontal thrust**:
  retract/load the shaft alongside the body, brace and shift weight,
  extend the points toward the actual target, contact on the existing
  Strike time, then withdraw and recover. The three points lead the action
  along the thrust axis. No overhead chopping, axe swing or sideways
  poleaxe arc as its basic attack. Keep any special action visually rooted
  in piercing/braced thrusts while preserving existing Bloody Rend semantics.
- Increase **actual authoritative basic attack range**, not just model
  length, client lunge or the impression of reach. The current base value
  is **1,100 simulation units = 1.1 tiles**, inherited by the five current
  forms. The new base must exceed that value. Measure the new weapon-tip
  world reach and compare candidate distances; select the smallest increase
  that gives the intended longer spear reach without remote-looking hits.
  Record the proposed integer value and before/after reach images. The
  final number is not fixed or claimed balanced by this planning document.
- Keep authority in core content. Never derive competitive range directly
  from client transforms. Update Trident's canonical Proposed value in
  GAME-DESIGN in the same implementation commit as the content change;
  follow existing ruleset/content version identity. Document inheritance
  for A1/A2/B1/B2 explicitly. Do not also increase damage, attack rate,
  Energy cost, HP, movement, Bloody Rend radius, bleed or other champions.
- Add focused range-boundary regression scenarios for both seats: the old
  edge, an enemy within the newly gained interval, exactly the new edge,
  and just outside it. Check inherited form range, target/Stronghold rules
  and visual contact; preserve the independent special's center/radius.
- D-33 evidence and the owner's final look/fun approval remain required to
  accept the exact balance value. Queue the existing focused lab scenarios
  against the frozen baseline and candidate, including reach-sensitive
  melee/frontline interactions and different compositions. Heavy lab/gate
  runs still require explicit authorization under section 10. If that
  evidence is unavailable, deliver the isolated range proposal/change as
  pending balance acceptance, continue all independent art work, and do
  not call the new value validated or merge/deploy it as an approved patch.

Fix `model.rs`, anatomy, grip, socket and impact tip as needed. The longer
shaft and tines must remain outside the
torso/head/fin volumes and in both hands in Idle, Walk, every basic/special attack,
Hit, Evolve, Victory, Death and transitions, across all five forms. Include
front, side and three-quarter samples, slow motion and real-time playback.
Use appropriate sockets/hand targets rather than shrinking the trident
until unreadable. Strengthen the shark silhouette, jaw/gill/fin read and
character-specific color blocking. Show before/after standing-height and
weapon-length measurements, the frontal thrust sequence, and gameplay
reach separately; visual size does not change a server hitbox. No physics engine is needed for a
fixed hand-authored kit clearance fix.

**Pyre Warden: visible emitted fire.** Render a recognizable directional
flame jet from the current nozzle, with a hot core, changing flame shape,
controlled embers/smoke and appropriate braced recoil. Reuse the existing
spectral/undead identity unless the individual color pass warrants a
documented variant. Use the event's `unit` to locate the visual socket when
available; retain a bounded world-space fallback after despawn. The ground
danger segment must still end exactly at authoritative `end`, mirrored
correctly for both seats. Allies within that segment visibly receive the
same existing damage/status cues as enemies. Do not make an attractive
cone imply a wider damaging area, add lingering damage or emit a second
flame from the generic basic-shot path. Verify Low as well as standard
quality; keep the essential jet and danger segment on Low.

**Old Finn: a readable, evolving fish.** Preserve his recognizable oilskin,
beard, rod and casting personality. Refine the projectile's fish silhouette
(head/body/tail/fins, contrasting belly/back and readable orientation) so
it is visibly a fish at gameplay size. Give Base, tier 1 and tier 2
progressively larger fish on both branches; exact cosmetic scales are
tuned and recorded after viewing, with Base < tier 1 < tier 2. Preserve
branch-A/branch-B visual identity and existing Echo behavior/counts.
Propagate only the needed presentation tier/scale through the existing
shot pipeline, with default behavior unchanged for other shots. Keep
release/flight/impact times, target and trajectory clearance contracts.
Add a readable carried/casting pose or momentary pre-release fish display
if necessary; do not slow the projectile to make a screenshot easier.

Complete the pilots' evidence and shared approach before using it on the
remaining roster. Continue ordinary work without waiting for subjective
sign-off; list owner approval as pending until actually received.

### S3 — Review and refine the remaining 43, one at a time

Follow the manifest in faction order, skipping the already-completed
pilots. For each champion: dossier -> model/color pass -> bespoke motion
pass -> weapon/effect pass -> all-form inspection -> paired evidence ->
record findings, files, validation and limitations -> logical local commit.
Do not mark a champion done because only a shared helper changed.

For similar pairs, compare them side by side: Archer/Farwatch/Celestial
Archer; Shieldbearer/Rampart/Bulwark; Blood Furnace/Pyre Warden;
Bloomkeeper/Antidote Brewer; Nester/Smuggler/Drillnest. Preserve their actual
abilities, weapon types, body plans and faction membership. Review summoned
entities with their parent champion so polished summoners do not emit
unreadable placeholders. The five auxiliaries remain outside the 46 count.

### S4 — Consolidated presentation and evidence

- Run the complete after-capture campaign once on the integrated final
  revision, using the baseline capture settings. Re-capture only affected
  entries after subsequent relevant changes.
- Verify normal match camera, target/seat mirroring, Troops model preview,
  training and P63 Victory. A catalog image alone does not verify combat.
- Reuse existing performance budgets from ART-DIRECTION/ARCHITECTURE; record
  comparable before/after frame-time, entity/particle lifetime, allocation
  and draw-call observations for the affected desktop scenes. Do not
  invent a new performance limit or claim an unrun stress test passed.
- Run focused validation under section 10. Finish the report with explicit
  missing evidence, owner decisions and platform checks still deferred.

## 7. Required evidence

Use `docs/media/p70/` for compact review indexes/contact sheets only.
Place full-resolution originals and recordings in ignored local storage
or the repository's established external review-artifact mechanism. Never
commit thousands of raw PNGs or large video binaries. Keep artifact URIs,
hashes, dimensions and capture metadata in the index; report if a reviewer
cannot access a remote artifact. Do not invent an upload destination.

The executor produces `docs/planning/P70-VISUAL-REVIEW.md` and an evidence
JSON index, linking **before | after for each of the 46 champions**:

1. Five-form comparison: Base/A1/A2/B1/B2 with consistent framing, both
   isolated and at normal match distance. This accounts for all 230 forms.
2. Front, profile and back silhouette/weapon inspection, plus a nameless
   roster lineup showing individual color and shape identity.
3. For every applicable clip and form, contact-sheet samples at 0%, 25%,
   50%, 75% and the actual release/strike; include the final pose for
   one-shot clips. De-duplicate coincident samples and compose frames into
   readable sheets instead of starting a game process for every sample.
4. A short temporal sequence/reel per champion covering applicable clips,
   transitions, primary attack, abilities and Victory at real time. Add
   slow-motion inspection for fast actions and clipping, clearly labeled.
5. Effects sequences show anticipation -> emission -> travel -> impact ->
   cleanup, with a target present. Distinguish raw pose reviews from real
   event-driven action evidence. Specials not represented by the ten
   generic clips get separate labeled coverage.
6. All 10 current clip IDs (`Idle`, `Walk`, `AttackMelee`, `AttackRanged`,
   `Cast`, `Hit`, `Death`, `Spawn`, `Evolve`, `Victory`) must be accounted
   for against every form. An inapplicable clip needs a reason tied to the
   actual kit; copying another champion's frames is not coverage.
7. Baseline/final source SHA, tooling SHA, seed/scenario, form/branch,
   clip/sample time, camera/aim, platform, quality, renderer and paths or
   URIs. Avoid player secrets and personal account data.

Current upper bound is **2,300 clip/form review slots per revision**, before
explaining inapplicable clips. This is coverage bookkeeping, not a demand
for 2,300 processes or separate screenshot files. Build reusable sheets
and capture batches sequentially, with resume and timeout support. First
capture baseline once, then after once; do not repeatedly run full
galleries after each small edit.

Before and after must use the same pose, camera, exposure, viewport, arena
and quality unless a documented capture-tool defect requires a correction
applied to both. Preserve originals. Crops, annotations and side-by-side
compositions are allowed as labeled review derivatives. Generated art,
paintovers or edited character pixels are never implementation evidence.

## 8. Proposed module changes

- Per-champion `art/kits/<kit>/{model,mod,motion}.rs` where needed, keeping
  model construction separate from significant animation logic.
- Shared `art/kits/common/`, `figure.rs`, `rig.rs`, `palette.rs` only for
  diagnosed shared issues; record each regression surface.
- `visuals/attacks.rs`, `vfx/shots.rs`, `vfx.rs`, `sync/actions.rs` for
  minimal cosmetic form propagation and accurate event-origin effects.
  Split a new coherent responsibility into its own module instead of
  adding all 46 characters' exceptions to a central match statement.
- Existing gallery/capture modules and small evidence scripts, isolated
  from production combat state and account/network paths.
- `docs/ART-DIRECTION.md` for individual color policy/approved evolution
  identity refinements; no unrequested change to gameplay canon.
- The **Trident-only** range exception can change its core content row,
  applicable focused regression scenarios and GAME-DESIGN Proposed value.
  Read `trident.rs` and `ruleset_version.rs` for existing contracts. No
  other core balance, server rule or protocol change is planned.

## 9. Acceptance and completion

- [ ] Manifest covers the fixed 46 champions and all 230 forms; auxiliaries
      are reviewed separately and never counted as champions.
- [ ] Every champion has an individual identity dossier and actual model,
      color and motion inspection, with paired before/after links.
- [ ] Personalized Idle, locomotion, primary action and Victory are visible;
      remaining applicable clips and abilities match individual anatomy.
      Common class motion or a color swap alone does not satisfy this.
- [ ] Names hidden, characters remain distinguishable by silhouette,
      equipment, color blocking and movement at match distance. Owner
      recognition/beauty approval is recorded as received or pending.
- [ ] Reported blue square/hole is reproduced, diagnosed and removed where
      present; intended openings and team readability are preserved.
- [ ] Trident has no visible major body/weapon penetration in all forms,
      actions and sampled transitions, including its special presentation.
- [ ] Trident is visibly taller, its trident is longer, and its primary
      attack is a frontal two-handed spear thrust, not an axe-like chop.
      Recorded measurements and sequences compare the same before/after pose.
- [ ] Trident's proposed real basic range exceeds the old 1.1 tiles;
      authority, boundary behavior and form inheritance are covered. The
      exact value has D-33 evidence/owner acceptance or is explicitly
      pending balance validation, never silently considered release-ready.
- [ ] Pyre Warden visibly emits fire; exact directional segment and
      friendly-fire cues remain correct for both seats and Low quality.
- [ ] Old Finn's projectile is recognizable as a fish and grows across
      tiers on both branches, while timing and existing Echo counts hold.
- [ ] Each evidence matrix cell has a real artifact or a documented
      inapplicability; absent/unrun evidence remains pending, never passed.
- [ ] No combat, cadence, targeting, hitbox, damage, class, lore, protocol,
      account or server behavior changes except the explicitly requested
      Trident basic range increase; no leaked capture/debug UI.
- [ ] Relevant checks and desktop scenarios are reported separately from
      deferred work. Cross-platform claims name platforms actually run.

The executor may deliver reviewed batches as work fronts, but P70 is not
complete until all 46 entries and applicable acceptance evidence are done.
An owner-pending aesthetic decision is not an approved visual result.
If Trident's balance evidence remains deferred, the art work front can be
handed over with its limits, but P70's balance acceptance stays open.

## 10. Validation policy and concrete handoff

**This planning delivery:** JSON/roster/source-path/link inspection and
`git diff --check` only. No game build, screenshot campaign or test run.

**Implementation fronts:** affected-package formatting and, only with an
inexpensive existing cache, one focused client check. Write meaningful
regression tests for changed form propagation/timing/capture correctness;
do not run crate/workspace test suites at every character commit.

```bash
cargo fmt -p tbg-client -- --check
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo check -p tbg-client --bin tbg-client
```

Reuse one development binary for the requested baseline campaign and one
for final consolidation when needed. Every compiling build/check/test uses
that same nonblocking host lock and at most two Cargo jobs. A busy lock,
missing tool or cold cache is recorded as deferred; do not poll, bypass,
clean caches, start duplicate builds or disturb active agents.

**Targeted consolidation:** the phase's requested before/after capture
campaign and animation inspection are the named visual scope; they are
not permission for release packaging, balance gates or a full suite.
Run the deduplicated affected tests once. Current narrow existing examples:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client \
  kits_keep_the_simulation_event_times -- --test-threads=1
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client \
  fish_lands_at_hit_and_clears_allies -- --test-threads=1
```

Queue new named regressions for branch-B capture, per-kit duration,
fish-scale progression/default shot isolation and flame origin/segment
mirroring if those paths change. For the Trident range exception, queue
the new named core range-boundary scenarios and record the required D-33
baseline/candidate evidence separately. Do not silently invoke a broad
balance gate under the animation test authorization. Assert meaningful contracts, not trivial
implementation details. Tests do not replace visual inspection.

Desktop web needs its own focused WASM/browser observation; native success
is not browser proof. Schedule an existing cached development WASM build
under the same lock at consolidation. If unavailable, state it is pending
and do not claim Linux/web parity. Windows runtime observation may be
coordinated with P68 without taking over its files. No full-suite or heavy
performance campaign without an explicit owner request.

## 11. Execution report template

Fill `docs/planning/P70-VISUAL-REVIEW.md` with:

- Base/content/tooling/final SHAs; scope and current 46/230/clip coverage.
- Blue artifact: reproducer, confirmed root cause, files and regression scope.
- Per champion: identity/color/motion decisions, affected forms, changes,
  paired evidence, known clipping/limitations, actual validation and status.
- Mandatory pilots: Trident height, longer weapon, clearance, frontal
  spear thrust and proposed authoritative range/balance evidence;
  fire direction/origin/friendly cues;
  fish recognition, tier sizes, release/impact and existing Echo behavior.
- Integrated lineup and Troops/training/Victory observations; platform and
  quality coverage; actual measured performance, if available.
- Commands actually run with outcomes; named deferred checks and reasons.
- Owner aesthetic approvals received/pending; deviations and blocked work.
- Local commits and review handoff. Stop for review; do not push, deploy or
  claim approval automatically.

## Planning report (historical planning delivery)

Prepared from source inspection only. The square/hole and Trident clipping
are owner-reported defects pending runtime reproduction. Individual briefs
are proposals. No before/after image, polished animation, performance
result, game validation or owner aesthetic approval is claimed by this plan.

Planning validation completed: JSON parsing; an independent Python audit
against `FULL_POOL` (46 unique IDs, 230 forms, 2,300 clip/form slots), all
individual briefs and three mandatory pilots; 121 source paths and SHA-256
hashes; the two new relative document links; whitespace inspection.
`git diff --check` passed. No Cargo, browser, capture, balance lab or VPS
command was run. The exact new Trident range remains Proposed/unselected.
