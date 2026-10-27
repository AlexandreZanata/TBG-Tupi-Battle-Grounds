# AGENTS.md — TBG 2.0 (Tupi Battle Grounds)

The only instruction file for every coding agent (Codex, Claude Code,
OpenCode, Cursor). Read it, then read only what your task needs.

## Mission

Ship a beautiful, fun, real-time 1v1 browser strategy game with a Rust
authoritative server, built mostly by AI agents, at minimal running cost.
**A playable game is the goal. Infrastructure is not.** This is an indie
MVP, not enterprise software: speed, visual quality, fun, working
multiplayer, low cost.

## Map

| Path | Contents |
|------|----------|
| `docs/GAME-DESIGN.md` | Canon: pillars, glossary, numbered rules, MVP values, fun metrics |
| `docs/ARCHITECTURE.md` | Stack, crate boundaries, simulation/network model, art pipeline, budgets, deploy, scaling |
| `docs/ROADMAP.md` | Phases, tasks with tiers and acceptance criteria, the next task |
| `docs/DECISIONS.md` | Why the big choices were made and what reopens them |
| `crates/tbg-core` | Deterministic rules, content tables, bot, protocol (shared) |
| `crates/tbg-client` | Bevy client → WASM/WebGL2 |
| `crates/tbg-server` | Axum/Tokio authoritative server |
| `archive/` | Legacy project, read-only (`archive/ARCHIVE-MANIFEST.md`) |

## Non-negotiables

1. **Game rules.** `docs/GAME-DESIGN.md` is canon. Use glossary terms
   exactly (`BattleLine`, `PlayerIntention`, `SlotCount`, `Archetype`,
   `Faction`, `Energy`, `Shop`, `Evolution`, `Synergy`, `Stronghold`); no
   synonyms. Change a rule or number only when your task says so; any new
   value goes into GAME-DESIGN marked **Proposed** in the same commit.
2. **Authority.** The server decides combat, `Energy`, shop, evolution,
   and outcomes. Clients send `PlayerIntention`s only. The seat comes from
   the connection, never from message content.
3. **Crate boundaries** (ARCHITECTURE §3). `tbg-core`: no I/O, graphics,
   async, clocks, floats in simulation state, unseeded randomness, or
   `HashMap` iteration in the simulation. `tbg-server`: never Bevy.
   `tbg-client`: never decides online outcomes.
4. **Security baseline.** No secrets in git. Validate and size-limit every
   inbound message; never panic on client input; rate limits and
   timeouts as specified; TLS for anything public.
5. **Archive.** Never import from, edit, or delete `archive/`.
6. **Git.** Never push, force-push, or rewrite history. One logical change
   per commit. No build outputs or large binaries in git.
7. **English only** in code, comments, docs, and commits.

## How to work a task

1. Pick a task ID from `docs/ROADMAP.md`. Read the files and doc sections
   it names — not the whole repository, and not `archive/` unless the
   task needs legacy material.
2. Climb the YAGNI ladder (adapted from Ponytail, MIT) and stop at the
   first rung that holds: Does this need to exist? Does the repo already
   have it? Does `std` or an existing dependency do it? Can it be one
   line? Only then write the minimum code that works.
3. Implement the smallest change that meets the task's *Done when*.
4. Verify (below) and look at the behavior yourself.
5. Commit, tick the roadmap box with the commit hash, and report: what
   changed, the commands you ran with their results, and known limitations.
6. Stuck after two honest attempts: write a three-line `> blocked:` note
   under the task (tried / exact error / suspected cause) and escalate one
   tier. Do not keep retrying the same approach.

## Autonomy

Make routine engineering decisions yourself: libraries within the stack,
module layout inside a crate, naming, refactors your task needs. Ask a
human only for: game-rule or product changes, anything paid or
irreversible (VPS, domain, license, deleting data), subjective sign-off on
look and fun, or requirements that genuinely conflict.

## Cost-aware model use

| Tier | Use for |
|------|---------|
| Cheap | Content tables, roster art specs, boilerplate, focused tests, docs, small fixes |
| Mid | Features with a clear spec: HUD, lobby, VFX, shop, synergy |
| Strong | Architecture, simulation core and combat, networking and concurrency, WASM/build failures, security review, bugs that resist two attempts |

Spend tokens on running the compiler and tests, not on re-reading files
or repeated AI reviews. Never run two agents on the same task.

## Code standards

- Function ≤ 80 lines, file ≤ 200 lines, cyclomatic complexity ≤ 10.
  Split instead of nesting. Content tables are exempt from the file cap.
- Each module starts with a `//!` line stating its responsibility.
  Comments explain *why*, not *what*.
- No new dependency without a one-line reason comment in `Cargo.toml`.
- No trait, generic, or layer for a single implementation; no frameworks.
- No `unwrap`/`expect` on network input or other untrusted data.
- Mark deliberate shortcuts with a `ponytail:` comment naming the limit
  and the upgrade path.

## Tests (proportional to risk)

Write tests where competitive correctness is at stake: determinism,
`Energy`, shop, evolution, synergy, outcomes, protocol validation, seat
ownership. Assert the documented contract (cite the rule ID), not the
implementation. One meaningful scenario beats ten trivial assertions. No
coverage targets. Never delete a useful test to make a change pass.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

Add per change type: client change → `trunk build --release` in
`crates/tbg-client/` and look at the page in a browser; visual change →
gallery screenshots at desktop and phone size; multiplayer change → two
clients finish a match, including a reconnect. A native build never
proves browser compatibility. Never claim a check you did not run; quote
failures exactly.

## Definition of done

Implemented · compiles (native and WASM where relevant) · verification
passes · behavior observed working · integrated (no orphaned code) · docs
updated when a rule, architecture choice, or decision changed · roadmap
box ticked · limitations reported.
