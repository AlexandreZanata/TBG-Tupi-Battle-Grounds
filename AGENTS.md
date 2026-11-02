# AGENTS.md — TBG 2.0 (Tupi Battle Grounds)

The only instruction file for every coding agent (Codex, Claude Code,
OpenCode, Cursor). Read it, then read only what your task needs.

## Mission

Ship a beautiful, fun, real-time 1v1 browser strategy game with a Rust
authoritative server, built mostly by AI agents, at minimal running cost.
**A playable game is the goal. Infrastructure is not.** This is an indie
MVP, not enterprise software: speed, visual quality, fun, working
multiplayer, low cost.

**Platform focus (owner rule, 2026-11-01): Linux, Windows and desktop
browsers, with Linux as the visual reference.** Mobile is a later maturity
stage of the game. Do not design,
verify, or fix phone layouts or touch input unless a task explicitly asks;
keep existing phone code compiling, but spend no effort on it.

**Local validation (owner rule, 2026-10-31): minimum per work front;
targeted tests once at consolidation; full suite only on an explicit owner
request.** This policy overrides older phase verification blocks and
per-step demands for broad checks. See *Verification* below and
`docs/VALIDATION.md`.

## Map

| Path | Contents |
|------|----------|
| `docs/GAME-DESIGN.md` | Canon: pillars, glossary, numbered rules, MVP values, fun metrics |
| `docs/ARCHITECTURE.md` | Stack, crate boundaries, simulation/network model, art pipeline, budgets, deploy, scaling |
| `docs/ROADMAP.md` | Phase index, order, status, executor and reviewer workflow |
| `docs/phases/PN-*.md` | One executable spec per phase: steps, exact types, commands, acceptance, report |
| `docs/DECISIONS.md` | Why the big choices were made and what reopens them |
| `docs/VALIDATION.md` | Minimal local checks, shared resource lock, consolidation, explicit full-suite procedure |
| `docs/ART-DIRECTION.md` | Medieval style guide, rig and motion rules, character procedure (visual phases P15+) |
| `docs/ui/DESKTOP-UI-CONTRACT.md` | Core visual correspondence: Linux reference, identical Windows and desktop web presentation |
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
6. **Git.** Never force-push or rewrite history. The owner explicitly
   authorized the P71 delivery workflow on 2026-11-01: commit completed
   changes, push the execution branch to GitHub, then reinstall the native
   Linux playtest. Apply that same workflow to each subsequent requested
   targeted adjustment. This does not authorize merging or public deployment. One logical change
   per commit. No build outputs or large binaries in git.
7. **English only** in code, comments, docs, and commits.
8. **Visual correspondence — core owner rule (2026-11-01).** Linux is
   the canonical reference for the entire in-game frontend. **Every visual
   change must look and behave exactly the same on Linux, Windows and
   desktop web.** This applies to every feature, fix, refactor, asset and
   optimization, across all screens, UI components, HUDs, modals, cards,
   typography, layout, loading/error states and transitions; all Arenas,
   cameras, models, Evolution forms, materials, colors, lighting, shadows,
   VFX and animations; and every champion's visible behavior, including
   spawning, movement, facing, attacks, abilities, hit/death feedback and
   celebrations, with matching timing and motion.
   Every such change must include an entry in the
   [visual correspondence contract](docs/ui/DESKTOP-UI-CONTRACT.md),
   identifying the affected cases and updating its matrix/motion coverage
   and shared presentation in the same change. Implement once and propagate
   to all three targets. Platform adapters may differ internally only to
   preserve identical presentation; no missing feature, visual downgrade,
   separate platform styling, tolerance or renderer/driver waiver is
   allowed without an explicit owner-approved contract change. Visible
   champion behavior remains subject to server authority and game canon.
   Follow `docs/VALIDATION.md`: queue scoped cross-platform checks at
   consolidation, record unverified targets as pending, and never claim
   parity from shared code, compilation or Linux-only screenshots.

## How to work a phase (executor)

1. Open the phase file named in `docs/ROADMAP.md`. Read only what its
   "Read first" list names — not the whole repository, and not
   `archive/` unless the phase says so.
