# Decisions

> Consequential technical and product decisions only, newest last. Each
> entry: decision, reason, and the condition that reopens it. Legacy ADRs
> stay in `archive/docs/adr/` and are never edited.

## D-01 · Rust + Bevy 0.20 browser client (WebGL2) — 2026-10-27

**Decision.** Keep Bevy 0.20 for the client, built to WASM for WebGL2 with
the `3d` and `ui` feature collections. Pin it for the whole MVP; no engine
upgrade mid-MVP.
**Why.** One maintained crate provides ECS, glTF, PBR, animation curves,
UI, input (touch enabled by `bevy_window`, part of both collections), and a
WebGL2 backend, and
it shares `tbg-core` with the server without bindings. Options considered:
*Three.js/TypeScript client* (smallest bundle and most mature on the web,
but a second language, no shared Rust rules, and a split toolchain for AI
agents); *macroquad* (tiny and simple, but weak 3D, no animation or glTF
pipeline); *three-d* (good WebGL2, small community, no UI or ECS);
*Fyrox* (weaker web story).
**Reopen if** the Phase 1 spike fails on iOS Safari or Android Chrome
after a one-day strong-agent timebox, brotli WASM exceeds 10 MB after
optimization, or the arena stress test drops below 30 fps on the
reference phone. The fallback to evaluate first is a Three.js client
speaking the same protocol.

## D-02 · Three-crate workspace at the repository root — 2026-10-27

**Decision.** `tbg-core`, `tbg-client`, `tbg-server` at the root
(supersedes former decisions 0001/0002 and the five-crate layout).
**Why.** The protocol is pure data both peers already get through
`tbg-core`; an asset-schema crate had no assets to describe. Fewer crates
mean fewer manifests and less context per task, with no boundary lost.
**Reopen if** a module needs dependencies that one side must not compile
(for example, server-only crypto in `tbg-core`).

## D-03 · Server-authoritative snapshots at 20 Hz / 10 Hz — 2026-10-27

**Decision.** The server simulates at 20 ticks/s and sends per-seat views
at 10 Hz; clients interpolate about 120 ms behind. No lockstep.
**Why.** It is the simplest model that is cheat-resistant, hides each
player's shop and `Energy`, and cannot desync. Bandwidth is small because
the lane holds about 20 units. This settles the legacy "10–20 ticks" open
question.
**Reopen if** measured bandwidth exceeds 6 KB/s per player (add delta
views first) or playtests report sluggish feedback at normal RTT.

## D-04 · Integer, dependency-free simulation core — 2026-10-27

**Decision.** `tbg-core` uses integer fixed-point state, an in-crate PCG32,
`Vec` ordering, and no dependencies except `serde`/`postcard` for the
protocol.
**Why.** Exact, reproducible outcomes on native and WASM; exact test
assertions; trivial to run headless thousands of times for balance.
**Reopen** never for the MVP.

## D-05 · Content as Rust `const` tables — 2026-10-27

**Decision.** Units, evolutions, synergies, and match constants live in
`tbg-core/src/content.rs`.
**Why.** Compiler-checked, zero parsing code, identical on both peers,
and easy for agents to edit. Balance changes need a rebuild, which takes
seconds for `tbg-core`.
**Reopen if** a non-programmer needs to tune values live (then RON files
loaded by the server and embedded in the client).

## D-06 · WebSocket + postcard, `ewebsock` on the client — 2026-10-27

**Decision.** Binary postcard frames over one WebSocket; `ewebsock` in the
client; Axum `ws` on the server. A `Hello` handshake rejects version
mismatches.
**Why.** Works in every browser, through Caddy and Cloudflare, with a
serde derive shared by both sides; about 5× smaller than JSON. WebRTC or
WebTransport add complexity a lane game does not need.
**Reopen if** head-of-line blocking is measured as a real problem.

## D-07 · Code-built figurines and animation; no rigs, no Blender — 2026-10-27

**Decision.** Units are rigid-part figurines assembled in Rust from mesh
builders on one shared 12-joint body plan, animated by a shared keyframe
library in code. Blender is only an escape hatch for a static part.
**Why.** The legacy skinned pipeline reached about 8% automatic pass rate
at 2–6 hours per unit (ADR-007). Rigid parts remove skinning, weight
painting, retargeting, export, and asset download. Agents iterate with the
compiler and a gallery screenshot. The toy-figurine style is a deliberate
look, not a placeholder.
**Reopen if** human review of the Phase 3 gallery rejects the look twice.

## D-08 · One client path for practice and online — 2026-10-27

**Decision.** `MatchLink::{Local, Net}` both produce `ServerMsg`; practice
runs `tbg-core` and the bot in the browser.
**Why.** A complete playable loop ships before networking exists, practice
mode costs nothing later, and it fills empty queues at launch.

## D-09 · Guest sessions, generated names, no accounts in the MVP — 2026-10-27

**Decision.** Random server-issued session tokens; display names generated
by the server.
**Why.** Accounts exist only to hold cosmetics (BR-C02), and there are no
cosmetics in the MVP. Generated names avoid moderation work.
**Reopen when** cosmetics or ranking enter scope.

## D-10 · SQLite for match records only, added in Phase 7 — 2026-10-27

**Why.** The MVP's only durable data is match telemetry for balance.
PostgreSQL triggers are listed in ARCHITECTURE §8.

## D-11 · One VPS: Caddy + systemd, no containers — 2026-10-27

**Decision.** One static binary under systemd behind Caddy (TLS, static
files, WS proxy). Retires the Docker Compose skeleton.
**Why.** Fewest moving parts for one Rust binary; a deploy is rsync plus
restart.
**Reopen if** a second always-on service appears.

## D-12 · Trunk for WASM builds; `bevy_ui` for all UI — 2026-10-27

**Why.** Trunk is the most widely used Bevy web build path (wasm-bindgen
and wasm-opt managed for us). `bevy_ui` keeps UI in Rust and in one
render path; an HTML overlay would split state across two languages.
**Reopen if** text input or accessibility needs exceed what `bevy_ui` can
do (room-code entry uses a share link plus an on-screen letter grid).

## D-13 · Single AGENTS.md; harness submodule and Cursor rules removed — 2026-10-27

**Decision.** Removed the `.agent-harness` submodule
(`GoodPraticesForLLMSandAgents`), its `agent-harness`/`agent-rules`
symlinks, `.cursor/rules/`, and `docs/AI-FIRST-MVP.md`. Everything still
useful is in `AGENTS.md`, which Codex, Claude Code, OpenCode, and Cursor
all read.
**Why.** Nine overlapping rule sources cost context on every task and
mostly encoded enterprise rigor the project had already overridden.
Recovery: tag `pre-simplify-20261027`.

## D-14 · MVP gameplay adaptations (need human playtest sign-off) — 2026-10-27

**Decision.** Buy and deploy are one intention; `Reposition` and the bench
are deferred; `Stronghold` win condition; 180 s + 60 s overtime with a
possible draw; slot cap 8; all values in GAME-DESIGN §5.
**Why.** Legacy design never defined a win condition, clock, numbers, or
the meaning of `Reposition`. A playable match needs all of them. Each is
labeled Proposed so it can be changed without arguing about canon.
**Reopen** after the first human playtest round (ROADMAP Phase 6).
