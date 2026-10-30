# P43 — Shared foundations, no behavior change (executor report)

> No reviewer spec existed for P43; the executor derived this work from
> ROSTER-EXPANSION §F and §G (P43) plus the unit sheets in §C. Deviations
> from the plan are listed under *Deviations*. Depends on P42
> (`phase/p42-roster-baseline` tip). Branch: `phase/p43-shared-foundations`.

## Goal (plan §G)

`Unit::is_summon()`; `Ability` enum with Hatch and Guard migrated;
`UnitFx`; `action_cd`; Hold rule for no-attack units; `Ruleset.pool`;
packed `fx` in views; protocol v13 with the new events (unused yet);
Practice preview flag. Behavior must not change.

## Commits (one logical change each)

- `0118c4f` refactor(p43): unit-level is_summon and action_cd
- `e58e3c3` feat(p43): Ability enum, Hatch/Guard migration, Hold rule
- `1b96ebd` feat(p43): UnitFx block, golden hashes updated once
- `52ad83d` feat(p43): Ruleset.pool, shop draws, practice preview
- `4c39b80` feat(p43): protocol v13, staged events, client blockout fallback
- (fmt + knobs fixes, this report — see below)

## Checks (real output)

- `cargo test -p tbg-core` (feature off) and `--features lab`: all green,
  including `lab_stats`; goldens pass with the new hashes below.
- Golden update (P43, once): tick/winner/reason verified identical for
  seeds 1/7/42 (2214/B, 3284/B, 1833/B); only hashes moved:
  det `0x23e057af9b0ffc5f`, `0xc74533b02893d54d`, `0xbca25a2593de46b1`;
  traces `0xd4d17533ea20cabf`, `0x44f36189d0ae2767`,
  `0x069f504c6b6dcbab`, `0x952b6552b9dcecea`, `0x344b6919239f50e9`.
- `gate --pairs 1024` on this branch: `gate.csv`, `roster.csv`,
  `duels.csv`, `duels.txt`, `gate.txt`, `payoff.csv` all byte-identical
  (`cmp`) to the P42 runs on the same seeds.
- `cargo fmt --all -- --check`: clean. `cargo clippy --workspace --
  -D warnings`: clean. `cargo test --workspace`: 39 suites ok (one full
  run showed 2 flaky `online.rs` timing failures;
  `records_insert_and_count`, `two_bots_finish_a_match`; both pass in
  isolation and on rerun).
- `cargo test -p tbg-core --features lab`: no failures.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`:
  clean.
- `cargo test -p tbg-server --test online`: 30 passed (includes
  `two_bots_finish_a_match`: two bots finish a real-WS match).
- `git diff <p42-tip> -- crates/tbg-core/src` outside `cfg` is exactly:
  the Ability/FormDef/UnitFx/pool/event/view additions, the Hatch/Guard
  dispatch, the Hold rule, the two P42 return-type sites untouched, and
  `pub mod lab` (P42).

## Deviations (spec-derived details, no reviewer spec existed)

- D1: form abilities are `[[Ability; 2]; 5]` (fixed pair per slot),
  decided here per §F.1 ("decided in P43"): Rotcap needs Poison + Echo on
  one form. Recorded as D-47.
- D2: `Ruleset::preview()` is built with `Box::leak` (core-side, like
  `duels::arena`); Practice uses the new `MatchState::with_preview()`.
  C10-safe (none of its forbidden words appear in client sources).
- D3: the lab's roster iteration (duels, favor, knobs) stays on
  `UNIT_KINDS`; only the Shop draws follow `rules.pool`. The per-pool lab
  iteration moves to P44, which must add duel/favor rows for the new
  kinds anyway. `CrateOpened{unit, opened}` renames the spec's illegal
  `crate` field (commented).
- D4: the three client wildcards carry `#[allow(unreachable_patterns)]`
  (unreachable until P44 adds kinds; `-D warnings` needs the allow).
- D5: no ROADMAP row exists for P43+ (reviewer writes specs and rows);
  left untouched. The Practice preview flag is wired (`with_preview`)
  but unobservable until P44 adds kinds.

## Blocked

Nothing. A real two-device online match stays owner-side (as in every
phase); the harness two-bot WS match passes locally.

## Reviewer checklist (suggested)

- Tick/winner/reason per golden identical to P42; only hashes moved.
- `cmp` gate CSVs yourself: `target/balance/p43-gate/` vs P42 outputs.
- `Comp`/`Favor`/`pool` untouched in behavior; C10 still passes.
- D-47 acceptable as an executor decision.
