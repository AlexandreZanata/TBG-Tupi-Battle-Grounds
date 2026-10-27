# AI-First MVP Policy — TBG 2.0

> Full development policy for the AI-built indie game phase.
> `AGENTS.md` is the entry point; this document is the reference.
> Gameplay canon lives in `docs/game-design/`; this file never overrides it.

## 1. Prime directive

TBG 2.0 is an AI-built indie game, not enterprise software. Optimize for
**speed + visual quality + fun + functional multiplayer + low cost**.
AI agents perform essentially all development; humans give product
direction, judge gameplay and visuals, and approve major decisions.

Default behavior: **implement, verify, report, continue.**

## 2. Core philosophy

1. Working software over elaborate architecture.
2. Playable features over theoretical perfection.
3. Visual quality and fun over internal complexity.
4. Fast iterations over extensive planning.
5. Simple solutions over sophisticated frameworks.
6. AI autonomy over constant human intervention.
7. Reuse libraries instead of reinventing functionality.
8. Build only what the current MVP requires.
9. Measure real problems before optimizing them.
10. Deliver visible progress in every phase.

## 3. AI autonomy rules

- Decide routine engineering questions independently; do not ask
  permission for ordinary choices with an established reasonable answer.
- Implement the simplest solution that satisfies the requirement.
- Resolve compilation errors autonomously; do not present architectural
  alternatives when a reasonable choice already exists.
- Do not write extensive planning documents for small tasks.
- Never stop at code: verify the behavior works; never declare success
  without evidence.
- Ask humans only for: major product decisions, irreversible operations,
  missing essential requirements, or genuinely conflicting constraints.

## 4. MVP-first engineering

Stage: **MVP / pre-release.** Priorities: working browser game, appealing
3D graphics, functional 1v1 real-time multiplayer, smooth desktop and
mobile play, automatic combat, evolution, synergies, responsive UI,
stable short sessions, low operational cost. Everything else is secondary.

Do not introduce: microservices, Kubernetes, event-driven infrastructure,
distributed databases, caching layers, enterprise patterns, DI
frameworks, generic framework development, complex monitoring,
horizontal scaling, observability systems, speculative features, or
large test frameworks. Straightforward code that agents can read and
debug fast. Localized, understood technical debt is acceptable when it
does not block the next milestone.

## 5. Minimal verification (replaces coverage gates)

No coverage percentages. No exhaustive suites. No test-per-feature
mandate. Prefer one meaningful gameplay smoke test over dozens of
low-value tests. Never remove useful existing tests to shrink the suite.

| Change | Required verification |
|--------|-----------------------|
| Ordinary code | Compiles; relevant existing checks pass; behavior verified directly; obvious runtime errors fixed; focused regression test only if meaningfully protective |
| Critical gameplay | Main mechanic works; state stays consistent; a full match can finish; a few focused rule tests |
| Multiplayer | Two real clients connect; matchmaking works; actions reach the server; state syncs; a full match completes; basic disconnect/reconnect works |
| 3D assets | Export/import succeeds; animations play; appearance is acceptable; basic mobile performance holds |

Skipped when irrelevant; never skipped where corruption of player data,
competitive fairness, or multiplayer results is at stake.

## 6. Pragmatic security baseline (non-negotiable, minimal)

- Never expose secrets or private credentials.
- Never trust client-reported damage, energy, inventory, or victories.
- Simulation stays server-authoritative; validate incoming messages.
- Enforce basic resource and message-size limits, plus basic rate limits.
- Prevent cross-player session control; use HTTPS/WSS publicly.
- Avoid unsafe database operations and obvious injection flaws.
- Avoid shipping known critical vulnerabilities.

No advanced anticheat, fraud detection, security monitoring, or
enterprise identity until gameplay validation demands it.

## 7. Technology rules

