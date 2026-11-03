# P76 — TBG Arena 5v5 (MOBA) foundation

> Owner request (2026-11-03): a new, separate 5v5 MOBA mode with its own
> core, documentation, plan and complete execution; playable on the local
> Linux install against 9 bots; visual base taken from the owner's reference
> image; immutable contracts and a functional, professional skeleton that
> another agent can continue. **No balancing** until the mode is fully
> functional and the owner explicitly asks.

Branch `phase/p76-moba-core`, based on GitHub `main` `c3dbc999`.
Canon: [GAME-DESIGN-MOBA](../moba/GAME-DESIGN-MOBA.md) ·
[ARCHITECTURE-MOBA](../moba/ARCHITECTURE-MOBA.md) ·
[CONTRACTS](../moba/CONTRACTS.md) · Decision D-112.

## Read first (next executor)

1. [docs/moba/README.md](../moba/README.md), then the three canon files above.
2. `moba/core-moba/src/lib.rs` (module map) and `moba/core-moba/tests/contract_moba.rs`.
3. `moba/client-moba/src/link/mod.rs` (the one data seam of the client).
4. This file's *Backlog* — pick the next phase from there.

Do not read `crates/*` or `archive/` for MOBA work; nothing there is used.

## Steps (all done)

| # | Step | Result |
|---|---|---|
| S1 | Separate workspace `moba/` (core, server, client), root workspace untouched | done — D-112 |
| S2 | Deterministic core: map, units, combat, movement, economy, levels, waves, camps, outcome | done — M-01…M-30 |
| S3 | Seat bots through `MobaIntention`s; `host::tick` shared by every host | done — M-04, M-26 |
| S4 | Wire protocol v1 + views (one shared world frame per tick) | done — M-40 |
| S5 | Authoritative room server (seats from connections, drop-in/out bots, limits) | done — M-41, M-42 |
| S6 | Client: valley scene from the core map, units, effects, RTS + minimap cameras | done |
| S7 | Client HUD after the reference image; menu; offline and online links | done |
| S8 | Linux installer and local install; docs, contracts, CI workflow | done |

## Acceptance

| Item | Status |
|---|---|
| Separate folders/crates; no dependency on or change to the 1v1 crates, lock or release inputs | met (root `Cargo.toml`/`Cargo.lock` unchanged; native version policy passes) |
| Offline match: player + 9 bots, playable to the end | met (Linux, observed; bot-only matches end by Stronghold or the 25:00 limit) |
| Online match on the local room server; seat from connection; bots fill/resume seats | met (contract tests + observed client) |
| Visual base follows the reference: valley, 3 lanes, diagonal river with bridges, central plaza crystal, stone camps, towers with team crystals, base plateaus, top scoreboard, role columns, player card, Troop cards, Destination, "Comprar e Enviar", minimap | met on Linux (captures below) |
| Installed on the local Linux desktop | met — `~/.local/bin/tbg-moba`, menu entry "TBG Arena 5v5" |
| Balance | out of scope (explicit owner rule) |
| Windows and desktop web correspondence | **pending** (see deferred checks) |

## Validation (consolidation level, MOBA workspace only)

All compilation ran under the shared host lock with two build jobs and a
dedicated target directory.

```text
cd moba && cargo fmt --all -- --check                                  → ok
cargo clippy --locked --workspace --all-targets -- -D warnings         → ok, 0 warnings
cargo test --locked --workspace                                        → 12 passed (10 core, 2 server), 0 failed
cargo check --locked -p tbg-moba-core -p tbg-moba-client --target wasm32-unknown-unknown → ok, 0 warnings
scripts: ci-test-lanes, check-native-inputs, check-patch-notes,
         check-native-version --base origin/main, test-ci-required-checks → ok
shellcheck moba/scripts/install-linux-moba.sh                          → ok
git diff --check                                                       → ok
```

Runtime observations (Linux, Xvfb 1600×900/1920×1080, Mesa software
renderer — not a performance measurement):

- `--capture` offline start, `--warp 444 --overview` mid-game at 07:27,
  `--online` against `tbg-moba-server` (server log: seat 0 joined, bot
  resumed on leave).
- Real X input via XTest: menu role pick and "Treinar contra 9 bots",
  Troop card click, Destination click, "Comprar e Enviar" (gold debited,
  Troops 1/8), right-click move (Commander walked to the point), hotkeys
  `5 V Enter 2 X Enter` (two Troops bought, gold 42 as expected).
- Simulation probe (opt-level 1): ~28 µs per tick at 170–210 live units;
  peak world frame 3 090 bytes. Identical habits per role keep bot-only
  matches even (7 vs 7 towers at 25:00 with the seed shift disabled).

![Menu](../moba/media/p76-menu.webp)
![1v1 champions on a lane at 7:03](../moba/media/p76-champions-lane.webp)
![Overview at 7:03](../moba/media/p76-overview.webp)

## Revision 2 — shared champions and the 1v1 battle HUD (owner, 2026-11-03)

Owner request: copy the 1v1 battle cards exactly (same frame, the same 3D
champion photos), use the 1v1 top HUD with the clock and each side's Gold
instead of HP, make the 5v5 champions exactly the 1v1 champions (both modes
share them), keep automatic Minion waves as the new element, then open a PR.

