# A-1 — Professional visual polish: troops, animation, targeted combat, Strongholds

**Status:** step 0 passed, step 1 audit complete; steps 2–14 pending (spec written 2026-10-29 against HEAD `f15432f` on
`phase/p40-arcana-trees`; `main` = `077abe7`). Writing this spec changes no
rule and enables nothing.
**Depends on:** nothing competitive. Reads (never edits) the P16-F3/F4
Stronghold contract in GAME-DESIGN §10.4–10.5.
**Branch:** `phase/a1-visual-polish`, created from `main` (step 0).
**Difficulty:** high (art direction, procedural animation, two UIs of
evidence). Steps are independently testable; one commit per step.
**Scope:** `tbg-client` only, desktop browsers only (AGENTS.md platform
focus). Zero lines may change in `tbg-core`, `tbg-server`, `tbg-balance`.

## Goal

The game looks like one hand-made miniature diorama:

1. Every troop has a distinct, readable silhouette by class and Faction,
   proportioned and painted like the arena.
2. Idle, walk, attack, hit, death, spawn and Evolution read as alive and
   match the existing simulation timing.
3. **Every attack visibly goes at its authoritative target**: archers aim,
   arrows leave the bow along the aim line, melee weapons swing toward the
   enemy, Strongholds turn their weapon before they fire.
4. The four Stronghold kinds (Treasury, Redoubt, Ballista, Banner) are
   recognizable by silhouette, grow visibly at Tier 1 / Tier 2 and by branch
   (Walls / Arms), and shoot in the right direction.
5. None of it costs frame rate or changes who wins.

## Invariants (absolute, checked in step 13)

| # | Invariant | Proof |
|---|-----------|-------|
| I1 | No gameplay value, rule, event, message or protocol field changes | `git diff main --stat -- crates/tbg-core crates/tbg-server crates/tbg-balance Cargo.lock` is **empty**; `PROTOCOL_VERSION` unchanged |
| I2 | Visuals never feed back into the simulation | the client never sends anything new; no new `ClientMsg` |
| I3 | Same seed + same intentions → same outcome | `cargo test --workspace` including replay/golden/contract tests passes **unmodified** (no golden updated) |
| I4 | The target of an attack is the one in `Event::AttackStart { from, to }`, never "nearest unit" | unit test + grep: no proximity search in `art/aim*` |
| I5 | Logical positions are untouched; visuals may only rotate or add small *cosmetic* offsets under the figure root | review of every `Transform` write |
| I6 | One mesh and one material per form, shared by all instances | step 13 asset-count check |

If a step seems to need a gameplay or event change, **stop**, write
`> blocked:` in the report, and ask the owner (O-A1-3). Expected need: none
(see Audit finding F1).

## Audit findings

Verified at planning time; step 1 re-verifies each one and records it with
`file:line` in the report.

| ID | Finding | Evidence |
|----|---------|----------|
| F1 | The authoritative target already travels with every attack: `Event::AttackStart { from: Target, to: Target }` (units *and* Strongholds). `Event::Hit { to, amount }` has **no attacker**; damage lands `WINDUP = 6` ticks (300 ms) after `AttackStart` for units, **in the same tick** for the Stronghold (`combat::stronghold_fire`, no windup) | `tbg-core/src/state.rs:132`, `combat.rs:134`, `content.rs:77` |
| F2 | The simulation lane is **1-D** (`UnitView.x` only). "Above / below / diagonal" exists only because the client spreads units over three presentation lanes: `lane_z(id) = [-0.8, 0, 0.8][id % 3]` | `tbg-client/src/sync.rs:95` |
| F3 | **Root cause of misaimed attacks:** the figure yaw is a constant ±65° chosen at spawn (`visuals::yaw`) and is never changed afterwards (`update.rs` only re-applies the same value). Nothing reads the target at `AttackStart` except to pick a projectile start/end | `visuals.rs:94,153`, `visuals/update.rs:31`, `sync/events.rs:139` |
| F4 | Melee reach is tiny compared with the lane spread: Shieldbearer range is 800 milli-tiles while two units on lanes −0.8 and +0.8 are 1.6 tiles apart sideways. Weapons therefore "hit the air" even if aimed; step 1 must tabulate real XZ distances at `Hit` per melee kind | `content.rs:197`, F2 |
| F5 | Archer arrows (`art/arrows.rs`) fly along the bow socket's rotation (fixed yaw) for a fixed `range / 0.12`, ignoring the target (`ponytail:` comment says so). `events.rs::attack` additionally calls the generic `visuals::spawn_projectile` for every Ranged archetype — likely a **second projectile** for native archers (verify) | `art/arrows.rs:34`, `sync/events.rs:166-172` |
| F6 | Only the Archer is a native rig-v2 figure (`FigureForm.native`). The other seven shop units are gray **blockouts** (`art/roster/blockout.rs`, "final art in P22–P29"); `Nester`, `Rampart`, `Beetling` use adapted legacy bodies. Foot-locking, spring secondaries and bow IK only run for native forms | `art/mod.rs:110-180`, `art/roster/*` |
| F7 | One shared clip set: `Idle, Walk, AttackMelee, AttackRanged, Cast, Hit, Death, Spawn, Evolve, Victory` for every unit. Hit is a flash + clip with no direction. Death lingers `CORPSE_SECS = 0.9` | `art/clips.rs`, `visuals/cues.rs` |
| F8 | **Stronghold kinds do not exist in code at HEAD** (no `StrongholdKind`, no view field, no protocol message, no tier). One code-built tower with three HP-driven pieces (base / middle / crown hide at ≤ 66 % and ≤ 33 %). The HUD and `damage_states` divide by the constant `STRONGHOLD_HP` | `arena/towers.rs`, grep of `StrongholdKind` = no hits |
| F9 | The kind names, stats, passives and the shared Walls/Arms tree are specified in `docs/GAME-DESIGN.md` §10.4–10.5 (Treasury, Redoubt, Ballista, Banner; Fortification T1 6 Energy, T2 10 Energy) and `docs/phases/P16F3`/`P16F4` — all **planned**. A competing, unmerged plan (`phase/p17a-strategic-strongholds`: Bastion/Artillery/Command, kind-exclusive trees, and a clashing "D-41") exists | `docs/GAME-DESIGN.md:513`, branch `phase/p17a-strategic-strongholds` |
| F10 | Stronghold shots: `AttackStart{from: Stronghold(seat), to: Unit}`; the shot has no visual weapon, only a generic projectile from `(world_x, 1.2, 0)` | `sync/events.rs:164` |
| F11 | Harness already on `main`: `?gallery=…` (with `silhouette`, `portrait`, `frames`), `?stress=N`, FPS/frame-time overlay (`overlay.rs`), `autoplay.rs`, `diagnostics/measure.js`. `?measure` fluidity diagnostics live only on unmerged `phase/p30c-desktop-fluidity` | `gallery/settings.rs`, `stress.rs`, `overlay.rs` |

## Coordination (read before step 0)

| Parallel work | Overlap | Rule |
|---------------|---------|------|
| Working tree on `phase/p40-arcana-trees` has uncommitted `tbg-balance` edits | Would pollute I1 | Do **not** work in that checkout. `git worktree add ../tbg-a1 -b phase/a1-visual-polish main` |
| `phase/p30c-desktop-fluidity` (view interpolation on the server clock, `sync.rs`) | Same files as aim wiring (`sync.rs`, `sync/events.rs`) | If it merges first, rebase and use its interpolation; do not re-implement. Aim reads each unit's **rendered** `Transform`, so it is independent of how interpolation is computed |
| `phase/p17a-strategic-strongholds`, P16-F3/F4 | Kind/tier data and names | A-1 builds the **look layer only** (step 10) behind `StrongholdLook`; the live mapping is one small function, see O-A1-1 |
| P16 (class animation), P22–P29 (per-unit), P30 (fluidity), P31 | Same goals | See O-A1-2: default is that A-1 supersedes P16 and P22–P29 for visuals; their files stay as reference |
| Stronghold tier/kind in `MatchView` (P16-F3) | A-1 needs `kind`, `tier`, `branch`, `max HP` per seat for the live wiring | Not added here (I1). Until it lands the live game renders the current tower as `Classic` |

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2–§5, §6, §8, §9, §10
- `docs/GAME-DESIGN.md` §3 (glossary), §10.4–10.5 (read only)
- `docs/ARCHITECTURE.md` §6–§7
- Core (read only): `state.rs` (`Event`, `Target`, `stronghold_x`), `combat.rs`
  (`stronghold_fire`), `content.rs` (`WINDUP`, `UNITS`, ranges)
- Client: `sync.rs`, `sync/events.rs`, `visuals.rs`, `visuals/{cues,update}.rs`,
  `art/{anim,clips,rig,feet,springs,arrows,bow,archer_aim,figure,mod}.rs`,
  `art/roster/**`, `arena/towers.rs`, `arena/mod.rs`, `vfx.rs`, `vfx/effects.rs`,
  `gallery/**`, `stress.rs`, `overlay.rs`
