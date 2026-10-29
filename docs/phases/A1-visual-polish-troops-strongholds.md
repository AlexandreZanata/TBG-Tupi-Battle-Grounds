# A-1 — Professional visual polish: troops, animation, targeted combat, Strongholds

**Status:** handed off at owner request; step 0 passed, step 1 partial (spec written 2026-10-29 against HEAD `f15432f` on
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

### Progress

Step 0 complete. Step 1 partially audited; steps 2–14 pending. Execution
was interrupted at the owner's request to continue in Antigravity.
No permanent gameplay or presentation behavior change has been made.
Temporary probes were saved as a patch and removed from client source.

The full resume instructions, corrected source recognition, evidence,
verification outputs, preliminary-measurement caveat and remaining steps
are in [A1-HANDOFF.md](../A1-HANDOFF.md).

A Chromium hardware-rendered autoplay match reached the result screen
(1:52, Stronghold destroyed). Formal performance baseline: not measured.
Firefox gameplay, online/reconnect and final acceptance: not run.
The preliminary F4 measurements require repeating with the corrected
probe; they are not accepted audit results.

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
