# P70 next cohort — Monsters, Celestials and Insects

Status: all twelve individual source implementations completed; final runtime
and visual review pending. Source and baseline preparation completed. No final render, runtime acceptance or owner visual approval yet.
The pilot consolidation lock returned 75; its runtime gate remains deferred.
Per owner minimal-validation policy, this independent art front continues while
that gate and Trident's separate balance evidence are pending. This is a
recorded sequencing deviation, not a claim that S2 passed.

## Evidence and boundaries

The immutable baseline is source `55510c5363e3f1e555fa5133208a5faf20c5825b`
with capture tooling `27f98acd98b51fa69eb09f4703bd2ca59a33ed41`.
Reviewed the [Monsters montage](../../.local/p70-evidence/derived-before/Monsters.png),
[Celestials montage](../../.local/p70-evidence/derived-before/Celestials.png),
[Insects montage](../../.local/p70-evidence/derived-before/Insects.png), and their
[derivative metadata](../../.local/p70-evidence/derived-before/derivatives.json).
These are resized/captioned Base Idle crops, not normal-match-distance proof.
Also inspected original five-form sheets for
[Bonehorn Victory](../../.local/p70-evidence/before-cohort/Bonehorn/ally-threequarter/09.png),
[Whisperer Victory](../../.local/p70-evidence/before-cohort/Whisperer/ally-threequarter/09.png),
[Celestial Archer ranged action](../../.local/p70-evidence/before-cohort/CelestialArcher/ally-threequarter/03.png),
and [Mantis melee](../../.local/p70-evidence/before-cohort/Mantis/ally-threequarter/02.png).
Static poses do not establish motion quality, live special cues or transitions.
All twelve `model.rs` and `mod.rs` files were read, along with the manifest,
phase/executor brief, shared motion/gear helpers, palette, timing and P63 hooks.

Source confirms Bonehorn, Ravager and Whisperer combine Idle and Victory;
Weaver's Victory is a fixed head tilt. Other kits mostly use common cheers.
The eight planted kits call `common::moves::idle`, which already applies aim,
then apply final aim again. Local authored Idle replaces this double application
while retaining one final `tools::face`. Preserve the unusual custom parent/rest
functions, four Bonehorn supports, six Ravager supports, six Whisperer tendrils,
eight Weaver legs, and the existing bipedal Jawguard/Mantis/Hornet body plans.
Do not reinterpret anatomical names as permission to change their limb count.

All proposals below are presentation only. Existing sockets, shot kinds, counted
procs, `Moment` counters/reduction/specials, five forms and authored life clips
remain. Melee contact stays 0.30 s; ranged release stays 0.18 s and cast release
0.22 s on their actual live paths. Keep bow string/nock/upright solve and P63's
post-kit celebration ownership. No shared palette, core, VFX, registry or capture
edits belong to this front. Existing palette colors provide the proposed blocks.

## Individual dossiers

### Bonehorn — heavy quadruped, not an upright tank

Baseline: horn, enormous pale mantle and black underbelly read; gray face and
box feet merge into the other Monsters. `model::{skull,mantle,legs}` and the
custom four-support rest are the anchors. Main warm dark leather hide, supporting
charcoal belly/hooves, ivory armor and turquoise eyes distinguish the body;
small rust brow plates connect the face to the warm hide. Retain side team cloth.
A1/A2 keep widening mantle/leg armor and taller dorsal spikes; B1/B2 keep longer
tusks and jaw guards, with a darker hide region rather than losing the new base.
Idle lowers the muzzle to sniff, then shifts the neck under the mantle. Walk
keeps the diagonal quadruped cycle with shoulder compression and head counterweight.
Primary keeps coil, front-leg brace and tusk shove at 0.30 s, with a clearer
head-first recovery. Victory plants the rear supports, presents the horn and
settles the mantle in a slow proud display. No bipedal arm cheer.

### Ravager — low six-support predator

