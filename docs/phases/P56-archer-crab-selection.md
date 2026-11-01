# P56 — Archer Kit, Cargo Crab Rework and Unit Selection

**Status:** implemented on `phase/units-archer-crab-selection` (local only, no push); ready for reviewer and human look approval.
**Branch:** `phase/units-archer-crab-selection` branched from `consolidate/main-20261030` (`b46676b`).

## Overview & Scope

This phase implements three major fronts in Tupi Battle Grounds:
1. **Archer Kit Refinement:** Migrates the Archer from legacy A-1 figure code into the standard visual kit library (`art/kits/archer/` with `model.rs` and `mod.rs`), providing 5 distinct forms, dedicated animations (full draw, aim elevation, 0.18s release recoil), own projectile dispatch, and eliminating legacy `arrows::release` / `anim.archer` double-arrow paths.
2. **Cargo Crab Rework (Visual & Core Logic, D-56):**
   - *Visual:* Upgrades the crab into a robust orange machine with heavy pincers, articulated legs, and an elevated passenger saddle mounted on the chassis. Adds a dedicated boarding animation (`Clip::Cast`) where the claw scoops up an ally, lifts it in an arc, and seats it in the saddle; mounted passenger sits in a dedicated saddle posture (`apply_mounted`) moving as one with the crab.
   - *Core Logic:* Server-authoritative deterministic pairing. Nearest eligible ally within 0.5 tile (tiebreak lowest ID) boards. United pair moves at `max(crab, passenger) * (1 + 20% carry boost)`. Single-target attacks target the crab first (pair is one single-target entity); area/splash damage continues reaching the passenger. Upon crab death, the passenger dismounts 0.2 tile behind the crab with its current HP intact and a 3s re-boarding lock.
   - *Protocol:* `PROTOCOL_VERSION` bumped to 16 to pack `FX_CARRIED` (`1 << 14`) in `UnitView.fx`.
3. **Unit Selection (Front-End Only):**
   - Troop nameplates are hidden by default to eliminate visual clutter on crowded battlefields.
   - Clicking any troop (ally or enemy) selects it; exactly one troop selected at a time.
   - Clicking outside (ground or sky) clears selection.
   - Selected troop receives a clean HUD-styled nameplate above its HP bar (which tracks elevation when mounted), a prominent luminous pulsing base ring, and subtle focus elevation (+0.07 Y). Selection automatically clears when the unit dies or upon match exit.

## Commits on Branch

1. `b4a4eb3` `feat(client): migrate Archer to standard art kit`
2. `bc2942b` `feat(core,client): rework Cargo Crab visuals and united boarding mechanics (D-56)`
3. `573e59e` `feat(client): implement clean unit selection and selective nameplates`
4. `d1816d2` `style: format files according to rustfmt`
5. `d591e75` `fix(balance,client): resolve clippy warnings`
6. `ba8fb3b` `docs: add P56 phase report, roadmap entry, and visual evidence`
7. `0c881ad` `style(balance): format cargo_crab_balance example with rustfmt`

## Proposed Game Design Numbers & Balance Evidence (D-56)

### GAME-DESIGN §11 Entry (Proposed)
`| Cargo Crab | Machines | Tank | 5 | 1 400 | 20 | 1.5 | 0.8 | 0.5 | Carries one non-Tank ally: pair moves at max(crab, passenger) × 1.20; single-target attacks hit crab first; dismounts on death with intact HP and 3 s lock (D-56, Proposed) |`

### Offline Balance Lab (`cargo_crab_balance`)
Evaluated across 1,024 matches with paired seeds and side inversion (Seat A and Seat B swapped) against representative roster champions (`Shieldbearer`, `Archer`, `BoneBrute`, `Hexer`, `Sawbot`, `SiegeWalker`, `Bloomkeeper`, `Rootcaller`):

