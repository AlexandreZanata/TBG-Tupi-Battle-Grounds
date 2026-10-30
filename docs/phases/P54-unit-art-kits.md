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
ARCHITECTURE §7 and DECISIONS (D-48). Fill in the report.

## Acceptance checklist

- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`,
      `cargo test --workspace`, WASM check, `trunk build --release` pass
- [ ] Timing contract unchanged (events at 0.30 / 0.18 / 0.22 s)
- [ ] Planted feet do not slide (test) for every unit; no walk/idle flicker
- [ ] Siege Walker muzzle points at its target; shells leave the muzzle
- [ ] Artillery barrel and ballista track their target; own projectiles
- [ ] Five forms per kit with the §5 Evolution reads; triangle counts logged
- [ ] Desktop screenshots in `docs/media/units/<unit>/` (≤ 300 KB each)
- [ ] 30-unit stress FPS recorded per kit

## Phase report

_Filled in at the end._