- Bevy 0.20 examples on disk (`~/.cargo/registry/src/index.crates.io-*/bevy-0.20.0/examples/`):
  `transforms/`, `animation/easing_functions.rs`, `3d/` (never copy older APIs from memory)

## Do NOT

- Change any simulation, server, balance or protocol code (I1).
- Add a dependency, glTF/skinning, Bevy's animation feature, physics, or full
  IK. (The existing two-bone `archer_aim.rs` stays as is; do not generalize it.)
- Search for "the nearest enemy" to decide an aim (I4).
- Move a unit's logical position, change `lane_z`, hitboxes, range, cooldowns.
  (Optional visual lane convergence is **gated by O-A1-5**.)
- Build a generic building generator, a second Stronghold gameplay system,
  kind selection UI, or any message for it (F8/F9).
- Add per-unit timers, per-frame allocations, new materials per instance, or
  particle spectacles.
- Touch `archive/`, push, merge, or force anything.

## Technical design (the contract for steps 4–12)

### D1. Aim model (client only, `art/aim.rs`, no Bevy types in the math)

```
AttackStart{from,to} ──► Aim { target: AimTarget, started: f64, hold: f32 }   (on the attacker root)
AimTarget = Unit(UnitId) | Stronghold(Seat)          (copied from the event, nothing else)
```

- **Source of truth:** `Aim` is created only by `Event::AttackStart`. While
  it is live, the target's world point is read from the target's **rendered**
  entity (`UnitEntities` O(1) lookup) or the known Stronghold point. If the
  target entity is gone (death, despawn), the **last known point is frozen**
  for the rest of the clip, then the aim expires.
- **Yaw:** `desired = atan2` of the XZ vector attacker→target in the
  attacker's parent frame (seat mirroring is already in world space because
  `sync::world_x` mirrors x for seat B — the math must work on world
  positions, never on seat-relative ones). Pure function
  `facing_yaw(from: Vec3, to: Vec3) -> Option<f32>` (None when closer than
  ε, keep current yaw).
- **Turn:** rotate the **figure child** (never the interpolated root) with a
  shortest-arc, rate-limited, critically-damped step
  (`turn_toward(current, desired, dt, max_rate)`), `max_rate` ≈ 540°/s
  *start* (tune at step 5). No wrap flips across ±π, no jitter when the
  target moves a few millimetres (dead-zone ≈ 2°).
- **Phases of a unit:** `Moving/Idle` → base yaw (±65° camera-turned, as
  today). `Aim` active → look-at yaw. After the attack clip ends the aim
  holds `AIM_HOLD` (≈ 0.5 s *start*, so a unit that attacks every second
  never snaps back and forth), then blends to base yaw over ≈ 0.25 s. A new
  `AttackStart` simply replaces the target (retarget = continue turning from
  the *current* yaw, never reset).
- **Cheapest rung first (YAGNI):** whole-figure yaw. Add a limited torso
  twist (±35°) for Ranged/Control only if the feet visibly pivot during the
  draw at step 5; add a weapon-only rotation only for the Stronghold turret
  (D4). Record the choice in the report.
- **Cost:** a marker component `Aiming` is inserted/removed on the transitions
  above, so the system iterates only aiming units (not all units, not
  quadratic).

### D2. Ranged and spell projectiles

- Origin: the weapon socket's world point at the clip's `Release` event
  (Archer bow string, Hexer staff tip, Bloomkeeper staff, Ogre arbalest
  muzzle), **not** `(unit centre + 0.6)`.
- Direction/endpoint: from the origin to the target's chest point **read at
  `Release`** (`+0.6 × target scale` above its feet; Stronghold: its hit point),
  re-read each frame only until impact so a moving target is still hit.
- Flight time = time remaining until the `Hit` lands (`WINDUP − Release`
  = 120 ms for archers, 80 ms for casts), clamped to [60, 250] ms; the
  projectile orientation follows its velocity (pitch included).
- Replace `FlyingArrow`'s "facing-directed range" and the duplicate generic
  `spawn_projectile` for native casters by this one path. Pool/reuse
  projectile entities; no per-projectile material.
- The arrow must never appear to leave sideways: assert in a unit test that
  `angle(arrow_dir, aim_dir) < 1°` at spawn and that the bow's forward axis
  at `Release` is within the draw tolerance of `aim_dir`.

### D3. Melee: directed strike, impact pairing, reaction

- **Strike profile table** (data, not code branches): per weapon class
  `{ windup_frac, arc_deg, torso_twist_deg, lunge, hit_stop_ms, impact }`.
  Classes: `Slash` (sword), `Smash` (club/mace/hammer), `Chop` (axe/cleaver),
  `Bash` (shield), `Mech` (saw/gear: spin-up and shove, no human arc).
  Assigned per unit form (so a bigger Tier-2 weapon gets a wider arc and
  longer windup, scaled by form `scale`). The `Strike` marker stays at
  **0.30 s** after `AttackStart` (ART-DIRECTION §4); every clip finishes before
  the unit's attack interval.
- Because the figure already faces the target (D1), the authored forward arc
  now points at it. Add a **cosmetic lunge** (body shifts ≤ 0.25 tile toward
  the target along the aim line, visual only, springs back) so the contact
  point reaches the target's near side. Never stretch limbs, never teleport.
- **If the XZ distance still exceeds weapon reach** (F4): play the directed
  strike anyway and put the **impact effect on the target** (spark + target
  recoil). Optional presentation-only lane convergence is Option B, behind
  O-A1-5; do not implement it unless step 7 proves Option A leaves visible
  misses.
- **Impact pairing:** `Hit` has no attacker. Keep a small ring per target:
  `AttackStart{from,to}` pushes `(from, to, due)`; the next `Hit{to}` for
  that target pops the **oldest** matching entry (FIFO) and uses its attacker
  for the **directional** hit reaction (target recoils ≤ 0.08 tile *away from
  the attacker*, additive, 120 ms) and the impact spark at the contact side.
  No match (status damage, splash, Stronghold fire) → neutral reaction. Stale
  entries expire after 1 s. Unit-test the FIFO and the expiry.
- Stronghold target: the aim point is the Stronghold hit point on the side
  facing the attacker (never its centre inside the mesh).

### D4. Stronghold look layer (art only)

```
StrongholdLook { kind: Classic|Treasury|Redoubt|Ballista|Banner, tier: 0..=2, branch: Walls|Arms }
```

- `arena/strongholds/`: `mod.rs` (spawn, per-seat component, transitions),
  `common.rs` (shared stone base ring, team banner slots, socket struct),
  `treasury.rs`, `redoubt.rs`, `ballista.rs`, `banner.rs` (T0 body + turret
  of each kind), `tier.rs` (add-on pieces by tier/branch, placed by a small
  per-kind transform table), `turret.rs` (pivot + muzzle + recoil),
  `damage.rs` (existing three HP states, thresholds unchanged).
- Meshes built once at startup, merged per piece group with the existing
  `arena::painted`, one material set shared by both seats; own/enemy differ
  only in banner/team slots. `Classic` = today's tower (the live fallback).
- **Identity** (silhouette first; palette from ART-DIRECTION §3 plus
  `GOLD`, `STONE`, `WOOD`, `ROCK`, `ALLY`, `ENEMY`):
  Treasury — squat vault keep, brass-banded gate, coin-chest roof, gilded
  crown, ballast of sacks; Redoubt — low, very wide, thick sloped walls,
  four corner buttresses, mantlet shield on the turret; Ballista — slim
  timber-and-iron platform with an oversized crossbow on a rotating
  mount, winch and bolt rack; Banner — tall narrow tower with a raised
  platform, two large pennons and a standard that flutters (cheap
  sine on a few vertices-free quads: rotate quads, no cloth sim).
- **Tier/branch add-ons** (shared pieces, kind-specific placement):
  T1 Walls — buttress ring and thicker plinth; T1 Arms — reinforced limbs,
  second bolt rack; T2 Walls — curtain-wall segments with a gate and a crest
  (the "slows hits" capability gets a faint cold tint on bolts); T2 Arms —
  double-limb war engine and a larger muzzle (the "splash" capability reuses
  `vfx::splash`). Each tier adds ≥ 3 visible changes (scale, a large-surface
  material change, a new silhouette piece), mirroring ART-DIRECTION §5.
- **Budget:** height ≤ 2.9 and footprint radius ≤ 1.15 world units (today
  ≈ 2.3 and 1.0) so a unit on lane z = ±0.8 stays visible from the match
  camera; triangles per kind/tier are measured and reported (no fixed
  maximum, D-30).
- **Transition:** when the authoritative `StrongholdLook` of a seat changes
  between two views, new pieces grow `0.01 → 1` with ease-out-back,
  staggered over ≤ 0.6 s, plus one shared dust/spark pooled burst; old pieces
  are replaced **under** the growth so there is never a gap. The first look
  after (re)join or reconnect is applied instantly with no animation.