Baseline: the low silhouette works; tiny ivory hooks and charcoal body read
as an indistinct small animal. Keep the exact six articulated supports and
socket hierarchy. Violet dorsal carapace, charcoal legs, ivory cutting edges
and rust scythe roots create three large regions. Broaden the scythe shoulder
root without moving its true contact tip. A1/A2 retain plated knees/spines;
B1/B2 retain wider blades/jaw teeth. Idle alternates a probing front-hook curl
and head tracking; Walk keeps the alternating tripods with a hunting chest
countertwist. Primary compresses the front pair, exposes the attacking blade,
contacts at 0.30 s and retracts. Victory raises the anterior hooks while the
four rear supports hold the low body. Counter-driven frenzy remains additive.

### Acidmaw — long-necked four-petal spitter

Baseline: turquoise throat is memorable, but neck/legs almost disappear and
head looks detached at crop scale. Preserve planted legs and all four head-parented
jaw sockets. Dark green hide, charcoal neck rear, ivory throat braces and turquoise
sac define a continuous neck; add a pale ventral neck strip with room for the sac.
A1/A2 retain neck rails/crest; B1/B2 retain larger armored sac. Idle uses sac
breathing, tiny alternating jaw openings and a neck bob. Walk counterbalances
neck and sac against planted steps. Spit visibly loads the throat, spreads petals,
then snaps at existing ranged release 0.18 s with the emitter still Head +Z.
Victory elongates the neck, fans the four petals and folds them back, arms low.
Counter and special-state sac cues remain; no synthetic acid event.

### Whisperer — suspended eye bell

Baseline: the single eye and hovering outline work; neutral cap is close in value
to Bonehorn/Ravager and the simple Victory is barely distinct from Idle. Use an
ivory crown over violet/dark lower bell, dark tendrils, turquoise eye and a narrow
bronze rim; retain its team ribbon. A1/A2 keep tall horns/side crests; B1/B2 keep
cage ribs/enlarged eye. Idle slowly tilts the bell while one tendril pair answers;
Walk banks and trails the existing six tendrils, without adding feet. Primary
folds the tendrils toward the eye, opens at the existing cast release and settles;
the actual Thought emitter remains `Prop(Back)`. Victory makes a restrained bell
orbit and a radial tendril unfurl. The independent fear pulse remains driven
only by `special`/`special_ready`.

### Solar Sentinel — sun disk with forward spear

Baseline: the shield gives identity, but ivory/navy torso resembles every other
Celestial. Gold breastplate, ivory edges and dark navy undercloth make the shield
and armor one clear sun-bearing mass. Keep the fitted S1 chest heraldry, sun
emblem clearance, shield split panels and spear length/socket. Add a short visor
brow to differentiate its closed helmet. A1/A2 retain broad armor and shield
crest; B1/B2 retain shoulder suns and expanded halo. Idle watches over the
shield rim; Walk counter-rotates shield and spear over heavy planted steps.
Primary visibly retracts the spear before the existing 0.30 s forward extension;
shield stays braced. Victory raises shield and spear in separate phases through
the hand IK targets, rather than having the common arm cheer overwritten by IK.
The real sun-opening grant retains its independent panel motion.

### Luminous Oracle — hooded ring-staff caster

Baseline: slim robe and staff read, but white robe repeats the faction. Violet
robe and lower sleeves, ivory hood/stole and gold ring form a distinct tall
column around a navy empty face. Retain rear team cape patch. A1/A2 retain broad
stole and taller/larger staff ring; B1/B2 retain navy robe emphasis and ring fins.
Idle inclines the hood toward a slowly orbiting staff ring; Walk takes small
planted steps with robe counter-sway. Primary draws staff inward, then presents
the ring at 0.22 s before a soft recoil. Victory traces a small staff arc and
opens the offhand, using IK targets so the fixed hold cannot erase the gesture.
Preserve the form-specific muzzle height and authoritative closing-hand special.

### Celestial Archer — angular bow and rigid feathers

