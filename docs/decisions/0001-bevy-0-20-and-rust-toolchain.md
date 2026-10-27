# 0001 — Bevy 0.20 and Rust Toolchain Baseline

**Status:** Accepted · **Date:** 2026-10-27

## Context

The reboot targets Bevy 0.20 for the browser client, subject to
compatibility checks against the actual environment.

## Verification performed

- `cargo info bevy` resolves **0.20.0** on crates.io (MIT OR Apache-2.0,
  rust-version 1.97.1) — the task's target version exists; no fallback
  needed.
- Environment toolchain is Rust 1.99.0, satisfying bevy 0.20's MSRV.
- `cargo info axum` resolves 0.8.9 (MIT), compatible with the pinned range.

## Decision

- Pin `bevy = "0.20"` for `tbg-client`.
- Pin `rust-toolchain.toml` to the `stable` channel (MSRV 1.97.1 via bevy).
- Re-validate on every Bevy minor bump; WebGL2/WASM support is rechecked
  in Phase 01, not assumed from this record.
