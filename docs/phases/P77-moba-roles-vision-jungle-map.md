# P77 — 5v5 role Shop slots, team vision, jungle monsters and the complete map refinement

> Owner request, 2026-11-03. Executable spec and handoff for the next agent.
> Builds on [P76](P76-moba-5v5-foundation.md) (PR #67). Every new rule and
> number is **Proposed**; no balancing (owner rule: balance only on an
> explicit request).

## 0. Handoff prompt (paste this into the next agent)

```text
You continue TBG Arena 5v5 (MOBA mode) of Tupi Battle Grounds.
Repository: github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds.
1. Read AGENTS.md, docs/VALIDATION.md, then docs/moba/README.md,
   docs/moba/GAME-DESIGN-MOBA.md, docs/moba/ARCHITECTURE-MOBA.md,
   docs/moba/CONTRACTS.md and this file (docs/phases/P77-moba-roles-vision-jungle-map.md).
2. Base: `git fetch origin`. If PR #67 (phase/p76-moba-core) is merged, branch
   `phase/p77-moba-roles-vision-map` from origin/main; otherwise branch it
   from origin/phase/p76-moba-core. Work in a new worktree under /data.
3. Execute sections 3–6 in order (A role slots → B jungle monsters →
   C team vision → D map refinement), one commit per step, never push to
   main, never edit crates/* (the 1v1 game) or the root Cargo files.
4. Validate per section 7, write the report in section 9 of this file, push
   the branch and open a PR to main only if the owner asked for it.
```

## 1. Read first

| File | Why |
|---|---|
| `moba/core-moba/src/{content.rs,shop.rs,intent.rs,bot.rs,protocol.rs,view.rs,state.rs}` | Shop, intentions, wire views |
| `moba/core-moba/src/sim/{spawn.rs,combat.rs,movement.rs,rewards.rs}` | camps, neutral behaviour, rewards |
| `moba/core-moba/tests/contract_moba.rs`, `moba/server-moba/{src/room.rs,tests/room.rs}` | contracts and the per-tick fan-out |
| `moba/client-moba/src/{link/mod.rs,units/mod.rs,units/models.rs,units/champions.rs}` | snapshot mirror, figures |
| `moba/client-moba/src/hud/{shop.rs,chrome.rs,minimap.rs}`, `camera.rs` | cards, minimap camera |
| `moba/client-moba/src/scene/*` | the generated valley (terrain, lanes, river, landmarks, forest) |
| `docs/moba/media/reference-map.webp` | **the owner's visual target for the map** |
| `docs/moba/media/reference-1v1-battle-hud.webp` | the 1v1 battle HUD the 5v5 HUD follows |
| `crates/tbg-client/src/arena/{trees,rocks,cliffs,water,dressing,props,clouds}.rs` | 1v1 scenery kits (read-only; see D) |

## 2. Fixed constraints (from AGENTS.md and D-112)

- Never edit `crates/*`, the root `Cargo.toml`/`Cargo.lock` or any 1v1
  release input: they would force a 1v1 desktop release. Share 1v1 code
  read-only, as P76 does (`#[path]` includes, `tbg-core` dependency, copied
  1v1 assets at install).
- `tbg-moba-core` stays deterministic: integers, id-ordered units, seeded
  RNG, no clocks/HashMap iteration/floats in state. The server stays the
  only authority online; the client never decides outcomes.
- Glossary of `docs/moba/GAME-DESIGN-MOBA.md`; every new number is
  Proposed there in the same commit. Contract tests are append-only: change
  one only with the rule it guards, named in the commit message.
- Wire protocol: PR #67 not merged yet → you may still change v1. Once it
  is merged, any wire change bumps `MOBA_PROTOCOL_VERSION` to 2.
- Every visual change adds a record to `docs/ui/DESKTOP-UI-CONTRACT.md` and
  `docs/ui/desktop-ui-contract.json` (Linux observed, Windows/web pending).
- Validation per `docs/VALIDATION.md`: shared lock, two jobs, scoped checks.

## 3. Section A — three role-locked Shop slots (rules M-19, M-20)

Owner rule: in 5v5 the Shop has **3 fixed slots** whose champions come only
from the **categories (archetypes) of the player's lane role**. For now:

| Role | Allowed archetypes | 1v1 pool size |
|---|---|---|
| Top | Dps, Tank | 21 |
| Mid | Control, Dps | 14 |
| Bot | Ranged, Dps | 17 |
| Support | Support, Control | 14 |
| Jungle | Dps, Tank, Ranged | 32 |

Later the owner will let players choose which category each of the 3 slots
accepts, so model it per slot from the start.

Steps:

- **A1 core data.** In `content.rs`: `pub const OFFERS: usize = 3;` (stop
  re-exporting the 1v1 `OFFERS`), and
  `pub fn slot_rules(role: Role) -> [&'static [Archetype]; OFFERS]` returning
  the role's set for all three slots (one table, documented Proposed).
- **A2 Shop.** `shop::redraw(rng, offers, locked, rules)` draws each
  unlocked slot from `POOL` filtered by that slot's archetypes, distinct
  across the three slots and excluding champions held by locked slots
  (keep the partial Fisher–Yates; build candidates per slot in slot order
  for determinism). `opening` takes the seat's rules. Purchase-redraw, lock
  and reroll semantics stay those of M-19.
- **A3 Player.** `Player.offers: [Champion; 3]`; `MobaMatch::new` passes
  `slot_rules(seat.role())`. Add `Player.slot_rules` (or derive from the
  role) so a future intention can change a slot's category.
- **A4 Wire.** `PrivateView.offers: [Champion; 3]`; add
  `slot_rules: [u8; 3]` (archetype bitmask per slot) so the HUD can label
  slots. `BuyTroop{slot}` and `ToggleLock{slot}` reject `slot >= 3`.
- **A5 Bots.** Bots buy the first affordable offer, preferring the role's
  first archetype; keep the reroll threshold.
- **A6 HUD.** Three 1v1 cards centred (same card geometry), the resource
  strip shrinks to the cards' width or stays at its current width centred —
  pick one and record it; hotkeys `1–3`; under each card's role line show
  the slot's allowed categories in muted text (e.g. "Dps · Tank").
- **A7 Tests.** Update `m19_*` (three offers) and add
  `m20_role_slots_only_offer_the_role_archetypes`: for every role and many
  rerolls, every offer's archetype is in its slot's set and offers stay
  distinct. Update `m10`, `m11` and the server room test only as needed.
- **A8 Docs.** GAME-DESIGN-MOBA M-19 rewritten, new M-20, CONTRACTS §3/§5.

## 4. Section B — jungle monsters (rules M-60…M-64)

Today every camp spawns one generic big and two small neutrals with
placeholder figures. Replace them with real camps:

- **B1 camp kinds (Proposed).** Four kinds, mirrored per side (camp ids
  0–3 Blue, 4–7 Red, same index → same kind), Tupi-themed and readable:
  | Camp | Monsters | Role |
  |---|---|---|
  | `Wolves` (lobo-guará pack) | 1 alpha + 3 pups | fast, low HP, gold |
  | `Golems` (stone golem) | 1 big + 2 small rock golems | slow, high HP, armour |
  | `Jaguar` (onça) | 1 elite | high damage, single target |
  | `Frogs` (sapo-cururu) | 1 big + 2 small, ranged spit | ranged pressure |
  Store kind in `map::camp(id)` (or a `CAMP_KINDS` table); add
  `NeutralKind` variants; stats, bounties (Gold/Essence/XP) and respawn
  per kind in `content.rs`. Keep passive-until-hit and the leash (M-09).
- **B2 behaviour.** Monsters fight back, return home and heal when
  leashed, respawn after their timer (existing rules, extend tests).
  Optional river objective at the plaza is an **owner decision** — do not
  add it without approval.
- **B3 figures.** New `client-moba/src/units/monsters.rs`: stylised
  low-poly figures in the 1v1 art language (vertex colours via
  `crate::art::mesh::Shape`, `Canopy`, `Rock`, `Bevel`, palette), per-kind
  idle breathing, attack lunge/bite, hit flash, death collapse and a
  spawn-from-ground animation when a camp respawns. Camp dressing per kind
  (bones, moss, lily pads, crystals) in `scene/landmarks.rs`.
- **B4 HUD/minimap.** Camp timer on the minimap when a camp is cleared and
  in vision (after C), neutral health bars as today.
- **B5 tests.** `m60_each_camp_spawns_its_kind`, `m61_monsters_stay_passive_until_hit`,
  `m62_leashed_monsters_heal_home`, `m63_cleared_camp_respawns_after_its_timer`.

## 5. Section C — team vision / fog of war (rules M-50…M-55)

Owner rule: a player only sees where **allied units have vision**.

- **C1 core.** `vision.rs` in `tbg-moba-core`: per tick, per team, an enemy
  or neutral unit is visible iff some allied unit (Troop, Minion,
  Commander, Tower, Stronghold) lies within that ally's `stats.sight` of
  it. Use the existing grid; deterministic; O(units × local). Structures
  are always visible (map knowledge); allied units always visible.
- **C2 authority / anti-cheat.** The world frame becomes **per team**:
  `view::world(m, team)` omits units the team cannot see and events it
  cannot know (e.g. an enemy's kill feed outside vision may still be
  public — decide and record; scores stay public). The room encodes two
  world frames per tick (Blue, Red) and sends each socket its team's frame;
  bandwidth stays O(team frame). The offline host uses the player's team.
- **C3 attacks into fog.** Projectiles/targets that leave vision stop being
  sent; the client must not crash on targets it never saw (already
  tolerant: missing ids are ignored — keep it so).
- **C4 client fog.** A fog texture (e.g. 256×256 R8, map space) rebuilt
  each frame from allied unit positions and sight radii (from a shared
  sight table in `core::content`), smoothed, drawn as a dark translucent
  layer over terrain and scenery (a custom `Material` or a plane with an
  alpha texture), with explored-but-not-visible areas dimmer than never
  seen if you add exploration memory (owner choice; default: two levels,
  visible / not visible). The minimap shows the same fog. Units outside
  vision are not rendered (they are not in the frame anyway); structures
  keep their last known state.
- **C5 tests.** `m50_enemies_outside_allied_sight_are_absent_from_that_team_frame`,
  `m51_structures_are_always_in_every_frame`, `m52_each_socket_receives_its_team_frame`
  (server), `m53_vision_is_deterministic`.
- **C6 performance.** Measure with the probe in section 7; keep the tick
  well under 1 ms and the per-team frame ≤ the P76 frame size.

## 6. Section D — complete visual refinement of the map

Target: `docs/moba/media/reference-map.webp`. Linux is the reference.

- **D1 reuse the 1v1 scenery kits read-only.** `arena/{trees,rocks,cliffs,
  water,dressing,props,clouds}.rs` depend only on `crate::art` and a parent
  `Part = (Shape, Color, Transform)` type, so the MOBA can include them with
  `#[path]` (same pattern as `art/`) inside a `scene/kit` module that
  defines `Part`. Replace the MOBA pines/bushes/rocks with the 1v1 ones
  (pine canopies, bushes, grass tufts, flowers, boulders) so both modes
  share one look. Never edit those files.
- **D2 terrain.** Richer ground: grass colour variation and patches,
  darker forest floor under trees, dirt around camps, sand on river banks,
  cliff faces with layered rock and moss on the valley rim, smoother height
  transitions, waterfalls where the river leaves the valley (reference).
- **D3 lanes.** Cobbled roads with edge stones and worn centre, gentle
  curves where the reference has them, lane-side props (fences, lamp
  posts/torches with the 1v1 flame, banners) away from walking paths.
- **D4 river and plaza.** Animated water (scrolling normals or vertex
  waves, foam at banks and around rocks), stone bridges matching the
  reference on Top/Bot, the plaza as a raised stone island with the glowing
  crystal and stairs.
- **D5 bases.** Fortified plateaus like the reference: walls with gates on
  each lane, team banners, braziers, a richer Stronghold crystal pedestal;
  tower models refined to the reference silhouette (stone shaft, team
  crystal, team trim), damaged states as HP drops.
- **D6 jungle.** Camp pits per monster kind (section B), denser paths
  through the jungle, bushes and rocks framing camps.
- **D7 light and atmosphere.** Warm key light and cool ambient as the
  reference, distance haze, optional clouds (`clouds.rs`), soft shadow
  cascades tuned for the overview and the lane camera.
- **D8 performance budget.** Desktop target 60 fps on the owner's laptop
  (RTX 4060) at 1920×1080 with ten bots and fog: merge static scenery into
  chunk meshes (as today), keep draw calls and triangle counts reported in
  the phase report, disable shadows on tiny props.
- **D9 captures.** Before/after captures (overview, lane, jungle camp,
  base, river) in `docs/moba/media/p77-*.webp`.

## 7. Validation commands

Build directory and lock (see `docs/VALIDATION.md`):

```bash
export CARGO_TARGET_DIR=/data/<your-target> CARGO_BUILD_JOBS=2
export PKG_CONFIG_PATH=<main checkout>/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig
cd moba
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock cargo fmt --all -- --check
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock cargo clippy --locked --workspace --all-targets -- -D warnings
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock cargo test --locked --workspace
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock cargo check --locked -p tbg-moba-core -p tbg-moba-client --target wasm32-unknown-unknown
```

Visual checks against a bot match (owner instruction): Xvfb uses the local
NVIDIA GPU through Vulkan, so captures are representative.

```bash
xvfb-run -a -s "-screen 0 1920x1080x24" env WAYLAND_DISPLAY= \
  $CARGO_TARGET_DIR/debug/tbg-moba --capture /data/<shots>/lane.png --seed 11 --warp 420 --role mid
xvfb-run -a -s "-screen 0 1920x1080x24" env WAYLAND_DISPLAY= \
  $CARGO_TARGET_DIR/debug/tbg-moba --capture /data/<shots>/overview.png --seed 11 --warp 420 --overview
```

Real input in a headless window: start `tbg-moba --play` in the background
inside `xvfb-run`, then `moba/scripts/xtest_input.py click:X:Y key:1 focus …`
and `import -window root shot.png` (ImageMagick). Window is 1600×900.
Online: `tbg-moba-server --bind 127.0.0.1:7979` + `tbg-moba --online
ws://127.0.0.1:7979/moba/ws`; stop only the processes you started (by PID).

Install for the owner: `moba/scripts/install-linux-moba.sh` (menu entry
"TBG Arena 5v5").

## 8. Acceptance

| Item | Evidence |
|---|---|
| 3 Shop slots per player, each offer inside its slot's role archetypes, locks/reroll/purchase redraw unchanged | m19/m20 + Linux capture of the three cards for two roles |
| Four jungle camp kinds with their own figures, behaviour and respawn | m60–m63 + capture of each camp |
| Each team receives only what it sees; fog drawn on map and minimap | m50–m53 + capture showing an enemy appearing when an ally walks in |
| Map refined towards the reference (sections D2–D7) | before/after captures, draw-call/triangle numbers |
| No `crates/*` or root Cargo change; 1v1 policy scripts pass | `check-native-version.py --base origin/main` etc. |
| Windows/web correspondence | pending (P81), recorded honestly |

## Owner steering — section E (2026-11-03)

Explicit follow-up during execution: players must not have personal
champion avatars or manual champion control. Purchases choose a Lane (or
the existing Jungle Destination), then movement/combat is automatic.
Add left-button ground dragging for camera navigation only. This revises
M-01/M-10/M-14/M-20/M-21/M-22/M-30/M-40/M-41/M-50–M-55/M-64 and adds
M-65/M-66; see the D-112 second amendment. Avatar XP/respawns/fountain and
manual orders from P76 are retired. Legacy enum tags are reserved, manual
orders rejected, and the wire moves to v2. The earlier Commander references
in sections A–D are historical input, superseded by this steering.

## 9. Report

**Level: work front, 2026-11-03.** Implementation is committed locally on
`phase/p77-moba-roles-vision-map`, worktree
`/data/tbg-p77-moba-roles-vision-map`. Base: `abc65648fcef` from
`origin/phase/p76-moba-core`; GitHub PR #67 was open/unmerged when checked.
The original checkout and other worktrees were preserved. No push, merge,
installation, release, deployment or balance run occurred.

Implementation commits:

- `2b35bb1a`: A — three role-locked Shop slots (M-19/M-20).
- `06463374`: B — four mirrored camp kinds, monster figures/motion and
  per-kind Proposed stats, rewards and respawns (M-60–M-64).
- `be736548`: C — per-Team snapshots, hidden data/event filtering,
  last-observed structure HP, fog material/mask and visible camp timers.
- `bb3a7f9d`: D — read-only shared vegetation/rocks, ground/path dressing,
  cobbles/kerbs/fittings, stone bridges/plaza/base refinements, waterfalls,
  animated water/flames, structure damage bands and lighting/haze.
- `0787ad65`: E — the owner's follow-up: no personal champion/avatar or
  movement control; per-purchase Lane/Jungle Destination; automatic
  movement/combat; left-drag map camera; strategic Seat/K/D UI; protocol v2.

All `crates/*`, root `Cargo.toml` and root `Cargo.lock` are identical to
both the P76 base and `origin/main`. No dependency was added. The only
shared 1v1 code is read-only. All five visual fronts have records in the
Markdown and JSON desktop correspondence contracts, with observations
explicitly pending.

### Checks actually run

Formatting and whitespace on the final code:

```bash
cargo fmt --manifest-path moba/Cargo.toml -p tbg-moba-core -p tbg-moba-client -p tbg-moba-server -- --check
git diff --check
python -m json.tool docs/ui/desktop-ui-contract.json > /dev/null
git diff --name-only origin/main -- crates Cargo.toml Cargo.lock
```

All exited 0; the last command printed no paths. A final server-test
fixture formatting pass also exited 0. Source checks below covered the
final game implementation; the subsequent change only bounded a test
fixture's wait and labelled historical screenshots.

Scoped compilation, from `moba/`, reusing the existing cache and the shared
host lock, always at two Cargo jobs:

```bash
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock \
  env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-moba-target \
  PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
  bash -c 'cargo check --locked -p tbg-moba-core -p tbg-moba-server --lib && cargo check --locked -p tbg-moba-client --bin tbg-moba'
```

Final real output, exit 0, no warnings:

```text
Checking tbg-moba-core v0.1.0
Checking tbg-moba-server v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.63s
Checking tbg-moba-core v0.1.0
Checking tbg-moba-client v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.54s
```

Earlier compilation caught four client errors: an overbroad edit had
copied projectile filtering into the generic Fade system, and Bevy 0.20
asset guards required mutable bindings. Those were fixed in D. A later
pass passed with two unused-binding/import warnings after avatar removal;
they were removed before the final clean compilation above. One HUD roster
parse error was corrected before formatting passed. No failure repeated
twice; there is no unresolved compiler failure.

Read-only 1v1 policy guards:

```bash
python scripts/check-native-version.py --base origin/main
python scripts/check-patch-notes.py --base origin/main
python scripts/check-native-inputs.py
python scripts/ci-test-lanes.py
```

Real results:

```text
Native version policy satisfied for 0.1.4-colossus.6.
Patch notes OK: 12 entries; current ruleset 16 / f2e253b98ccb313a
Native inputs use canonical UTF-8/LF bytes; binary assets are untouched.
Test lanes cover the workspace with unified features: {'test': '--workspace --exclude tbg-balance --features tbg-core/lab', 'balance': '-p tbg-balance --features tbg-core/training'}
```

The first version-policy attempt failed because the restricted shell could
not resolve github.com; its approved network-enabled retry passed. The
other guards passed immediately. Their evidence remains applicable:
subsequent changes were limited to `moba/` and documentation, with the
unchanged 1v1 paths inspected again on the final tree.

### Regression coverage written, not executed

There are now 22 core contract scenarios and 4 room scenarios. New coverage
includes role-slot membership/uniqueness/locks, camp composition/passivity/
leash/respawn, exact sight boundary and hidden data, remembered structure
HP, deterministic Team frames, socket Team isolation, no avatars,
rejected manual commands, per-purchase lane choice/automatic walking and
Troop bounty/score bookkeeping. Existing scenarios remain; affected
assertions/API calls follow their revised rules and D-112 amendment.

No `cargo test`, workspace Clippy, WASM compilation, build/package,
benchmark or gameplay campaign ran. These checks are pending, not passes.

### Decisions and deviations

- The resource strip keeps its original centred 862 px width. Three
  166×225 cards with 8 px gaps stay centred below it; category captions
  are below the Archetype line. Hotkeys are 1–3.
- Slot rules derive per slot from Role, one content table; there is no new
  category-changing intention. Any Role may send each purchase anywhere.
- The P77 draft reused M-20, which P76 used for avatar progression.
  The owner's E revision retires the avatar clause and leaves M-20 as
  the slot rule, without retaining unused XP/level systems.
- Protocol v1 could still change because P76 was unmerged. E deliberately
  moves to v2 to make the removal of avatar fields and manual controls
  explicit and reject old clients. Existing enum discriminants remain
  reserved; manual-order variants return ManualControlDisabled.
- Troop K/D/team score replace the retired avatar kill score. Card-price
  bounties stay unchanged, neutral kills do not score for a Team, and no
  new kill bonus is introduced. No balancing or river objective was added.
- Fog has two levels with a cosmetic inner feather, no exploration
  memory. Public Troop kill counts, Towers, team Gold and outcomes remain
  visible; hidden purchases/camp events/target ids do not. Structure HP
  remains last-observed outside sight. The client never decides visibility
  online; shared sight is used only for presentation of received state.
- D1's assumption that every listed 1v1 scenery file depends only on Part
  was inaccurate: cliffs/water depend on 1v1 geometry/theme/materials;
  props/life also depend on authored 1v1 sites. The adapter therefore
  includes trees.rs and rocks.rs directly and builds MOBA-specific stone,
  river and fitting geometry. No copies of the 1v1 world layout and no 1v1
  edits were introduced. Flames follow the 1v1 nested-cone language.
- Runtime instrumentation logs static mesh batches/triangle counts before
  culling, explicitly excluding water/crystal/flames, units and shadow
  passes. It is not a measured GPU draw-call count. Section 7 names a
  performance probe but provides no actual probe command/source; no new
  benchmark framework was added.

### Deferred consolidation and owner acceptance

Reuse `/data/tbg-moba-target`, the PKG_CONFIG_PATH above, the nonblocking
host lock and `CARGO_BUILD_JOBS=2`. One reviewer should deduplicate and run:

```bash
# Focused contracts; the full-match scenario stays outside this work-front scope.
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-moba-target \
  cargo test --locked -p tbg-moba-core --test contract_moba -- --skip m30_a_ten_bot_match_always_ends --test-threads=1
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-moba-target \
  cargo test --locked -p tbg-moba-server --test room -- --test-threads=1
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-moba-target \
  cargo clippy --locked -p tbg-moba-core -p tbg-moba-server --lib -- -D warnings
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-moba-target \
  cargo clippy --locked -p tbg-moba-client --bin tbg-moba -- -D warnings
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-moba-target \
  cargo check --locked -p tbg-moba-core -p tbg-moba-client --target wasm32-unknown-unknown
```

Reasons: work-front policy defers test execution, Clippy and extra targets;
compilation does not compile these integration tests or validate shaders.
No local full suite was requested. Full workspace suites, the ten-bot
full-match probe, 60 fps measurement and packaging/install remain deferred
until the relevant explicit owner scope.

A scoped native build and Linux runtime are still needed to validate the
embedded fog shader/pipeline, gesture handling and appearance. Capture
`docs/moba/media/p77-{overview,lane,camp-wolves,camp-golems,camp-jaguar,camp-frogs,base,river}.webp`
with the P77 binary and compare to the historical P76 captures and owner
reference. Verify left dragging on map; starting on cards/locks/resources/
roster/modal; dragging across UI; release/focus loss; minimap drag; zoom;
Space/base centre. Buy champions into different lanes without any movement
input; verify no avatar/Auto/right-click movement; see an enemy enter/leave
allied sight, loss of projectile targets, monster spawn/hit/death motion,
and timers disappearing outside vision.

No new capture was generated and no P77 runtime, frame time, triangle or
GPU draw-call measurement is claimed. Windows/desktop-web pixel and motion
checks remain pending (P81); Linux-only compilation proves no visual
correspondence. Owner look/fun sign-off, balance and release remain open.

### Linux installation follow-up (owner request, 2026-11-03)

The owner explicitly requested updating the existing separate 5v5 Linux
playtest. Ran the existing installer with `CARGO_TARGET_DIR=/data/tbg-moba-target`
and the PKG_CONFIG_PATH recorded above. The installer holds the shared
nonblocking validation lock and uses two Cargo jobs. Cargo briefly waited
for its package cache, then continued compiling; the existing process
finished during a chat interruption. No second build was started.

The existing `TBG Arena 5v5` menu entry and both `~/.local/bin/tbg-moba*`
links now target `~/.local/opt/tbg-moba/0.1.0-e0cbfe6c3645/`.
Installed `tbg-moba --version`: `TBG Arena 5v5 0.1.0 · MOBA protocol 2`.
Installed client and cached build have identical SHA-256:
`bc34e28453de785f406078489090b711b67653e470bcbf7b1457d35ece4be076`.

Scoped installed-binary smoke command:

```bash
timeout --signal=TERM --kill-after=5s 45s xvfb-run -a -s '-screen 0 1920x1080x24' \
  env WAYLAND_DISPLAY= RUST_LOG=warn /home/iiii/.local/bin/tbg-moba \
  --capture /tmp/tbg-p77-linux-update/installed-overview.png \
  --seed 11 --warp 60 --role mid --overview
```

Exit 0. Visually inspected the resulting 1600×900 image: three Shop cards,
strategic Seat HUD, map/minimap fog and refined scene loaded. No game or
shader errors; the log contains only Xvfb XSETTINGS/display-size warnings.
Local evidence: `/tmp/tbg-p77-linux-update/smoke.log` and the PNG above.
This supersedes the installation/initial shader-startup deferrals only.
Full visual/motion/gesture checks, tests, performance, Windows/web and
owner look/fun approval remain pending. No 1v1 package or installation
was changed.

### Territory vision revision (owner, 2026-11-03)

Implemented M-50/M-67–M-69: full live home-half vision, scouting only from
allied Troops/Minions across the river, and permanent original-sight circles
around destroyed enemy Towers. One shared integer Territory model drives
server visibility and the map/minimap mask. Tower sites come from the fixed
map and surviving public structures; no wire-schema change or new radius.
Updated the affected M-50 camp/boundary contract and added mirrored home,
river-crossing/retreat, destruction-only, exact-radius and new-match reset
scenarios in `contract_vision_territory.rs`. Formatting and `git diff --check`
pass. Native compilation/install and a scoped runtime observation will be
consolidated with the owner's subsequent five-card/presentation revision.
Tests are written, not yet executed; queue at consolidation:
`cargo test --locked -p tbg-moba-core --test contract_vision_territory -- --test-threads=1`,
`cargo test --locked -p tbg-moba-core --test contract_moba m5 -- --test-threads=1`
and `cargo test --locked -p tbg-moba-server --test room m52 -- --test-threads=1`
under the shared nonblocking lock and two-job policy. Windows/web coverage
remains pending; no visual parity or full verification is claimed.

### Five-card revision (owner, 2026-11-03)

Restored the original five-card row and hotkeys 1–5. Slots 1–3 use the
Role's allowed Archetypes; slots 4–5 use all Archetypes from the full
roster. Locks, rerolls and purchases keep all five offers distinct. Protocol
v3 carries five offers and masks. Updated M-19/M-20 and their existing
boundary test; added many-draw checks proving both open slots draw outside
the Role and that a locked fifth card survives a fourth-card purchase.
Formatting and whitespace pass; focused compilation/install and runtime
will follow the champion presentation/ability work in this request.
Queue `cargo test --locked -p tbg-moba-core --test contract_moba m20 -- --test-threads=1`
at consolidation, under the shared lock with two Cargo jobs. No tests or
extra platform builds ran on this front.

### Automatic abilities and body spacing (owner, 2026-11-03)

M-70/M-71 activate shared base champion abilities in the authoritative
5v5 simulation. Cadence, heal, control, shields, counters, pulses, summons
and hit attribution advance by fixed ticks. The 2D adapter uses swept body
movement, disk contact for area payloads, aimed segments, and Team-filtered
combat cues. The client compiles the 1v1 weapon release/strike/footfall
system and uses its champion rigs, clips, projectiles and impact recipes.
Copied completion/flame and cask recipes are kept separate from the
BattleLine snapshot plumbing. The unrestricted Shop caption is localized.
No purchased Evolution choices or 1v1 input files changed.

Validation at consolidation (shared lock, two build jobs):

```bash
cargo fmt --manifest-path moba/Cargo.toml -p tbg-moba-core -p tbg-moba-client
git diff --check
cargo test --offline --locked --manifest-path moba/Cargo.toml -p tbg-moba-core \
  --test contract_champion_combat --test contract_vision_territory --test contract_moba \
  -- --skip m30_a_ten_bot_match_always_ends --test-threads=1
```

Results: format/whitespace passed; 4 champion combat, 3 territory and 22
existing MOBA contracts passed (29 total; one full-match contract skipped).
The combat checks cover solid contact plus melee reach, Weaver's real
windup/root, Mantis's fourth-hit bonus and poison credit after caster death.
Full workspace, balance, full champion motion gallery and Windows/web
visual correspondence remain deferred.

### Consolidated Linux playtest installation (2026-11-03)

`moba/scripts/install-linux-moba.sh` built the client and room server
with `CARGO_TARGET_DIR=/data/tbg-moba-target` and the previously recorded
`PKG_CONFIG_PATH` (37.87 s, dev profile). The existing `TBG Arena 5v5`
menu entry and both local commands resolve to `0.1.0-f6d2b000f3ac`;
`--version` reports MOBA protocol 3. Installed and cached client SHA-256:
`c77a43d0e97a52b264b27bf9fe7e45f729590f94b749688f0ca79789b8afb348`.

```bash
cargo test --offline --locked --manifest-path moba/Cargo.toml -p tbg-moba-server \
  --test room m52 -- --test-threads=1
timeout --signal=TERM --kill-after=5s 45s xvfb-run -a -s '-screen 0 1920x1080x24' \
  env WAYLAND_DISPLAY= RUST_LOG=warn /home/iiii/.local/bin/tbg-moba \
  --capture /tmp/tbg-p77-final-linux/overview.png --seed 11 --warp 60 --role mid --overview
```

M-52 passed (1 room contract, 3 filtered; 30 focused tests including core).
The installed smoke exited 0. Inspected the 1600×900 overview: five cards,
Portuguese unrestricted captions, the Destination panel, full home-half
sight, dim enemy half and local scout reveal appear in map and minimap;
units are present and both Teams have scored. The smoke log has only Xvfb
XSETTINGS/display-size warnings. This screenshot does not prove full
motion, gestures, per-champion correspondence, performance, or parity
with Windows and desktop web; those checks remain pending.

### Walk continuity and crowded exits (M-72)

The 5v5 arrival-time interpolation could repeatedly slow, stop or jump.
Local playback now uses the simulation accumulator and the actual prior
snapshot. Online playback buffers two ticks and slews a monotonic server
clock. The shared Animator now accounts for the threefold parent scale
when advancing the gait. Terrain grounding samples the existing road
height and the bounds of the physical bridge decks.

Each purchase or wave spawns immediately at the chosen exit. Same-team
mobile units spread with bounded pressure using PK-01 body sizes, with
lateral bias on the road. This is a planar adaptation of the BattleLine
spacing and persistent bypass rules. The 1v1 longitudinal slot solver
does not run in the 5v5 map. Opponents and fixed structures still use
swept body contact. The earlier waiting-queue approach was superseded
by the owner's immediate deployment requirement.

Focused core results: 22 MOBA contracts, 4 champion combat contracts and
2 deployment contracts passed. The deployment fixture buys eight Colossi
at once on each side, including four for one exit. It checks immediate
appearance and forward progress from each spawn position. It also checks
a static obstacle bypass. The 25-minute bot match was excluded. Full
motion comparison and Windows/web verification remain pending.

Installed Linux build `0.1.0-e48e3108e9ad` using the local 5v5
installer (29.56 s). The installed binary exited successfully after a
base capture with `--seed 11 --warp 57 --role bot`. The inspected image
shows units beyond the base plaza on the roads, with no game or shader
errors in the log. Evidence: `/tmp/tbg-p77-route-exits/base.png` and
`/tmp/tbg-p77-route-exits/smoke.log`. This is a startup and placement
observation. The complete motion comparison remains pending.

### Scaled champion attack reach (M-18)

The figures and collision bodies were three times larger while the
printed basic range was still in unscaled BattleLine coordinates.
Combat acquisition also stopped at eight units even for longer printed
ranges. Use the existing 3× figure scale for basic reach and explicit
cast distances, and scan at least the weapon's range. Keep Team vision
and area radii independent. The shared Animator and projectile recipes
consume the same authoritative positions across platforms.

28 focused core contracts passed across `contract_champion_range`,
`contract_champion_combat` and `contract_moba`, excluding the 25-minute
bot match. The new tests cover every Shop champion and a Farwatch
firing at 36 units without stepping toward the target.

Installed Linux build `0.1.0-2b7775169a4b` with the local installer
(9.90 s). The installed client completed the 60-second Bot overview
capture with exit 0 and only Xvfb display warnings. Evidence:
`/tmp/tbg-p77-range/playtest.png` and `/tmp/tbg-p77-range/smoke.log`.

### Committed aggro and Nexus exits (M-07, M-72)

Target selection preserves an eligible focus when other enemies move
closer. It chooses a new target only after invalidation or leaving the
pursuit radius. Heals and control casts retain valid recipients. Shared
rendering faces the interpolated target during pursuit and lets stale aim
expire after the attack clip. Purchases and waves originate at the
Nexus plinth. Both Nexus Towers and the Mid Base Tower were moved off
the Mid road's clearance envelope so large champions can leave it.

34 focused contracts passed across aggro, deployment, champion range
and combat, territory vision and existing MOBA rules (full bot match
excluded). The deployment test verifies the forward position and
distance of all eight immediate spawns from each Nexus, then checks
their advance. The aggro test changes the nearest enemy during pursuit
and verifies retargeting after the locked target leaves. Full desktop
motion comparison remains pending.

The Linux installer built and installed `0.1.0-0a0afee693ab` in
13.04 s. The installed client exited successfully after the local
`--seed 11 --warp 16 --role mid` capture. The inspected image shows
the clear Mid corridor and Minions on the three lanes. Xvfb reported
display warnings only. Evidence: `/tmp/tbg-p77-nexus/base.png` and
`/tmp/tbg-p77-nexus/smoke.log`.

## 10. After P77 (renumbered backlog)

P78 champion Evolution forms and coverage beyond the automatic base kits ·
P79 online rooms for real players · P80 presentation polish (sound,
scoreboard, settings) · P81 Windows and desktop web · P82 balance (only on
explicit owner request).


### M-73–M-75 correction delivery and installed Linux motion (2026-11-03)

**Level:** owner-requested focused consolidation and local Linux installation.
Worktree `/data/tbg-p77-moba-roles-vision-map`, branch
`phase/p77-moba-roles-vision-map`. Implementation commits `22827552dd93`
and `0a7b3f241141`. No push, merge, deployment, full workspace suite,
balance gate, release build or Windows/WASM build was run.

Implemented camera edge removal with captured left dragging; persistent
Lane/Jungle missions and cargo mission inheritance; deterministic body-safe
routes with continuous melee approach, fixed detours, progress/rejection
rules and soft allied pressure; shared water/crossing/plaza geometry;
per-death Gold/Essence receipts, personal versus Team HUD notices and
vision-safe origins; authoritative activity driving shared 1v1 gait/attack
posture. The plaza radius is Proposed 12 instead of 10 so existing widest
bodies have a valid crossing. Champion reach remains 3×; economy values,
neutral retaliation/leash, immediate Nexus spawning and the 1v1 sources
remain unchanged. Wire v4 carries receipt/activity additions.

**Reproduction and fixes.** The initial 40-case Jungle scenario exposed
incomplete clears, leash resets and a body contact position missed by a
coarse route lattice. The final scenario uses Shieldbearer, BoneBrute,
CargoCrab, Bonehorn and Colossus against all eight Camps, with 20× HP and
4× basic damage in the fixture only so the assertion concerns navigation
and attack completion rather than balance. All 40 combinations cause damage
and clear their Camp within the bound. The first Linux motion run additionally
exposed pressure pinning small bodies at a route's first seed cell, and
cargo contents inheriting Static. Removed redundant route corners by swept
visibility and kept the parent's mission on the immobile crate. Added a
mixed eight-champion Nexus/cargo regression; the final installed run no
longer showed that initial cohort accumulated at the Nexus.

**Checks actually run.** All compiling commands used the nonblocking shared
lock, two Cargo jobs and `/data/tbg-moba-target`; tests used one test thread.
The final source passed 28 distinct focused tests across these selections
(repeated affected selections only after relevant fixes):

```bash
# From this worktree; reuse for every Cargo command below.
export CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-moba-target
export PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock   cargo test --locked --manifest-path moba/Cargo.toml -p tbg-moba-core   --test navigation --test contract_deployment --test contract_aggro   --test contract_champion_combat -- --test-threads=1
# The newly added mixed_crowd scenario was rerun alone after its fixture
# excluded unrelated enemy-wave deaths; it passed. Production code unchanged.
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock   cargo test --locked --manifest-path moba/Cargo.toml -p tbg-moba-core   --test navigation mixed_crowd -- --test-threads=1
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock   cargo test --locked --manifest-path moba/Cargo.toml -p tbg-moba-core   --test contract_champion_range --test rewards -- --test-threads=1
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock   cargo test --locked --manifest-path moba/Cargo.toml -p tbg-moba-core   --test contract_moba m6 -- --test-threads=1
flock -n -E 75 /tmp/tbg-validation-$(id -u).lock   cargo test --locked --manifest-path moba/Cargo.toml -p tbg-moba-core   --test contract_moba m40 -- --test-threads=1
```

Final evidence totals: navigation 7 passed; deployment 2; aggro 1;
champion combat 4; champion range 2; rewards 5; M-60–M-65 selection 6;
M-40 wire round-trip/hostile frames 1. Reward scenarios include real splash,
stored lethal poison after source removal, owned summon credit, simultaneous
source death, all monster Gold/Essence values, ownerless individual kills,
Tower payment to all five allied Seats, once-only grants and hidden-origin
redaction. Replay scenarios compare every state hash and encoded Team World
under competing targets beside water and the rim. Large and small lane
walkers cross valid geometry and retain their original Lane.

Native client `cargo check --locked --manifest-path moba/Cargo.toml
-p tbg-moba-client --bin tbg-moba` passed (1.88 s at the first consolidated
revision). The final installer compiled both client and server with the same
lock and two jobs: `Finished dev profile ... in 30.44s`, then
`Installed TBG Arena 5v5 0.1.0-0a7b3f241141 (dev profile)`.
Package formatting, `git diff --check` and contract JSON parsing passed.

**Installed artifacts.** Menu entry `TBG Arena 5v5`, and both local binary
links point into `/home/iiii/.local/opt/tbg-moba/0.1.0-0a7b3f241141/`.
Client SHA-256: `368c53be52ac2898c0d8b820789ce1df0c0ed21c7a5d0f5484c1357d8d82b681`.
Server SHA-256: `4bbcb46d20d82c85b9b102d1a467c42f73c65fd0b1c104462318b0bed3ffc182`.
No 1v1 source, root Cargo manifest/lock or archive file was changed by this work.

**Linux motion/input evidence.** Ran the installed binary in an isolated
native X11/Xvfb session, 1600×900, seed 11, 90-second offline warp, Jungle
Seat; actual XTest mouse/keyboard input, 20 fps video plus per-frame rig and
unit telemetry. This is a scoped native Linux observation, not owner device
sign-off or a complete roster/performance certification. Saved evidence
outside Git at `/data/tbg-moba-logs/p77-missions-0a7b3f241141/`:

- `linux-motion.mp4` (53.4 s), `input-results.json`, `motion.csv` and
  `analysis.json`: left/right/top/bottom and two corner holds, release,
  focus loss/return, drag across the HUD, HUD-origin gesture, minimap,
  wheel and Space-to-base. Each stationary interval had exactly one
  camera pose across all its recorded frames (18–26 frames per short
  hold; 513 in the 30-second fixed-camera observation). Drag capture
  cleared on release/focus loss; no new press meant no resumed dragging.
- `jungle-melee.mp4` (18 s), `golems.png`, `golems-{3,9,15}.png`: melee
  contact/strikes, retargeting between Golems, damage, monster removal and
  subsequent route resumption. Sawbot and Rootcaller later continued toward
  the next Camp. Native sequence shows the `+6 Ouro · Pessoal · Selva` notice.
- `team-notice.png`, extracted at 21.5 s of the first video, shows both
  `+5 Ouro · Pessoal · Selva` and `+25 Ouro +2 Essência · Equipe · Torre`.
  Correct recipients and once-only grants are established by core tests;
  the screenshot demonstrates their distinct HUD presentation.
- The sampled motion analysis contains 74,388 champion-frame records,
  including 8,807 Attack records with displacement and **zero Attack/Walk
  records**. Facing stayed within the existing shared 4π rad/s turn limit
  (log rounding gives a measured maximum 12.569 rad/s). This supports the
  pressure/stance fix in the observed scenes; it does not certify absence
  of every possible roster animation issue. The evidence contains changing
  frames and telemetry, not just a static screenshot.
- `manifest.json`, `final-tests.log`, `client.log` and input scripts record
  identity and reproduction. Runtime emitted only the Xvfb XSETTINGS/display
  size warnings, with no panic. Test windows/processes created here were closed.

**Deferred, not passes.** Windows and desktop web matching pixel/motion
captures for the P77 L cases; full Linux roster motion and owner look/fun
sign-off; online two-client receipt/camera/vision journey. The exact queued
scenarios are stationary edge/capture/focus, visible and hidden-origin
personal/team receipts, eight Camps with both body sizes, mixed Nexus
crowding, water/rim pursuits and post-combat mission continuation. A scoped
WASM compile remains `flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/data/tbg-moba-target cargo check --locked
--manifest-path moba/Cargo.toml -p tbg-moba-client --bin tbg-moba --target
wasm32-unknown-unknown`; run at cross-platform consolidation, not as evidence
of visual correspondence. No full-suite or release-readiness claim is made.

### M-76 decision-engine follow-up (2026-11-03, in progress)

Owner report: deployed Troops still stall and repeatedly switch targets.
The focused moving-contact regression reproduced loss of a reachable living
Shieldbearer target at tick 40. An incremental A* experiment restored long
crossings but failed the Colossus camp-clear scenario twice.

> blocked: incremental planner integration; tried bounded partial paths,
> then persistent search frontiers. Exact remaining failure:
> `melee_reaches_damages_and_clears_every_camp`: Colossus Camps 1, 3, 5, 7
> deal damage but do not clear within the existing 120-second fixture.
> Suspected cause: interaction between refreshed contact positions, crowd
> movement and leash. This planner attempt is withdrawn; do not ship it or
> claim its validation. Continue the independent decision/aim changes and
> use the previous proven route solver for the contact correction.


The final implementation restores the previous complete route search and
adds continuous bridge-centre graph edges. The narrow-lattice issue was
reproduced from native seed 11: Bulwark/Sawbot radius 3 cannot fit any grid
centre inside the bridge's remaining centre corridor. A separate Leechroot
was born from cargo on invalid terrain. Continuous bridge connections and
same-bank placement resolve those cases without changing map width.
A moving patient now remains committed while followed into legal action range.

Consolidation: 42 focused tests passed on the integrated M-76 working tree,
including 13 decision scenarios, 7 navigation scenarios (40 melee/Camp
combinations), aggro 1, champion combat 4, range 2, deployment 2, rewards 5,
M-6x 6, M-08 1 and M-40 1. The initial 46-champion exit matrix is a bounded
mechanics check, not full roster visual acceptance. Native client/server dev
build passed in 18.09 seconds. All compilation/tests held the shared
nonblocking lock with two Cargo jobs and one test thread. No full suite.
The exact command batch and logs are retained for the final evidence record.
Native final motion/installation results follow after observation.

Follow-up native finding on `2b75170252da`: a Lane route could fail globally
because a distant mobile fight occupied the future waypoint. The Unit then
never approached acquisition range. M-76 now plans unengaged Lane travel
against terrain/permanent structures, while every actual step still checks
current bodies and nearby combat acquires normally. Local Jungle/neutral
routes retain contested-body planning; applying the relaxed mission search
to Jungle first failed Bonehorn Camp 4, so it was restricted to Lane missions.
The focused rerun passed all 14 decision and 7 navigation tests. Deployment
also passed with the same Lane planning change. Together with the unaffected
consolidated contracts, 43 distinct focused tests passed. Logs preserve the
intermediate failure and the successful final scoped rerun.

Native follow-up on `54f755cfb109` recorded 1,114 authoritative ticks:
0.820 ms median, 2.278 ms p95, 21.404 ms maximum core tick time in this
instrumented dev/Xvfb sample. No target reacquisition cycle within 40 ticks
was recorded. Two mission stalls remained at passive Camps (Ravager and
Leechroot): Lane units cannot attack those neutrals, so they must remain
route obstacles. The final filter keeps neutral bodies and immobile allies,
while still allowing approach to a distant hostile front. A regression uses
both observed positions and asserts departure with neutral HP unchanged.
All 15 decision plus 7 navigation tests passed after this correction.
