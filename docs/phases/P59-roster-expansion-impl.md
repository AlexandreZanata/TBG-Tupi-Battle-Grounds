> **Owner authorization update (2026-10-31):** implement all 21 character kits
> and catalog abilities, one commit and real screenshot per character. Follow
> [P59-EXECUTION](../roster/P59-EXECUTION.md). Earlier deferrals below describe
> the previous scope.

# P59 — Roster expansion: implementation (status report)

Status: **W0 and W1 are done** (D-62 and D-63 for W0; D-64 for W1).
The owner-authorized complete roster front is active in draft PR #22, stacked on
PR #20. Shared anatomy/gallery support is implemented (D-67). The four Monsters (R08–R11) now
have dedicated five-form kits and authoritative abilities (D-68–D-71), with
screenshot/minimum validation evidence in [P59-EXECUTION](../roster/P59-EXECUTION.md).
The other 17 character deliveries remain pending. No new kind is in a public Shop/lab pool;
this is an implementation report, not W10 balance acceptance or W11 release.

## What is in place

| Item | Where | State |
|---|---|---|
| Planning, renumbered from P56 to P58 | `docs/phases/P58-roster-expansion-planning.md`, `docs/roster/P58-*.md` | merged |
| Registry in GAME-DESIGN §13 (Proposed; §13.7 records decisions taken) | `docs/GAME-DESIGN.md` §13 | Proposed |
| Part A: archer kit, carrier id (protocol 18, D-61), passengers, crab rule (D-60), name tags | `phase/units-fixes` | merged here |
| W0: six shared slots and the unit shield grant path (D-62, D-63) | `crates/tbg-core/src/roster_slots.rs`, `UnitFx`, `UnitView.shield` | commits `a0c71b9`, `bd9964f` |
| W1: three Factions, arrays sized by `FACTION_COUNT`, Insects movement synergy (D-64) | `crates/tbg-core/src/types.rs`, `content.rs`, `ruleset.rs`; palette, death styles, gallery | commit `c968b2c` |
| Monsters pilot: four inactive base kinds, no abilities or kits (D-66) | `UnitKind`, `UNITS`; outside every pool | commit `2b28591` |
| English names for the roster (D-65) | `docs/roster/P59-NAMES.md` | decided |

**Protocol numbers.** The shield took 19 (D-63) and the Factions took 20 (D-64).
R08 self-reduction took 21 (D-68), R10 cue 22 (D-70), and independent special
snapshots/cues took 23 (D-71). The plan said 19 for the Factions. The shield changed the wire first, so the
Factions take the next number; this deviation is recorded here.

**The audit's protocol number is stale.** P58 §2 reads "protocol 16". The
current value is 23 (D-71). Update the audit at consolidation.

## Waves

- **W0: shared contracts. Green (D-62, D-63).** Six slots in `UnitFx` (bleed,
  weakening, queued post-Root Slow, caster's mark, timed self-reduction, ability
  heal-cut), all inert until a unit uses them. The unit shield grant path with
  expiry, and the `UnitView.shield` field (protocol 19). The view and event
  extensions for the six slots wait for the first unit wave that shows an effect,
  and that wave carries its own protocol bump.
- **W1: factions. Done (D-64, protocol 20).** Monsters, Celestials and Insects are
  appended. `FACTION_COUNT` sizes every array. Synergy at 500 / 1 000 bp. Palettes,
  death styles and gallery rows cover the seven Factions. ART-DIRECTION §3 and
  §11.2 have rows for the three new Factions (Proposed).
- **Pilot: Monsters R08 Bonehorn, R09 Ravager, R10 Acidmaw, R11 Whisperer. Kinds done (D-66).**
  Inactive kinds and base rows from GAME-DESIGN §13.1 are registered. All four
  now own their kits, animation and abilities; their first desktop lineups were observed.
  Per-character evidence is recorded in P59-EXECUTION before each next delivery.
- **W2–W6: rules for the other 17 units** (timers, counted strikers, area pulses,
  shield pair, fusion). The 2026-10-31 owner update authorizes implementing
  the catalog abilities on this front; balance evidence remains deferred.
- **W7–W9: evolution visuals, models, animation profiles** for all 21 units and
  the three drills. This is the visual part of the mission, unit by unit after
  the pilot.
- **W10: tests.** Deterministic per-unit tests and the cross-cutting cases from
  the plan. Only a small laboratory sanity run; the full 256/2048-pair matrices
  are for the owner to run later.
- **W11: release to the online pool.** Owner decision, after look and fun sign-off.

## Owner items, answered by the agent (D-65)

The owner asked for the best choice on each open item, documented. Balance and
rule values stay Proposed for the balance agent.

1. **Bastion (R01).** Renamed Bulwark. The Stronghold kind keeps Bastion (D-46).
2. **Crematumba (R20).** Catalogue values kept as the Proposed base: cost 6, HP 800, movement 23.
3. **Brocatanque (R21).** Catalogue values kept: trigger 4 500, merge radius 200, drill range 600, Duobroca speed 73.
4. **Monsters Synergy damage.** 500 / 1 000 bp (D-64).
5. **Shield semantics and Cargo Crab `shield_bp`.** One shared path; the Cargo Crab value stays dormant (D-63).
6. **R21 with no merge partner.** The drillnest keeps producing; no merge happens. Implemented with R21.
7. **Shop gate floor.** Not changed here; the balance lab's gate belongs to the balance agent.
8. **Names.** All English (D-65, `docs/roster/P59-NAMES.md`).

