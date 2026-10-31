# Consolidation 2026-10-30 — every open PR in one branch

**Branch:** `consolidate/main-20261030`, from `origin/main` 6ef155b.
**Goal:** one tree with every open PR, the live roster release, the P55
champion kits (the character direction we keep) and the themed arenas,
ready for the next VPS release. Nothing was pushed or merged into `main`.

## What is in it (first-parent order)

| Commit | Source |
|---|---|
| 328c591 | #7 `docs/p41-i18n-spec` |
| 5aba13e | #12 `docs/p46-live-spectate-spec` (P47 plan) |
| 64fb911 | #9 `phase/p41-i18n` (lobby markup merged by hand with the Stronghold tab and chip) |
| 604623c | #14 `codex/ranking-ui`, first part |
| 3663dc1 | #5 `phase/p30a-network-audit` (D-41 renumbered D-54) |
| 5975727 | #16 `release/roster-online` (the build live on the VPS, d06279a) — see *Known issues* |
| d52d27d | #15 `phase/p54-unit-art-kits` + `phase/p55-champion-kits` (D-51..D-53) |
| 0625d54 | #11 `phase/a1-troops-port` (D-47; kits take precedence) |
| de034aa | decisions ordered D-47..D-54, roadmap status |
| ea254b1 | `phase/p30d-server-performance` + `phase/p30d-review` |
| b46676b | #14 `codex/ranking-ui`, later commits: lobby v2 art, Faction frames, island seascape |
| 0cc07b5 | `phase/arenas-gameplay` (ARENAS-G, D-55) |

Every open PR head (#5, #7, #9, #11, #12, #14, #15, #16) is an ancestor
of the branch (`git merge-base --is-ancestor`).

## Conflict choices

- Characters: the P54/P55 kits win over A-1's generic troop polish; A-1
  aim, hit-stop, recoil and faction deaths apply only to units without a
  kit (D-47 note).
- Lobby: P41 i18n attributes kept on the v2 markup of #14; the account
  link now reads "Sign in with key" in `en.json` and `pt-BR.json`.
- Arena scene: the Forest uses #14's owner-approved seascape and light;
  Tundra, Volcanic, Ruins and Storm keep their own sea and light.
  Codex's themed towers are dropped; every arena uses the D-46 towers.
- `measure.js`: frame history (P30) and first-frame time (#14) both kept.

## Known issues

- Merge commit 5975727 brings in #16 but carries the message of a P30-A
  fix (a shell mistake while committing). Its content is the correct
  merge; history is not rewritten (AGENTS.md).
- `tbg-balance` C7 fails on the base branch too (a SiegeWalker ×3 cost
  check); pre-existing, not touched here.
- Branches without a PR were left as they are: `phase/a1-visual-polish`,
  `phase/p17a-strategic-strongholds`, `docs/roadmap-p18-p39`,
  `phase/p56-roster-expansion-planning`.
- Balance of the arena values and the lab measurement are deferred by
  the owner; the values stay **Proposed**.

## Checks (minimal, owner request)

`cargo fmt --check`; `cargo clippy --workspace --all-targets -D
warnings`; `cargo check -p tbg-client -p tbg-core --target
wasm32-unknown-unknown`; `cargo test --workspace --exclude tbg-balance`:
311 passed, 0 failed; Node `tests/*.test.js`: 30 passed;
`trunk build --release`.