2. Work on branch `phase/<pN-name>`. Do the steps in order; apply the
   minimal validation policy to each step, queue broader checks for
   consolidation, commit after each step, and never push.
3. The phase file is the implementation spec; *Verification* below governs
   when its checks run. Do not invent rules, numbers, message
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
7. Finish the phase report with the validation level, real command output,
   measurements, and exact deferred checks with reasons. Commit it and stop
   for review; deferred validation is never a pass.

## Autonomy

Make routine engineering decisions yourself: libraries within the stack,
module layout inside a crate, naming, refactors your task needs. Ask a
human only for: game-rule or product changes, anything paid or
irreversible (VPS, domain, license, deleting data), subjective sign-off on
look and fun, or requirements that genuinely conflict.

## Roles and cost

| Role | Model | Does |
|------|-------|------|
| Executor | cheaper model (e.g. DeepSeek) | Implements one phase file, runs minimal scoped checks, queues tests, writes the phase report |
| Reviewer | strong model (e.g. Claude, Codex) | Reviews the diff; owns deduplicated targeted validation at consolidation; fixes or returns issues, merges, updates the roadmap status; writes new phase specs |
| Owner | human | Devices, VPS/domain, look and fun sign-off, product decisions, license |

Choose the smallest useful check and reuse valid evidence for the same code.
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
Writing a regression test does not require running a crate or workspace
suite on that work front; queue its exact command for consolidation.

Contract tests (`tests/contract_*.rs`, `tbg-balance/tests/contract.rs`)
are append-only: change one only in the same commit as the GAME-DESIGN or
DECISIONS change it guards, and name that change in the commit message.
Balance numbers change only with `tbg-balance` evidence (D-33); gate
thresholds are product decisions, never test fixes.

## Verification

Follow `docs/VALIDATION.md`. Local work defaults to **work-front mode**:

- Docs/comments/assets only: inspect the diff and relevant links or file
  formats; run `git diff --check`. No Cargo, Trunk, or browser build.
- Rust changes: formatting for the affected package and, if the existing
  cache makes it inexpensive, one `cargo check -p <crate>` narrowed to
  `--lib`, `--bin <name>`, or the actual target. Defer cold Bevy builds or
  additional target builds. Do not check native and WASM by habit.
- Do not run `cargo test` by default on individual fronts. Write needed
  regression tests and hand their exact commands to consolidation. An
  explicit request for a particular test authorizes only that scope.
- No automatic workspace check/clippy/test, release or packaging builds,
  balance gates, benchmarks, ignored tests, repeated galleries, or full
  multiplayer scenarios. Older phase acceptance lists are a backlog for
  consolidation, not permission to start these on every front.

Every Rust/Trunk validation that compiles must use the same nonblocking
host lock across worktrees, with at most **2 Cargo build jobs**. Example:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo check -p tbg-core --lib
```

Lock busy (exit 75), missing lock support, or unavailable cache → record
the check as deferred and continue independent work. Do not wait, poll,
bypass the lock, raise parallelism, run `cargo clean`, or create a fresh
target directory to evade contention. Preserve active sessions and builds.

**Consolidation:** one designated reviewer integrates the fronts and runs
the union of affected tests once, under that lock, with required focused
target/browser/multiplayer checks. A consolidation or review request does
not authorize the full suite; defer unrelated or expensive acceptance work.

**Full suite:** only an explicit owner request for the full suite (or named
heavy checks) authorizes that scope. Run it once on the consolidated tree,
sequentially under the lock, using the reference commands in
`docs/VALIDATION.md`. Do not infer permission from "finish", "review",
"merge", a phase checklist, or a failed focused check.

Report commands, results, and deferred checks separately. Re-run only after
relevant code changes or unresolved failures. A native build never proves
browser compatibility; deferred checks never support a verified claim.

## Definition of done

Work-front delivery: implemented · minimal validation passed or explicitly
deferred · relevant tests written · no orphaned code · docs/report updated ·
limitations and consolidation checks listed. Consolidation records the
targeted checks actually completed on the integrated tree. Do not claim full
verification or release readiness while applicable acceptance is deferred;
the owner controls when the full suite runs.
