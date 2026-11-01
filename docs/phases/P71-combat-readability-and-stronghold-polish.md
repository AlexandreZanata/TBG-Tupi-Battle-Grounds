# P71 — Combat readability, fire identity and Stronghold polish

**Status:** execution in progress; Trident motion refinement blocked, see execution report.
**Planning branch:** `codex/p71-combat-visual-plan`.
**Execution branch:** `phase/p71-combat-visual-polish`.
**Art baseline:** P70 `b2680c3` (runtime source `c481369`), preserving the
46-champion cohort and its 230 forms. Resolve onto a reviewed descendant if
other fronts integrate first; record the actual execution SHA and installer.
**Delivery:** local commits, genuine renderer evidence and a native Linux
playtest candidate. Owner authorization on 2026-11-01 adds GitHub push after
commits and native Linux reinstall afterward, including subsequent targeted
adjustments. No merge, public deployment or release packaging.
Do not overwrite the owner's current worktree or P70 originals.

## 1. Owner outcome and scope

The owner reports Trident's, Whisperer's and Bonehorn's HP bars entering their bodies,
Rattle & Tick's weapon clipping through the ground, redundant blue
team bands, hard-to-read Evolution cubes and buggy tower shots. They request
bright yellow outlined Evolution stars, red/orange flame, a white/red/black
Pyre Warden and professional-looking tower performances and progressive
breakage for every 5% HP lost.

This phase changes client presentation only. Preserve authoritative combat,
Stronghold kinds, Fortification, damage, range, cadence, targets, HP/max HP,
friendly fire, projectile impact semantics, Energy, protocol and outcomes.
P70's Proposed Trident range is not reopened here; its D-33 acceptance stays
pending. Tower degradation is cosmetic and never disables its actual weapon.
Desktop Linux is the first playtest target; share the same art with desktop
web and Windows. Mobile and a new physics/shader/particle framework are out.

Interpret Evolution markers consistently: Base has no stars, A1/B1 has one,
A2/B2 has two. Stars have the same yellow/outline on both branches and seats;
branch identity remains in equipment and the status panel. Do not replace
Fortification with unit Evolution terminology.

## 2. Read first, then only the source for the current step

- [AGENTS.md](../../AGENTS.md), [VALIDATION.md](../VALIDATION.md).
- [P70 review](../planning/P70-VISUAL-REVIEW.md) and its
  [fixed roster](../planning/P70-VISUAL-MANIFEST.json).
- [ART-DIRECTION](../ART-DIRECTION.md): team identity, P70 palette,
  Strongholds, motion and budget sections; apply this owner override.
- [GAME-DESIGN](../GAME-DESIGN.md): BR-SK01–09, Fortification, and the
  Proposed P71 cosmetic contract. [DECISIONS](../DECISIONS.md): D-46/D-52,
  the P54 kit preservation decision and the dated P71 owner follow-up.
- [ARCHITECTURE](../ARCHITECTURE.md) §§3, 6, 7 and 10 only.
- This phase's source map below; use local Bevy 0.20 examples when APIs change.

## 3. Baseline observations and source map

Owner screenshot: `/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-28-59.png`,
1920 × 1080, SHA-256
`29359afaf162a5a9eef744fdea30fe2e43a1e53e8044dc930c8f1ccd0512bcc6`.
It shows the HP bar cutting through Trident's upper silhouette. It is a
reported installed-build frame, not a temporal proof of any tower-shot bug.
Do not invent animation findings from this still.

Additional owner screenshot:
`/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-46-37.png`,
1920 × 1080, SHA-256
`f606ea8f2c650df016791d61361cf76b5d8c7a19d787a4b3a8d6e4f98d07c19e`.
It shows selected Base Whisperer with its HP bar crossing the floating upper
body beneath the name label. Whisperer is a second mandatory regression case,
not merely part of the generic roster sweep. Preserve both owner originals.

Additional owner ground-clearance report:
`/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-48-11.png`,
1920 × 1080, SHA-256
`632c529801fb534e15ee66eefa26dc0c9f44594cacbfa4ebbdcb4ac1148b9e42`.
The selected unit is Base Rattle & Tick; the owner reports its weapon entering
the ground. Preserve this frame and reproduce the motion before attributing
it to a particular clip, socket, pivot or IK constraint. The club model and
pose sources are `art/kits/rattle_tick/{model,mod}.rs`.