Baseline: face, halo and long bow are clear, but rectangular white front looks
like generic armor; the static draw already has real string/nock motion. Navy
chest/upper cloth, a narrower ivory central plate, gold bow and ivory feather
planes form a slimmer silhouette. Keep the diagonal team sash and quiver patch.
A1/A2 retain precise larger bow/arm armor; B1/B2 retain dark plate and split bow
ornaments. Idle adjusts bow elbow and head with independent rigid feather settling;
Walk carries the upright bow close while the free hand balances. Primary keeps
0.12 s draw, 0.18 s release and all string/nock/upright solving, with chest load
and wrist follow-through. Victory presents the bow vertically, free hand over
heart, then opens the feather ornaments. No new arrows or purification event;
counted-shot and `special` halo remain authoritative.

### Justiciar — segmented judicial construct

Baseline: broad square chest and banded forearms identify it; white front is
another faction clone. Charcoal breastplate center, ivory shoulders/face/gauntlet
segments and a compact gold sun make the rigid mass unique. Add short gold
shoulder brackets and keep a separate team pauldron. A1/A2 keep broad chest and
collar; B1/B2 retain dark forearm stacks and raised shoulder fins. Idle aligns
one wrist stack against the chest sun, with small mechanical head correction.
Walk has alternating square shoulder counterweights over heavy short steps.
Primary chambers the right gauntlet, presents its palm at cast release 0.22 s
and recoils into alignment. Victory locks the forearms across the chest then
opens them ceremonially. Keep the independent real two-palm special stamp and
chest mechanism pulse.

### Jawguard — broad carapace and mandibles

Baseline: rounded moss shell reads, but arms are generic dangling blocks and
the face lacks definition. Retain the bipedal authored rig. Dark chitin head,
moss shell, broad amber brow/edge and amber mandibles frame a clear beetle face;
small ivory shell seams keep the dome readable. A1/A2 preserve pale reinforced
shell and dorsal horn; B1/B2 preserve dark face and larger jaws. Idle braces the
forearms beside the shell and performs an asymmetric jaw click; Walk carries
shell weight with low shoulder counter-swing. Primary draws the head back,
opens jaws, plants arms and clamps at 0.30 s. Victory lifts the head and spreads
mandibles under a small shell bow, never a human overhead cheer. Reduction and
expiry-ready counter cues retain their final additive priority.

### Mantis — praying scythes and triangular face

Baseline: narrow body distinguishes it; the curved blades are small and merge
into amber antennae. Leaf-green thorax/head, dark joints, pale cutting edges and
amber blade roots separate body from weapons. Broaden the scythe ring section,
keeping its forward contact tip and existing parent. A1/A2 preserve pale shin
plates/crest; B1/B2 retain dark wider blade backs and elbow teeth. Idle grooms
one folded scythe, then restores prayer; Walk counterbalances high knees with
folded arms and a leading head. Primary adds an exposed elbow load before the
variant-specific 0.30 s cut. Victory draws both blades down into a symmetric
fan, then closes to prayer. Real fourth-strike crossing remains the `special`
overlay; do not imply a second simulation hit.

### Hornet — banded abdomen and paired wings

Baseline: opaque wings are legible but the green torso hides wasp identity;
the stinger resembles a small handheld needle. Amber thorax with dark banding,
chitin head/limbs, linen wings and a thicker amber stinger root create a wasp
block at match scale. Keep four wings, existing planted bipedal motion, abdomen
and real stinger emitter. A1/A2 keep pale venom torso/sacs and longer stinger;
B1/B2 keep darker torso and larger wing ornaments. Idle folds/opens paired wings
against an abdomen response; Walk keeps feet planted with a forward-ready sting
and wing counterweight. Primary draws the sting hand back, loads abdomen, then
extends at 0.18 s through the existing IK. Victory fans wings and bows abdomen
while presenting the stinger safely away from the head. Every-third venom cue
remains counted; the flourish must not fake flight or an extra projectile.

