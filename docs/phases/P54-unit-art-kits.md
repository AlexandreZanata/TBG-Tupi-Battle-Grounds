# P54 — Unit art kits: own models, motion, aim and projectiles

**Depends on:** `main` (P16-F4 modular Strongholds merged). Independent of
the roster expansion (P42–P53): it touches only client art, effects, and
Stronghold presentation, never `tbg-core` or the protocol.
**Branch:** `phase/p54-unit-art-kits`.
**Difficulty:** high (architecture + art + animation quality).
**Owner request (2026-10-30):** review the walk of every character; give
each character its own code-built asset library — own projectile, own
attack ("shot" or "hit") animation, correct aim (the Siege Walker points
the tip of its bazooka at the target it fires at), correct walking; give
cannon towers their own projectiles and make them aim at the characters.
Start with **Siege Walker, Bone Brute, Rootcaller and the Strongholds**;
make their Evolutions striking, with their own animations, and the
champions much more beautiful. High frame rate on modest machines stays a
hard requirement. Art budgets may grow up to 100 % where needed (owner,
same day).

## Goal

1. A **unit art kit** format: one folder per character that owns its
   figure (five forms), its signature motion layered over the shared clip
   library, its gait, its aim rig, and its attack style (own projectile,
   muzzle effect and impact, or melee impact). Kits are plain data plus
   function pointers in a static registry — no trait, no engine layer.
2. **Gait v2 for every unit**: walk cadence driven by the distance the unit
   actually travels (no foot sliding at any speed, slow or Evolution
   bonus), feet planted in world X *and* Z with a lateral leg roll, a
   start/stop hysteresis so units never flicker between walk and idle, and
   two-bone legs for adapted (legacy) figures too.
3. **Aim**: kit units with a target turn their body toward it and pitch
   their weapon to the launch angle of the shot they are about to fire;
   the projectile leaves the real muzzle position.
4. **Strongholds aim and shoot their own projectiles**: the Artillery
   carriage and barrel track their target (yaw + elevation), recoil along
   the aimed axis and fire a cannonball with smoke from the muzzle; the
   Sentinel ballista tracks and fires a bolt; towers without a gun fire a
   crossbow bolt from their top.
5. Three kits at final quality: **Siege Walker** (shoulder bombard /
   "bazooka"), **Bone Brute**, **Rootcaller**, each in five forms with
   Evolution reads per ART-DIRECTION §5 (scale, large-surface color change,
   new silhouette piece, tier-2 signature artifact with glow).

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §4, §5, §8, §9
- `docs/GAME-DESIGN.md` §5 rows for the three units (stats, Evolution tree)
- `docs/ARCHITECTURE.md` §7
- Client: `src/art/` (anim, curves, clips, feet, figure, rig, archer_*),
  `src/visuals/`, `src/vfx*`, `src/sync/events.rs`, `src/arena/strongholds/`

## Do NOT

- Change any simulation value, event, timing, message or protocol. All of
  this is cosmetic; `tbg-core` is read only.
- Move the timing contract (ART-DIRECTION §4): melee Strike at 0.30 s,
  ranged Release at 0.18 s, cast Release at 0.22 s after `AttackStart`.
- Add textures, glTF, skinning, shaders or dependencies.
- Touch the other units' figures (only the shared gait changes them).
- Allocate per-entity materials: every effect and figure reuses shared
  meshes and materials (performance requirement).

## Design

### Kit format (`crates/tbg-client/src/art/kits/`)