Additional owner Bonehorn report:
`/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-53-42.png`,
1920 × 1080, SHA-256
`e21aa4676033ae7341a99b37093ce7dfe8e782ef4bdda8b640bfa2a8018d1104`.
The selected Base Bonehorn's HP bar is partly covered by its tall back armor.
The owner also reports a diagonal-looking walk; the still shows its oblique
body orientation but cannot establish its trajectory. Verify body facing and
foot motion against actual displacement in a recording.

Additional owner Drillnest report:
`/home/iiii/Pictures/Screenshots/Screenshot_2026-11-01_07-56-28.png`,
1920 × 1080, SHA-256
`3d8c8a65cf33fda2fcce2e1089c124199910b8635ea4fda24c2c256671cfc734`.
Selected Base Drillnest is visibly oblique to BattleLine; the owner reports
it moving diagonally. Record chassis facing versus actual travel before
attributing this to gameplay movement. Preserve this original with the
other feedback frames. Drillnest joins Bonehorn as a mandatory locomotion
alignment case.

Confirmed source limitations on P70:

- `visuals.rs::spawn_form_visual` uses `1.05 * form.plan.scale + 0.25`
  for ordinary unit bar height. `visuals/update.rs` repeats that formula
  after Evolution. Neither uses the actual tall shark model.
- `visuals/assets.rs` supplies a `tier_cube`; spawn/update use branch
  gold/silver materials. Bars are billboarded in `visuals/cues.rs`.
  `visuals/names.rs` and status badges share that screen region.
- `art/mod.rs::native_form` substitutes ENEMY for parts colored ALLY.
  Shared `art/kits/common/body.rs` and individual models author team patches.
  Removing only one shared tabard leaves many individual team markers.
- `visuals/update.rs` substitutes status colors for the team ring. Preserving
  unambiguous foot allegiance is a dependency of removing body team marks.
- Pyre's model is `art/kits/pyre_warden/model.rs`; its runtime jet and ground
  segment are `vfx/flame.rs`, currently green/white/gold with a green strip.
- `arena/strongholds/{mod,parts}.rs` implement only three damage states
  (two-thirds/one-third thresholds), with disappearing pieces. `dress`
  despawns/rebuilds children on look changes; `towers.rs` separately hides
  Classic cloth. A new damage policy must update both paths.
- `arena/strongholds/{aim,motion,gunners}.rs` control aiming, shutters,
  recoil and emission; `{classic,treasury,bastion,artillery,command}.rs`
  own distinct models. Existing `aim::fire` can emit a fallback top bolt
  after a defender disappears. Review that presentation explicitly.

Tower defects to reproduce, not yet confirmed: muzzle/mesh pose divergence,
side mirroring, closed shutters at emission, aim delay versus impact,
overwritten pending orders, reload seams, and damage/Fortification rebuilds
interrupting a shot. Tie each confirmed fix to a real before/after sequence.

## 4. Execution order and acceptance

### S0 — Freeze feedback and affected baselines

1. Create the execution worktree from the reviewed P70 descendant. Record
   source SHA, binary SHA, quality, camera, viewport and actual installed path.
2. Reuse immutable P70 evidence where the scene matches; preserve the owner's
   screenshot as a hashed original. Capture only missing baselines needed
   for HUD, Pyre and tower firing/degradation. Originals stay ignored;
   version compact derivatives and a linked evidence index.
3. List all 46 champions, 230 forms, five auxiliaries separately, and
   20 selectable Stronghold forms (four kinds × Base/A1/A2/B1/B2), both sides.
   Classic Base is a compatibility fixture, not a fifth selectable kind.
4. Write a tower bug ledger: kind/form/seat, target position, HP stage,
   order/emission/impact timestamps, reproduction and suspected source.
   A generic improvement cannot substitute for a reported bug reproduction.

Acceptance: fixed coverage inventory and reproducible immutable baseline;
no old capture is relabeled as new, and missing evidence is explicit.

### S1 — HP anchors, Evolution stars and persistent team rings

1. Put unit overhead layout in one focused module reused by spawn and
   Evolution update. Store a stable per-form head/body/attached-backgear
   envelope in the existing figure cache. Include custom anatomy and scale;
   exclude detached projectiles and the swept tip of long weapons, so a
   polearm does not place the HP bar a spear-length above its owner.
2. Calibrate clearance in the normal desktop camera using the projected top
   silhouette, not an arbitrary taller constant for Trident. Keep the bar
   stable through breathing/walking and its camera-facing bottom edge clear
   of head/helmet/body. Avoid per-frame mesh scans or a jittering height.
   Account for mounted Cargo Crab passengers, custom creatures, Evolution,
   spawn/reconnect and world/root offsets. Hide overheads on existing death.
