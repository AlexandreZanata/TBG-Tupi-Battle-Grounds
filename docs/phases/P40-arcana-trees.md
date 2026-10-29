# P40 — Arcana trees: 15 nodes, 12 points and three supremes

**Depends on:** active Arcanas (D-40), the P16-C Ruleset lab and the
existing HTML editor. This is a product extension requested by the owner
on 2026-10-29 (D-42); it does not reopen the P31 release gate.
**Branch:** `phase/p40-arcana-trees` (built on the P16-F owner revision).
**Status:** implemented and integrated; automated checks and lab
screening run; human/browser look check and full P34 confirmation remain.
**Difficulty:** large. Core schema, conditional combat hooks, protocol,
persistence migration, editor and lab in one feature.

## Goal

Keep the three families (War, Precision, Vitality) and the six original
nodes, but add two children and one supreme per family. Every player has
12 free points; a family costs 8. The player may finish one family and
spend 4 elsewhere, or play a 4/4/4 hybrid without a supreme. Supremes
must be conditional, powerful and counterable; the hybrid must remain
viable.

## Audit summary

Reused rather than replaced:

| Need | Existing mechanism |
|------|--------------------|
| Preset storage | `ArcanaPreset { ranks }`, `Ruleset.arcana_bp`, `stats_with` |
| Authority | `SetArcanas` validated while idle, frozen at pairing, both presets in `MatchStart` |
| Persistence | one account BLOB + `tbg.arcanas` local document; match/ranked rows |
| Editor | `assets/arcana/{arcana.json,preset.js,editor.js,editor.css}` |
| Lab | `tbg-balance arcana` over paired, seat-swapped seeds |
| Stats formula | additive bp plus one truncation (BR-X01); summons neutral |

## Node model

Indices 0–5 keep their old serialization order and values. Indices 6–14
are appended:

| Index | Node | Cost | Requires |
|-------|------|------|----------|
| 6 | War / Shieldwall | 1 | Guard 1 |
| 7 | War / Battle Rush | 1 | Assault 1 |
| 8 | War / Combined Arms | 2 | Guard 2, Assault 2, Shieldwall 1, Battle Rush 1 |
| 9 | Precision / Hunter's Focus | 1 | Piercing 1 |
| 10 | Precision / Quickdraw | 1 | Endurance 1 |
| 11 | Precision / Deadeye | 2 | Piercing 2, Endurance 2, Hunter's Focus 1, Quickdraw 1 |
| 12 | Vitality / Unshaken | 1 | Resilience 1 |
| 13 | Vitality / Emergency Care | 1 | Renewal 1 |
| 14 | Vitality / Purification | 2 | Resilience 2, Renewal 2, Unshaken 1, Emergency Care 1 |

`COST` is separate from rank: a supreme is rank 0–1 and costs 2 points.
`ArcanaPreset::complete` validates the real cost (12), per-node maxima and
`PREREQUISITES`. `migrate_legacy` keeps a six-rank preset and raises the
original nodes to rank 2 in index order; it never selects a child or
supreme. Default: `[2,2,2,2,2,2,0,0,0,0,0,0,0,0,0]`.

## Effects and integration points

- **War:** Shieldwall 6% Tank damage reduction while a conventional DPS
  ally lives; Battle Rush 12% DPS movement while a Tank ally lives;
  Combined Arms 5% Tank reduction and 8% DPS damage while both live.
  `MatchState::army` caches living Tank/DPS counts once per combat tick
  and decrements on death, so a hit never scans the army.
- **Precision:** Hunter's Focus 12% Ranged damage against a target already
  slowed/rooted at hit resolution; Quickdraw 15% attack speed within 2
  tiles, clamped by `MIN_INTERVAL`; Deadeye +100 milli-tiles (0.1 tile) of
  range for conventional Ranged.
- **Vitality:** Unshaken 25% shorter incoming slow/root duration on
  Support/Control; Emergency Care 15% direct-heal efficiency below 40% HP
  before the heal; Purification removes slow and halves remaining root
  after a real heal on a conventional ally.
- **Shared:** original six nodes keep the same additive HP/damage/heal
  basis points. Summons receive no Arcana and never satisfy a conditional.
  Deadeye does not modify Strongholds.

## Protocol, persistence and migration

