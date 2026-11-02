# P70 mandatory pilots: source audit

**Scope:** read-only source inspection on execution branch
`phase/p70-champion-visual-polish`, initially at `60416ae`, descending from
the fixed 46-champion base `55510c5363e3f1e555fa5133208a5faf20c5825b`.
No model, animation, VFX, gameplay value or contract was changed for this
audit. No rendering, screenshot capture, compilation or test was run.
All runtime appearance, clipping, measurements and owner approval remain
pending. In particular, no new Trident range is selected here.

## Shared constraints and useful seams

- `art/anim.rs::Animator::pose` samples the shared clip before the kit's
  pose layer. A bespoke kit must replace every relevant shared transform,
  rather than relying on a partial adjustment to remove a generic attack.
- `art/clips.rs::Clip::events` fixes melee Strike at 0.30 s, ranged Release
  at 0.18 s and Cast Release at 0.22 s. The shared kit recovery length in
  `art/kits/common/moves.rs` is 0.62 s. Pyre Warden overrides basic action
  lengths to 0.48 s; its canonical cadence is 0.50 s.
- `visuals/cues.rs::play_cue` selects AttackMelee for Tank/Dps,
  AttackRanged for Ranged and Cast for Control/Support. Thus the normal
  pilot clips are Trident AttackMelee, Pyre Warden AttackMelee and Old Finn
  Cast. The other two generic attack IDs need an explicit inapplicability
  record, not invented gameplay coverage.
- `art/celebration.rs::celebration_for` already selects the kit's Victory.
  `signature_for` currently returns `None`; `finish` constrains root yaw
  and displacement. Author individual Victory behavior in the existing
  pilot kits and keep this result-scene ownership. A catalog preview alone
  does not demonstrate the result-scene bounds or transitions.
- Figures face +Z, their left side is +X, and `WeaponR` is a zero-offset
  child of HandR. `tools::reach` solves chest-space arm IK; the second hand
  must target a point on the same weapon transform. A visually requested
  hand target beyond the two-bone reach does not establish a valid grip.

## Trident

### Existing identity and source findings

The brief calls for a taller shark, readable gills/jaw/dorsal fin, a lighter
belly against a cool back, and a warm rust/bone three-tine weapon. Existing
forms already distinguish bone armor/fin growth on A and a red belt/wider
weapon assembly on B. Preserve these identities while strengthening their
large surfaces and silhouettes.

Relevant implementation:
`crates/tbg-client/src/art/kits/trident/{mod,model}.rs`.

- `plan` uses the Large plan, size 0.96, arm length 0.24 and torso width
  multiplied by 1.05. `body::plan` also applies the lane scale and existing
  +8%/+16% tier progression. This is source geometry, not a measured
  standing height.
- The shaft is a cylinder of length 0.95 centered at local z=0.18, so its
  endpoints are approximately -0.295 and 0.655. The tine tips reach
  approximately z=0.915. The registered impact point is z=0.88. These are
  weapon-local extents; actual forward world reach remains unmeasured.
- `grip` requests right/left hand targets with different x coordinates
  (-0.14 and -0.04), despite the shaft running along weapon-local +Z.
  The requests are not collinear. Some left-hand requests exceed the
  existing two-segment reach (0.24 + 0.216), especially at extension.
- The rear shaft extends behind the right hand while the hand is close
  to the torso centerline. This is a concrete intersection risk requiring
  visual and transform inspection, not a confirmed runtime diagnosis.
- The primary action's chest rotates by -12 degrees and advances the
  requested grips only slightly; the independent special adds a -55
  degree chest sweep. `Death` skips the grip function altogether.
- The current grip recovery ends at 0.65 s while the kit action duration
  is 0.62 s. Inspect the recovery/blend boundary rather than assuming an
  endpoint screenshot proves continuity.

### Minimal implementation after immutable baselines

Keep a compact `model.rs` and move significant pose logic to the kit's
`motion.rs` if needed. Increase height through coherent leg/torso/shoulder
and arm proportions, retaining uniform Evolution growth. Use one weapon
geometry definition for length, both hand anchors and the impact tip.
Place the shaft's rear and front intentionally relative to the torso;
simply lengthening the current centered cylinder increases penetration risk.