| Configuration | Matches | Win Rate | Draw Rate | Avg Duration | Passenger Survival |
|---|---|---|---|---|---|
| Cargo Crab (Alone) | 1 024 | 62.5 % | 12.5 % | 36.76 s | N/A |
| Cargo Crab + Archer (Mounted) | 1 024 | 81.2 % | 0.0 % | 14.71 s | 81.2 % |
| Cargo Crab + Bloomkeeper (Mounted) | 1 024 | 50.0 % | 18.8 % | 36.53 s | 68.8 % |

**Key Findings:**
- The +20% carry boost (`2 000` bp over `max(crab, passenger)`) allows the pair to close distance quickly and establish frontline pressure.
- Single-target absorption successfully insulates squishy damage dealers (Archer survival rate 81.2%), providing massive value when paired with high-DPS units.
- Geometry counterplay holds: Area splash damage (e.g. Siege Walker) reliably damages the passenger sitting at `distance = 0`.

## Verification Results

### 1. Formatting
```bash
cargo fmt --all -- --check
# Clean (exit 0)
```

### 2. Clippy
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
# Clean, zero warnings (exit 0)
```

### 3. Core & Client Unit / Contract Tests
```bash
cargo test -p tbg-core
# All contract and rule tests passed, including contract_cargo_crab (7 passed)

cargo test -p tbg-client
# All 94 tests passed, including every_kit_form_builds_within_the_doubled_budget,
# every_kit_has_five_distinct_forms, kits_keep_the_simulation_event_times,
# and selection_lifecycle_select_switch_deselect_and_unit_death.
```

### 4. WASM Target Check
```bash
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
# Compiles cleanly without errors
```

### 5. Web Release Build (Trunk)
```bash
cd crates/tbg-client && trunk build --release
# Built successfully into dist/
```

## Visual Evidence (`docs/media/p56/`)

- `archer_lineup.jpg`: Complete Archer kit lineup showing 5 distinct forms (Base, A1 Longbow, A2 Marksman, B1 Volley, B2 Arrow Rain) across idle, walk, and bow draw/aim/attack.
- `archer_range.jpg`: Live firing range with Archers taking aim and loosing arrows against enemy Shieldbearers down the lane.
- `cargo_crab_lineup.jpg`: Complete Cargo Crab lineup showing orange machine aesthetic with brass chassis, articulated mechanical legs, heavy pincers, passenger saddle, and 5 distinct variants (Armored Carrier A1/A2, Turbo Carrier B1/B2).
- `unit_selection.jpg`: Practice match with 40-unit armies demonstrating clutter-free battlefield (all names hidden by default, clean HUD presentation).

## How to Test in Browser

1. Start authoritative server: `cargo run -p tbg-server`
2. Start web client dev server: `cd crates/tbg-client && trunk serve --port 8080`
3. URLs for inspection:
   - Archer Kit Lineup: `http://localhost:8080/?gallery=archer&lineup` (1600×900)
   - Archer Range Practice: `http://localhost:8080/?gallery=archer&range`
   - Cargo Crab Lineup: `http://localhost:8080/?gallery=cargo_crab&lineup`
   - Practice Match with Crab Boarding & Unit Selection: `http://localhost:8080/?autoplay=practice&army=40`
     - Click on any ally or enemy unit: tag appears above HP bar, luminous base ring pulses, unit figure slightly elevates.
     - Click on ground or sky: selection clears, tag disappears.
     - Spawn Cargo Crab and Archer: crab boarding pincer animation triggers, Archer sits mounted on saddle and shoots while pair moves together at +20% speed.

## Real Limitations Declared

1. **Balance Lab:** Measured in headless duels with paired seeds and side inversion. It models unit-to-unit interactions and speed dynamics accurately, but does not capture late-game 15-node Arcana synergies, tower aura combinations, or shop cycling economics.
2. **Subjective Approval:** Visual styling of the orange machine crab and Archer kit aesthetics require human review by the owner.

> **Read this first.** Sections above this one describe P56 as it stood before Part A. Where they disagree with this report (protocol 16, the selection test, the Cargo Crab carry bonus), this report, D-60 and D-61 are the record.

