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

## 10. After P77 (renumbered backlog)

P78 1v1 champion abilities and Evolutions in 5v5 (automatic, no Commander kit/orders) ·
P79 online rooms for real players · P80 presentation polish (sound,
scoreboard, settings) · P81 Windows and desktop web · P82 balance (only on
explicit owner request).
