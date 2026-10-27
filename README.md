# TBG — Tupi Battle Grounds

> **Technical reboot in progress.** This repository is the new TBG 2.0
> workspace: a clean, minimal, documented Rust-first foundation.
> **The game is not playable yet.**

An open-source, browser-first, real-time 1v1 strategy game featuring
automatic single-lane combat, evolving units, and faction synergies.

## Vision

No micromanagement. You build, position, and evolve — your units fight on
their own on a single `BattleLine`. Depth comes from shop decisions,
deployment timing, branching evolutions, and faction synergies, all
settled by an authoritative server so competitive outcomes are fair.

## Main gameplay mechanics

| Mechanic | Summary |
|----------|---------|
| Live 1v1 PvP | Both players share one real-time match; no turns, no waiting |
| Single `BattleLine` | One lane, no lateral movement |
| Automatic combat | Deployed units move and fight without player input |
| `Energy` economy | Time-regenerated in-match resource; overspend is rejected, never debt |
| Continuous `Shop` | Three rotating unit offers per cycle, server-synchronized timers |
| Branching `Evolution` | In-match upgrade trees per deployed unit |
| Faction `Synergy` | Same-faction (or defined-combo) field bonuses |
| Cosmetics-only monetization | Skins, VFX, emotes, board themes — never stats |

Full rules: [`docs/game-design/`](docs/game-design/) (glossary, mechanics,
factions, traceability to legacy sources).

## Target architecture

| Layer | Choice |
|-------|--------|
| Language | Rust (Cargo workspace, `stable` channel) |
| Client | Bevy 0.20 → WebAssembly, WebGL2 target, browser-first |
| Server | Rust + Axum + Tokio, authoritative fixed-tick simulation |
| Protocol | Versioned WebSocket messages (`tbg-protocol`) |
| Database | SQLite (initial persistence) |
| Assets | glTF 2.0 / GLB via offline Codex-driven procedural pipeline + headless Blender |
| Deployment | Static CDN + one small VPS, Docker Compose |

Details: [`docs/architecture/`](docs/architecture/) ·
Delivery plan: [`docs/ROADMAP.md`](docs/ROADMAP.md) ·
Decisions: [`docs/decisions/`](docs/decisions/).

## Current development status

**Phase 00 — Repository Reboot** (workspace scaffolding, preserved rules,
documented architecture). No gameplay, networking, accounts, or 3D
pipeline implemented. See the roadmap for acceptance criteria and the
exact next task (Phase 01: Bevy/WASM browser rendering prototype).

## Repository organization

| Path | Purpose |
|------|---------|
| `crates/tbg-core/` | Deterministic game domain (no graphics/DB/network) |
| `crates/tbg-protocol/` | Network message contracts |
| `crates/tbg-client/` | Bevy browser application |
| `crates/tbg-server/` | Axum/Tokio backend (never depends on Bevy rendering) |
| `crates/tbg-assets/` | Asset schemas + validation contracts |
| `content/` | Units, factions, balance, packaged assets (data, Phase 02+) |
| `tools/asset-factory/` | Offline generators, rigs, animations, validators (Phase 02+) |
| `docs/` | Game design, architecture, decisions, roadmap |
| `tests/` | Cross-crate integration tests (Phase 03+) |
| `deploy/` | Compose skeleton + VPS checklist |
| `archive/` | Legacy implementation backup — read-only, see [`archive/ARCHIVE-MANIFEST.md`](archive/ARCHIVE-MANIFEST.md) |
| `agent-harness/` `agent-rules/` | Agent development standards (symlinks into `.agent-harness/`) |

## Build and test

Prerequisites: stable Rust (`rust-toolchain.toml` applies automatically).

```bash
cargo metadata --no-deps --format-version 1 > /dev/null
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

WASM client check (Phase 01 acceptance, not yet passing):

```bash
rustup target add wasm32-unknown-unknown
cargo check -p tbg-client --target wasm32-unknown-unknown
```

A native `cargo check` passing does **not** demonstrate
WebAssembly/mobile-browser compatibility.

## Contributor guidance

1. Read [`AGENTS.md`](AGENTS.md), then `docs/game-design/GLOSSARY.md` —
   ubiquitous language is mandatory in code and docs.
2. Smallest correct diff; one logical change per commit; English only.
3. Contract-first tests: assert documented contracts, never mirror code.
4. Respect crate dependency rules (`docs/architecture/RUST-WORKSPACE.md`).
5. Never import from `archive/` into active code; never commit secrets,
   generated artifacts, or large binaries.

## Legacy implementation

The previous Unity/Nakama/Python implementation is preserved but inactive
under [`archive/`](archive/) (manifest: `archive/ARCHIVE-MANIFEST.md`;
pre-reboot snapshot: git tag `pre-tbg20-20261027`, branch
`backup/pre-tbg20-20261027`). Its gameplay concepts are authoritative
references; its technologies are retired.

## License

No root `LICENSE` file exists yet — **this must be resolved before any
public release or asset distribution** (tracked as a Phase 01
prerequisite risk). The project objective remains MIT open-source
distribution, with dependency and asset licenses audited per release.
Current notices: [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
(Ponytail MIT rules) and
[`archive/THIRD_PARTY_NOTICES.md`](archive/THIRD_PARTY_NOTICES.md)
(legacy MB-Lab/Mixamo notices for removed binaries).
