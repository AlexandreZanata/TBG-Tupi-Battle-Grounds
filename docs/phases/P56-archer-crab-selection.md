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