### Weaver — low eight-legged spinner

Baseline: small moss abdomen blends with the rest of the Hive; the eight-leg
outline is sound. Violet abdomen, dark lower legs, ivory dorsal silk ridges and
amber spinner establish a unique low body; keep the separate dorsal team patch.
A1/A2 preserve silk rails/frame and pale leg tips; B1/B2 retain wider front legs,
darker body and split nozzle. Idle sequentially tests front feet and tilts the
spinner; Walk preserves alternating four-leg groups with rear body lag. Primary
braces the front legs, compresses abdomen and pitches the spinner at cast release
0.22 s; source emitter stays on Head-parented WeaponR. Victory performs a small
front-to-back leg ripple with at least four supports down and an abdomen bow.
No humanoid planting, full-body spin, lost legs or synthetic web event.

## Validation and pending review

Implementation must keep full Base/A1/A2/B1/B2 distinctions and review Spawn,
Hit, Death and Evolve for interference with new geometry. Static baselines do
not contain guaranteed live `special` states. After final formatting/diff review,
queue one consolidated client diagnostic run and affected five-form originals,
then real normal-camera engagements, both seats, special cues, Low readability,
transitions, nameless lineup and P63 Victory. No compile/test/render claim belongs
to this preparation record. Owner visual approval remains pending.

## Work-front implementation result

All twelve owned kit directories now change both model/color composition and
Idle, Walk, primary action and Victory. The dossiers above describe the resulting
source intent. Shape additions are bounded mesh parts created by figure builders,
never per-frame assets. No semantic palette additions were needed. The eight
planted kits no longer call the aiming common Idle before their final aim call.
The shared life-clip functions, unusual rest/parent maps, attack socket/tip/muzzle
contracts, durations, shot/impact recipes and special-state feeds are unchanged.
Hornet's new free-arm pose is scoped to Idle/Walk/primary/Victory so it does not
replace the inherited life-clip forearm. P63 still resolves and bounds Victory.

A read-only analytic check used the actual shoulder rest coordinates
`(±(torso.x/2+0.05), torso.y*0.9, 0)` and arm span `1.9*arm`, including conservative
Cartesian combinations of cast, special and salute endpoints. Evolution only
changes root scale, so these local bounds cover all five forms:

- Oracle offhand: maximum 0.313984 versus span 0.323000; staff hand 0.302132.
- Solar shield: maximum 0.314006 versus span 0.323000; spear hand 0.314261.
- Hornet sting: maximum 0.356071 versus span 0.361000.

The initial proposed Oracle forward presentation exceeded its short arms;
raising the offhand and reducing its cross-body close restored reach. Solar's
low wrist plus forward thrust also exceeded reach; the wrist now rises with
thrust while keeping its original resting height. These are analytic reach
bounds only, not proof of elbow, cloth, socket or body clearance in rendered
frames. Orbit/staff grip orientation and simultaneous specials still require
real review. No unrelated front was formatted or modified.

Validation level: work-front source/format inspection only. Commands run:

```text
rustfmt --edition 2021 --config skip_children=true crates/tbg-client/src/art/kits/{bonehorn,ravager,acidmaw,whisperer,solar_sentinel,luminous_oracle,celestial_archer,justiciar,jawguard,mantis,hornet,weaver}/{mod,model}.rs
exit 0, no output

git diff --check -- <the twelve owned kit directories> docs/planning/P70-NEXT-MONSTERS-CELESTIALS-INSECTS.md
exit 0, no output
```

A source comparison confirmed all twelve `KIT` definitions are unchanged,
including the attack emitter configuration and duration tables. Evidence-link
inspection found all eight local links present. Rust parsing/formatting is not
a compile check. No Cargo builds, tests, renderer launches, captures or commits
were run on this front. Runtime/body clearance, normal-match readability,
loop continuity, special combinations, native/web compatibility and owner
acceptance remain deferred to the designated consolidation.
