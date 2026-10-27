# Roadmap — TBG 2.0

> Phased delivery plan. Phase 00 is the repository reboot (this task).
> Phases 01–06 are explicitly **not started**.

## Phase 00 — Repository Reboot (current)

Archival, cleanup, business-rule preservation, Rust workspace,
architecture documentation.

Acceptance criteria:

- [x] Previous code recoverable (backup branch + tag, archive intact)
- [x] Archive undamaged; git history intact; submodules/symlinks valid
- [x] Business rules documented and traceable to legacy sources
- [x] Legacy tech decisions explicitly retired with replacement direction
- [x] Rust workspace resolves (`cargo metadata`), fmt/check/test pass
- [x] README describes the new project; no playable-game claims
- [x] No legacy runtime in the active build path

## Phase 01 — Browser Rendering Foundation

Bevy 0.20 + Rust/WASM + WebGL2; basic arena scene; desktop and mobile
browser smoke tests.

Acceptance: `cargo check -p tbg-client --target wasm32-unknown-unknown`
passes; arena renders in current Chrome/Safari/Firefox on desktop and one
mobile browser; native build is dev-only. WASM compatibility is **not**
demonstrated by a native `cargo check` — it requires the WASM target run.

## Phase 02 — Procedural Asset Factory

Modular character generation; Codex-controlled headless-Blender scripts;
shared animation structures; GLB validation gates; first playable
character model in `content/assets/`.

Acceptance: one base body class with full named animation set passes all
six quality gates in `docs/architecture/ASSET-PIPELINE.md`.

## Phase 03 — Deterministic Battle Simulation

`tbg-core`: units, energy, shop, automatic combat, evolution, synergies;
local bot matches (no network).

Acceptance: scripted bot-vs-bot matches run deterministically from a seed
(same seed → same outcome), covered by contract tests in `tests/`.

## Phase 04 — Authoritative Multiplayer

`tbg-server` (Axum + Tokio) + `tbg-protocol` over WebSocket: matchmaking,
reconnection, deterministic synchronization; SQLite persistence and
schema migrations.

Acceptance: two real browsers complete a match; disconnect/reconnect
resyncs; version mismatch rejected; no client-trusted outcomes.

## Phase 05 — Content and Polish

Faction content (8-unit starter set → 12-unit MVP roster), animation
improvements, VFX, audio, mobile UX, performance budgets.

Acceptance: full starter roster playable in bot matches; asset budgets
met; Lighthouse/mobile smoke tests pass.

## Phase 06 — Public Beta

VPS deployment, static hosting, player accounts, ranking, load testing,
monitoring, dependency/license audit.

Acceptance: production deploy via `deploy/`; backup/restore drilled;
ranked matches stable under target CCU.