Author load, brace, forward extension at 0.30 s, brief contact, withdrawal
and recovery. Keep the two hands collinear on the shaft, solve both within
reachable arm lengths, and plant the feet when the hips shift. Replace the
special's sideways chest sweep with a braced piercing presentation while
leaving its event, target center and damage radius unchanged. Define safe
weapon handling for Hit, Spawn, Evolve, Victory and Death as well as Idle
and Walk. Review previous/next-pose blends, both attack variants and aim
mirroring across Base/A1/A2/B1/B2.

Measure standing bounds and weapon-tip world reach from the same captured
poses before selecting the smallest useful integer range above 1,100.
Do not infer the competitive value from local shaft length alone.

### Authoritative range and contract scope

`crates/tbg-core/src/content.rs` currently defines Trident as
`[1250, 55, 40, 1100, 28, 0, 0]`; all four evolved forms inherit its range
without a range modifier. The Proposed P70 section in GAME-DESIGN leaves
the new integer unselected. Only the range field may change.

`targeting.rs::enemy_in_range` uses inclusive `distance <= range`, Guard
priority and the existing nearest-unit/Stronghold rules. Test the real
target-selection or combat-start path; `hits::resolve_hit` alone is not a
range-boundary test because range is decided when the windup starts.
For both seats and all five forms, cover the old edge, the newly gained
interval, the exact candidate edge, candidate+1, and Stronghold reach.
Keep carried-target and Guard eligibility unchanged.

`trident.rs::resolve` centers Bloody Rend on the primary target and reads
its independent radius/bleed values from abilities. Preserve the existing
R07 regressions and add a case establishing that the proposed basic-range
increase does not enlarge that special radius.

Changing `content.rs` requires incrementing `RULESET_VERSION` in
`ruleset_version.rs` and appending the actual watched-source hash to
`tests/fixtures/ruleset-versions.txt`; current revision is 2. The frozen
Trident row in `tests/contract_roster.rs::proposed_p59_bases_match_section_13`
also records 1,100. Follow the contract-change rule: document the changed
GAME-DESIGN Proposed value in the same logical commit and name that change
in the commit message. Do not silently weaken or remove the contract.

D-33 baseline/candidate evidence and final owner acceptance remain open.
No balance gate or heavy lab run is authorized by this source audit.

## Pyre Warden

### Existing identity and source findings

The distinguishing pieces are a coffin furnace, rigid hose and a braced
nozzle, with funerary iron/bone against localized spectral flame.
Relevant files are `art/kits/pyre_warden/{mod,model}.rs`,
`sync/actions.rs`, `vfx.rs` and core `pyre_warden.rs`.

- The kit uses `Attack::None`. Its authoritative basic attack is rendered
  through `Event::ProfaneFlame`, not the generic projectile release.
- `sync/actions.rs` currently ignores the event's `unit` field and sends
  only mirrored `start` and `end` to `vfx::profane_flame`.
- The current effect starts at `(start, 0.55, 0)` and consists of one
  static green tapered shape plus the exact ground strip. The shared
  spike mesh is explicitly translated so its base is at the origin;
  it is not accidentally centered on `start`.
- Weapon-local nozzle exit is approximately `(0, 0.02, 0.46)` normally
  and `(0, 0.02, 0.60)` for A2. Centralize the exit alongside the nozzle
  construction before using it as the emitter point.
- The ground strip is centered at `(start+end)/2`, length `abs(end-start)`,
  and its existing animation fades thickness while preserving length.
- Core flame damage includes allies within the inclusive directional
  segment. Hit/status events already drive their feedback independently
  of the visible jet. Preserve them unchanged.

### Minimal implementation after immutable baselines

Keep the authoritative start/end and exact ground strip. Use the event's
unit to find its Animator weapon joint and sample its GlobalTransform
after transform propagation, as existing real-joint projectile releases
do. A small transient cue consumed in PostUpdate can retain the event
endpoints and emitter reference without using previous-frame transforms.
If the source has disappeared, use an explicitly bounded world-space
fallback; never drop the ground segment with the source.

Render the jet with a narrow hot core, a few offset/changing spectral
flame lobes and restrained embers/smoke using existing cached meshes and
materials. Keep essential jet and strip independent of optional quality
effects. Do not add a generic shot or extend the represented damage area.
Use the existing 0.30 s authoritative contact and 0.48 s pose recovery;
retain both-seat direction and friendly-fire feedback.

## Old Finn

### Existing identity and source findings

Preserve the oilskin/sou'wester, beard, pipe, rod and creel. Branch A adds
larger catch/creel and the giant catfish; B adds net/second rod and a full
net/lantern. Relevant files are `art/kits/old_finn/{mod,model}.rs` and
`vfx/shots.rs`.