## Part A report: units fixes (branch `phase/units-fixes`, 2026-10-30)

Part A is integrable alone. It is built on `origin/main` (merged at `e3b977f`, which carries PR #17), with the Cargo Crab rule branch `phase/crab-rules` merged in. Nothing is pushed, merged into `main`, or deployed.

### Commits (oldest first)

| Commit | Item | Subject |
|---|---|---|
| `dc4f9a9` | A1, A2 | The Archer runs on its kit with a curved, drawable longbow (native path removed) |
| `69e0802` | A4 | `UnitView.carrier` names a passenger's carrier (protocol 18) |
| `d8d8eef` | A4, A5 | Passengers sit on their carrier's saddle and board with an arc; crab visual rebuilt |
| `231c667` | A6 | A selected unit's name is the only one shown, and it follows |
| `99dfc65` | A3 | Merge of `phase/crab-rules`: carry bonus is the larger of base and branch (D-60) |
| `3a632af` | fix | Clippy clean-up after the archer and passenger changes |
| (docs commit) | | D-61, D-47 update, this report, ROADMAP row |

### What was fixed

- **A1, Archer bow.** The old bow was two straight cylinders and a fixed string. Now each limb is four segments ending exactly on its tip (`gear::limb_points`), so the bow is an arc. The string's two halves pivot on the limb tips and turn toward the nock on draw (`gear::bow_string`), forming a V, snap at Release (0.18 s) and wobble after it. The nocked arrow shows only from the start of the shot until Release (`Attack::Shot.loaded`), so the projectile replaces it and no second arrow appears. The bow is held upright in idle, walk, aim and hit (`gear::bow_upright`), so its long axis is vertical and the shot runs level. Arrow Rain (B2) fires its own `ShotKind::ArrowStorm`: smaller, faster arrows with a spark trail. Rig: `Socket` gains `StringTop`, `StringBottom` and `Nock` as children of the bow socket.
- **A2, native Archer path removed.** `archer_v2`, `archer_v2_clips`, `archer_v2_tests`, `archer_aim`, `bow.rs`, `arrows.rs`, `roster::humans::archer_v2` and its gear, `roster::archer_v2`, `Animator.archer`, the `bow_half` field, and the native branch in `build_library`. `Easing::Hold` went with them (its only callers were the native poses). `sync/events.rs` no longer needs `is_archer`. Kept: the Shieldbearer native figure (`roster::humans::shieldbearer_v2`), `legacy.rs` and `adapter.rs` (the 12-joint compatibility layer that the other native figures still use), and `roster::blockout` (review blockouts for every kind).
- **A3, Cargo Crab rule (D-60, merged).** The carry bonus is `max(base, branch)`, not `base + branch`. Base +20 %; B1 +20 %; B2 +20 % and +40 % during its surge; cap +50 % (BR-X03). Contract tests cover A1, A2, B1, B2 with and without surge, the cap, and a slowed passenger. The old test that pinned `PROTOCOL_VERSION == 16` is gone; `contract_protocol` pins 18. The balance measurement (`cargo_crab_balance`, 128 pairs with a baseline) is **not run**: the owner postponed balancing. The rule is Proposed.
- **A4, carrier id and mount.** `UnitView.carrier: Option<UnitId>` (D-61, protocol 18). `visuals/mount.rs` seats a passenger on its carrier's real `x, z`, at the saddle height from the kit (`cargo_crab::model::seat`, scaled by the form's growth), with its hips on the seat (`hip_height` from the passenger's own plan). It runs after `sync_units`. `Scene::place` is not edited (the P57 branch edits it). Its carried-passenger block still sets `pos.z = lane_z(crab.id)`, but `mount` overwrites that position each frame; **delete that block once P57 lands**.
- **A5, boarding and visual.** `Event::Boarded` inserts `Boarding` on the passenger: a 0.5 s arc from its place into the saddle, while the crab plays `Cast`. A view that stops carrying a passenger starts `Descent`: a 0.4 s step back to the ground. A reconnect sends the full view with the carrier, so passengers appear seated with no arc, because the arc only starts from the event. The Cargo Crab is rebuilt: a saturated forge-orange carapace on a deeper belly; thick piston legs in place of rods; broad plated and toothed pincers in place of thin cones; the saddle keeps its height.
- **A6, selection.** `visuals/names.rs` splits the rule from the projection: `show` (the inspected unit, while alive; orphan labels despawn) and `position` (desktop only; off-screen hides). The old `ui/inspect.rs` test that only wrote and read the resource now runs the death and match-end systems. `names.rs` has four system tests on two units: selecting shows one name, switching moves it, deselect hides it, a dying unit hides it, and match end despawns the names with their units.

### Verification run (exact results)

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test -p tbg-client --bin tbg-client`: 100 passed, 0 failed (this includes the four archer geometry tests, the four name-tag tests, the selection and kit tests, and `every_kit_form_builds_within_the_doubled_budget`).
- `cargo test -p tbg-core --features lab`: every test binary `ok`, 0 failed (includes `contract_protocol`, `contract_cargo_crab` (10 passed), `contract_determinism`).
- `cargo test --workspace --exclude tbg-balance`: `cargo test --workspace --exclude tbg-balance`: 44 test binaries, **316 passed, 0 failed, 1 ignored** (the ignored test is a full 1x match that takes 2 to 4 minutes; the code marks it for manual runs).
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: exit 0.
- `trunk build --release` in `crates/tbg-client`: `trunk build --release` in `crates/tbg-client`: success, built from the final sources (a forced rebuild after the last edit).
- `node --test tests/*.test.js` in `crates/tbg-client`: `node --test tests/*.test.js` in `crates/tbg-client`: 30 passed, 0 failed.

Not run, by choice: `cargo test -p tbg-balance --test contract` (the owner asked for minimal tests; C7 is fixed on `main` by D-58, so it can run later) and the balance lab matrices.

### Visual evidence (`docs/media/p56/`)

Captured in the gallery at 1920×1080 with headless Chrome (GPU flags), from the release build:

- `archer-lineup-before.jpg` / `archer-lineup-after.jpg` (`?gallery=archer&lineup`). Before: the bow is a straight stick pointing forward. After: the bow is a curved longbow, vertical in idle and walk, and in the attack row the string is drawn in a V with the arrow nocked.
- `archer-range-before.jpg` / `archer-range-after.jpg` (`?gallery=archer&range`).
- `cargo-crab-lineup-before.jpg` / `cargo-crab-lineup-after.jpg` (`?gallery=cargocrab&lineup`). Before: thin rod legs and thin cone pincers. After: thick piston legs, plated blocky pincers, a more saturated forge-orange carapace and a darker belly. The saddle cushion is visible in both.
- `cargo-crab-cast-after.jpg` (`?gallery=cargocrab&frames&clip=Cast`): contact sheet of the boarding gesture at 0, 25, 50 and 75 %.
- `unit-selection-before.jpg`: the P56 selection capture. No after capture was made.

**Not captured:** a 1920×1080 match-camera close-up, the boarding arc inside a live match, the mounted pair walking and attacking, the dismount, and a click-through selection. Practice does not force a crab to board, so these need a scripted match before they can be captured. The gallery shows the animations, not the match behaviour.

### Limitations

- **Look sign-off is the owner's.** The crab and the bow are built to the style guide, but approval is a human decision.
- **The mounted pair and the boarding arc are not captured in a live match.** A practice match does not force a crab to board, so the arc is covered by reasoning and by the event path, not by a screenshot.
- **The carrier id costs one postcard tag byte per unit per view.** Not measured yet; if the bytes matter, a per-view `carried` list costs nothing when empty (D-61 reopen condition).
- **Selection in a live match** is covered by the system tests and the browser gallery only; a click-through capture was not made.
- **Balance** of the carry rule is deferred by the owner. The Shop and online pools are unchanged.
