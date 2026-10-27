# AGENTS.md — TBG 2.0 (Tupi Battle Grounds)

> **Read this first** in any new agent session (Cursor, Claude Code, Codex,
> Windsurf, etc.). This repo is the **TBG 2.0 game workspace**: a Rust
> Cargo workspace for a browser-first 1v1 strategy game.
> `archive/` holds the retired implementation (read-only backup).

**Language:** 100% English — code, comments, docs, commits, and all agent output.

---

## 1. What this repo is

| Is | Is not |
|----|--------|
| Rust workspace: `tbg-core`, `tbg-protocol`, `tbg-client`, `tbg-server`, `tbg-assets` | Unity, Nakama, or Python application |
| Browser-first game (Bevy → WASM/WebGL2) + authoritative Rust server | Playable game yet (Phase 00 scaffolding only) |
| Contract-first, YAGNI, documented codebase | Harness-only repo (that era ended 2026-10-27) |

**Authoritative references, in order:** game rules
(`docs/game-design/`) → architecture (`docs/architecture/`) → roadmap
(`docs/ROADMAP.md`) → decisions (`docs/decisions/`).

---

## 2. Business rules (game domain — never reinterpret)

- Ubiquitous language is mandatory: [`docs/game-design/GLOSSARY.md`](docs/game-design/GLOSSARY.md).
  Use `BattleLine`, `PlayerIntention`, `SlotCount`, `Archetype`, `Faction`,
  `Energy`, `Shop`, `Evolution`, `Synergy` exactly. Never invent synonyms.
- Numbered rules live in [`docs/game-design/BUSINESS-RULES.md`](docs/game-design/BUSINESS-RULES.md)
  with status (confirmed / adapted / proposed / retired).
- Never invent balance numbers or mechanics: anything not established is
  listed in [`docs/game-design/MECHANICS.md`](docs/game-design/MECHANICS.md)
  and [`docs/game-design/TRACEABILITY.md`](docs/game-design/TRACEABILITY.md).
- `tbg-core` owns the domain; `tbg-server` runs it authoritatively;
  `tbg-client` renders and predicts only.

---

## 3. Engineering standards

- **Crate boundaries** (`docs/architecture/RUST-WORKSPACE.md`): `tbg-core`
  takes no graphics/DB/network deps; `tbg-server` never depends on Bevy
  rendering; `tbg-protocol` is types only; `tbg-assets` is contracts only.
  No circular dependencies.
- **Size caps:** 80 lines/function, 200 lines/file, cyclomatic complexity
  ≤ 10 (inherited Agent Harness rule; applies to all new Rust/Python).
- **Contract-first tests:** before any test, read
  `agent-rules/04-testing/contract-first-tests.md`. Assert documented
  contracts; never mirror implementation. Pyramid: unit 75% / integration
  20% / E2E 5%.
- **Minimal implementation (Ponytail):** smallest correct, readable,
  verifiable change. No speculative abstractions, frameworks, or features.
  `.cursor/rules/ponytail.mdc` always applies.
- **Security:** server validates every intention; never trust
  client-reported damage/balances/results; secrets in environment only;
  see `agent-rules/03-security/` and `docs/architecture/NETWORKING.md`.
- **Verification after each edit:** run the smallest gate covering the
  change (`cargo fmt`, `cargo check -p <crate>`, `cargo test --workspace`).
  Never claim tests passed without running them.

---

## 4. Tooling instructions

- Toolchain: stable Rust; `rust-toolchain.toml` applies automatically.
  MSRV 1.97.1 (via bevy 0.20).
- Workspace commands (repo root):
  `cargo metadata --no-deps`, `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo test --workspace`.
- WASM client gate (Phase 01+):
  `cargo check -p tbg-client --target wasm32-unknown-unknown`.
  Native check success never counts as WASM validation.
- Harness helpers: `agent-harness/rules-path.sh`,
  `agent-harness/resolve-rules.sh <keywords>`. Rule library lives in
  `agent-rules/` (symlink into `.agent-harness/` — never edit inside
  the submodule).
- Task-scoped rules: resolved via `resolve-rules.sh`, 2–6 files only.

---

## 5. Legacy archive (read-only)

- Game-design sources: `archive/docs/` (ADRs, glossary, infra overview).
- Manifest and recovery guide: [`archive/ARCHIVE-MANIFEST.md`](archive/ARCHIVE-MANIFEST.md).
- **Never** import from `archive/` into active code, CI, or builds.
- **Never** delete or rewrite `archive/`; git history + tag
  `pre-tbg20-20261027` are the preservation mechanisms.
- Legacy names in the glossary's "retired" table (`CharacterDefinition`,
  `CharacterCatalog`, …) must not be reused for new concepts.

---

## 6. Agent protocol

1. State which rule files you loaded (brief list).
2. Ask when domain/business-rule answers are blank — never assume them.
3. Smallest diff; one logical change per commit.
4. Verify after each edit; record failed or skipped validations honestly.
5. English only in all artifacts.
