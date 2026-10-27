# 0002 — Workspace at Repository Root (No Nested `tbg/` Directory)

**Status:** Accepted · **Date:** 2026-10-27

## Context

The reboot brief sketches a `tbg/` tree containing `Cargo.toml`,
`crates/`, `docs/`, `archive/`, `.agent-harness/`, and `.cursor/` —
i.e. the full repository contents.

## Decision

The repository root **is** the workspace root. `Cargo.toml`,
`rust-toolchain.toml`, `crates/`, `content/`, `tools/`, `tests/`, and
`deploy/` sit beside `archive/`, `.agent-harness/`, and `.cursor/`
instead of inside a nested `tbg/` directory.

Rationale: a nested directory would duplicate the repo boundary, break
the existing submodule/symlink layout, and add path noise for zero
isolation benefit. The adaptation is structural only; all specified
members exist with their specified responsibilities.
