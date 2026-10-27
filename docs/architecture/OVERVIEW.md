# Architecture Overview — TBG 2.0

> Rust-first, browser-first reboot of Tupi Battle Grounds. The game rules in
> `docs/game-design/` are authoritative; this document describes the system
> that will run them. Nothing below is implemented yet except the workspace
> scaffolding (Phase 00).

## Principles

1. **Server-authoritative competition.** The backend runs game rules and
   simulation; the browser renders, animates, and interpolates. Client
   input is always an intention, never a verdict.
2. **Shared deterministic domain.** One Rust crate (`tbg-core`) holds the
   rules, compilable for native and WASM targets without graphics,
   database, or network code.
3. **Browser-first delivery.** The playable client is a Bevy application
   compiled to WebAssembly, served as static files alongside game assets.
4. **Solo-maintainable operations.** One small self-hosted VPS, SQLite
   persistence, Docker Compose deployment. No Kubernetes, no managed
   service required before metrics demand it.
5. **Minimal, boring dependencies.** Each crate does one job; circular
   dependencies are forbidden; heavy frameworks stay at the edges
   (Bevy in the client, Axum/Tokio in the server).

## System map

```
┌──────────────────────────────────────────────────────────────┐
│ BROWSER (desktop + mobile)                                   │
│  tbg-client (Bevy 0.20 → WASM/WebGL2)                        │
│  rendering · animation · VFX · audio · input → intentions    │
└──────────────────────────┬───────────────────────────────────┘
                           │ WebSocket (tbg-protocol messages)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│ VPS (single small node)                                      │
│  tbg-server (Axum + Tokio, native)                           │
│  matchmaking · fixed-tick simulation (tbg-core) · accounts   │
│  SQLite persistence                                          │
└──────────────────────────┬───────────────────────────────────┘
                           │ static files (CDN or same VPS)
                           ▼
                   WASM bundle + GLB assets + content JSON
```

## Crate responsibilities

| Crate | Depends on | Must never depend on |
|-------|-----------|----------------------|
| `tbg-core` | std only (+ tiny deterministic helpers) | bevy, axum, tokio, rusqlite, network, fs in game logic |
| `tbg-protocol` | serde | bevy, axum, game logic |
| `tbg-client` | bevy, tbg-core, tbg-protocol | axum, tokio server runtime, rusqlite |
| `tbg-server` | axum, tokio, tbg-core, tbg-protocol, rusqlite | bevy (renderer forbidden) |
| `tbg-assets` | serde/validation only | bevy render pipeline, heavy generation runtimes |

Dependency direction: `tbg-client` → `tbg-core` ← `tbg-server`;
`tbg-protocol` is shared types only; `tbg-assets` is contracts only.

## Technology choices (locked for the reboot)

| Layer | Choice | Reference |
|-------|--------|-----------|
| Language | Rust (pinned toolchain, see `rust-toolchain.toml`) | — |
| Game framework | Bevy 0.20 (verified on crates.io 2026-10-27) | `RUST-WORKSPACE.md` |
| Browser runtime | WebAssembly (`wasm32-unknown-unknown`), WebGL2 graphics target | `NETWORKING.md`, Phase 01 acceptance |
| Backend | Axum + Tokio (native) | `INFRASTRUCTURE.md` |
| Communication | WebSocket with versioned `tbg-protocol` messages | `NETWORKING.md` |
| Database | SQLite (via rusqlite) for initial persistence | `INFRASTRUCTURE.md` |
| Asset format | glTF 2.0 / GLB, validated before packaging | `ASSET-PIPELINE.md` |
| Asset production | Codex-driven procedural scripts → headless Blender → GLB | `ASSET-PIPELINE.md` |
| Deployment | Static CDN + one small VPS, Docker Compose | `INFRASTRUCTURE.md`, `deploy/` |
| License posture | MIT objective; every dependency and asset license audited | root `README.md`, `THIRD_PARTY_NOTICES.md` |

## What changed from legacy (retired stack)

| Legacy (retired) | 2.0 direction | Rationale |
|------------------|---------------|-----------|
| Unity C# client | Bevy/Rust/WASM browser client | Solo-maintainable, open-source, no engine licensing weight |
| Nakama + TypeScript match module | Rust authoritative server (`tbg-server`) + `tbg-core` | One language, deterministic shared domain, same single-VPS model |
| PostgreSQL via Nakama | SQLite | Zero-ops persistence at initial scale; upgrade path kept |
| Per-character MB-Lab/Mixamo rigging | Modular shared-rig procedural factory | Legacy pipeline measured ~8% auto pass rate (ADR-007); not viable |
| Native mobile distribution | Browser-first (mobile browsers included) | Reach without store pipelines; native deferred |

## Document map

- [`RUST-WORKSPACE.md`](RUST-WORKSPACE.md) — workspace layout, crate contracts, toolchain.
- [`NETWORKING.md`](NETWORKING.md) — protocol, tick model, reconciliation, reconnection.
- [`ASSET-PIPELINE.md`](ASSET-PIPELINE.md) — Codex → Blender → GLB → validation flow.
- [`INFRASTRUCTURE.md`](INFRASTRUCTURE.md) — VPS topology, deploy, persistence, ops.
- [`../ROADMAP.md`](../ROADMAP.md) — phased delivery plan.
- [`../decisions/`](../decisions/) — per-decision records.
