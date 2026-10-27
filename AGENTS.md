# AGENTS.md — TBG 2.0 (Tupi Battle Grounds)

> **Read this first** in any new agent session (Cursor, Claude Code, Codex,
> Windsurf, OpenCode, etc.). This repo is the **TBG 2.0 game workspace**:
> an AI-built indie browser game plus its authoritative Rust server.
> `archive/` holds the retired implementation (read-only backup).

**Language:** 100% English — code, comments, docs, commits, and all agent output.

**Prime directive:** TBG 2.0 is an AI-built indie game, not enterprise
software. Optimize for **speed + visual quality + fun + functional
multiplayer + low cost**. Default behavior: **implement, verify, report,
continue.** Full policy: [`docs/AI-FIRST-MVP.md`](docs/AI-FIRST-MVP.md).

---

## 1. What this repo is

| Is | Is not |
|----|--------|
| AI-built browser game (Bevy → WASM/WebGL2) + authoritative Rust server | Enterprise software; Unity/Nakama/Python app |
| MVP/pre-release: playable 1v1 first, polish early, harden later | Perfect, exhaustively tested, webscale system |
| Rust workspace: `tbg-core`, `tbg-protocol`, `tbg-client`, `tbg-server`, `tbg-assets` | Microservices, Kubernetes, distributed infra |

**Authoritative references, in order:** game rules
(`docs/game-design/`) → AI-first policy (`docs/AI-FIRST-MVP.md`) →
architecture (`docs/architecture/`) → roadmap (`docs/ROADMAP.md`) →
decisions (`docs/decisions/`).

---

## 2. AI autonomy (agents do the work)

AI agents own architecture within project boundaries, Rust, WASM/Bevy,
multiplayer, gameplay, UI/UX, procedural assets, validation, debugging,
and necessary docs — working autonomously from spec to verified code.

- Decide routine engineering questions independently; reuse libraries
  instead of reinventing; simplest solution that satisfies the requirement.
- Resolve compilation errors autonomously; no architecture bake-offs
  when a reasonable choice exists; no planning documents for small tasks.
- Never stop at code and never declare success without evidence.
- Ask humans only for: major product decisions, irreversible operations,
  missing essential requirements, genuinely conflicting constraints.

---

## 3. MVP-first delivery

Build only what the current milestone requires (`docs/ROADMAP.md`).
Working features beat perfect unfinished ones; visible progress every
phase. No microservices, Kubernetes, distributed DBs, caching layers,
enterprise patterns, generic frameworks, complex monitoring, scaling
systems, speculative features, or test frameworks heavier than the
feature. Localized, understood tech debt is acceptable when it does not
block the next milestone. Build order: arena → units → auto-combat →
two-player connection → authoritative sim → playable 1v1 → polish.

---

## 4. Minimal verification (no coverage gates)

No coverage percentages, no exhaustive suites, no test-per-feature
mandate. Prefer one meaningful gameplay smoke test over dozens of
low-value tests. Verify per change: compiles, existing checks pass,
behavior works, full match can finish (gameplay), two real clients play
a full match incl. reconnect (multiplayer), assets import/play/look
right on mobile. Focused regression tests only where meaningfully
protective. Never skip correctness where player data, fairness, or
multiplayer results are at stake.

---

## 5. Pragmatic security baseline (mandatory, minimal)

Never expose secrets. Never trust client-reported damage, energy,
inventory, or victories — simulation stays server-authoritative with
validated messages, size/resource limits, and basic rate limits. No
cross-player session control; HTTPS/WSS publicly; no unsafe DB
operations or obvious injection flaws; no known critical
vulnerabilities shipped. Advanced anticheat, fraud detection, and
enterprise identity wait until after gameplay validation.

---

## 6. Business rules (game domain — never reinterpret)