## Not measured, not verified

- `UnitFx` size is not measured. Hashes moved in W0 and W1; the goldens were refrozen with tick, winner and reason unchanged (D-62, D-64).
- The carrier id from Part A costs one postcard tag byte per unit per view (D-61). The shield costs one byte per unit while no shield stands (D-63).
- PR #11 and #15 states were not re-checked (no network).
- Audit §1 and §6 were not reaudited; the "reviewed #16" daggers in §6 are stale.
- `crates/tbg-core/tests/contract_roster.rs:162` asserts `UNIT_KINDS.len() == 10`. It stays true until a kind joins the pool.

## Verification for the shield (commit `bd9964f`)

| Command | Result |
|---|---|
| `cargo fmt --all` (then `--check` clean) | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo test -p tbg-core --features lab --no-fail-fast` | exit 0; 215 passed, 0 failed |
| `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown` | exit 0 |

## Verification for W0 (commit `a0c71b9`, the core slots)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings; test code of every crate compiled, not run |
| `cargo test -p tbg-core --features lab --no-fail-fast` | exit 0; 212 passed, 0 failed |
| `cargo test -p tbg-balance --test contract` | exit 0; 7 passed, 0 failed; the binary took 530 s in a debug build |
| `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown` | exit 0 |

## Verification for W1 (commit `c968b2c`)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo test -p tbg-core --features lab --no-fail-fast` | exit 0; 215 passed, 0 failed |
| `cargo test -p tbg-balance -- --skip c7_planted` | exit 0; 26 passed (c7 skipped, see below) |
| `cargo test -p tbg-server --test ranked --test leaderboard` | exit 0; 10 passed |
| `cargo test -p tbg-client --bin tbg-client` | exit 0; 100 passed |
| `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown` | exit 0 |

Golden check for W1: the hashes in `contract_determinism` and `contract_lab`
moved once. Against the pre-W1 commit `bd9964f`, all five seeds gave the same
tick, winner and reason, and the old trace probe reproduced the old goldens
exactly before the refreeze.

## Not run

- `cargo test -p tbg-server` as a whole (the online and integration suites), and `cargo test --workspace`.
- `c7_planted_imbalances_are_caught_for_every_unit` in W1 (about nine minutes in a debug build). It passed in the W0 contract run.


## Integration review — 2026-10-31

Merged `main` (P47) and `release/p47-arenas` (`92829e8`) into the candidate,
retaining live battles, spectating, arena scenery, Shop portraits, tower-click
Fortification, and the Leave/Ranking fixes. Protocol stays 20. Roster values,
`ALL_KINDS`, `FULL_POOL`, `UNIT_KINDS`, deferred kits and dormant effects stay as
D-60–D-66 define them. No dependency was added.

The merge exposed a spectator integration error: `PublicView` still used four
Factions and called the old unit serializer. It now uses `FACTION_COUNT` and the
battle tick for shield expiry. A non-contract regression test checks that seats
and spectators agree on shields and Synergy, including the expiry tick. The
palette labels now name all seven Factions; its camera frames nine rows and its
existing studio floor keeps arena scenery from obscuring the swatches.

| Command | Observed result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo test -p tbg-core --features lab --no-fail-fast` | exit 0; 220 passed |
| `cargo test -p tbg-client --bin tbg-client` | exit 0; 106 passed |
| `cargo test -p tbg-balance -- --skip c7_planted` | exit 0; 26 passed; 216.6 s |
| `cargo test -p tbg-server --test ranked --test leaderboard --test spectate --test live_api` | exit 0; 19 passed |
| `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown` | exit 0, no warnings |
| `node --test crates/tbg-client/tests/*.test.js` | exit 0; 11 test files passed |
| `trunk build --release` in the client crate | exit 0; a WASM-only unused binding found in the first build was fixed |
| `cargo build --release -p tbg-server --bins` | exit 0; 30.97 s |
| `cargo audit --json` | exit 0; 0 vulnerabilities; informational unmaintained `ttf-parser` advisory RUSTSEC-2026-0192 |

The first server-test attempt was denied loopback sockets by the execution
sandbox (`Operation not permitted`); the same command passed with local
networking enabled. Tests and limits were not weakened.

**Observed browser behavior (desktop Chrome).** Two independent browser contexts
clicked Casual, played to Stronghold destruction, and received identical
`MatchEnd` bytes. One connection was closed during play and resumed with a
second `MatchStart`; every handshake used protocol 20. No page error. Ranking
hid every home panel and its close control was clickable. Watch followed by
Leave returned to the lobby, removed `?watch=`, cleared `tbg_watch`, and reduced
the server watcher count from 1 to 0. Desktop palette captures were inspected at
1920×1080 and 1280×800; the final camera review remains part of release validation.

**Limits.** No phone testing, full local workspace run, repeated C7 run, new kits,
ability waves, or online-pool expansion. GitHub CI and the production rollout
are separate release gates; this report does not claim either has completed.
Owner look and fun sign-off remain open.
