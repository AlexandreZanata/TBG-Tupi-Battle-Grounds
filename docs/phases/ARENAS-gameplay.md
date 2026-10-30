# ARENAS-G — Themed arenas in the real game

**Status:** in progress on `phase/arenas-gameplay` (from the consolidation
branch, carrying the five visual arenas of `codex/themed-arenas`).
**Goal:** the arena is part of the match. The server draws it at pairing,
both seats play under the same small modifier, practice lets the player
choose it, the scene switches without a reload, and a lobby page presents
the five arenas.

## Read first

- `docs/ARENAS.md`, `docs/phases/ARENAS-visual-variants.md` (the scenes).
- `docs/GAME-DESIGN.md` §Energy, §Evolution, §Strongholds; BR-X01 (one
  truncation), BR-E05 (Interest), BR-SK01 (frozen at pairing).
- `crates/tbg-core/src/{sim,state,ruleset,intent,view,protocol,replay}.rs`,
  `crates/tbg-server/src/{lobby/matchmaking,match_task,store}`,
  `crates/tbg-client/src/{arena,local,sync,ui}`, `crates/tbg-balance`.

## Rules (all values **Proposed** until the lab measures them)

| Arena | Effect (both seats) | Path |
|---|---|---|
| Verdant Forest | none (neutral reference) | — |
| Frozen Tundra | troop movement −10 % | `move_bp` beside Evolution/Augments, one truncation |
| Volcanic Forge | troop attack speed +8 % | `atk_bp` beside Evolution/Augments/Synergy; interval = base × 10 000 / (10 000 + atk_bp), then the existing cap and `MIN_INTERVAL` |
| Ancient Ruins | first successful Evolution of each player costs 1 Energy less | the Evolution price, once per player, consumed only on success; never below the existing 1-Energy floor |
| Storm Valley | base Energy regeneration +8 % | +800 bp beside Interest's bp on the base rate; the existing remainder carries rounding |

Troops include summons and cargo (they move and attack under the same
sky). Strongholds, Augment and Arcana values are unchanged.

## Design

- `tbg_core::Arena` (stable ids 0–4) and one effect table
  (`content::ARENAS`, copied into `Ruleset::arenas` so the lab can tune it).
  `MatchState::arena` is fixed at creation; each `PlayerState` keeps
  `arena_discount_used`. No float, clock or new RNG stream.
- The view carries the seat's effective Evolution prices (base, Augment
  discount, arena discount), so the HUD never shows a different number.
- `ServerMsg::MatchStart` carries the arena (protocol v15); a reconnect
  gets the same `MatchStart`. Replays and match rows record the arena
  (SQLite v6: nullable `arena` columns; older rows untouched).
- The server draws uniformly from the OS RNG in the one pairing path
  (casual, ranked, rooms, challenges, rematches), never from the match
  seed or a simulation stream.
- Client: an `ActiveArena` resource; changing it despawns only the scene
  entities and lights and builds the new scene. `MatchStart` sets it; the
  lobby previews the hub's choice; `?arena`/`?theme` stay inspection
  tools that never override a match.
- Lobby: an Arenas dialog (nav tab) with five cards: real scene
  thumbnails, theme text, exact effects (from `assets/arenas/arenas.json`,
  pinned to the Rust table by a contract test), "same for both sides",
  "online the arena is drawn", select and train.
- In match: a start banner with the arena and its effects, and a small
  chip that reopens them.
- Lab: `--arena` for `run`, and an `arenas` command that compares each
  arena with the forest on paired seeds, both seats, several policies.

## Steps

1. Spec and roadmap row (this file).
2. Core rules, view prices, protocol v15, replay; tests.
3. Server draw, MatchStart, records and migration; tests.
4. Lab support and measurement; tune values on evidence; GAME-DESIGN.
5. Client scene switching, MatchStart wiring, practice choice.
6. Arenas hub and in-match HUD.
7. Verification, browser evidence, report.

## Acceptance

Neutral arena keeps every existing contract; each effect is exact and
symmetric, applied once, deterministic, and shown as applied; the
discount is used once and only on success; the server owns the draw, both
clients and reconnects see the same arena; practice uses the chosen one;
repeated scene switches add no entities or lights. The commands in
AGENTS.md pass, plus `trunk build --release` and desktop browser checks.

## Phase report

_Filled in at the end._