3. Replace cube geometry with a cached five-point star plus a slightly larger
   dark silhouette behind it. Bright unlit yellow (`#FFD43B`, Proposed) and
   near-black (`#17120C`) outline retain contrast on snow and dark arenas.
   At 1080p begin with roughly 10–12 px stars and a one-pixel outline, then
   calibrate at actual game zoom. No font-dependent Unicode glyph or bloom
   is required. Share bar billboard orientation, disable shadows and prevent
   coplanar z-fighting. Center one star; align two above the bar with space.
4. Move name/status offsets into that same layout so bleed/shield/mark icons,
   labels and stars do not cover one another. Preserve legible fill and HP
   colors. Stars represent tier, never team, branch or combat strength stats.
5. Keep the ally/enemy team ring visible even while slow/root/mark applies.
   Reuse existing status-badge seams or a separate small status accent;
   do not overwrite the ring's blue/red allegiance. Preserve any existing
   accessible shape distinction; where absent, author a subtle solid/notched
   ring difference using shared geometry and verify both at match distance.

Acceptance: Trident, Whisperer and Bonehorn Base/A1/A2/B1/B2 bars clear their bodies
on both sides during Idle/Walk/primary action/Evolve. For Whisperer, include
the floating body's upper envelope, hood/attachments and hover motion; the
bar must stay stable and clear without clipping the name label or Evolution
stars. Reproduce the selected Base case from the second owner screenshot.
For Bonehorn, include the highest back armor/carapace and its gait bob,
not only the head or generic humanoid height.
Check the other 43 champions' five-form envelopes with
existing evidence plus affected captures. Base/one/two-star transitions,
reconnect, names, statuses, passenger overheads and death stay consistent.
No need to build a general-purpose HUD framework.

### S1b — Rattle & Tick weapon ground clearance

1. Reproduce the owner-reported Base weapon penetration at the ordinary
   match camera and actual BattleLine ground height. Inspect the complete
   club geometry, grip/socket transform, arm reach and animated pose in
   `art/kits/rattle_tick/{model,mod}.rs`, including custom parent transforms.
2. Correct the local carry angle, hand target, weapon pivot or swing path
   responsible for the intersection. Keep the weapon visibly held and the
   heavy character grounded; do not lift the whole unit, hide the weapon,
   crop the camera or shorten it merely to mask a bad transform. Preserve
   Tick's shoulder position and independent performance.
3. Check Base/A1/A2/B1/B2 on both sides through Idle, Walk, primary attack,
   special overlays, Hit, Spawn, Evolve, Victory, Death and their blends.
   The weapon can visibly contact the surface where intended, but must not
   penetrate it; a grounded Death pose must also remain coherent.
4. Add a focused `p71_` regression using transformed generated weapon vertices
   against the actual ground plane over the reproduced clip and its blend
   boundaries. Queue it with the existing client consolidation filter.
   Capture an ordinary-speed before/after sequence, not only a safe rest pose.

Acceptance: no club volume visibly sinks below the arena surface in the
reviewed five-form sequences; grip and body clearance remain intact. Record
minimum sampled world-space clearance and sampling limits. Keep existing
Strike/release timing, damage, range, cadence, target rules and hitboxes.
A finite sampled geometry test supports, but does not replace, rendered review.

### S1c — Trident anticipation and distance-aware spear thrust

Owner follow-up: the trident already passes into the opponent before the
thrust reads as an attack. Start the visible preparation before the weapon
reaches the opponent; raise/retract the weapon in the load, then aim its
points toward the target. Against a close melee opponent, carry a more
pronounced downward incline so extension reads as a controlled spear thrust.
This reopens P70's unresolved live contact/axis acceptance, including B2.

1. Reproduce the complete approach → ready → load → contact → withdrawal
   sequence with an actual opponent that closes during windup. Inspect
   `art/kits/trident/{motion,model,weapon,tests}.rs`, the Animator's aim/blend
   handling and actual rendered target/body positions. A studio target at
   fixed range is insufficient. Record first visual overlap versus the
   existing AttackStart/Strike events before choosing the repair.
2. Keep the ready/approach pose from already projecting the points through
   the opponent. Reposition/retract the two-hand guard and raise the load
   enough to clear the approaching body before extension. Cosmetic ready
   preparation may precede contact; actual attack/cast onset still follows
   the existing authoritative cue. Do not anticipate damage, change target
   selection, add a server cast event or trigger a fake early hit.