| Step | Result |
|---|---|
| R1 Troops are `tbg_core::UnitKind` champions; stats/prices from their 1v1 `UnitDef`; heal and splash honoured (M-18) | done |
| R2 Per-seat Shop with the 1v1 rules, locks and a paid reroll; buying by slot (M-19); protocol v1 intentions updated before first merge | done |
| R3 Client compiles the 1v1 `art/`, `vfx/impacts.rs`, `vfx/shots.rs` in place; champions and Commanders use the 1v1 rigs, clips, shots and impacts | done |
| R4 HUD in the 1v1 battle layout: score plates with crests around the round clock (Gold · Kills · Towers instead of HP), Arena chip, `resources.webp` strip (Gold, Essence/Upgrade, Army, Reroll), five `shop-card.webp` cards with 1v1 portraits, names, archetypes, Cost and lock | done |
| R5 Minion waves every 30 s per lane attacking Troops, Commanders and structures on the way | kept from S2 (documented as §9a) |
| R6 Installer copies the 1v1 `battle/` and `lobby/native-icons/` assets; RNG seeds no longer collide (splitmix64) | done |

Validation of the revision (MOBA workspace, shared lock, two jobs):

```text
cargo fmt --all -- --check                                              → ok
cargo clippy --locked --workspace --all-targets -- -D warnings          → ok
cargo test --locked --workspace                                         → 13 passed (11 core, 2 server)
cargo check --locked -p tbg-moba-core -p tbg-moba-client --target wasm32-unknown-unknown → ok
```

Observed on Linux (Xvfb on the local NVIDIA RTX 4060 through Vulkan):
offline start, 7:03 lane fight and overview (`--seed 11 --warp 420`),
installed binary loading the copied 1v1 assets with zero load errors, and
real X input: lock on card 1 (closed-lock icon), Reroll (card 1 kept, the
other four redrawn, 6 Gold spent), card purchase. Online: `tbg-moba-server`
assigned seat 0, the bot resumed it on leave. Bot-only probe: seeds 1–3
reach 25:00 with Towers falling on both sides, ~25–48 µs per tick, peak
world frame ~2.7 KB.

![Locked card after a reroll](../moba/media/p76-card-lock.webp)

## Deviations

- The 1v1 non-negotiable glossary is not reused: the MOBA has its own
  glossary (Gold, Essence, Troop, Commander…) in its canon, all Proposed.
- `moba/` is its own Cargo workspace (not `crates/*` members) because a root
  manifest change is a 1v1 desktop release input (`scripts/native_release_policy.py`)
  and would force a 1v1 version bump and publication.
- The river is presentation only (no slow) to keep the first rules small.
- Role names stay in English in both languages (as in the reference image).

## Deferred checks (exact commands, with reasons)

| Check | Why deferred |
|---|---|
| Desktop web runtime and visual comparison against Linux (the WASM client compiles) | no web packaging/hosting for the MOBA yet (P80) |
| Windows build, package and visual comparison | no Windows packaging for the MOBA yet (P80) |
| Real-GPU frame-time measurement on the owner's desktop (NVIDIA) | only software-rendered Xvfb was used |
| Ten humans on a real network | needs the owner and devices (P78) |
| Owner look and fun sign-off | owner |

## Known limitations (honest list for the next agent)

- 1v1 champion abilities and Evolutions are not active in 5v5 yet (base
  form and base stats only); Upgrade tiers are a MOBA percentage boost.
- The 1v1 sources are shared by `#[path]`, so a 1v1 refactor of `art/` or
  `vfx/` can break the MOBA build (the MOBA workflow watches those paths).
- No Commander abilities, items, fog of war, pings, chat, sounds or music.
- One room per server process; no matchmaking, reconnect token, TLS or metrics.
- Commander Auto is simple (follows the lane front, retreats at 25 % HP).
- Lane units overlap loosely in big fights (soft separation only).
- No delta compression; ~60 KB/s per player at peak.
- Menu shows the valley without units (no match running behind it).
- Portraits are rendered once per frame by small cameras (cheap, but always on).

## Backlog — next phases (each its own spec before work starts)

1. **P77 — Champion abilities and Evolutions in 5v5, Commander kit and orders.**
   Run each champion's 1v1 abilities and Evolution forms in the MOBA core;
   one active ability per Commander role (Q),
   attack-move, stop, health/cooldown HUD, ability VFX; bots use them.
   Contracts: append `MobaIntention` variants (protocol v2 only if needed).
2. **P78 — Online rooms for real players.** Several rooms per process,
   lobby/matchmaking endpoint, reconnect tokens to the same seat, latency
   ping display, metrics, load test with 10 socket clients, TLS via Caddy.
   VPS work only on explicit owner request.
3. **P79 — Presentation polish.** Rigged walk/attack animations, sounds,
   music, kill-feed portraits, Tab scoreboard, settings (volume, edge pan),
   fog of war rendering after the rule exists.
4. **P80 — Windows and desktop web.** Packaging, WASM build/hosting,
   correspondence matrix and captures per the visual contract.
5. **P81 — Balance (only when the owner asks).** Offline lab for the MOBA
   core in the spirit of D-33; numbers change only with evidence.
6. **Integration** of the mode into the 1v1 launcher/lobby — owner decision.