- Ubiquitous language is mandatory: [`docs/game-design/GLOSSARY.md`](docs/game-design/GLOSSARY.md).
  Use `BattleLine`, `PlayerIntention`, `SlotCount`, `Archetype`, `Faction`,
  `Energy`, `Shop`, `Evolution`, `Synergy` exactly. Never invent synonyms.
- Numbered rules in [`docs/game-design/BUSINESS-RULES.md`](docs/game-design/BUSINESS-RULES.md)
  (confirmed / adapted / proposed / retired). Never invent balance numbers.
- `tbg-core` owns the domain; `tbg-server` runs it authoritatively;
  `tbg-client` renders and predicts only. A gameplay rule changes only
  when a task explicitly authorizes a product-design change.

---

## 7. Engineering standards

- **Crate boundaries** (`docs/architecture/RUST-WORKSPACE.md`): `tbg-core`
  takes no graphics/DB/network deps; `tbg-server` never depends on Bevy
  rendering; `tbg-protocol` is types only; `tbg-assets` is contracts only.
  No circular dependencies.
- **Size caps:** 80 lines/function, 200 lines/file, cyclomatic ≤ 10.
- **Contract-first tests** (relaxed): assert documented contracts, never
  mirror implementation; failing-first only where meaningfully protective.
- **Minimal implementation (Ponytail):** smallest correct, readable,
  verifiable change; `.cursor/rules/ponytail.mdc` always applies, with the
  MVP-first exceptions in `docs/AI-FIRST-MVP.md` §13.
- **Cost-aware models:** cheap models for boilerplate/content/simple
  scripts and fixes; strong models for architecture, hard compiler or
  Bevy/WASM problems, sync/determinism, critical gameplay bugs. Read only
  task-relevant files. On repeated cheap-model failure: stop retrying,
  write a concise diagnostic handoff.
- **Verification after each edit:** smallest gate covering the change
  (`cargo fmt`, `cargo check -p <crate>`, `cargo test --workspace`,
  browser smoke checks). Never claim tests passed without running them.

---

## 8. Tooling instructions

- Toolchain: stable Rust; `rust-toolchain.toml` applies automatically.
  MSRV 1.97.1 (via bevy 0.20). This file is the single policy source for
  Codex, Claude Code, and OpenCode; `.cursor/rules/` is the same policy
  surfaced for Cursor (`mvp-first.mdc` takes precedence on conflicts).
- Workspace commands (repo root):
  `cargo metadata --no-deps`, `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo test --workspace`.
- WASM client gate (Phase 01+):
  `cargo check -p tbg-client --target wasm32-unknown-unknown`.
  Native check success never counts as WASM validation.
- Harness helpers: `agent-harness/rules-path.sh`,
  `agent-harness/resolve-rules.sh <keywords>`. Rule library lives in
  `agent-rules/` (symlink into `.agent-harness/` — never edit inside
  the submodule). Task-scoped rules: resolved via `resolve-rules.sh`,
  2–6 files only. MVP-first project overrides win over inherited
  enterprise rigor — see `docs/AI-FIRST-MVP.md` §13.

---

## 9. Legacy archive (read-only)

- Game-design sources: `archive/docs/` (ADRs, glossary, infra overview).
- Manifest and recovery guide: [`archive/ARCHIVE-MANIFEST.md`](archive/ARCHIVE-MANIFEST.md).
- **Never** import from `archive/` into active code, CI, or builds.
- **Never** delete or rewrite `archive/`; git history + tag
  `pre-tbg20-20261027` are the preservation mechanisms.
- Legacy names in the glossary's "retired" table (`CharacterDefinition`,
  `CharacterCatalog`, …) must not be reused for new concepts.

---

## 10. Agent protocol

1. State which rule files you loaded (brief list).
2. Implement, verify with evidence, report, continue — one logical change
   per commit; ask humans only for the §2 escalation cases.
3. Verify after each edit; record failed or skipped validations honestly.
4. Definition of done: implemented, compiles, behavior verified,
   integrated, no obvious critical regression, visually acceptable where
   applicable, limitations reported.
5. English only in all artifacts.