3. Blend the guard origin, elevation and visual extension with the actual
   displayed target distance/height. At ordinary spear distance, deliver the
   frontal point-led thrust; at close melee distance, start with hands/shaft
   higher and farther back and point the spear downward toward the opponent.
   Calibrate the elevation and extension against target silhouettes instead
   of one fixed pitch or an arbitrary new range threshold. This remains a
   thrust, not an overhead axe chop: the points lead toward the target and
   the shaft stays aligned through contact and withdrawal.
4. Preserve rigid weapon length and the existing two-hand IK anchors. Keep
   hands reachable and the shaft/butt outside Trident's own torso, jaw and
   fins; the tip must not sweep through the ground or pass out the far side
   of a small opponent before the intended contact. Use bounded cosmetic
   contact depth based on rendered geometry, not a new gameplay hitbox or
   collision simulation. Do not solve the defect by shrinking/disappearing
   the weapon or moving the authoritative unit farther away.
5. Fit the new load/extension/recovery inside the existing attack timeline:
   the primary Strike stays at 0.30 s and authoritative damage/cadence stay
   unchanged. Preserve Bloody Rend's independent channel/timing and inspect
   overlapping basic/special poses. Raise/retract before contact and withdraw
   along the same line without a sideways snap, teleport or grip jump.
6. Review all five Trident forms, both seats, near/normal range and an enemy
   moving inward; include short/tall targets, basic/special overlap, retarget,
   target death, Walk/Idle transitions and Hit/Evolve interruptions. Verify
   live transforms and aim weight rather than accepting a direct pose sample
   with forced full aim as proof of correct runtime contact.

Acceptance: normal-speed footage shows readable preparation before first
unintended body overlap, a more downward point-led thrust at close distance,
contact at the existing cue and a controlled return. Keep current HP-bar
clearance, own-body/ground clearance and grip regressions. Add focused `p71_`
regressions for the reproduced close-range path, both-side aim, finite/reachable
poses and transition clearance; queue existing `p70_trident_` regressions once
at consolidation because this step changes the previously validated pose.
Record before/after tip/shaft paths, event timestamps, spacing and sampling
limits. Current range 2000 remains Proposed under P70; this is not permission
to increase range again, change movement or delay the opponent's approach.

### S1d — Bonehorn straight-facing locomotion and overhead clearance

1. Make Bonehorn a mandatory S1 HP-anchor regression alongside Trident and
   Whisperer. Inspect `art/kits/bonehorn/{model,mod}.rs` and the projected
   whole-body silhouette, including the tall back armor in all five forms.
   Place its HP bar above that envelope through the complete walking bob,
   Idle and Evolution; keep the name/stars clear and the anchor stable.
2. Reproduce the apparent diagonal walk while logging/rendering the unit's
   actual displacement and body forward axis. The shared `visuals.rs::yaw`
   deliberately presents units obliquely (65/-65 degrees), and Bonehorn's
   current Walk poses do not explicitly align the body root to travel.
   Treat this as a source lead, not proof of a diagonal simulation path.
3. Align Bonehorn's visual longitudinal body axis, head/horn and gait with
   its actual direction of advance down BattleLine. Keep the feet stepping
   along that same axis rather than sideways under a rotated torso. Apply
   the smallest kit-specific orientation correction using existing figure/
   pose seams; do not rotate the shared camera or change every other kit's
   facing to fix one creature. Preserve the four-legged weight and stance.
4. Handle Walk-to-Idle/attack/recovery smoothly, with correct mirrored facing
   on both sides, no double yaw, sudden spin, foot sliding or reversal. Keep
   the bar billboard readable after the corrected visual rotation and check
   horn/body/ground clearance during contact. Actual targeting and movement
   remain authoritative; do not change lane position, speed, spacing, range,
   stats, hitboxes or model size to disguise the problem.
5. Record normal-speed Base/A1/A2/B1/B2 approach and stop/attack transitions
   on both sides. Add a focused `p71_` orientation regression comparing the
   rendered forward axis with travel under both mirrored parent frames,
   plus the shared form-aware anchor check. Queue these for consolidation.

Acceptance: Bonehorn visibly walks straight along its direction of advance,
with coherent foot placement. The bar stays wholly above the tall armor in
ordinary game views without name/star overlap. Before/after footage includes
motion, not just a differently posed still; disclose finite sampling limits.

### S1e — Bonehorn head-inclusive body occupancy

Owner follow-up: treat Bonehorn's head as part of its body blocking area;
it currently enters enemies. Include the forward head/muzzle and horn in
the visible occupied volume, rather than spacing units only from its trunk
or foot ring. This is a mandatory contact repair alongside S1d's facing fix.