Rust · Bevy · WebAssembly · WebGL2 · Axum + Tokio · WebSocket ·
SQLite · GLB/glTF · Codex + procedural scripts · headless Blender ·
static CDN + small VPS. No Unity, Unreal, Node app servers, Python
runtime services, or commercial multiplayer infra without evidence and
explicit approval. Python is allowed for offline Blender scripts and
dev tooling. The server never runs a graphical renderer.

## 8. Gameplay rules are preserved

`docs/game-design/` is canon: real-time 1v1, automatic combat, single
`BattleLine`, energy regen, three-offer continuous shop, branching
evolution, synergies, five archetypes, four factions, slot sizes,
server-authoritative outcomes, cosmetics-only monetization. A gameplay
rule changes only when a task explicitly authorizes a product-design
change. Obsolete implementations stay retired in `archive/`.

## 9. Asset and multiplayer priorities

Visuals: professional-looking stylized low-poly; modular parts, shared
rigs and animation libraries, procedural geometry, consistent palettes;
automated GLB export; mobile-efficient. Fewer polished characters beat
many unfinished ones: check silhouette, proportions, colors, animation
readability, materials, camera, consistency, performance. Never rebuild
the retired per-character rig/weight-paint pipeline.

Build order: arena rendering → procedural units → automatic combat →
two-player connection → authoritative simulation → playable 1v1 →
gameplay improvements → polish → content. Smallest viable multiplayer
first: one server, in-memory matches, SQLite only for necessary data.

## 10. Cost-aware model use

Cheap models: boilerplate, repetitive code, JSON content, asset
variations, simple Blender scripts, docs, small fixes, routine
refactoring and validation. Strong models: architecture decisions,
hard compiler problems, Bevy/WASM integration, sync and determinism,
critical gameplay bugs, pipeline failures, hard performance issues.
On repeated cheap-model failure: stop retrying, write a concise
diagnostic handoff. Read only task-relevant files; never load the whole
repo into context.

## 11. Rapid iteration and visual-first delivery

Per task: understand outcome → inspect relevant files → simplest working
solution → minimal checks → fix failures → report function plus
remaining issues. No unrelated refactors, no elegance rewrites, no
abstractions before multiple concrete uses. A working feature beats a
perfect unfinished one. Target desktop, Android, iPhone, and tablet
browsers; never claim a platform without testing it; fix unfinished
presentation before adding complexity.

## 12. Documentation discipline

Maintain: `README.md`, `AGENTS.md`, this file, `docs/game-design/`,
`docs/architecture/`, `docs/ROADMAP.md`. Document decisions future
agents need; keep docs synced with implementation. No redundant
architecture papers, planning artifacts, or verbose progress reports.

## 13. Exceptions to inherited harness rules

The Agent Harness submodule (`.agent-harness/`, read-only) targets
enterprise rigor. Project-level overrides, documented here and enforced
via `.cursor/rules/mvp-first.mdc`:

| Inherited requirement | MVP-first exception |
|-----------------------|---------------------|
| Domain ≥ 90% coverage; 75/20/5 pyramid automated in CI | Section 5 above; focused tests only |
| Failing-test-first TDD for every change | Failing-first only where meaningfully protective |
| Use-case documents before code | Brief decision notes; full use cases only for critical gameplay flows |
| OWASP Top 10:2025 + Agentic 2026 programs, MFA, LGPD/GDPR process | Section 6 baseline only |
| Versioned-API/error-format ceremony for every endpoint | Minimal versioned protocol (`tbg-protocol`); ceremony deferred |

Mandatory regardless of stage: ubiquitous language, crate boundaries,
size caps, no secrets in repo, server-authoritative fairness,
`archive/` preservation, English-only artifacts.

## 14. Definition of done (normal MVP feature)

Implemented · compiles · behavior verified · integrated · no obvious
critical regression · visually acceptable where applicable ·
limitations reported. Not required: 100% coverage, exhaustive edges,
architecture review, security certification, full optimization, long
docs, multiple AI approvals. Server-authoritative gameplay and
multiplayer state keep their fundamental correctness requirements.
