# AGENTS.md — TBG 2.0 (Tupi Battle Grounds)

The only instruction file for every coding agent (Codex, Claude Code,
OpenCode, Cursor). Read it, then read only what your task needs.

## Mission

Ship a beautiful, fun, real-time 1v1 browser strategy game with a Rust
authoritative server, built mostly by AI agents, at minimal running cost.
**A playable game is the goal. Infrastructure is not.** This is an indie
MVP, not enterprise software: speed, visual quality, fun, working
multiplayer, low cost.

**Platform focus (owner rule, 2026-10-29): desktop browsers only, for
now.** Mobile is a later maturity stage of the game. Do not design,
verify, or fix phone layouts or touch input unless a task explicitly asks;
keep existing phone code compiling, but spend no effort on it.

## Map

| Path | Contents |
|------|----------|
| `docs/GAME-DESIGN.md` | Canon: pillars, glossary, numbered rules, MVP values, fun metrics |
| `docs/ARCHITECTURE.md` | Stack, crate boundaries, simulation/network model, art pipeline, budgets, deploy, scaling |
| `docs/ROADMAP.md` | Phase index, order, status, executor and reviewer workflow |
| `docs/phases/PN-*.md` | One executable spec per phase: steps, exact types, commands, acceptance, report |
| `docs/DECISIONS.md` | Why the big choices were made and what reopens them |
| `docs/ART-DIRECTION.md` | Medieval style guide, rig and motion rules, character procedure (visual phases P15+) |
| `crates/tbg-core` | Deterministic rules, content tables, bot, protocol (shared) |
| `crates/tbg-client` | Bevy client → WASM/WebGL2 |
| `crates/tbg-server` | Axum/Tokio authoritative server |
| `crates/tbg-balance` | Offline balance lab: millions of headless matches, gate, search (D-33) |
| `archive/` | Legacy project, read-only (`archive/ARCHIVE-MANIFEST.md`) |

## Non-negotiables

1. **Game rules.** `docs/GAME-DESIGN.md` is canon. Use glossary terms
   exactly (`BattleLine`, `PlayerIntention`, `Archetype`,
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

## How to work a phase (executor)

1. Open the phase file named in `docs/ROADMAP.md`. Read only what its
   "Read first" list names — not the whole repository, and not
   `archive/` unless the phase says so.
2. Work on branch `phase/<pN-name>`. Do the steps in order; run each
   step's check before the next; commit after each step; never push.
3. The phase file is the spec. Do not invent rules, numbers, message
   types, dependencies, or architecture it does not name. If something is
   ambiguous, take the simplest reading and record it under *Deviations*.
4. Climb the YAGNI ladder (adapted from Ponytail, MIT) and stop at the
   first rung that holds: Does this need to exist? Does the repo already
   have it? Does `std` or an existing dependency do it? Can it be one
   line? Only then write the minimum code that works.
5. Bevy 0.20 changed many APIs. Copy patterns from the Bevy examples on
   disk (`~/.cargo/registry/src/index.crates.io-*/bevy-0.20.0/examples/`),
   never from memory of older versions.
6. Same error twice → stop that step, write a `> blocked:` note in the
   phase report (tried / exact error / suspected cause), continue only
   with steps that do not depend on it.
7. Finish by filling in the phase report with real command output and
   measurements, commit it, and stop for review.

## Autonomy

Make routine engineering decisions yourself: libraries within the stack,
module layout inside a crate, naming, refactors your task needs. Ask a
human only for: game-rule or product changes, anything paid or
irreversible (VPS, domain, license, deleting data), subjective sign-off on
look and fun, or requirements that genuinely conflict.

## Roles and cost

| Role | Model | Does |
|------|-------|------|
| Executor | cheaper model (e.g. DeepSeek) | Implements one phase file, runs every check, writes the phase report |
| Reviewer | strong model (e.g. Claude, Codex) | Re-runs acceptance checks, reviews the diff, fixes or returns issues, merges, updates the roadmap status; writes new phase specs |
| Owner | human | Devices, VPS/domain, look and fun sign-off, product decisions, license |

Spend tokens on running the compiler and tests, not on re-reading files.
Never run two agents on the same files at the same time.

## Code standards

- One responsibility per module and per function. Split by
  responsibility — a new concern or a second reason to change gets its own
  module or folder — so the code stays decoupled and can grow. Never build
  a "god file" that mixes concerns. There is no fixed line limit: a long
  file or function is a signal to check its cohesion, not a rule to obey.
- Keep control flow shallow (cyclomatic complexity ≤ 10); split instead of
  nesting. Content tables may be long.
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

Contract tests (`tests/contract_*.rs`, `tbg-balance/tests/contract.rs`)
are append-only: change one only in the same commit as the GAME-DESIGN or
DECISIONS change it guards, and name that change in the commit message.
Balance numbers change only with `tbg-balance` evidence (D-33); gate
thresholds are product decisions, never test fixes.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

Add per change type: client change → `trunk build --release` in
`crates/tbg-client/` and look at the page in a browser; visual change →
gallery screenshots at desktop size (phone only when a task asks, see
*Platform focus*); multiplayer change → two
clients finish a match, including a reconnect. A native build never
proves browser compatibility. Never claim a check you did not run; quote
failures exactly.

## Definition of done

Implemented · compiles (native and WASM where relevant) · verification
passes · behavior observed working · integrated (no orphaned code) · docs
updated when a rule, architecture choice, or decision changed · roadmap
report filled in · limitations reported.