Source clarification: `visuals/layout.rs` already separates drawn bodies
using scale-based circular footprints; its seats are laid out independently.
The core remains one-dimensional and is not driven by rendered meshes
(D-57). Implement this request through the existing presentation occupancy
seam, including opposing units, rather than silently introducing physical
head colliders or changing attack ranges in the simulation.

1. Reproduce Bonehorn's head overlap in live approach/contact, on both sides,
   using corrected S1d orientation. Measure the form's body footprint from
   transformed torso, head, muzzle, horn and attached armor. Exclude transient
   effects and unheld projectiles. Cache stable form extents; do not perform
   per-frame vertex collision or move the gameplay coordinate to follow a
   bobbing head. A narrow forward extent must not inflate every unit's radius
   or unnecessarily widen Bonehorn on all sides.
2. Use head-inclusive occupancy when resolving visual contact with allies
   and enemies. The current independent-seat layout cannot by itself prove
   cross-team clearance. Add the minimum bounded cross-side reconciliation
   needed for Bonehorn's frontal volume; keep unit ordering, allegiance and
   its four-legged gait coherent. Check existing broad-phase grid reach
   assumptions when extents grow, so distant buckets do not miss its head.
3. Stop the drawn body from advancing through an opponent before contact.
   Head/horn may make brief intentional contact at the existing strike, but
   must not remain buried in or emerge through the enemy. Keep both figures
   within their legible combat relationship; avoid oscillation, sudden rank
   jumps, sideways sliding or shifting a whole formation every head bob.
   Verify different enemy sizes and Bonehorn-versus-Bonehorn, including a
   crowded frontline and enemies moving inward during windup.
4. Any necessary drawn offset remains cosmetic and bounded: retain server x,
   simulation movement/stopping, target identity, range checks, damage and
   event timing. Preserve credible contact for Bonehorn and the opponent's
   weapon after separation; do not create a visible miss with remote damage.
   Ring, HP bar, stars, name, selection/picking and attack target anchors must
   follow the final displayed figure consistently. Do not change body size,
   camera or gameplay reach to mask overlap.
5. Add focused `p71_` cases for forward head occupancy, both seats and five
   forms, mixed-size/close opposing bodies, grid-boundary candidates and
   stable unchanged-view resolution. Preserve existing useful layout tests;
   any necessary client-only front-position exception must be explicit and
   limited to this contact policy, not a weakened universal no-overlap claim.
   Record actual before/after approach and strike sequences at normal speed.

Acceptance: the head is visibly part of the area occupied by Bonehorn; it
stops at opponents rather than entering their bodies. Verify contact, gait,
HUD and targeting cues together in all five forms and both seats, recording
maximum cosmetic offsets and finite sampling limits. No head-based damage,
new targetable zone or authoritative body-block mechanic is implied. If the
requested look cannot fit the existing combat spacing credibly, document that
specific conflict for product review before proposing a mechanical change.

### S1f — Drillnest chassis and track alignment

1. Reproduce the selected Base Drillnest approach from the owner screenshot
   in normal-speed footage. Inspect `art/kits/drillnest/{model,mod}.rs`,
   the shared figure yaw, Walk root/track motion and special overlay. Compare
   its longitudinal chassis/track axis with actual displayed travel; a static
   oblique silhouette alone does not prove diagonal simulation movement.
2. Align the chassis, both tracks and attached production assembly with the
   direction of advance, using the existing kit/figure seams. Keep track
   motion consistent with displacement so the machine rolls forward rather
   than sliding sideways. Handle both sides and parent transforms without
   applying yaw twice or changing the camera/global roster facing.
3. Review Base/A1/A2/B1/B2 in Walk, Idle, start/stop, Evolve, Hit, Death,
   Victory and actual drill production. In particular, special overlays must
   not reset the root orientation back to the unwanted diagonal during
   production or erase locomotion alignment. Keep production sockets and
   spawned cosmetic cues consistent with the displayed assembly.
4. Preserve Drillnest's authoritative movement, range, production trigger,
   intervals, spawn coordinates, living drill cap and fusion rules. It is a
   producer, not a conventional shooting tank: do not add a cannon attack
   merely to match the owner's colloquial vehicle description. Check actual
   Microdrill production and the existing higher-rank presentation without
   silently changing any simulation position to follow the cosmetic chassis.
