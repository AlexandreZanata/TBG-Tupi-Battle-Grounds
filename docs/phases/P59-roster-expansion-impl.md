# P59 — Roster expansion: implementation (status report)

Status: **W0 partly done** (six of its seven contracts, D-62). W1 and the pilot have not started. This branch holds the planning (P58, Part B) and the
Part A units fixes (`phase/units-fixes`, merged on 2026-10-30). No unit of the
expansion (R01–R21) exists in code yet. This file records where the work stands
and what the next session must do. It is a status report, not a finished wave.

## What is in place

| Item | Where | State |
|---|---|---|
| Planning, renumbered from P56 to P58 | `docs/phases/P58-roster-expansion-planning.md`, `docs/roster/P58-*.md` | merged |
| Registry moved from GAME-DESIGN §11 to §13 (active §11 roster and §12 Arenas unchanged) | `docs/GAME-DESIGN.md` §13 | merged; numbers Proposed |
| Reaudit against today's code (Shop 25 kinds, pool `FULL_POOL`, protocol, UnitFx, faction arrays) | `docs/roster/P58-AUDIT-AND-INTEGRATION.md` §2–§5, §7 | merged |
| Completion register (values not in the owner's text), all Proposed | GAME-DESIGN §13.6 | merged |
| Part A: Archer kit, carrier id (protocol 18, D-61), passengers, crab rule (D-60), name tags | `phase/units-fixes` | merged here |

**The audit's protocol number is stale.** P58 §2 reads "protocol 16". After
Part A the value is 18 (D-61). Update the audit when the next session starts.

## Waves (from P58; none started)

- **W0 — shared contracts. Six of seven in (D-62); not green.** The six slots are
  in `UnitFx`: the bleed (R07), the weakening (R13), the Slow queued behind a
  Root (R19), the caster's mark (R03), the timed self-reduction, and the ability
  heal-cut beside the pair. Each is inert until a unit uses it. Tick, winner and
  reason are identical on all five golden seeds; the hashes moved once and were
  refrozen. **Not done:** the unit shield grant path with expiry and its view field
  (R12/R14), blocked on owner decision 5; and the view and event extensions for the
  six slots, deferred to the first unit wave that shows an effect (that wave
  carries the protocol bump). Because W0 is not green, W1 has not started.
- **W1 — factions. Not started;** it waits for W0 to be green (the shield item is the blocker). Append `Monsters`, `Celestials`, `Insects` after the four
  existing factions. Generalise every fixed-size-4 array (the reaudit lists ten:
  `FACTIONS`, `counts`, `SYNERGIES`, palette `FACTIONS: [[Color; 8]; 4]`, …).
  Synergies at 2 and 4. Bump the protocol to the next free number (19 at the
  time of writing; check with `git grep -n "PROTOCOL_VERSION: u16"` across refs
  first). Add the three rows to `docs/ART-DIRECTION.md` §3 and §11.2 as Proposed.
- **Pilot — Monsters R08 Grumak, R09 Rasgarr, R10 Esporax, R11 Murmúrio.** Rules,
  protocol, client, kit (`src/art/kits/<unit>/`), animations, gallery. Its report
  goes here before the next wave starts.
- **W2–W6 — the other 17 units**, each with its evolution tree (W7), kit (W8) and
  animations (W9) in the same cycle. R01 Bastion goes last, after the owner
  decides its name.
- **W10 — tests.** Deterministic per-unit tests and the cross-cutting cases from
  the plan. Only a small laboratory sanity run; the full 256/2048-pair matrices
  are for the owner to run later.
- **W11 — release to the online pool.** Owner decision, after look and fun sign-off.

## Owner decisions still open

From the reaudit (P58 §7 and the completion register):

1. **Bastion.** R01's name collides with the existing Stronghold kind `Bastion` (D-46). Not renamed.
2. **Crematumba** cost, HP and movement (C3–C4).
3. **Brocatanque** production, radius and merge rules (C8–C10).
4. **Monsters synergy** damage: 5 or 10 %. Existing MaxHp and AttackSpeed synergies are 20 and 40 %.
5. **R12/R14 shield** semantics, and whether the dormant Cargo Crab `shield_bp` is granted or removed.
6. **R21** fallback when there is no merge partner.
7. **Shop gate floor**: 0.6/N in the plan, 2.5 % per kind in `gate.rs` today.
8. **Names.** R08–R11 and the unsplit R12–R19 Evolution names are Portuguese placeholders; the English-only rule needs the owner to name them.

## Not measured, not verified

- No cargo run in this branch: `UnitFx` size, state hash and view bytes are unmeasured.
- The carrier id from Part A costs one postcard tag byte per unit per view (D-61).
- PR #11 and #15 states were not re-checked (no network).
- Audit §1 and §6 were not reaudited; the "reviewed #16" daggers in §6 are stale.
- Left alone by the docs agent (code, not docs): `crates/tbg-core/src/view.rs` lines 24–26 describe unfinished fx packing, and `crates/tbg-core/tests/contract_roster.rs:162` asserts `UNIT_KINDS.len() == 10`.

## Verification to run in the next session

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p tbg-core --features lab
cargo test -p tbg-client --bin tbg-client
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```