- **Turret (D4a):** a pivot entity (yaw about Y, small pitch) carrying the
  weapon and a `Muzzle` child. On `AttackStart{from: Stronghold(seat), to}`:
  slew to the target's point in ≤ 100 ms (≈ 900°/s), fire (recoil 60–90 ms),
  projectile from the `Muzzle` along muzzle→target (D2). Because the
  simulation applies Stronghold damage in the same tick, the target's
  **visual** hit reaction is deferred by the projectile flight (≤ 150 ms) so
  the shot is seen to connect; HP bars are never deferred. The base never
  rotates. Between shots the turret rests at the last aim (or lane centre
  before the first one). No pre-aiming, no new windup (I1).
- **Damage and destruction:** keep three HP states (the 4-state target of
  ART-DIRECTION §6 stays archived); add hit shake of the piece group and a
  collapse sequence (pieces drop and fade ≤ 1.2 s, then despawn). Max HP for
  the fraction comes from the view when a field exists, else the constant
  (F8).

### D5. Unit art standard (written into `docs/ART-DIRECTION.md` §11)

- One proportion table per class on the rig-v2 anatomy (≈ 1.05 tall, head
  0.26 of height): Tank — shoulder:hip 1.4, limb thickness ×1.25, prop
  ≥ 0.7 of body height, wide stance; DPS — shoulder:hip 1.15, big
  asymmetric weapon, forward lean; Ranged — slim, weapon taller than the
  body, off-hand free; Support — rounded, light colours, glowing staff top;
  Control — tall hood/crown, staff or symbol with a glow accent. Large body
  plan = ×1.5 as in §5.
- Faction rules (colours exactly from §3): Humans steel/oak/gold and linen;
  Undead bone/rusted iron/soul-fire and violet cloth; Machines iron/brass/forge
  glow, **mechanical joints, no human limb arcs**; Plants bark/moss/bloom/
  sap, organic curves.
- Evolution: scale +8 % / +16 %, one large-surface colour/material change,
  one new silhouette piece, Tier 2 carries the branch artifact with accent
  glow (§5 already specifies this; A-1 implements it for every unit).
- Detail density: large structural shapes only (nothing readable below ≈ 6 px
  at the match camera); smooth-shaded large forms, flat small details.
- Shared with the arena: one unit material (`FigureLibrary.material`), same
  light and fog, ring/pip/HP-bar system untouched.

## Steps (one commit each: `feat(a1): <step title>`)

### 0. Preflight
Worktree and branch from `main` (see Coordination). Record `HEAD`,
`git status`, `rustc`, `trunk`, browser versions, GPU (`WebGL2` renderer
string). **Check:** `cargo build --workspace` and
`cargo check -p tbg-client --target wasm32-unknown-unknown` on the untouched
branch.

### 1. Audit report (no code)
Fill the report's audit tables with `file:line`: F1–F11 confirmed or
corrected; per unit form (11 kinds × 3 tiers/branches in `UNITS`): class,
Faction, current body (native / blockout / legacy), weapon and weapon class,
silhouette verdict, class/Faction identity verdict, animation issues,
Evolution issues. **Measure** in the live client (a temporary debug log
in a vs-bot match via `autoplay`, removed before commit): the XZ distance attacker→target at each `Hit` for every melee kind (F4),
whether native archers spawn two projectiles (F5), per-unit entity count and
triangles per form. **Check:** the report tables are complete; the root-cause
screenshot of a misaimed diagonal attack is committed to `docs/media/a1/`.

### 2. Baselines before any change
On the untouched code: (a) **aim lab before sheets** (the lab is added in
step 5, so first commit it as a standalone `?gallery=aim` against the old
behaviour — it only spawns units and issues attack cues); (b) performance
baseline for 10 / 40 / 100 / 200 units **per side** with `?stress=2N` and one
real vs-bot match with `autoplay`, 60 s each, 3 runs, median: FPS, frame
time p50/p95/p99 (from `overlay.rs` diagnostics or a `PerformanceObserver`
in `diagnostics/measure.js`), JS heap and `WebAssembly.Memory` bytes after
60 s, Bevy entity count, `Assets<Mesh>`/`Assets<StandardMaterial>` lengths,
CPU (Chrome `Performance.getMetrics` / `chrome://tracing` task duration);
draw calls only if a tool is available (Spector.js) — otherwise write "not
measured". State whether the GPU is real or software; **never mix the two in
one comparison**. **Check:** `docs/media/a1/baseline/` + a table in the
report.

### 3. Visual standard
Write ART-DIRECTION §11 (D5) and its unit proportion/faction/evolution
tables; add the "Tupi look" checklist (palette, scale, light, pivots, floor
height, rim). Docs only.

### 4. Aim core (pure Rust + tests)
`art/aim.rs`: `facing_yaw`, `turn_toward`, `AimTarget`, `Aim` state machine
(`Active → Hold → Release`), dead-zone, expiry, frozen last point, and the
`ImpactRing` FIFO (D3). No Bevy types except `Vec3`/`f32`.
**Tests (native, assert the contract; cite I4):** target front / up / down /
diagonal ±45° / near (0.4) / far (6) / behind; mirrored seat B gives the
mirrored yaw; shortest arc across ±π; no flip when the target jitters
< 2°; retarget keeps current yaw; target removed keeps last point; hold and
release timings; FIFO pairing and 1 s expiry. **Check:** `cargo test -p
tbg-client aim`.

### 5. Aim wiring and Archer prototype
- `sync/events.rs`: on `AttackStart` from a unit, insert `Aim` (copy of
  `from/to`), push `ImpactRing`; remove the nearest-position projectile for
  native casters; send Stronghold targets to the same path.
- `visuals/cues.rs` / new `visuals/aim.rs`: the single system for `Aiming`
  units writes the figure child's yaw; base yaw otherwise.
- `art/arrows.rs`: D2 projectile (origin socket, endpoint at target chest,
  flight = time to `Hit`, oriented by velocity); delete the facing-directed
  path and its `ponytail:` note.
- `?gallery=aim` lab (stays as the permanent regression tool): attacker at
  the centre, targets on a ring of 8 angles × {near, far}, own side and
  mirrored enemy side, buttons/keys for retarget, kill-target-mid-draw, and
  "crowd of 12 attackers around 3 targets"; `&unit=<kind>&tier=&branch=`.
**Check:** unit tests green; contact sheets (0/25/50/75/100 %) of Archer vs
the 8 angles committed; arrow endpoints land within 0.15 tile of the target
chest in the lab (assert numerically in a test using the pure math).

### 6. Melee prototype
Shieldbearer (mace, `Smash`) first: strike profile table, lunge, directed
reaction via `ImpactRing`, impact spark at the contact side, hit-stop 50 ms
on the **target's animator only**. Second melee check on the Bone Brute
cleaver (`Chop`) to validate the table, not to finish its art.
**Check:** lab sheets for front / diagonal / above / below / moving target /
retarget at the first frames / target dies mid-swing / target despawned /
two attackers on one target / 12 melee in a clump / Stronghold target /
walk→attack→walk / seat-mirrored teams.

### 7. Prototype gate (reviewer checkpoint; step D of the brief)
Before/after sheets side by side; re-run the 40-per-side baseline; table
of the F4 reach distances with a verdict per kind: *Option A suffices* or
*propose Option B* (O-A1-5). **Check:** I1 diff empty; `cargo test
--workspace` green; frame time p95 not worse than +10 % against step 2;
no new duplicate projectile. Stop here and write the interim report if any
check fails twice.

### 8. Troop polish by Faction (one commit per Faction)
For each unit: build the native rig-v2 body from the §11 table, replace the
blockout/legacy form, keep the code id and the shop/inspect icon working;
do the three tiers and both branches per ART-DIRECTION §5 motifs (a weapon
change implies a new `StrikeProfile`, scaled by form `scale`).
1. Humans: Shieldbearer (kite shield, flanged mace; A tower shield + plate +
   crest, B spear + pauldrons + back banner), Archer (verify v2 forms keep
   working with D1/D2).
2. Undead: Bone Brute (hunched giant, cleaver, ribcage armour; A bone plates
   and skull shield, B twin cleavers / scythe), Hexer (hooded, skull staff;
   A crooked tall staff and crown, B censer and green mist).
3. Machines: Sawbot (saw-blade arm, `Mech` profile; A twin saws, vents; B
   riveted plates, tower helm), SiegeWalker/Ogre Archer (geared great-bow
   with a muzzle socket; A double-limb, B barbed / hammer-head bolts),
   Rampart (wall-like Tank; no human limbs).
4. Plants: Bloomkeeper (flowering staff, leaf cloak, petal aura), Rootcaller
   (small treant, root claws; Strangle/Overgrowth silhouettes), Nester and
   its Beetling summon (Beetling uses the Nester's tier/branch look,
   cheapest possible mesh).
**Per-unit check:** 64 px black silhouette is identifiable among all units
(`?gallery=…&silhouette`); base/T1/T2 are named correctly at match-camera
distance; triangles logged; one mesh + one material per form (I6).