5. Reuse a small existing orientation helper if it naturally supports both
   Bonehorn and Drillnest; otherwise keep the correction within each kit.
   Do not build a new motion framework or change every champion's facing.
   Add focused `p71_` mirrored-axis/transition regressions, including the
   production overlay, and queue them for targeted consolidation.

Acceptance: all five forms visibly roll straight along travel on both sides;
tracks, body and production assembly stay coherent through starts, stops
and production. Preserve grounded geometry, bar/star/name alignment and
existing production cues. Supply real before/after motion, source/binary
provenance and sampling limits; no runtime fix is claimed by this plan.

### S2 — Remove body team markers across the roster

1. Audit shared body helpers and all 46 kit models, five auxiliaries and
   tower defenders. Remove strips, patches, arm bands and body emblems whose
   only role is team color, on allies and enemies. Keep clothing/armor as
   coherent geometry: replace a structural tabard with intrinsic cloth,
   rather than leaving a torso hole or deleting a silhouette-defining coat.
2. Give decorative surfaces intrinsic kit/faction colors. Keep meaningful
   natural blues/reds (shark skin, robes, fishing coat, fire equipment).
   Do not delete every blue/red vertex or globally desaturate the roster.
3. Resolve color-equality remapping in `native_form`: an intrinsic color must
   not silently turn red on the enemy. Reuse identical team-neutral unit
   meshes when appropriate; retain variants only where a real need remains.
   Do not add an abstraction solely to replace a color.
4. Keep architectural Stronghold flags/banners and allegiance UI; the owner
   removed body identification, not castle ownership or War Banner's identity.
   Tower defenders follow the intrinsic-body palette policy too.
5. Compare allied/enemy model, portrait, Troops and P63 appearances using the
   same source assets. Refresh affected generated portraits through the
   established renderer path only; do not leave old team patches in cards.

Acceptance: no body patch remains solely to mark ally/enemy; the 46 identities
remain distinct and foot allegiance/statuses remain readable in dense combat.
Update ART-DIRECTION and the evidence index in the same logical change.

### S3 — Pyre Warden becomes a fire unit

1. Preserve the coffin-furnace/nozzle/hose silhouette and all five form
   equipment differences; recolor the kit with ivory/white armor, strong
   red furnace panels and charcoal/black fuel pack, gloves, hose and seams.
   Metal may remain a restrained neutral. Remove the team chest badge.
   A2/B2 must remain distinguishable by structure and material placement.
2. Replace spectral green/violet glow with a warm orange/red furnace core,
   orange body of flame, red outer tongues and a small yellow-white hottest
   center. No green jet or green ground segment remains for this effect.
   Use named semantic colors without recoloring other Undead effects.
3. Retain the real nozzle socket, propagated transform, event endpoint,
   ground danger segment, 0.20 s jet lifetime, trigger and current damage/
   friendly-fire rules. Essential nozzle-to-target flame survives Low;
   sparks/smoke are optional. Fire damage to allies must still read clearly.
4. Check release/aim/Walk/Victory and nozzle clearance in all forms. Record
   unobstructed and crowded actual live flames in both seats, standard/Low,
   rather than accepting event counts as recognizable fire.

Acceptance: white/red/black identity is visible at normal distance; flame
reads as red/orange fire, exits the nozzle, and covers precisely the
unchanged visual danger interval. Existing endpoint/timing tests remain valid.

### S4 — Distinct, synchronized Stronghold performances

Work through each kind and all five Fortification forms individually:

- **Bastion:** weighty masonry, readable opening shutter, falconet runs out
  before its muzzle emits, restrained recoil and recovery; B2 Sentinel's
  ballista has tension/release and a bolt aligned to its rail.
- **Artillery:** aligned carriage/barrel, clear load/brace/fire/recoil/return,
  purposeful gear motion, smoke with a short bright flash; A and B cadence
  differences come from existing authoritative intervals, not slowed shots.
- **Treasury:** hinged trapdoors clear the coin mortar, coherent opening/
  release/closing, a readable purse/coin volley with the existing trajectory;
  vault/roof and Fortification additions remain recognizable.
- **Command:** observer/archer turns with stable footing, draws, releases
  from the visible bow and returns; banner secondary motion supports the
  shot rather than hiding it. War Banner radius and passive remain exact.
- **Classic compatibility:** crossbow/sentry aim, release and reset remain
  coherent in reference/training scenes; do not expose it in the selector.