```text
kits/mod.rs           registry `kit(UnitKind) -> Option<&'static Kit>`, Kit, Moment
kits/gait.rs          Gait data + the shared locomotion layer
kits/aim.rs           figure-space aim solve (yaw, ballistic pitch)
kits/siege_walker/    mod.rs (Kit), model.rs (five forms), motion.rs (clips)
kits/bone_brute/      idem
kits/rootcaller/      idem
art/ik.rs             general two-bone IK (legs and arms)
vfx/shots.rs          styled projectiles: parabola, tangent facing, trails
vfx/impacts.rs        muzzle, impact, slam, root and spore recipes
arena/strongholds/aim.rs  turret aim, muzzles, tower shots
```

```rust
pub struct Kit {
    pub figure: fn(tier: u8, branch: u8) -> FigureSpec, // native rig v2
    pub pose: fn(&Moment, &mut Pose),   // signature layer over shared clips
    pub gait: Gait,                     // stride, bob, lean, arm swing, stomp
    pub attack: Attack,                 // Shot { joint, muzzle, style } | Melee { impact }
}
```

`Moment` carries the clip, its time, the form, the aim (target point in
figure space and a 0..1 weight), seconds since the last attack, and the
attack variant (alternating A/B). The Animator calls the kit layer after
sampling the shared clip, exactly like the Archer layer does today.

**Glow layer:** a `PartSpec` may be marked `.glow()`; glow parts merge
into a second mesh per joint drawn with one shared unlit vertex-color
material (eyes, furnace, runes, spores). One extra draw per glowing joint.

### Timing

| Kit | Clip | Event | Projectile flight | Arrives |
|-----|------|-------|-------------------|---------|
| Siege Walker | AttackRanged 0.5 s (×1.25 recovery for Large: 0.62 s) | Release 0.18 s | 0.12 s | 0.30 s |
| Bone Brute | AttackMelee 0.62 s | Strike 0.30 s | — | 0.30 s |
| Rootcaller | Cast 0.6 s | Release 0.22 s | ground ridge 0.08 s | 0.30 s |
| Stronghold | — | `AttackStart` | 0.25 s (cannonball 0.3 s) | — |

### Performance rules

- Kit poses are a few `sin`/`lerp` per joint; aim is solved once per unit
  per frame; no allocations per frame.
- Effects reuse a handful of shared meshes/materials; each effect is a
  short-lived entity that only scales and moves. A shot with a trail
  spawns ≤ 12 puffs in its life; impacts ≤ 16 entities.
- Measure: `?gallery=<unit>&stress=30&measure` (30 copies of one kit unit
  walking and attacking) on desktop; the report records FPS.

## Steps

### 1. Plan
This file and the ROADMAP row. **Done when** committed.

### 2. Kit framework, IK and glow layer
`art/ik.rs`, `art/kits/{mod,aim}.rs`; `FigureLibrary` builds kit units from
their kit; glow meshes and material; `Animator` gains the kit layer, aim
state and per-kit clip duration; `UnitVisual` keeps the aim target.
Check: `cargo test -p tbg-client` (IK reaches its target; aim yaw/pitch
point the muzzle at the target). **Done when** existing figures render
unchanged and tests pass.

### 3. Gait v2 (every unit)
Distance-driven phase, world-XZ foot locks with lateral roll, two-bone
legs for every figure, start/stop hysteresis, per-kit gait (Large stomp
dust). Check: unit test — a planted foot stays within 1 mm of its anchor
over a contact half-cycle at any speed. **Done when** every unit walks
without sliding in `?gallery=<unit>&stress=30`.

### 4. Shots and impacts
`vfx/shots.rs`, `vfx/impacts.rs`; `sync/events.rs` stops spawning the
generic ball for kit units and Strongholds and hands them the target.
Check: unit test — a shot starts at the muzzle and ends at the target;
its launch tangent equals the aim pitch. **Done when** non-kit units still
show the generic projectile.

### 5. Siege Walker kit
Ogre gunner with a shoulder bombard. Forms: Base (bronze bombard, harness,
team loincloth, powder keg) · A1 Longshot (long barrel, brass scope,
chest plate) · A2 Artillery (twin-bore great bombard, gold muzzle, plumed
helm, glowing boiler) · B1 Shrapnel (flared blunderbuss bell, spiked
pauldron, shell bandolier) · B2 Concussion (drum-fed hammer bombard,
glowing bands, iron jaw). Motion: braced aim (body yaw + chest pitch to
the ballistic angle), recoil kick and half-step back, backblast, reload
between shots, heavy stomp walk, drop-in spawn. Shots per form: shell,
long shell, great shell, shrapnel burst, hammer shockwave.

### 6. Bone Brute kit
Hunched giant skeleton. Forms: Base (rusty cleaver, violet rags) · A1
Ossified (bone plate armor, buckler) · A2 Bone Wall (skull tower shield,
horned helm, green runes) · B1 Butcher (twin cleavers, blood apron, hooks)
· B2 Reaper (great scythe with glowing edge, hooded cloak, skull crown).
Motion: hunched breathing idle, lumbering stomp walk, overhead slam and
side swing alternating, shield bash + chop (A2), double chop (B1), wide
scythe sweep (B2), rise-from-the-ground spawn. Impacts: dust ring, bone
chips, crack; Reaper crescent matching splash 1.2.

### 7. Rootcaller kit
Treant shaman. Forms: Base (bark body, moss, root claws, leaf crown) ·
A1 Deep Roots (longer claws, vine coils, foot roots, deep moss) · A2
Strangle (thorn claws, thorn crown, glowing amber heart) · B1 Spores
(mushroom cap, spore sacs) · B2 Overgrowth (great glowing cap, back
mushrooms, moss cloak). Motion: swaying idle, rooted shuffle walk, raise
and slam-into-ground cast with channel, sprout-from-the-ground spawn.
Effect: a root ridge runs along the ground to the target and roots erupt
around it for the root duration; spore cloud for the B branch (splash).

### 8. Strongholds aim and own projectiles
Turret yaw/pitch on the Artillery carriage and barrel, Sentinel ballista
yaw/pitch, recoil along the aimed axis, muzzle flash and smoke, cannonball
/ ballista bolt / crossbow bolt per kind, impact dust. Shots are driven by
the render-delayed `AttackStart` events (replaces the view-time queue).
Gallery towers track a moving dummy. Check: unit test — the aimed barrel
axis points at the target within 1°.

### 9. Review tools, verification, report
`?gallery=<unit>&range`: the five forms fire at a dummy with full kits.
`?gallery=<unit>&stress=N` uses that unit. All AGENTS verification,
`trunk build --release`, browser screenshots (desktop) of each kit's
forms and range, a Practice match observed, FPS measured. Update
ARCHITECTURE §7 and DECISIONS (D-51). Fill in the report.

## Acceptance checklist

- [x] `cargo fmt`, client clippy/tests, WASM check pass; `trunk build --release`
      passed at `ecacd71` (workspace tests not run: no core/server change)
- [x] Timing contract unchanged (events at 0.30 / 0.18 / 0.22 s)
- [x] Planted feet do not slide (test) for every unit; no walk/idle flicker
- [x] Siege Walker muzzle points at its target; shells leave the muzzle
- [x] Artillery barrel and ballista track their target; own projectiles
- [x] Five forms per kit with the §5 Evolution reads; triangle counts logged
- [x] Desktop screenshots in `docs/media/units/<unit>/` (≤ 300 KB each)
- [x] 30-unit stress FPS recorded per kit (60 FPS each, idle machine)

## Phase report

### What was built

| Area | Files | Result |
|------|-------|--------|
| Kit format | `art/kits/{mod,aim,gait,tools,tests}.rs` | `Kit { figure, pose, gait, attack, evolve, durations }`, a static registry `kits::kit(UnitKind)`; `Attack::Shot { from, muzzle, shot, flash }` or `Attack::Melee { from, tip, impact }` |
| IK | `art/ik.rs` | Two-bone hinge IK (no twist at knees/elbows), used by legs, held weapons and reaches |
| Gait v2 | `art/feet.rs`, `art/anim.rs`, `visuals/cues.rs` | Cadence = distance travelled / stride for every figure; feet locked in world X and Z while planted; lateral steps solved, not slid; 0.15 s stop hysteresis; animation in `PostUpdate` after all movement; review figures walk on a treadmill |
| Aim | `kits/aim.rs`, `visuals.rs` (`engage`), `anim.rs` | A unit keeps its target for its interval + 0.7 s; kits turn the body so an off-center weapon lines up exactly (`line_yaw`) and raise it to the launch tangent of their own shot |
| Glow layer | `art/figure.rs`, `art/mod.rs` | `.glow()` parts merge into one unlit mesh per joint (eyes, furnace, runes, souls, spores) |
| Shots and impacts | `vfx/shots.rs`, `vfx/impacts.rs`, `vfx.rs` | 13 shot kinds, 14 impacts, 6 muzzles, footfall dust, 3 Evolution bursts; shared meshes/materials only |
| Kit attacks | `visuals/attacks.rs`, `sync/events.rs` | Release fires the kit's shot from the real muzzle (with flash and backblast); Strike lands the impact at the blade/target; kits skip the generic ball |
| Siege Walker | `kits/siege_walker/` | Ogre gunner, shoulder bombard held by both hands (IK), braced aim, recoil, crank reload, drop-in spawn, skyward salute on Evolve |
| Bone Brute | `kits/bone_brute/` | Hunched skeleton, soul in the ribcage; slam / sweep alternation, shield bash (Bone Wall), double chop (Butcher), reaping sweep (Reaper); rises from the ground; roars on Evolve |
| Rootcaller | `kits/rootcaller/` | Treant shaman; claws driven into the ground, root ridge to the target, roots for the real root duration, spores on branch B; sprouts on spawn; blooms on Evolve |
| Strongholds | `arena/strongholds/{aim,motion,artillery,bastion,parts,mod}.rs` | Artillery carriage/barrel and Sentinel ballista slew onto the target and fire their own cannonball/bolt from the muzzle; others loose a crossbow bolt; gallery towers track a dummy |
| Review | `gallery/range.rs`, `gallery/settings.rs`, `stress.rs` | `?gallery=<unit>&range[&evolve]`; `?gallery=<unit>&stress=N` cycles the unit's five forms |

### Timing table (ART-DIRECTION §4 unchanged)

| Kit | Clip | Length | Key times | Event |
|-----|------|--------|-----------|-------|
| Siege Walker | AttackRanged | 0.62 s | brace 0–0.12, Release + recoil 0.18–0.21, recover 0.24–0.55 | Release 0.18 s, shell 0.12 s → 0.30 s |
| Siege Walker | Idle (engaged) | loop | crank 0.5–1.55 s after each shot | — |
| Siege Walker | Evolve | 0.9 s | raise 0–0.27, salute 0.34, settle 0.68–0.9 | — |
| Bone Brute | AttackMelee | 0.62 s | windup 0–0.20, strike 0.20–0.30, hold to 0.40, recover to 0.62 | Strike 0.30 s |
| Bone Brute | Evolve | 0.9 s | gather 0–0.36, roar 0.27–0.86 | — |
| Rootcaller | Cast | 0.6 s | raise 0–0.14, slam 0.14–0.22, channel to 0.45, pull out to 0.6 | Release 0.22 s, ridge 0.08 s → 0.30 s |
| Rootcaller | Evolve | 0.9 s | sink 0–0.27, bloom 0.32–0.9 | — |
| Strongholds | — | — | slew ≤ 0.15 s onto a new target | cannonball 0.30–0.34 s, bolts 0.22 s |

Tests: `kits_keep_the_simulation_event_times`, `unit_shots_land_on_the_simulation_hit`.

### Triangles per form (gallery log / `every_kit_form_builds_within_the_doubled_budget`)

| Kit | Base | A1 | A2 | B1 | B2 |
|-----|------|----|----|----|----|
| Siege Walker | 1 824 | 2 222 | 2 148 | 2 270 | 2 450 |
| Bone Brute | 3 376 | 3 688 | 3 904 | 3 860 | 3 986 |
| Rootcaller | 1 460 | 2 248 | 2 928 | 2 190 | 2 754 |

All within the P24 budget doubled by the owner (≤ 6 000).

### Deviations

- **Siege Walker identity.** P24 planned an ogre with a geared arbalest;
  the owner's request (2026-10-30) names a bazooka, so the kit carries a
  shoulder bombard. The display name stays "Ogre Archer" (an open owner
  decision, D-51).
- **Base branch.** P54 was cut from `origin/main` (6ef155b), not from the
  in-progress roster expansion (P42–P48B), which the owner asked to keep
  apart.
- **Steps committed by area**, not one commit per numbered step: the kit
  framework, the shots and the three kits only compile together.
- **Archer** keeps its own release layer and facing-directed arrows; it
  gains gait v2 only (its target-aware arrow is a follow-up kit).

### Verification (real output)

The phase changes only `crates/tbg-client` and docs
(`git diff --stat origin/main..HEAD -- crates/tbg-core crates/tbg-server
crates/tbg-balance` is empty), so the checks were scoped to the client and
run at low priority (`nice -n 19`, `-j 4`) at the owner's request: the
machine was saturated by another session's `tbg-balance gate` and a full
`cargo test --workspace` plus LTO release builds made it unusable.

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | ok |
| `cargo clippy -p tbg-client -- -D warnings` | ok (no warnings) |
| `cargo test -p tbg-client` | `60 passed; 0 failed` (new: IK reach/hinge, foot lock X/Z at three headings and both gaits, stride path, adapted legs on whole segments, shot endpoints/tangent, unit and cast shots land at 0.30 s, kit event times, kit budgets, bombard aim, off-center yaw, tower barrel aim on both sides) |
| `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown` | ok |
| `trunk build --release` | ok at `731d07d` (final code), 1 min 05 s on an idle machine (owner ran it); earlier ok at `ecacd71` (38 min under load) |
| Browser (headless Chrome, GPU, 1600×900) | gallery forms, `&range`, `&clip=walk`, `?gallery=strongholds` observed on the dev build |

Not run: `cargo test --workspace` (no core/server/balance change), a
Practice match and a two-client online match (no protocol or simulation
change).

### Performance

`?stress=30&measure&shadows=1`, release build at `731d07d`, 1920×1080,
headless Chrome on the Intel iGPU (ANGLE GL), idle machine (load ≈ 4–7),
16 s per run:

| Stress (30 units) | FPS | WebGL draws / frame (max) |
|-------------------|-----|---------------------------|
| Mixed roster (mostly shared-library units) | 60.0 | 413.5 (419) |
| Siege Walker, five forms cycled | 60.0 | 475.0 (481) |
| Bone Brute, five forms cycled | 60.0 | 497.9 (502) |
| Rootcaller, five forms cycled | 60.0 | 471.4 (476) |

All runs hold the 60 FPS vsync cap. Kit units cost 14–20 % more draws
than the mixed roster (glow layers, more jointed meshes), diagnostic only
(D-28). An earlier run during another session's lab gate (load 40–57)
gave 7–14 FPS and is discarded. Phone measurement is out of scope
(desktop-only focus).

### Evidence (`docs/media/`)

- `units/siegewalker/p54-forms.jpg`, `p54-range-aim.jpg` — five forms;
  every bombard aimed at its own dummy with the pennant up.
- `units/bonebrute/p54-forms.jpg`, `p54-range.jpg` — five forms; swings
  and a ground impact on the dummies.
- `units/rootcaller/p54-forms.jpg`, `p54-range-roots.jpg` — five forms;
  the root ridge running to the target, a spore ring.
- `p54/strongholds-aim.jpg` — every kind and form; Artillery barrels and
  the Sentinel ballista turned onto moving dummies, cannonballs in flight.
- `p54/gait-legacy-legs.jpg` — adapted Shieldbearer forms walking with
  bent knees and no torn leg meshes.

### Owner approval (2026-10-30)

The owner reviewed the release build on `localhost` and **approved
everything**: the three kits, gait v2 and the aiming Artillery (frozen,
D-51). Follow-up requested the same day and done here (D-52): animated
defenders and lobby-faithful dress for the four other towers, using the
lobby portraits (`assets/stronghold/<kind>.webp`) as reference, in the
same low-poly style:

| Tower | Defender and animation | Dress after the portrait |
|-------|------------------------|--------------------------|
| Classic | crossbowman walks round the crown to face the target, aims, recoils | steps, lintel, door studs, arrow slits, gold band |
| Bastion | gun port: shutter flies open, a falconet runs out, fires, runs back | arched gate on steps, conical corner turrets with pennants, crenellated keep with flag |
| Treasury | roof dormer: trapdoors open, a brass mortar lobs a purse of gold (gold impact) | eight-sided terracotta roof, gold dome and spire, finials and pennants, team banners, steps, windows, striped awnings |
| Command | archer on the platform turns, draws and looses | cloth panels on the rail, heraldic banner, arched door, masonry courses, steps |

Evidence: `docs/media/p54/towers-dressed-own.jpg`,
`towers-dressed-enemy.jpg` (mirrored side), `towers-defenders-zoom.jpg`.
Checks after the follow-up: fmt ok, client clippy ok, `cargo test -p
tbg-client` 60 passed (tower barrel aim on both sides covers the new
`orbit` solve), `trunk build --release` ok at `4292d0d` (1 min 13 s), and
browser review of both sides.

How it was done and how to make the next kit or defender:
[`docs/art/UNIT-ART-KITS.md`](../art/UNIT-ART-KITS.md).

### Still open

- The Siege Walker's display name ("Ogre Archer" vs. its bombard).
- Next kits: Archer (target-aware arrows), then the other units.

