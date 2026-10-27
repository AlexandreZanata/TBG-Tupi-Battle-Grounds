# Rust Workspace — TBG 2.0

> Layout and dependency rules for the Cargo workspace at the repository
> root. The workspace is scaffolding only: placeholder library entry
> points and smoke tests, no gameplay.

## Layout

```
Cargo.toml              # workspace root (members: crates/*)
Cargo.lock              # pinned resolutions, committed
rust-toolchain.toml     # pinned stable toolchain
crates/
  tbg-core/             # deterministic game domain (no I/O, no rng without seed)
  tbg-protocol/         # versioned network message contracts
  tbg-client/           # Bevy app (WASM/browser target)
  tbg-server/           # Axum + Tokio backend (native only)
  tbg-assets/           # asset definition schemas + validation contracts
content/                # units/, factions/, balance/, assets/ (data, Phase 02+)
tools/asset-factory/    # generators/, rigs/, animations/, validators/ (Phase 02+)
tests/                  # cross-crate integration tests (Phase 03+)
deploy/                 # Compose files, VPS provisioning notes
docs/                   # architecture/, game-design/, decisions/, ROADMAP.md
archive/                # legacy backup (never a workspace member)
```

The repository root **is** the workspace root. There is no nested `tbg/`
directory: the task diagram's `tbg/` level maps to the repo root so that
`archive/`, `.agent-harness/`, and `.cursor/` stay top-level siblings of
the new workspace, as the diagram itself shows.

## Crate contracts

### tbg-core

- Pure deterministic game domain: factions, archetypes, slots, energy,
  shop rotation, evolution trees, synergies, combat resolution (Phases 03+).
- `no_std`-compatible discipline: no filesystem, no network, no wall-clock,
  no unseeded randomness in game logic. (Standard library use is allowed;
  nondeterministic I/O is not.)
- Must compile for `wasm32-unknown-unknown` as well as host targets so the
  client can reuse it for prediction without forking logic.

### tbg-protocol

- Versioned message contracts between `tbg-client` and `tbg-server`
  (intentions upstream, snapshots/results downstream).
- Serde-based types with an explicit `PROTOCOL_VERSION`; breaking changes
  bump the version and are rejected across versions.
- No game logic, no transport code — types only.

### tbg-client

- Bevy 0.20 application: rendering, animation, effects, audio, input.
- Owns presentation and client-side interpolation only; imports `tbg-core`
  for prediction-shaped types and `tbg-protocol` for messages.
- Primary build target: `wasm32-unknown-unknown` + WebGL2 (Phase 01).
  Native desktop builds are a development convenience, never the
  compatibility claim.

### tbg-server

- Native-only Axum + Tokio backend: matchmaking, authoritative fixed-tick
  simulation (driven by `tbg-core`), SQLite persistence.
- Must **not** depend on Bevy's renderer or any graphics stack.
- Binds transport (`tbg-protocol` over WebSocket) to simulation.

### tbg-assets

- Asset definitions, JSON schemas, and validation contracts for units,
  factions, balance tables, and GLB acceptance gates.
- No heavy generation runtimes (no Blender scripting deps, no ML
  inference). Generation lives in `tools/asset-factory/` as offline
  scripts; this crate only describes and validates their outputs.

## Dependency rules

- Allowed: `tbg-client → {tbg-core, tbg-protocol, tbg-assets}`,
  `tbg-server → {tbg-core, tbg-protocol, tbg-assets}`,
  anything → `tbg-protocol`, anything → `tbg-assets` (types/schemas only).
- Forbidden: any dependency of `tbg-core` on the other four crates;
  any dependency of `tbg-server` on Bevy rendering; any dependency of the
  game crates on `tools/` scripts.
- Adding a dependency requires a one-line justification in the crate's
  `Cargo.toml` comment or `docs/decisions/`.

## Toolchain and versions (pinned 2026-10-27)

| Crate / tool | Version | Verification |
|--------------|---------|--------------|
| Rust toolchain | `stable` channel (see `rust-toolchain.toml`); MSRV 1.97.1 imposed by bevy 0.20 | `rustc --version` (env: 1.99.0) |
| bevy | 0.20.0 | `cargo info bevy` — exists on crates.io |
| axum | 0.8.x | `cargo info axum` — 0.8.9 current |
| tokio | 1.x (full features off; `rt-multi-thread`, `macros`, `net`, `sync`, `time`) | resolved by `cargo metadata` |
| serde / serde_json | 1.x / 1.x | protocol + asset schemas |
| rusqlite | 0.32.x (bundled SQLite) | avoids system libsqlite3 dependency |

`Cargo.lock` is committed. Version bumps are deliberate, single-crate
commits with `cargo metadata` re-resolved.

## Commands (run at repo root)

```bash
cargo metadata --no-deps --format-version 1 > /dev/null  # resolve check
cargo fmt --all -- --check                               # formatting gate
cargo check --workspace                                 # typecheck all crates
cargo test --workspace                                  # unit + integration tests
```

WASM smoke test (Phase 01 acceptance; not claimed until it passes):

```bash
rustup target add wasm32-unknown-unknown
cargo check -p tbg-client --target wasm32-unknown-unknown
```