Use actual render-delayed Stronghold shot orders as the cue source. Make
muzzle flash, visible weapon, projectile origin/tangent and recoil use the
same evaluated release pose. Fix mirrored axes, hinge pivots, target changes,
reload/idle seams and any real order-loss bug found at S0. Review current
SETTLE delay against the existing impact timing; never add a dramatic pause
that makes a projectile arrive after authoritative damage. Cosmetic anticipation
cannot require a new server event or predict damage.

Do not hide a firing weapon and retain a floating bolt as a final look.
While alive, retain a plausible attached emitter or a visible reserve firing
port for every form/stage, synchronized with its shot. Degradation must not
reset the shot counter/aim or rebuild a moving weapon every 5% crossing.
Do not change mechanics to repair a visual trajectory.

Acceptance: actual basic shots on both sides, near/far/off-axis targets,
retarget/death, repeated attacks, Fortification during recoil and damage
crossings are aligned and recover cleanly. Record each fixed bug and each
kind's final signature sequence; a static gallery alone cannot pass motion.

### S5 — Twenty cumulative 5% degradation steps

Use only authoritative HP and that Stronghold's current max HP, including
Fortification. Compute the display stage safely with integer intermediates:

```text
max = max(authoritative_max_hp, 1)
hp = clamp(authoritative_hp, 0, max)
stage = floor((max - hp) * 20 / max)   # 0..20, using i64 arithmetic
```

Stage 0: HP >95% through 100%; first change at HP <=95%; then <=90%,
<=85% ... <=5%; stage 20 is zero HP. Thus there are 21 visual states,
including intact and destroyed. Every 5% boundary adds a readable cumulative
change: not a whole tower chunk on every minor hit, and no random rearranging.

Proposed art progression (thresholds are cosmetic tuning, never gameplay):

- 100%: complete structure, clean surfaces, intact machinery.
- 95–80%: a new crack/chip per step, dust and small edge fragments.
- 75–60%: broken crenellations, chipped facing, exposed timber, light smoke.
- 55–40%: wider fractures, missing secondary panels/roof tiles, localized
  small red/orange fire at authored combustible joints.
- 35–20%: several persistent flame pockets, charred edges, sagging trim,
  larger missing decorative pieces; firing mechanism remains readable.
- 15–5%: heavily fractured silhouette, exposed supports and denser smoke;
  keep the base, kind signature, team flag/marker and a credible live emitter.
- 0%: one bounded collapse into matching rubble and residual fire/smoke;
  outcome controls and the existing match completion remain immediately usable.

Each kind/form needs authored damage regions, loss thresholds and burn sockets.
Use masonry fragments for stone, splinters for timber and bent/detached metal
trim for machinery. Flames attach to combustible/interior fuel regions, not
arbitrary floating points. Maintain a cumulative deterministic damage mask
for a given look/stage; use a stable cosmetic seed for debris variation only.

Implementation should extend current reusable `Piece` metadata/cache and
split responsibilities into focused damage/transition modules. Prefer stable
children with visibility/material/pose changes on a stage transition. Build
meshes/materials once per form/part, never twenty full towers or new meshes
per frame. Change the old three-state loss constants, turret availability,
Classic cloth thresholds and gallery damage controls together. Keep old
working geometry and omit a new scene graph/physics dependency.

Handle state changes deliberately:

- A hit skipping several bands sets the complete final damage mask; play a
  bounded combined debris burst instead of replaying every skipped explosion.
- Initial snapshot/reconnect renders the current state directly, with no
  fake historical hit/debris/collapse burst. Repeated identical views do not
  accumulate emitters or retrigger transitions.
- Real healing may restore stages from the new HP fraction. Fade in repaired
  regions and reduce fire; do not implement a permanent worst-ever damage
  counter that visually contradicts HP. This grants no actual healing.
- Fortification changes max HP using BR-SK07's authoritative rounded ratio;
  recompute from that view and carry damage onto new geometry. Do not flash
  an intact upgraded tower or detach the active emitter. A boundary crossing
  from authoritative integer rounding is legitimate, not a free visual heal.
- At zero HP, use the terminal view and suppress live fire orders. Cleanup
  finishes on rematch, leaving training, scene change and reconnect teardown.

Reuse shared VFX meshes. Persistent fire/smoke should use a small fixed set
of sockets per tower with bounded density. Cap debris bursts, expire pieces
and reuse the existing simple VFX lifecycle; do not create an unbounded pool.
Low quality keeps the same structural damage and at least essential flames,
reducing smoke/sparks/debris. Reduced motion keeps damage readability without
shake/flashing. No extra dynamic lights or shadow-casting smoke are required.