- The kit selector is `shot: |_, _| ShotKind::Fish`, discarding form.
- `look(Fish)` uses one fixed body scale. The cached fish body has narrow
  dimensions approximately 0.098 by 0.126 by 0.252, one cone tail, one
  visible eye and a thin rectangular fin. No runtime readability claim
  follows from these dimensions.
- Finn's Cast releases at 0.22 s; `FISH_FLIGHT` is 0.08 s and impact lands
  at 0.30 s. `fish_arc` guarantees a minimum apex offset of 2.0 world units.
- Authoritative Echo is defined in core content; the basic shot path does
  not decide Echo count. Do not change core Echo while improving the fish.

### Minimal implementation after immutable baselines

Carry cosmetic tier through the existing selector, for example by changing
the presentation-only variant to `ShotKind::Fish { tier: u8 }` and selecting
`tier.min(2)` in Old Finn's kit. The look table can choose the tier scale;
the flight path should recognize all Fish variants while leaving every
other shot unchanged. This avoids a new protocol or generic attack layer.

Give the cached mesh a fuller contrasting belly/back, readable forked tail,
paired eyes and fins. Tune strictly increasing Base/tier1/tier2 scales on
both branches from match-distance evidence. Preserve the trajectory,
release, flight, impact and cleanup times. Inspect silhouette while the
existing roll follows the tangent, not only in an isolated mesh image.
If a carried/pre-release fish is needed, use the existing loaded-prop
hook; do not slow its flight to create visibility.

## Blue square/hole investigation

No runtime cause is confirmed by this audit.

- `common/body.rs::tabard` builds a literal ALLY-colored thin rectangle
  on `Secondary(Spring::Tabard)`, plus a gold hem and diamond. Its four
  current callers are Archer, Longaxe, Shieldbearer and Solar Sentinel.
  A shared tabard fix alone cannot establish a fix across all 46.
- Pyre Warden independently adds a small rectangular ALLY chest patch.
  Trident has a diagonal team-colored chest band and two hip strips.
  Old Finn has a rectangular team neckerchief. They do not call the
  shared tabard helper.
- The shared tabard is parented to Hips with a spring, while the torso
  follows Chest. Its rest surface sits near the torso front. Relative
  motion and overlap therefore need inspection rather than a blanket
  recolor or removal.
- `mesh::paint` duplicates indexed vertices and computes flat normals;
  `figure::build_layer` merges transformed parts per joint. Nothing read
  here establishes a universal missing-face or inverted-normal defect.

Use immutable before frames and a temporary diagnostic scene to isolate
the exact piece on humanoid, undead and unusual bodies, both teams and
front/side/back, stationary and animated. Toggle one candidate mesh at a
time; inspect transforms, closure, normals, culling and depth overlap.
Preserve intentionally skeletal openings and a readable team marker in
the eventual fix. Remove diagnostic toggles from production.

## Deferred targeted validation

These commands were **not run**. New test names below are proposed work,
not claims that those tests already exist. Execute the deduplicated union
once at consolidation, with the nonblocking shared lock and two build
jobs. A busy lock or cold cache remains deferred.

Existing relevant tests:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client kits_keep_the_simulation_event_times -- --test-threads=1
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client fish_lands_at_hit_and_clears_allies -- --test-threads=1
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 cargo test -p tbg-core --lib --features lab r07_ -- --test-threads=1
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 cargo test -p tbg-core --test contract_roster proposed_p59_bases_match_section_13 -- --test-threads=1
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 cargo test -p tbg-core --test contract_ruleset_version -- --test-threads=1
```

Suggested new regressions: `r07_basic_range_boundaries_both_seats_and_forms`
and `r07_basic_range_keeps_stronghold_and_special_rules` in the core R07
test module; `fish_form_scale_preserves_flight_and_other_shots`,
`profane_flame_preserves_mirrored_segment_and_nozzle_origin` and
`profane_flame_source_loss_keeps_bounded_fallback` in affected client
modules. A `p70_` prefix can group newly added client regressions into one
filtered consolidation invocation.

Runtime evidence still required: all five forms, actual event/release
sequences, both seats, Low/default quality for flame, complete applicable
clip/transitions, Troops/training and P63 Victory. Native observation does
not establish desktop web or Windows compatibility. D-33 heavy lab
commands require separate explicit owner authorization.
