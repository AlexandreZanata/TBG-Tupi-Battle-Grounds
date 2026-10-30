# Unit art kits — how the visual upgrade was built, and how to make the next one

P54 gave the Siege Walker, Bone Brute and Rootcaller their own code-built
"asset library": model, animations, aim, projectile and impacts. It also
made every unit walk without sliding and turned the Strongholds' shots into
animated weapons that aim at their targets. The owner approved the result on
2026-10-30 (D-48, D-49). This page records how it was done and is the guide
for the next characters.

Read with: `docs/phases/P54-unit-art-kits.md` (spec and report),
`docs/ART-DIRECTION.md` §4–§5, `docs/ARCHITECTURE.md` §7.

## 1. Goals and constraints

- **Each character has its own library in code:** five forms, its own
  attack animation, its own projectile and impact, aiming at the real
  target from the real weapon (the Siege Walker's bazooka tip).
- **Beautiful and readable at match distance**, in the game's low-poly,
  vertex-colored, toy-diorama style. No textures, no glTF, no skinning,
  no new dependencies.
- **Cheap to run.** Measured result: 30 kit units at 60 FPS (vsync cap) at
  1920×1080 with shadows on an Intel iGPU; 14–20 % more draw calls than the
  mixed roster.
- **Cosmetic only.** The simulation decides everything. The ART-DIRECTION
  §4 event times never move: melee Strike at 0.30 s, ranged Release at
  0.18 s, cast Release at 0.22 s after `AttackStart`.

## 2. How it was done (process)

1. **Plan first.** The phase spec (`P54-unit-art-kits.md`) fixed the file
   layout, the kit format, the timing table and the acceptance checks. It
   was committed before any code.
2. **Framework before characters:**
   - two-bone hinge IK (`art/ik.rs`);
   - gait v2 (`art/feet.rs`, `art/anim.rs`);
   - the kit registry (`art/kits/mod.rs`) and the aim math (`kits/aim.rs`);
   - the glow layer (`art/figure.rs`);
   - shots and impacts (`vfx/shots.rs`, `vfx/impacts.rs`).

   Each piece has a unit test for its contract: a planted foot does not
   drift, IK reaches its target, a shot starts at the muzzle and leaves
   along the aim tangent, and unit shots land at 0.30 s.
3. **Three kits** — model, motion and attack per character — then the
   tower aim (`arena/strongholds/aim.rs`).
4. **Visual review rounds** in a real browser. Each round took full-size
   screenshots of the gallery, `&range` and walk cycles, wrote down every
   problem, fixed them all in one batch and rebuilt (§7). Three rounds fixed:
   - the bombard's hands hiding the ogre's face;
   - the cleaver cutting through the ground;
   - the Reaper's hood covering the skull;
   - roots hidden under the target;
   - legacy legs tearing at the knee once knees bent.
5. **Owner review** on `localhost` with the gallery links (§7). The owner
   approved, then asked for defenders and lobby-faithful dress on the
   gunless towers (D-49), using the lobby portraits in
   `crates/tbg-client/assets/stronghold/<kind>.webp` as reference.
6. **Light verification only** (owner request): fmt, client clippy, client
   tests, WASM check, one release build, browser review. Core, server and
   balance were untouched, so their suites were not re-run.

## 3. Architecture (data flow)

```text
sim Event::AttackStart{from,to}
  └─ sync/events.rs ─► UnitVisual::engage(target)       (kit units skip the generic ball)
                     └► TowerOrders (Stronghold shots)
visuals/cues.rs   ─► Animator.aim_at = target point (chest height) each frame
art/anim.rs       ─► shared clip sample → kit pose layer (Moment: clip, t, form, aim, since_attack, variant, clock, walk)
                  ─► gait v2 legs while walking (distance-driven phase, world-locked feet)
PostUpdate        ─► transforms propagate
visuals/attacks.rs ─► on Release: shots::fire(kit shot, real muzzle → aim_at) + muzzle flash/backblast
                   └► on Strike: impacts::land(kit impact, at target within reach)
vfx/shots.rs      ─► parabola in fixed time, nose on tangent, trail → impact on arrival
arena/strongholds/aim.rs ─► orders → slew yaw/elevation → fire from muzzle (gunners.rs for defenders)
```

| Path | Responsibility |
|------|----------------|
| `art/kits/mod.rs` | `Kit { figure, pose, gait, attack, evolve, durations }`, `Attack::{Shot, Melee}`, `Moment`, registry `kit(UnitKind)` |
| `art/kits/<unit>/model.rs` | the five forms (`spec(tier, branch)`), plan, weapon points (muzzle, tip) |
| `art/kits/<unit>/motion.rs` | the pose layer: keyframe tables (`tools::play`) and procedural IK |
| `art/kits/<unit>/mod.rs` | the `KIT` constant: which shot/impact/muzzle per form |
| `art/kits/tools.rs` | `set/add/shift`, `window/hold/snap`, `reach` (arm IK + grip), `plant` (legs keep feet down), `stance`, `fk`, `face` |
| `art/kits/aim.rs` | `flight`/`tangent` (the shot's parabola), `line_yaw` (an off-center gun lines up), `pitch` |
| `art/kits/gait.rs` | stride, lift, bob, sink, stomp (`LIGHT`, `HEAVY`) |
| `vfx/shots.rs`, `vfx/impacts.rs` | `ShotKind` look table, trails; `Impact`, `Muzzle`, `Burst` recipes |
| `visuals/attacks.rs` | Release/Strike → shot or impact from the real joint transforms; footfall dust |
| `arena/strongholds/{aim,gunners,motion}.rs` | turret solve, defenders, piece motions (`Kick`, `Swivel`, `Aim`, `Mounted`, `Hatch`) |

## 4. Conventions

- **Rig v2:** 17 body joints plus springs (`Back`, `HeadGear`, `Tabard`)
  and sockets (`WeaponR` on `HandR`, `OffhandL` on `HandL`, `Back`).
  - Figures face +Z. The left side is +X.
  - Rest offsets come from `rig::rest`. Parts are given in the joint's own
    space.
- **A weapon held in the hands** goes on `Prop(Socket::WeaponR)`.
  - Pose it with `tools::reach(.., Some(grip))`, so the hand frame *is* the
    weapon frame and the other hand reaches a grip point on it.
  - This lets the weapon pitch, recoil and be cranked independently of the
    chest (Siege Walker, Reaper scythe).
- **Poses** are Euler XYZ degrees per joint, plus an offset.
  - `Rx` positive tilts +Z down. Arm up is about `x = -180`.
  - Keyframe tables use normalized time over the kit's own clip length.
    Put the event's moment at `event_seconds / length` (0.30 / 0.62 = 0.484).
- **Feet:** any pose that moves the hips calls `tools::plant(..)` so the
  soles stay on the ground.
- **Forms:** `Form::of(tier, branch)` → `a`, `b`, `t2`. Every tier changes
  three things (ART-DIRECTION §5):
  - scale: the kit plan uses `×(1 + 0.08 × tier)`;
  - a large surface;
  - a new silhouette piece.

  Tier 2 adds a glowing signature (`.glow()`).
- **Team color:** paint team pieces `ALLY`. The library builds the enemy
  copy by recoloring them to `ENEMY`. Give every kit at least one team
  piece readable from the 55° match camera: a pennant, armband or sash.
- **Budgets:** 6 000 triangles per form at most (P24's 3 000, doubled by
  the owner). The tests print the counts. Current forms:
  - Siege Walker: 1.8–2.5 k;
  - Bone Brute: 3.4–4.0 k;
  - Rootcaller: 1.5–2.9 k.
- **Timing:** kits may lengthen a clip's *recovery* (`durations`, e.g.
  0.62 s for Large attacks); they never move an event.
  - A unit shot flies `UNIT_FLIGHT` = 0.12 s (Release 0.18 s → 0.30 s).
  - A cast's ground ridge runs `CAST_FLIGHT` = 0.08 s (0.22 s → 0.30 s).

## 5. How to add the next kit (checklist)

1. Create `art/kits/<unit>/{mod,model,motion}.rs`. Copy the closest
   existing kit:
   - ranged: Siege Walker;
   - melee: Bone Brute;
   - cast: Rootcaller.
2. In `model.rs`:
   - write `plan(tier)`, `spec(tier, branch)`, and the weapon points
     (`muzzle`/`tip` in socket space);
   - keep each builder function to one body area (legs, chest, head, arms,
     weapon).
3. In `motion.rs`, write `pose(&Moment, &mut Pose)`:
   - match on `Idle`, `Walk`, the attack clip, `Spawn` and `Evolve`;
   - turn toward `m.aim` with `tools::face`, or with `line_yaw` for an
     off-center weapon;
   - raise the weapon to `aim::pitch(aim::tangent(muzzle, target, shots::arc(kind), 0.0))`;
   - add an engaged idle loop with `m.since_attack` (reload, crank, flex).
4. In `mod.rs`, write the `KIT` constant: gait, attack (shot or melee per
   form), muzzle flash, evolution burst, and longer recoveries if needed.
5. Register the kit in `kits::kit()` (a `static` plus a match arm).
6. Add a shot, impact or muzzle variant only when no existing one fits.
   Effects reuse the shared meshes and materials in `VfxAssets`; never
   create a material per entity.
7. Extend `kits/tests.rs`:
   - the kit is in `KITS` (budget, event times);
   - for a ranged kit, the weapon points along its shot, like
     `the_bombard_points_along_its_shells_flight`.
8. Review (§7):
   - `?gallery=<unit>`, `&range`, `&range&evolve`, `&clip=<attack>&slow=0.25`;
   - `&clip=walk`, `&stress=30`;
   - fix everything in one batch, then rebuild.

## 6. How to add a tower defender

In `arena/strongholds/gunners.rs`:

- Add `PartDef`s built facing +X with the origin at the pivot.
- Place them with `.aimed()` so they mirror onto the enemy tower, and give
  them the right motion:
  - `Swivel`: a figure that walks round the tower axis;
  - `Mounted(kick)`: a weapon carried by that figure;
  - `Kick(kick)`: a fixed gun. A negative kick runs it out of a port;
  - `Hatch(angle)`: shutters and trapdoors, which swing open about their
    local X axis.
- Return a `Turret` from `gunners::turret`:
  - `orbit: true` for carried weapons;
  - `arc` limits a port's yaw;
  - gate it on the damage level that removes its pieces.

Leave the Artillery alone (approved).

## 7. Review and verification recipes

- **Build (light):** run in `crates/tbg-client`, reusing the main checkout's
  compiled dependencies:

  ```bash
  CARGO_TARGET_DIR=<main>/target CARGO_BUILD_JOBS=4 nice -n 19 trunk build --dist <dir>
  ```

  - Use `--release` for the final check. It takes about 1 min on an idle
    machine and over 30 min when another job saturates the CPU.
  - Serve the result with `python3 -m http.server <port>`.
- **Screenshots at full size:** headless Chrome with
  `--headless=new --use-angle=gl-egl` keeps the GPU. SwiftShader is too
  slow, and Vulkan has no WebGL2 in headless. Take 4–8 frames 0.1 s apart
  for attacks, and stitch them into one sheet to review timing.
- **Performance:** `?stress=30&measure&shadows=1`, plus `&gallery=<unit>`
  for one unit. `diagnostics/measure.js` reports FPS and WebGL draws per
  frame. Measure only on an idle machine.
- **Checks:**
  - `cargo fmt --all -- --check`
  - `cargo clippy -p tbg-client -- -D warnings`
  - `cargo test -p tbg-client`
  - `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`

  Run the workspace suites only when core, server or balance change.

## 8. Lessons

- **Watch the face.** A weapon carried at the shoulder put the forearms in
  front of the face. Carry the grip at chest height and let the barrel rest
  on the shoulder.
- **Two-bone knees tear one-piece legs.** Adapted (legacy) legs must ride
  whole thigh, shin and foot segments (`adapter::segment`), and the base
  body needs overlapping thigh and shin capsules.
- **Top-down readability:**
  - effects at a target's feet hide under the target, so make them larger
    and lighter-colored than the ground;
  - a hood or cloak must sit behind the face;
  - team color needs a piece visible from above.
- **Aim from the line, not the pivot.** An off-center gun misses by its
  offset unless the body turns by `line_yaw`. A defender on a tower rim
  aims from the tower axis.
- **Order systems by schedule.** Animation runs in `PostUpdate` before
  transform propagation, after all movement. Releases read joint
  `GlobalTransform`s after propagation.
- **Respect the machine.** Batch fixes per rebuild, run builds with `nice`
  and limited jobs, and never run two heavy builds in the same target
  directory.