Acceptance: 20 selectable forms × 21 states × 2 sides = 840 static matrix
entries, plus Classic Base × 21 × 2 = 42 reference entries. These are coverage
requirements, not permission to build/run hundreds of individual campaigns.
Prefer shared contact sheets, batched state inspections and focused temporal
transitions once at consolidation. Record pending entries explicitly.

### S6 — Consolidate evidence and deliver the Linux playtest

1. Produce the per-unit HUD/team-mark review and Pyre before/after, the tower
   bug ledger, damage contact sheets and actual temporal sequences. Preserve
   original source/binary hashes, both seats, quality and ordinary combat speed.
2. Inspect at normal game distance on Frozen Tundra (owner screenshot), a
   dark arena and the standard arena; use 1280×720 and 1920×1080 desktop
   layouts. Check stars on both dark and bright backgrounds and busy lineups.
3. On the final integrated code, run the focused checks below once. List
   source, real results, known failures and exact deferred scenarios in
   `docs/planning/P71-COMBAT-VISUAL-REVIEW.md`; never pass a recorded-only item.
4. Install the verified native development candidate beside the current
   Linux build, preserving saved data and a rollback launcher. Keep the
   application's TBG shortcut and validate the installed binary/assets.
   Test Troops, local bot entry, Evolution and Stronghold shots in that build.
   Do not replace P70's evidence binaries or publish a release.
5. Present the concrete playtest and comparison pack for owner look/fun
   review. Update roadmap and stop for review with local commits.

## 5. Validation policy and focused regression queue

Planning now: Markdown/source-path inspection and `git diff --check` only.
No Cargo, Trunk, runtime campaign or installer mutation in this planning task.

Implementation work fronts: affected-package formatting, then one inexpensive
cached client check if available; queue tests until consolidation. Every
compiling command uses the shared nonblocking host lock and two jobs:

```sh
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo check --offline --locked -p tbg-client --bin tbg-client
```

Reuse the actual integrated tree's existing cache/target and installed native
pkg-config path where needed. Busy lock (75), missing lock support or cold
cache means deferred, never wait/poll/bypass/clean or invent a new target.

Write narrow `p71_` regressions for the documented contracts that can fail:

- Shared spawn/Evolution anchor selection and stable body clearance using
  actual custom figure bounds; Base/one/two-star counts and team-neutral color
  semantics. Rendered projection still requires visual evidence.
- Stage boundaries at/beside every 5% threshold, clamping and i64 overflow
  safety, varied kind max HP including Bastion 5175 and Fortification.
- Cumulative masks, live emitter availability, healing/Fortification/view
  idempotence and terminal/reconnect cleanup. No mechanical core test needs
  changing merely to accept a client damage mask.
- Actual turret configurations on both sides, release transform/tangent,
  authoritative-order cardinality and no duplicated cosmetic shot on rebuild.
  If a real order-loss bug exists, preserve its minimal reproduced scenario.

At consolidation run this selected union sequentially under the same wrapper:

```sh
cargo test --offline --locked -p tbg-client --bin tbg-client p71_ -- --nocapture --test-threads=1
cargo test --offline --locked -p tbg-client --bin tbg-client arena::strongholds:: -- --nocapture --test-threads=1
cargo test --offline --locked -p tbg-client --bin tbg-client pyre_jet -- --nocapture --test-threads=1
```

Update client-only three-state tests to the new cosmetic contract while
retaining coverage; contract tests remain append-only under AGENTS.md.
Do not repeat unaffected P70 tests by habit. A changed muzzle/hand geometry
also queues its existing grip regression, named in the execution report.

A native build may be needed for the requested Linux playtest; do it once
under the lock. Cached WASM checking and desktop-browser runtime inspection
are separate consolidation evidence; Windows runtime is reported separately
if unavailable. No full suite, workspace sweep, D-33 gate, release build,
benchmark or packaging build is implied. Measure obvious entity/VFX growth
with ordinary captures; a heavy performance campaign needs separate scope.

## 6. Completion and report structure

Record one logical local commit per step. The final report must include:
actual base/candidate and installation path; owner requests versus Proposed
art tuning; confirmed defect causes; per-step implementation/files; passing
commands; real screenshot/video provenance; fixed and remaining bugs;
coverage matrix; deferred commands/scenarios with reasons; deviations;
rollback information; and pending owner subjective approval.

Do not close P70's unrelated acceptance items through this phase, conflate
Fortification with Evolution, claim every tower bug fixed from one still,
or call P71 complete with missing applicable evidence. The owner approves
look and fun after a concrete local playtest is available.