### 9. Full animation pass
Parameter profiles, not new keyframe tracks per unit: idle breath amplitude
and phase offset, walk cadence = speed with foot-lock for every native form
(no sliding at normal speed), attack = `AttackStart → anticipation (≥ 25 %)
→ Strike at 0.30 s → follow-through → recovery` by profile, Ranged/Control/
Support aim at the target (`Cast` aims at the ally for heals; the effect
spawns at the staff socket), directional `Hit`, faction **death styles**
(Humans fall, Undead collapse into the ground, Machines spark and power
down, Plants wilt and sink) built from the existing `Death` clip + sink/fade
parameters, `Spawn` and `Evolve` that preserve the unit's identity and facing
(orientation never resets on an Evolution). Every attack clip finishes before
the unit's `interval`. Corpses and temporary effects are removed on a fixed
schedule. **Check:** timing unit tests (Strike/Release at their contract
times ± 1 frame, clip < interval) and a corpse/effect leak test (entity
count returns to the pre-battle value after a 60 s fight plus 3 s).

### 10. Stronghold models (D4, Tier 0)
`arena/strongholds/*` for the four kinds + `Classic`, per-seat spawn, team
banners, sockets, three HP states, hit shake, collapse. `?gallery=strongholds`
shows 4 kinds × 2 seats at the match camera and in silhouette. Practice mode
and online both keep rendering `Classic` through `look_for(view, seat)` until
the kind data exists (O-A1-1). **Check:** silhouettes of the four kinds are
distinct at 64 px; footprint/height budget respected; sheet with units on
lanes ±0.8 next to each tower proving visibility.

### 11. Tier and branch kit + transition
`tier.rs` add-ons and the growth transition (D4). `?gallery=strongholds`
shows every kind at T0, T1 Walls, T1 Arms, T2 Walls, T2 Arms (a key cycles
the tiers and plays the transition). **Check:** each step is distinguishable
from the previous at match-camera distance (three named changes per tier,
listed in the report); the transition has no frame with a missing piece
(frame-by-frame sheet); reconnect snapshot test: the first look is applied
with no animation.

### 12. Stronghold attack
Turret pivot, muzzle origin, recoil, deferred target reaction (D4a) for all
kinds and tiers (T1/T2 change the weapon geometry but not the socket
contract). **Check:** the aim lab runs with a Stronghold as the attacker
against the 8 angles × near/far for each kind; numerical test that
`angle(muzzle_forward at fire, muzzle→target) < 3°`; the projectile never
leaves from a point more than 0.2 from the muzzle socket.

### 13. Performance, regression and compatibility
- Re-run the step 2 matrix (same machine, same GPU type, same browser).
  Acceptance: p95 frame time ≤ baseline + 10 % at 40 and 100 per side, no
  new frame-time spike class; the 200-per-side run is reported even if
  worse (state the environment limit); memory ≤ baseline + 5 %.
- Asset reuse: `Assets<Mesh>` and `Assets<StandardMaterial>` length constant
  while spawning 200 more units of existing forms; entity count per unit and
  per Stronghold reported.
- I1: `git diff main --stat -- crates/tbg-core crates/tbg-server
  crates/tbg-balance Cargo.lock` is empty. I3: full `cargo test --workspace`
  passes with no golden/contract edit.
- Browsers: Chromium desktop (primary) and Firefox desktop: gallery routes,
  one vs-bot match to the end, and two clients finishing an online match on
  the local server including one reconnect (AGENTS.md verification).
- Commands (quote real output): `cargo fmt --all -- --check`, `cargo clippy
  --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo
  check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`,
  `trunk build --release` in `crates/tbg-client/` and WASM size (brotli)
  against the ≤ 6 MB ceiling.