- `PROTOCOL_VERSION` 11: `SetArcanas { ranks: [u8; 15] }` and
  `ArcanasSelected` carry the new schema; older peers are refused.
- Account BLOB: 15 bytes. A 6-byte row is migrated and written back on
  first load. Match rows now store 15 bytes; old rows are untouched.
- Browser: schema 2 `{schema,ranks}`; a 6-rank document migrates on load.
  A partial draft is editor-only; the game falls back to the default.
- Editor refuses a removal that would invalidate an active dependent and
  names the dependent. Add is blocked with cost, rank and prerequisite
  reasons. Keyboard interaction is preserved; the same banners and emblems
  are reused; the layout stacks and scrolls on small screens.

## Verification

Run on this worktree:

```text
cargo fmt --all -- --check
  exit 0
cargo clippy --workspace -- -D warnings
  Finished `dev` profile [unoptimized + debuginfo]; exit 0
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
  Finished `dev` profile; exit 0
node --test crates/tbg-client/tests/arcana.test.js
  6 passed, 0 failed
trunk build --release (crates/tbg-client)
  Finished release profile; success
```

`cargo test --workspace --no-fail-fast` reports 238 passed, 1 ignored
and exactly one failure: the pre-existing
`tbg-balance/tests/contract.rs::c7_planted_imbalances_are_caught_for_every_unit`
(`Shieldbearer ×2 HP and damage was not caught`). The same failure was
reproduced on `f15432f^` in an isolated worktree before any P40 change,
so P40 did not introduce it; its thresholds were not changed. The other
core, server, client and balance suites pass (215 + 23).

Lab command:

```text
tbg-balance arcana-named --pairs 512 --threads 8
  war-full         49.0% [45.0 – 53.4]
  war-nosup        50.2% [46.2 – 54.4]
  precision        55.0% [50.6 – 58.9]
  vitality         47.9% [43.8 – 52.1]
  hybrid           53.2% [49.1 – 57.3]
  hybrid-children  48.6% [44.4 – 52.8]
  defensive        48.9% [44.7 – 53.1]
  aggressive       50.2% [46.0 – 54.4]
  control          51.0% [46.7 – 55.5]
  ranged-supreme   55.0% [50.8 – 59.2]
  legacy-6pt       48.0% [43.8 – 52.3]
```

The complete 1 842-preset screen with 8 pairs produced no interval
outside 45–55%, but its intervals are too wide for confirmation. The
named block above is the usable 512-pair evidence. Precision is the
strongest family at the top of the gate; the retired six-point default
is 48.0% against the new default, and the 4/4/4 hybrid is 53.2%. War and
Vitality supremes are not compulsory on these seeds.

Deadeye was tuned from the requested +0.5 tile to +0.1 tile after the
variants measured 60.0% (+0.5), 56.8% (+0.25) and 55.0% (+0.1) against
the default (512-pair block). The original +0.5 value is not approved.

Not measured yet: per-node activation frequency, damage prevented, time
alive and Augment interactions. Those need counters in the lab, not
guesses; they remain open review work. Human playtests and the full P34
confirmation matrix are also open.

## Files

Core: `arcana.rs`, `ruleset.rs`, `content.rs`, `state.rs`, `combat.rs`,
`hits.rs`, `augment.rs`, `sim.rs`, `protocol.rs`, `tests/arcana.rs`,
`tests/contract_protocol.rs`, `tests/contract_protocol_server.rs`.
Server: `lobby/arcanas.rs`, `store/arcanas.rs`, `store/matches.rs`,
`store/ranked.rs`, server tests.
Client: `config.rs`, `assets/arcana/*`, `tests/arcana.test.js`.
Lab: `src/arcanas.rs`, `src/commands.rs`, `src/cli.rs`, `src/knobs.rs`,
`tests/arcana_effects.rs`.
Docs: GAME-DESIGN §9, DECISIONS D-42, ARCHITECTURE §5/§15, ROADMAP, this
report.

## Deviations

- Deadeye +0.5 tile was reduced to +0.1 tile on lab evidence (above).
- Mobile layout was implemented although the current owner platform focus
  is desktop, because this task explicitly asks for it.
- The balance lab still reports the pre-existing C7 failure; thresholds
  were not changed to hide it.
