# P70 Trident pilot

**Status:** art implementation ready for targeted consolidation; no after-render,
compiled result, measured range candidate or owner visual approval yet. The
immutable 46-champion static baseline gate completed before these edits.

## Evidence and decisions

Inspected authentic original ally three-quarter sheets
`.local/p70-evidence/before-smoke-display/Trident/ally-threequarter/00.png`,
`02.png` and `09.png`. They show the previous gray humanoid body, a separated
upper-head/jaw silhouette, a foreshortened short weapon and little distinction
in Victory. Source findings and competitive boundaries are recorded in
[P70-PILOT-SOURCE-AUDIT.md](P70-PILOT-SOURCE-AUDIT.md).

The new five-form model retains the existing Evolution growth, bone armor on
A and wider trident/red cloth on B. Its larger legs and torso support the shark
head; the fitted jaw, dark mouth seam, paired gills, dorsal fin and contrasting
belly establish the animal silhouette. Shared semantic `SHARK_SLATE` and
`SHARK_BELLY` colors replace the gray skin. Narrow belt tails and a fitted arm
band carry the team color. B2 gains small backward barbs rather than large
rings around the weapon head.

Source dimensions, **not executed pose measurements**:

- The leg parameter changes from 0.38 to 0.46, torso from approximately
  0.483/0.34/0.24 to 0.48/0.42/0.26 and upper arm from 0.24 to 0.35.
  The standard lower arm is 0.315, making the two-segment reach 0.665.
- The weapon runs from local z=-0.40 to the actual central tine tip z=1.12:
  1.52 figure units versus the original nominal 1.21, about 25.6% longer.
  At Base scale 1.728, these construction lengths are 2.62656 and 2.09088
  world units. Actual generated-mesh bounds remain queued below.
- The right-hand anchor is the weapon origin and the left-hand anchor is
  z=0.18. The shaft slides along its own axis from -0.06 to +0.14.
  A chest-space guard at (-0.27, 0.32, 0.31), yaw 50 degrees and bounded
  pitch [-12, 28] degrees keeps the support hand within available reach.
  At rest, the rear shaft projects outboard of the torso rather than
  behind its center. These geometric observations are not a rendered
  clipping pass or continuous-collision proof.

The weapon has one chest-parented rigid frame, with both wrists solved onto
its anchors. This prevents independent wrist motions from changing shaft
placement and retains the grip in Death, Hit, Spawn, Evolve and Victory.
The full kit pose replaces inherited shared attack transforms. Idle breathes
through the jaw/fin; Walk carries a braced weapon with an opposing head/fin
motion; Victory gives a restrained two-handed salute. P63 continues to own
Result placement, bounds and transitions through the existing kit seam.

The primary attack loads until 0.18 s, extends to the existing **0.30 s Strike**,
holds briefly and recovers by 0.62 s. Bloody Rend uses the same piercing line
on its independent presentation channel. Shared event times and gameplay
special radius are unchanged. AttackRanged and Cast receive safe gallery
poses; they are not claims of live Trident attack modes.

## Measurement and validation handoff

Owned files are `art/kits/trident/{mod,model,motion,weapon,tests}.rs` in
`tbg-client`. No shared body, palette, core, ruleset or contract file was
edited by this front.

Completed work-front commands (exit 0):

```text
rustfmt --edition 2021 crates/tbg-client/src/art/kits/trident/{mod,model,motion,weapon,tests}.rs
git diff --check -- crates/tbg-client/src/art/kits/trident docs/planning/P70-TRIDENT.md
```

The following focused command is **queued, not run**. The parent reported
that the shared validation lock was busy (exit 75) for its attempted pilot
build; this front performed no compilation or tests and did not poll it.

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client \
  p70_trident_ -- --nocapture --test-threads=1
```

The queued regressions use the existing `pose_world` hierarchy and generated
meshes. They report maximum sampled wrist-to-anchor error and minimum distance
between sampled spear lines and actual hips/chest/head/jaw/fin triangle
surfaces. They reject intersections across five forms, ten clips, actual
clip events, special phases and intermediate Euler blends into Idle/Hit/Death.
They also print actual posed standing bounds, generated weapon length and
load/Strike/recovered tip positions toward both local Z directions, while
checking frontal line alignment and the 0.30 s event. The clearance samples
include the shaft center and four surface lines plus the three tine axes;
they do not prove every point of the weapon volume clear for every continuous
frame. Hands intentionally touch the weapon; arms are not treated as torso
collision surfaces by this diagnostic.

## Remaining acceptance

Run the queued diagnostics and inspect after evidence for all forms, both
seats, frontal/profile/three-quarter views and real target contact. Review
life-clip transitions, aim pitch extremes, Result placement and the relevant
Low/reduced-motion browser paths. A native diagnostic is not browser proof.

**The authoritative range remains 1100. No candidate is selected.** Synthetic
aim points and weapon-local length cannot establish a combat range. Calibration
must combine the reported tip with an actual opposing body and the live
cosmetic lunge/spacing; current Trident inherits the shared Slash lunge, 0.20
world units at Base. Select the smallest useful integer only from that evidence.
The later authorized core change must preserve every unrelated stat and
Bloody Rend radius, update the Proposed GAME-DESIGN entry and ruleset version,
and add both-seat/five-form boundary and special-radius regressions. Existing
D-33 balance-evidence requirements remain pending, not implicitly waived by
this art work.