### 14. Documentation and report
ART-DIRECTION §11 final, a DECISIONS entry (next free ID; "aim from
`AttackStart`, look layer for Strongholds, A-1 supersedes P16/P22–P29 if the
owner chooses"), ROADMAP row, a short GAME-DESIGN note only if a canon
statement changed (none expected). Fill the phase report; do not claim any
item not run.

## Mandatory case matrix (every cell has a named check)

| Case | Archer | Melee | Stronghold turret | How verified |
|------|:------:|:-----:|:-----------------:|--------------|
| Target straight ahead | ✔ | ✔ | ✔ | unit test + lab sheet |
| Above / below lane, diagonal ±45° | ✔ | ✔ | ✔ | unit test + lab sheet |
| Near / far | ✔ | ✔ | ✔ | unit test + lab sheet |
| Retarget (also at the first frames) | ✔ | ✔ | ✔ | unit test + lab |
| Target dies or is removed mid-animation | ✔ | ✔ | ✔ | unit test (frozen point) + lab |
| Several targets close together | ✔ | ✔ | ✔ | lab (12 attackers, 3 targets) |
| Blue and red side (seat A and B) | ✔ | ✔ | ✔ | unit test + sheet |
| Attack a Stronghold | ✔ | ✔ | — | lab + live match |
| Movement ↔ attack transitions | ✔ | ✔ | — | live match, sheet |
| Weapon classes sword/club/axe/shield/mech | — | ✔ | — | lab per class (see step 6, 8) |
| Tier 0/1/2 and branches | ✔ | ✔ | ✔ | gallery + lab with `tier`, `branch` |
| Projectile origin = weapon/muzzle | ✔ | — | ✔ | numeric tests (D2, step 12) |

## Owner decisions

| ID | Decision | Default if the owner does not answer |
|----|----------|--------------------------------------|
| O-A1-1 | Stronghold dependency: build only the look layer now and wire it when P16-F3/F4 (names **Treasury, Redoubt, Ballista, Banner**, shared Walls/Arms) lands. If the P17-A plan wins instead, only the mapping table and names change | Look layer now; live game shows `Classic` until the data exists |
| O-A1-2 | Does A-1 supersede P16 (class clips) and P22–P29 (per-unit phases) for visuals? | Yes; mark them "superseded by A-1" in the ROADMAP at step 14 |
| O-A1-3 | A change to `tbg-core` events or the protocol, **only** if a step proves it indispensable | No change (expected) |
| O-A1-4 | Look sign-off per Faction and for the four Strongholds from gallery sheets and one live match | Block merge until recorded |
| O-A1-5 | Option B: presentation-only lane convergence during melee engagement (client z only), if Option A leaves visible misses | Not implemented |

## Acceptance checklist

- [ ] I1–I6 hold; the empty-diff command output is in the report
- [ ] All `AGENTS.md` verification commands pass (quoted); `trunk build --release` passes
- [ ] Aim unit tests (D1–D3) pass; they cite the invariant they guard
- [ ] Lab sheets prove every cell of the case matrix; before/after pairs exist
- [ ] Every shop unit and the Beetling use a native rig-v2 form; silhouettes distinct at 64 px
- [ ] Tier 0/1/2 + both branches readable for every unit and each Stronghold kind
- [ ] Stronghold transition has no missing-piece frame; reconnect applies the look instantly
- [ ] Performance matrix before/after at 10/40/100/(200) per side, with environment stated
- [ ] No new per-instance mesh/material; no leaked entities after a battle
- [ ] Chromium and Firefox desktop checked; no phone or touch work done
- [ ] Owner decisions O-A1-1…5 recorded

## Risks and what to do

| Problem | Action |
|---------|--------|
| Yaw rotation makes feet pivot or slide while shooting | Torso twist for Ranged/Control only (D1), still no IK |
| Melee still misses at lane offsets (F4) | Directed strike + impact on target (Option A); evidence at step 7; Option B only with O-A1-5 |
| Hit has no attacker (F1) | FIFO `ImpactRing`; unmatched hits get the neutral reaction; never guess by proximity |
| Stronghold shot and damage share a tick | Fast slew + deferred *visual* reaction ≤ 150 ms; never add a windup |
| Interpolation work (P30-C) lands mid-phase | Aim reads the rendered `Transform`; rebase, do not duplicate |
| Stronghold tree changes (P17-A) | `StrongholdLook` + a mapping table isolate it |
| 200-per-side is unreachable on the dev machine | Report the real limit and the environment; do not invent numbers |
| Visual size hides the lane | Footprint/height budget (D4) and the lane-visibility sheet |

## Reviewer checklist

- `git diff main...phase/a1-visual-polish --stat` touches no file under `crates/tbg-core`, `crates/tbg-server`, `crates/tbg-balance`.
- No proximity search decides an aim; the target always comes from `AttackStart`.
- No attack clip is longer than its interval; Strike/Release contract times asserted.
- No per-frame allocation in the aim/animation systems; `Aiming` marker limits iteration.
- Meshes/materials per form shared; entity counts after battle return to normal.
- Strongholds are one look layer, not a second gameplay system; names match GAME-DESIGN §10.4.
- Report quotes real output and states what was **not** measured or run.

## Phase report

### Execution environment and step 0

- Date: 2026-10-29 (America/Cuiaba).
- Worktree: `/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/tbg-a1`.
- Branch: `phase/a1-visual-polish`, created from `main` at
  `077abe7e4cdd0ac06b8bddc64d849bdb69d053c0`.
- Initial `git status --short`: empty before copying this untracked spec
  from the owner's checkout. That checkout and its edits were not modified.
- `rustc 1.99.0 (b940084d7 2026-09-28)`; `trunk 0.22.1`.
- `Google Chrome 156.0.8078.73`; `Mozilla Firefox 158.0.1`.
- Chromium WebGL2: available. Renderer:
  `ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)`.
  **Software GPU in the initial headless session only.** Browser profile: `/tmp/tbg-a1-chrome`; local DevTools
  port 9331. Hardware and software results must not be combined.
- Subsequent desktop audit used hardware:
  `ANGLE (Intel, Mesa Intel(R) Graphics (RPL-P), OpenGL 4.6)`;
  profile `/tmp/tbg-a1-chrome-gpu`, port 9333, viewport 1280×800/DPR 1.
  No formal before/after performance comparison has been collected.
- `PROTOCOL_VERSION = 9` (`crates/tbg-core/src/protocol.rs:15`).

Untouched-source preflight, both exit 0:

```text
$ cargo build --workspace
   Compiling tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/tbg-a1/crates/tbg-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 05s

$ cargo check -p tbg-client --target wasm32-unknown-unknown
    Checking tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/tbg-a1/crates/tbg-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 46s

$ git diff main --stat -- crates/tbg-core crates/tbg-server crates/tbg-balance Cargo.lock
(no output)
```

### Step 1: Audit report (no code)

#### 1.1 Verified Findings F1–F11

| Finding | Status | Exact `file:line` | Verified Evidence / Correction |
|---|---|---|---|
| F1 | Confirmed | `crates/tbg-core/src/state.rs:83,85`, `crates/tbg-core/src/content.rs:48`, `crates/tbg-core/src/combat.rs:47–67,97–119` | Authoritative target is passed in `Event::AttackStart { from: Target, to: Target }` for both units and Strongholds. `Event::Hit { to, amount }` carries no attacker. Units deal damage after `WINDUP = 6` ticks (300 ms). Stronghold deals damage in the same tick (`combat::stronghold_fire`) with no windup. |
| F2 | Confirmed | `crates/tbg-client/src/sync.rs:90,95` | The simulation is strictly 1-D (`UnitView.x`). The 3 presentation lanes `lane_z(id) = [-0.8, 0.0, 0.8][(id.0 % 3) as usize]` are a client-only cosmetic spread. `sync::world_x` mirrors X coordinates for Seat B. |
| F3 | Confirmed | `crates/tbg-client/src/visuals.rs:94,153`, `crates/tbg-client/src/visuals/update.rs:31`, `crates/tbg-client/src/sync/events.rs:139,150` | Figure yaw is fixed at `+65.0°` (Seat A) or `-65.0°` (Seat B) at spawn. On Evolution, `update.rs` re-applies the exact same fixed yaw. On `AttackStart`, `events.rs` only triggers `VisualCue::AttackStart`, never orienting the figure toward the target entity. |
| F4 | Corrected & Measured | `crates/tbg-core/src/content.rs:169,183,197,230,239` | 1D base melee ranges: Shieldbearer 0.8, Sawbot 0.8, Rampart 0.8, BoneBrute 1.0, Beetling 0.6. Presentation lane separation is up to 1.6 in Z. Real 3D XZ distance at `Hit` reaches up to 1.887 (BoneBrute), 1.789 (Sawbot), 1.705 (Beetling), 1.000 (Rampart), 0.994 (Shieldbearer), while 1D DX remains strictly within range (max 1.0, 0.8, 0.6, 0.8, 0.79). |
| F5 | Confirmed & Measured | `crates/tbg-client/src/art/arrows.rs:33–48`, `crates/tbg-client/src/sync/events.rs:166–172` | Confirmed duplicate projectile for native Archer: on `Event::AttackStart`, `events.rs` immediately spawns a generic straight-line projectile via `visuals::spawn_projectile`. Then 0.18s later at `ClipEvent::Release`, `art/arrows.rs` spawns a native `FlyingArrow` oriented by fixed socket yaw (`velocity: rotation * Vec3::Z * range / 0.12`). Logged 53 native arrows and 341 generic projectiles during audit matches. |
| F6 | Corrected & Confirmed | `crates/tbg-client/src/art/mod.rs:109–150`, `crates/tbg-client/src/art/roster.rs:101–115`, `crates/tbg-client/src/art/roster/blockout.rs` | Planning audit was inaccurate: live non-native units are NOT gray blockouts; they are colored legacy bodies adapted to rig v2 (`adapter::build`). Only Archer 0/0, 1/0, and 2/0 use native rig v2 (`archer_v2(tier)`). Archer 1/1, 2/1 and all other 10 kinds use adapted legacy forms. Gray native blockouts exist solely in `FigureLibrary.blockouts` as separate review assets. Nester, Rampart, and Beetling recolor Bloomkeeper, Shieldbearer, and Rootcaller. |
| F7 | Confirmed | `crates/tbg-client/src/art/clips.rs:14–174`, `crates/tbg-client/src/visuals/cues.rs:34` | Shared clip set: `Idle`, `Walk`, `AttackMelee`, `AttackRanged`, `Cast`, `Hit`, `Death`, `Spawn`, `Evolve`, `Victory`. `STRIKE_TIME = 0.30s`, `RELEASE_TIME = 0.18s` (ranged) / `0.22s` (cast). `CORPSE_SECS = 0.9s`. `Hit` is a non-directional flash tint + clip. Foot-locking and spring secondary physics only run for native figures. |
| F8 | Confirmed | `crates/tbg-client/src/arena/towers.rs:33–78` | No `StrongholdKind`, tier, branch, or view field exists in codebase at HEAD. The existing tower is a single procedural mesh with base/middle/crown hiding at `<= 2/3` and `<= 1/3` HP against the constant `STRONGHOLD_HP`. |
| F9 | Confirmed | `docs/GAME-DESIGN.md:472–517` | Treasury, Redoubt, Ballista, Banner names and the shared Walls/Arms tree are planned in GAME-DESIGN §10.4–10.5 (lines 472–517 on main). The glossary is in §2 (not §3). A-1 implements only the visual presentation layer (`StrongholdLook`), keeping live gameplay mapped to `Classic`. |
| F10 | Confirmed | `crates/tbg-client/src/sync/events.rs:108–111`, `crates/tbg-client/src/arena/mod.rs:84–85` | Stronghold attack events originate from logical `(world_x, 1.2, 0.0)` at `x = 0` (Seat A) and `x = 16` (Seat B). The visible arena towers are placed at `x = -0.9` and `x = 16.9`. Turret muzzle and hit sockets must be placed on the visible structure. |
| F11 | Confirmed | `crates/tbg-client/src/gallery/settings.rs`, `crates/tbg-client/src/stress.rs:43`, `crates/tbg-client/src/overlay.rs`, `crates/tbg-client/src/autoplay.rs`, `crates/tbg-client/diagnostics/measure.js` | Diagnostic and gallery harness exists on main: `?gallery=...`, `?stress=N` (spawns 2N total units, divided by parity), `?autoplay=...`, FPS/draw overlay. `diagnostics/measure.js` hooks WebGL calls and frame timings. |

#### 1.2 Corrected Live Melee Reach Measurements (F4)

Recorded in live browser gameplay under hardware rendering (`ANGLE (Intel, Mesa Intel(R) Graphics (RPL-P), OpenGL 4.6)`) using the corrected FIFO probe (Hit/Heal consumption, Stronghold accounting, 5-tick minimum windup delay, 20-tick TTL):

| Melee Kind | Archetype | 1D Range | Observed Hits | 1D DX Min | 1D DX Max | DZ Values | Real XZ Min | Real XZ Avg | Real XZ Max | Theoretical Max $\sqrt{\text{range}^2 + 1.6^2}$ |
|---|---|---|---|---|---|---|---|---|---|---|
| Shieldbearer | Tank (Humans) | 0.800 | 10 | 0.590 | 0.790 | 0.0, 0.8 | 0.590 | 0.832 | 0.994 | 1.789 |
| Sawbot | Dps (Machines) | 0.800 | 20 | 0.250 | 0.800 | 0.0, 0.8, 1.6 | 0.790 | 1.141 | 1.789 | 1.789 |
| Rampart | Tank (Machines) | 0.800 | 4 | 0.550 | 0.600 | 0.0, 0.8 | 0.550 | 0.768 | 1.000 | 1.789 |
| Bone Brute | Tank (Undead) | 1.000 | 87 | 0.480 | 1.000 | 0.0, 0.8, 1.6 | 0.540 | 1.415 | 1.887 | 1.887 |
| Beetling | Dps (Plants) | 0.600 | 83 | 0.250 | 0.600 | 0.0, 0.8, 1.6 | 0.540 | 0.965 | 1.705 | 1.709 |

*Analysis:*
In 1D simulation space, every melee unit strictly respects `|dx| <= range`. However, because presentation lanes are separated by up to `dz = 1.6`, the true Euclidean distance in XZ space at `Hit` reaches up to $1.887$ for Bone Brute and $1.789$ for Sawbot. Without aim orientation and lunge, melee units visually swing at empty air when engaging enemies on diagonal lanes.

#### 1.3 Duplicate Projectile Bug Audit (F5)

- Live audit confirmed duplicate projectile emissions for native Archer:
  1. `crates/tbg-client/src/sync/events.rs:166–172`: on `Event::AttackStart`, `visuals::spawn_projectile` fires an immediate generic projectile along the vector from attacker to target.
  2. `crates/tbg-client/src/art/arrows.rs:33–48`: 0.18s later at `ClipEvent::Release`, `art/arrows.rs` spawns a native `FlyingArrow` oriented by fixed socket yaw (`velocity: rotation * Vec3::Z * range / 0.12`).
- 53 native arrows and 341 generic projectiles were captured in the targeted audit session. Native arrows flew horizontally along fixed direction `(-0.9063, 0.0, +0.4226)` regardless of target position.
- Resolution in Step 5: eliminate generic projectile spawn for native casters/archers; unify projectile emission through D2 socket origin and chest target endpoint.

#### 1.4 Per-Unit Entity Count and Triangles Per Form (55 Forms)

Total forms: 11 kinds × 5 forms (`(0,0), (1,0), (2,0), (1,1), (2,1)`) = 55 forms.
Triangles extracted from startup registration logs (`docs/media/a1/baseline-form-triangles.txt`); entity counts verified via `A1_ENTITIES` probe in `crates/tbg-client/src/visuals.rs`.

| Unit Kind | Faction | Archetype | Native? | Form 0/0 (Base) | Form 1/0 (A1) | Form 2/0 (A2) | Form 1/1 (B1) | Form 2/1 (B2) | Max Tris | Entity Count |
|---|---|---|---|---|---|---|---|---|---|---|
| Shieldbearer | Humans | Tank | Adapted | 880 | 1,000 | 808 | 898 | 1,098 | 1,098 | 31 |
| Archer | Humans | Ranged | Base+A Native | 1,178 + 74 | 1,274 + 74 | 1,406 + 74 | 926 (adapt) | 950 (adapt) | 1,406 (+74) | 34 (native) / 31 (adapt) |
| Bone Brute | Undead | Tank | Adapted | 932 | 968 | 1,028 | 836 | 864 | 1,028 | 31 |
| Hexer | Undead | Control | Adapted | 832 | 992 | 1,112 | 1,100 | 1,340 | 1,340 | 31 |
| Sawbot | Machines | Dps | Adapted | 948 | 1,096 | 1,208 | 972 | 984 | 1,208 | 31 |
| Siege Walker | Machines | Ranged | Adapted | 880 | 880 | 880 | 936 | 964 | 964 | 31 |
| Bloomkeeper | Plants | Support | Adapted | 1,376 | 1,376 | 1,496 | 1,400 | 1,470 | 1,496 | 31 |
| Rootcaller | Plants | Control | Adapted | 880 | 880 | 1,120 | 1,200 | 1,440 | 1,440 | 31 |
| Nester | Plants | Support | Adapted (recolor) | 1,376 | 1,376 | 1,496 | 1,400 | 1,470 | 1,496 | 31 |
| Rampart | Machines | Tank | Adapted (recolor) | 880 | 1,000 | 808 | 898 | 1,098 | 1,098 | 31 |
| Beetling | Plants | Dps | Adapted (recolor) | 880 | 880 | 1,120 | 1,200 | 1,440 | 1,440 | 31 |

#### 1.5 55-Form Roster Silhouette, Weapon & Evolution Audit

| Kind | Form | Name | Faction | Class | Current Body | Current Weapon | Weapon Profile | Silhouette Verdict | Identity Verdict | Animation / Rig Issues | Evolution Issues |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Shieldbearer | 0/0 | Base | Humans | Tank | Adapted | Sword + round shield | Slash | Generic round shield; lacks kite shield | Uses generic human body; sword conflicts with mace identity | Fixed ±65° yaw; no lunge; strikes air | Baseline |
| Shieldbearer | 1/0 | Bulwark | Humans | Tank | Adapted | Sword + gold-rim shield | Slash | Slightly larger shield, still round | Minimal visual distinction from base | Same fixed yaw and no reaction pairing | Scale +8% present in scale formula, but mesh changes minor |
| Shieldbearer | 2/0 | Fortress | Humans | Tank | Adapted | Sword + tower shield + crest | Slash | Distinct tower shield and red crest | Reads as Tank; lacks flanged mace | Weapon swing doesn't match tower shield weight | Scale +16%; good crest silhouette |
| Shieldbearer | 1/1 | Vanguard | Humans | Tank | Adapted | Spear (0.85m) + buckler | Slash / Thrust | Reads as spearman rather than tank | Diverges from Tank archetype | Spear uses sword slash clip | Weapon class changes to spear without unique clip |
| Shieldbearer | 2/1 | Champion | Humans | Tank | Adapted | Long spear (1.05m) + banner | Slash / Thrust | Back banner and pauldrons give strong silhouette | Good silhouette, but weapon is spear not mace | No directional aim for spear tip | High triangle count (1098); good banner |
| Archer | 0/0 | Base | Humans | Ranged | Native v2 | Yew shortbow | Ranged | Good hooded silhouette; offhand bow | Strong human archer identity | Spawns generic projectile + native arrow | Baseline |
| Archer | 1/0 | Longbow | Humans | Ranged | Native v2 | Yew longbow | Ranged | Green hood, feather; taller bow | Clear progression | Duplicate projectile bug; arrow ignores target | Scale has NO growth (`1.0x` in code) contrary to +8% rule |
| Archer | 2/0 | Marksman | Humans | Ranged | Native v2 | Gilded recurve bow | Ranged | Half-cape, gold trim, gilded bow | Excellent noble archer identity | Arrow flies along fixed yaw, missing target on diagonal | Scale grows only +7% (`1.07x`) instead of +16% |
| Archer | 1/1 | Volley | Humans | Ranged | Adapted | Dual quivers + recurve bow | Ranged | Drops native rig! Uses adapted legacy | Loss of visual fidelity from native A branch | BowRig may linger; feet slide without lock | Regression to legacy adapted body |
| Archer | 2/1 | Arrow Rain | Humans | Ranged | Adapted | Cape + double quivers | Ranged | Broad shoulders from cape | Silhouette blurred by boxy legacy parts | Lacks native secondary springs | Adapted body; does not match A branch fidelity |
| Bone Brute | 0/0 | Base | Undead | Tank | Adapted | Bone club | Smash | Hunched giant with ribs; distinct | Strong undead brute silhouette | Club swings horizontally across lane, missing diagonal | Baseline |
| Bone Brute | 1/0 | Ossified | Undead | Tank | Adapted | Bone club + bone chestplate | Smash | Bulkier torso | Reads as tank | Swings air at lane z=0.8 offset | Scale +8% |
| Bone Brute | 2/0 | Bone Wall | Undead | Tank | Adapted | Bone club + ribcage shield | Smash / Bash | Wide silhouette with rib shield on left hand | Excellent defensive silhouette | Shield is static, no bash animation | Scale +16%; shield gives great shape |
| Bone Brute | 1/1 | Butcher | Undead | Tank | Adapted | Iron cleaver | Chop | Asymmetric right arm | Reads as aggressive DPS/tank hybrid | Swing is generic attack clip | Weapon change to cleaver has no Chop profile |
| Bone Brute | 2/1 | Reaper | Undead | Tank | Adapted | Scythe (1.15m iron shaft) | Chop / Slash | Tall scythe polearm silhouette | Very distinct, grim reaper motif | Scythe blade clips ground during idle/walk | High reach weapon still hits air at dz=1.6 |
| Hexer | 0/0 | Base | Undead | Control | Adapted | Skull staff | Cast | Hooded cone head + staff; slender | Distinct mystical silhouette | Cast projectile leaves from center, not staff tip | Baseline |
| Hexer | 1/0 | Withering | Undead | Control | Adapted | Staff + glowing eye orbs | Cast | Orbs add top-heavy silhouette | Mystical undead identity | Cast clip has no target aim | Scale +8% |
| Hexer | 2/0 | Doom | Undead | Control | Adapted | Large staff + glowing ring aura | Cast | Floating glowing ring above staff | Strong tier 2 magical silhouette | Ring is rigidly parented to hand joint | Scale +16%; aura readable |
| Hexer | 1/1 | Plague | Undead | Control | Adapted | Censer on chain (left hand) | Cast | Asymmetric censer and spores | Strong alchemical/plague theme | Spores are static balls, no motion/smoke | Good branch divergence |
| Hexer | 2/1 | Pestilence | Undead | Control | Adapted | Censer + 5 floating spore balls | Cast | Wide halo of green spores | High silhouette volume (1340 tris) | Spore spheres clip into body during animation | Very high triangle count for adapted body |
| Sawbot | 0/0 | Base | Machines | Dps | Adapted | Right-hand circular saw | Mech | Boxy head, glowing eye slot, spinning saw | Strong steampunk automaton look | Human limb arcs look unnatural on robot | Baseline |
| Sawbot | 1/0 | Overclock | Machines | Dps | Adapted | Twin saws (both hands) | Mech | Wide aggressive dual-arm silhouette | Reads immediately as high-DPS mech | Both arms swing with human punch motion | Scale +8% |
| Sawbot | 2/0 | Shredder | Machines | Dps | Adapted | Twin saws + dual exhaust pipes | Mech | Exhaust stacks on back give vertical read | Clear evolution from base and A1 | Exhaust pipes are static, no smoke particles | Scale +16%; 1208 tris |
| Sawbot | 1/1 | Plating | Machines | Dps | Adapted | Single saw + front/back heavy plates | Mech | Wider, armored torso | Looks more like Tank than DPS | Armor plates clip into arms during attack | Good branch differentiation |
| Sawbot | 2/1 | Juggernaut | Machines | Dps | Adapted | Large saw (r=0.27) + tower helm | Mech | Tall rectangular headplate | Distinct heavy juggernaut | Tower helm clips into shoulders on walk | High mass silhouette |
| Siege Walker | 0/0 | Base | Machines | Ranged | Adapted | Torso cannon barrel (0.95m) | Ranged (Mech) | Slender tall legs, torso barrel; NOT arbalest | Conflicts with spec (cannon vs arbalest) | Recoil is whole body rock; no weapon aim | Baseline |
| Siege Walker | 1/0 | Longshot | Machines | Ranged | Adapted | Long cannon barrel (1.25m) | Ranged (Mech) | Extremely long protruding barrel | Clear long-range read | Barrel clips through obstacles and ally units | Identical triangle count to base (880) |
| Siege Walker | 2/0 | Artillery | Machines | Ranged | Adapted | Long barrel + brass muzzle ring | Ranged (Mech) | Brass muzzle accentuates barrel tip | High visual clarity at muzzle | Barrel doesn't elevate or traverse to aim | Scale +16%; same 880 tris |
| Siege Walker | 1/1 | Shrapnel | Machines | Ranged | Adapted | Triple cannon barrels (3x) | Ranged (Mech) | Multi-barrel array across torso | Strong volley artillery silhouette | 3 barrels shoot 1 generic projectile | Distinct wide profile |
| Siege Walker | 2/1 | Concussion | Machines | Ranged | Adapted | Triple barrels + rear brass boiler | Ranged (Mech) | Rear counterweight boiler balances silhouette | Industrial heavy artillery | Boiler doesn't steam or glow | Good weight balance |
| Bloomkeeper | 0/0 | Base | Plants | Support | Adapted | Blossom staff + leaf cloak | Cast | Flower-petal head ring + cloak | Distinct floral druid silhouette | Heal cast has no directional aim | Baseline; 1376 tris |
| Bloomkeeper | 1/0 | Blossom | Plants | Support | Adapted | Larger blossom petals | Cast | Head petals expand from 0.07 to 0.09 | Subtle change, hard to read at camera | Petal ring clips into torso on walk | Minimal silhouette change from base |
| Bloomkeeper | 2/0 | Grove | Plants | Support | Adapted | Blossom ring halo behind head | Cast | Outer halo ring gives regal priest silhouette | Reads as high tier healer | Halo is rigid, rotates with head | Scale +16%; 1496 tris |
| Bloomkeeper | 1/1 | Bramble | Plants | Support | Adapted | Bark armor slabs on torso | Cast | Thick wooden chest plate | Reads tougher, more durable | Armor slabs clip into arms | Scale +8% |
| Bloomkeeper | 2/1 | Thornwarden | Plants | Support | Adapted | Bark armor + 5 thorny wood cones | Cast | Spiky chest plate with 5 horn/cones | Aggressive thorny silhouette | Spikes look somewhat chaotic at distance | Good branch differentiation |
| Rootcaller | 0/0 | Base | Plants | Control | Adapted | Twin vine whips (both hands) | Slash / Cast | Spiky sprout ears + vine arms | Small woodland creature / treant | Vine whips are rigid pills, don't bend | Baseline; 880 tris |
| Rootcaller | 1/0 | Deep Roots | Plants | Control | Adapted | Long vine whips (0.7m vs 0.5m) | Slash / Cast | Longer arms reach further | Subtle at camera distance | Long rigid pills look stiff in attack clip | Minimal change |
| Rootcaller | 2/0 | Strangle | Plants | Control | Adapted | Long whips + leaf wrist cuffs | Slash / Cast | Leaf cuffs add green silhouette volume | Reads as evolved plant controller | Cuffs clip into body on walk | Scale +16%; 1120 tris |
| Rootcaller | 1/1 | Spores | Plants | Control | Adapted | Mushroom cap on head (r=0.24) | Slash / Cast | Domed toadstool cap silhouette | Completely changes silhouette; very readable | Cap clips into shoulders during idle breath | Excellent silhouette branch |
| Rootcaller | 2/1 | Overgrowth | Plants | Control | Adapted | Large cap (r=0.30) + 3 body mushrooms | Slash / Cast | Huge spotted mushroom crown + torso clusters | Highly readable fungus treant | Torso mushrooms clip into arms on attack | Scale +16%; 1440 tris |
| Nester | 0/0 | Base | Plants | Support | Adapted (recolor) | Recolor of Bloomkeeper (NESTER) | Cast | Identical to Bloomkeeper except color | Lacks unique silhouette; placeholder | Reuses Bloomkeeper animation directly | Reuses Bloomkeeper forms |
| Nester | 1/0 | Brood | Plants | Support | Adapted (recolor) | Recolor of Bloomkeeper A1 | Cast | Identical to Bloomkeeper A1 | No hive or insect thematic geometry | No egg or nest props | Recolor placeholder |
| Nester | 2/0 | Hive | Plants | Support | Adapted (recolor) | Recolor of Bloomkeeper A2 | Cast | Identical to Bloomkeeper A2 | Lacks wasp/bee hive silhouette | Petal ring recolored to yellow/brown | Recolor placeholder |
| Nester | 1/1 | Venom | Plants | Support | Adapted (recolor) | Recolor of Bloomkeeper B1 | Cast | Identical to Bloomkeeper B1 | No venom glands or stingers | Same bark slab geometry | Recolor placeholder |
| Nester | 2/1 | Blight | Plants | Support | Adapted (recolor) | Recolor of Bloomkeeper B2 | Cast | Identical to Bloomkeeper B2 | No blight motifs | Same chest thorns | Recolor placeholder |
| Rampart | 0/0 | Base | Machines | Tank | Adapted (recolor) | Recolor of Shieldbearer (RAMPART) | Slash (Mech) | Scaled-up Shieldbearer (`scale=1.25`) | Human knight proportions clash with Mech faction | Uses human walk and sword swing | Reuses Shieldbearer forms |
| Rampart | 1/0 | Bastion | Machines | Tank | Adapted (recolor) | Recolor of Shieldbearer A1 | Slash (Mech) | Large round shield, colored gunmetal | Still reads as human knight in armor | No mechanical joints or tread/wall silhouette | Recolor placeholder |
| Rampart | 2/0 | Citadel | Machines | Tank | Adapted (recolor) | Recolor of Shieldbearer A2 | Slash (Mech) | Tower shield recolored to iron/orange | Reads as heavy knight, not mechanical wall | Knight crest on helmet looks medieval human | Recolor placeholder |
| Rampart | 1/1 | Spiked | Machines | Tank | Adapted (recolor) | Recolor of Shieldbearer B1 | Thrust / Slash | Spearman knight recolored | Not a tank, not a machine wall | Spear thrust clip | Recolor placeholder |
| Rampart | 2/1 | Siegebreaker | Machines | Tank | Adapted (recolor) | Recolor of Shieldbearer B2 | Thrust / Slash | Knight with back banner recolored | Completely contradicts Machines aesthetic | Back banner is medieval fabric | Recolor placeholder |
| Beetling | 0/0 | Base | Plants | Dps | Adapted (recolor) | Recolor of Rootcaller (`scale=0.55`) | Slash | Scaled-down Rootcaller treant | Reads as mini-treant, NOT a beetle/insect | Uses treant whip arms, not mandibles | Summon inherits Nester tier/branch |
| Beetling | 1/0 | Brood | Plants | Dps | Adapted (recolor) | Recolor of Rootcaller A1 (`scale=0.55`) | Slash | Mini-treant with longer arms | Still not an insect | No beetle carapace or legs | Reuses Rootcaller A1 |
| Beetling | 2/0 | Hive | Plants | Dps | Adapted (recolor) | Recolor of Rootcaller A2 (`scale=0.55`) | Slash | Mini-treant with leaf cuffs | Treant silhouette | Same whip animation | Reuses Rootcaller A2 |
| Beetling | 1/1 | Venom | Plants | Dps | Adapted (recolor) | Recolor of Rootcaller B1 (`scale=0.55`) | Slash | Mini-treant with mushroom cap | Mushroom head looks fungal, not insectoid | Mushroom cap wobbles with head | Reuses Rootcaller B1 |
| Beetling | 2/1 | Blight | Plants | Dps | Adapted (recolor) | Recolor of Rootcaller B2 (`scale=0.55`) | Slash | Mini-treant with large mushroom + body fungi | Reads as mushroom goblin | High triangle count for small summon (1440 tris!) | Reuses Rootcaller B2 |

#### 1.6 Evidence Artifacts

- [Misaimed Diagonal Attack Screenshot](docs/media/a1/misaimed-diagonal.jpg) (`docs/media/a1/misaimed-diagonal.jpg`, 133 KB): live scene showing Bone Brute on lane $z = -0.8$ and Shieldbearer on lane $z = +0.8$ engaging across a 1.6-tile diagonal gap with fixed $\pm 65^\circ$ yaw, visibly swinging at empty air.
- [Autoplay Match Result](docs/media/a1/autoplay-result.jpg) (`docs/media/a1/autoplay-result.jpg`, 83 KB): complete match vs bot (DEFEAT / Stronghold destroyed, 1:52).
- [Targeted Match Result](docs/media/a1/targeted-result.jpg) (`docs/media/a1/targeted-result.jpg`, 83 KB): targeted practice match deploying Rampart, Sawbot, Nester, and Beetlings (DEFEAT, 1:35).
- [Baseline Form Triangles](docs/media/a1/baseline-form-triangles.txt) (`docs/media/a1/baseline-form-triangles.txt`): actual startup logs for all 55 forms.
- [Targeted Audit Console JSON](docs/media/a1/preliminary-targeted.json) (`docs/media/a1/preliminary-targeted.json`, 237 KB): full CDP console capture of corrected reach measurements and projectile events.
- [Temporary Audit Probe Patch](docs/media/a1/temporary-audit-probe.patch) (`docs/media/a1/temporary-audit-probe.patch`): exact diff of the temporary instrumentation used to gather live measurements, preserved for reproduction.

### Step 2: Baselines before any change

#### 2.1 Formal Performance Baseline Matrix (Hardware GPU: Intel RPL-P OpenGL 4.6)

Measured across 3 runs of 60 seconds each on Chrome 156 (`1280x800`, DPR 1, port 9333) with release client. Medians computed across the 3 independent runs:

| Scenario | Mode / URL Param | Units (Total / Per Side) | Median FPS | Frame Time p50 (ms) | Frame Time p95 (ms) | Frame Time p99 (ms) | JS Heap (MB) | Wasm Mem (MB) | Bevy Entities | Mesh Assets | Material Assets | WebGL Draws / Frame |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **10 per side** | `?stress=20&measure` | 20 (10 / 10) | 84.9 | 6.1 | 34.1 | 134.6 | 24.36 | 89.50 | 993 | 1,100 | 25 | 359 |
| **40 per side** | `?stress=80&measure` | 80 (40 / 40) | 109.6 | 6.1 | 26.3 | 36.5 | 28.86 | 94.69 | 2,932 | 1,100 | 25 | 452 |
| **100 per side** | `?stress=200&measure` | 200 (100 / 100) | 60.5 | 12.3 | 41.9 | 55.1 | 30.67 | 104.44 | 6,809 | 1,100 | 25 | 630 |
| **200 per side** | `?stress=400&measure` | 400 (200 / 200) | 29.7 | 23.9 | 75.5 | 385.2 | 17.72 | 139.75 | 13,273 | 1,100 | 25 | 927 |
| **Autoplay Practice** | `?autoplay=practice&measure` | Live vs-bot match | 107.0 | 6.1 | 23.8 | 31.5 | 18.32 | 85.31 | 971 | 1,100 | 25 | 265 |

*Performance Observations:*
- Steady-state frame time median (p50) remains at 6.1 ms (~160 Hz display cadence) up to 80 active units and in live practice gameplay (p50 = 6.1 ms, FPS = 107.0).
- At 200 total units (`stress=200`), frame time p50 settles at 12.3 ms with 60.5 FPS, satisfying the 60 FPS desktop requirement (ART-DIRECTION §9).
- At extreme stress of 400 units (`stress=400`, 13,273 ECS entities), frame time p50 is 23.9 ms (29.7 FPS), testing high-density crowd bounds.
- CPU task duration tracking (`Performance.getMetrics` TaskDuration) requires continuous tracing/devtools profiling; marked "not measured" outside tracing sessions.
- Draw calls scale from 265 (in live practice) to 452 (80 units) and 927 (400 units) thanks to Bevy mesh sharing and batching.

#### 2.2 Baseline Media Artifacts

- [Aim Lab Before Screenshot](docs/media/a1/baseline/baseline-aim.jpg) (`docs/media/a1/baseline/baseline-aim.jpg`, 35 KB): standalone `?gallery=aim` lab displaying baseline Archer and targets before aim orientation changes.
- [Stress 20 Units Baseline](docs/media/a1/baseline/baseline-stress-20.jpg) (`docs/media/a1/baseline/baseline-stress-20.jpg`, 158 KB): 10 units per side walking the lane.
- [Stress 80 Units Baseline](docs/media/a1/baseline/baseline-stress-80.jpg) (`docs/media/a1/baseline/baseline-stress-80.jpg`, 178 KB): 40 units per side walking the lane.
- [Stress 200 Units Baseline](docs/media/a1/baseline/baseline-stress-200.jpg) (`docs/media/a1/baseline/baseline-stress-200.jpg`, 185 KB): 100 units per side stress test.
- [Stress 400 Units Baseline](docs/media/a1/baseline/baseline-stress-400.jpg) (`docs/media/a1/baseline/baseline-stress-400.jpg`, 185 KB): 200 units per side stress test.
- [Autoplay Match Baseline](docs/media/a1/baseline/baseline-autoplay.jpg) (`docs/media/a1/baseline/baseline-autoplay.jpg`, 136 KB): 60-second in-match state vs bot.
- [Baseline Performance JSON](docs/media/a1/baseline/baseline-performance.json) (`docs/media/a1/baseline/baseline-performance.json`, 6.4 KB): complete 3-run raw data capture and median metrics.

#### 2.3 Step 5 Aim Wiring & Archer Prototype Verification

- **Aim System Wiring (`visuals/aim.rs`):** Single authoritative system `update_aim` drives figure child yaw toward authoritative target (`Quat::from_rotation_y(aim.current_yaw)`). Base yaw (`v.yaw`) maintained when idle or walking.
- **Targeted Projectile (`art/arrows.rs`):** Implemented D2 flight from bow socket directly to target chest (`Vec3::new(0.0, 0.55, 0.0)`), duration clamped to 0.12s, rotation aligned to flight velocity. Removed legacy facing-directed range calculation and redundant generic projectile spawning on native archers.
- **Contract Tests:**
  - `test_arrow_endpoint_accuracy_across_8_angles`: Verified discrete 60 Hz integration arrives within < 0.15 tile of target chest across all 8 angles x near/far.
  - `test_arrow_spawn_direction_matches_aim_dir`: Verified arrow horizontal flight direction aligns with aim direction within < 1.0 deg at spawn across all 8 angles.
- **Aim Lab (`?gallery=aim`):** Attacker at center, targets on 8-angle ring x {near, far}, retargeting (1-8 and space keys), target kill test (K key), near/far toggle (N key), crowd mode (12 attackers around 3 targets), and unit query parameter `&unit=`.
- **Media Artifacts:**
  - [Archer Aim 8 Angles Contact Sheet](docs/media/a1/archer-aim-8angles.jpg) (`docs/media/a1/archer-aim-8angles.jpg`, 277 KB): 4x2 montage demonstrating directional aiming across all 8 angles in the aim lab.
  - [Archer Attack Frames Sheet](docs/media/a1/archer-attack-frames.jpg) (`docs/media/a1/archer-attack-frames.jpg`, 56 KB): 0%, 25%, 50%, 75%, 100% frame-by-frame draw and release.

### Progress

Step 0 complete (preflight verification).
Step 1 complete (audit report, corrected F4 live melee reach measurements for all 5 melee units, F5 duplicate projectile verification, 55-form roster audit, triangle and entity counts, misaimed diagonal attack screenshot).
Step 2 complete (aim lab before sheets, formal 5-scenario performance baseline across 3 runs of 60s, hardware GPU measurements, baseline screenshots and raw metrics committed).
Step 3 complete (authored ART-DIRECTION §11 D5 visual standard, class proportion table, faction morphology/material rules, evolution scale progression rules +8%/+16%, and Tupi checklist).
Step 4 complete (aim core pure Rust in art/aim.rs with facing_yaw, turn_toward, Aim state machine Active/Hold/Release, deadzone, frozen point on despawn, ImpactRing FIFO and 1s expiry; all 11 contract tests passing).
Step 5 complete (aim wiring system in visuals/aim.rs, D2 targeted arrow flight in art/arrows.rs, duplicate projectile fix, 8-angle lab contact sheet, 2 contract tests passing).
Steps 6–14 pending.

### Owner decisions

- O-A1-1: apply the supplied default; look layer only, live `Classic`.
- O-A1-2: apply the supplied default on completion; supersede visual phases
  P16/P22–P29 only once the replacement work is implemented.
- O-A1-3: no event/protocol change authorized or made.
- O-A1-4: owner visual sign-off pending; merge remains blocked.
- O-A1-5: no lane convergence authorized or implemented.

### Deviations

- The supplied spec was untracked in the owner's original checkout and
  absent from `main`; it is copied into this branch as part of step 0.
- The original checkout was at `6ee3495`, with an edited ROADMAP and an
  untracked `.claude/` directory and A-1 spec, rather than the planning-time
  balance edits described above. Isolation is still required and preserved.
- The spec calls the glossary §3; its actual location on `main` is §2.
- This explicitly requested sibling worktree is not an app-managed worktree;
  the app attachment tool returned `The checkout exists but is not a managed
  worktree.` Commands use its absolute working directory.
