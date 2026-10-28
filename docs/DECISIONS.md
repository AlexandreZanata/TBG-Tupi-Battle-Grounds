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
**Reopen if** the P1 spike fails on iOS Safari or Android Chrome
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
**Reopen if** human review of the P6 gallery rejects the look twice.

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

## D-10 · SQLite for match records only, added in P5 — 2026-10-27

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
**Reopen** after the first human playtest round (P7).

## D-15 · Skeleton first: online validated before any 3D art — 2026-10-27

**Decision.** Phase order is browser spike → rules → skeleton game with
placeholder shapes → online multiplayer with latency metrics and bot load
tests → validation on the owner's VPS and domain → only then 3D art and
content. `visuals.rs` is the seam that later swaps placeholders for
figurines without touching game, network, or UI code.
**Why.** Multiplayer, latency, hosting, and capacity are the risks that
can sink the project; art is the most expensive thing to redo. Proving the
risky parts with cheap visuals first means art is never built on top of a
broken foundation.

## D-16 · Executor/reviewer workflow with phase files — 2026-10-27

**Decision.** Each phase has an executable spec in `docs/phases/` written
for a cheaper executor model (for example DeepSeek): exact types,
algorithms, file lists, commands, and a report section. A strong reviewer
model re-runs acceptance checks and reviews each phase before the next
starts.
**Why.** Cheap models follow precise specs well but invent badly. Putting
the decisions in the spec and verifying by commands keeps quality high and
token cost low.
**Reopen if** an executor needs more than two review rounds per phase.

## D-17 · Accounts = unique name + generated key; no email or password — 2026-10-28 (planned, P11)

**Decision.** A player chooses a unique `PlayerName` and receives an
`AccountKey` (`Name-XXXX-XXXX-XXXX-XXXX`, 80 random bits). The key is the
only credential; the server stores a SHA-256 of the secret part. No
email, password, recovery, or personal data. Supersedes D-09 for players
who create an account; guests stay as in D-09.
**Why.** Ranking needs stable identities. A generated high-entropy key
needs no password rules, hashing cost, email service, or PII handling,
and works on any device by copy/paste. The cost is "lost key = lost
account", stated clearly at creation.
**Reopen if** lost-key requests become common, or cosmetics/purchases make
account recovery a requirement.

## D-18 · Elo rating with placement K; FIFO pairing that ignores rating — 2026-10-28 (planned, P12)

**Decision.** Ranked uses Elo (start 1000, floor 100, K 40 for the first
20 games then 24, ±1 minimum on decisive results; GAME-DESIGN §5).
Pairing takes the two longest-waiting players of a queue regardless of
rating.
**Why.** Elo is fair (the gain depends on both ratings), tiny, and easy
to explain; few players at launch make rating windows pointless — they
would only lengthen waits. Glicko-2 was considered: better uncertainty
handling, but more state and harder to explain for no gain at this scale.
**Reopen when** more than ~200 ranked players are active daily or rating
gaps in pairings become a measured complaint (then: rating windows that
widen with wait time, and maybe Glicko-2).

## D-19 · One SQLite writer task for accounts, ratings, and records — 2026-10-28 (planned, P11)

**Decision.** All SQL runs on one dedicated thread that owns the
connection; async code talks to it by channel; schema migrations are
numbered and run at startup. Rating updates are single transactions.
**Why.** Player data now matters; one writer removes lock contention and
"database is locked" errors while keeping SQLite's zero-ops simplicity.
**Reopen** with ARCHITECTURE §8's PostgreSQL triggers (two or more servers).

## D-20 · HTML overlay for text entry — 2026-10-28 (planned, P11)

**Decision.** Name and key entry use one small HTML `<input>` overlay
shown by the client; everything else stays `bevy_ui`.
**Why.** D-12's reopen condition is met: names and pasted keys need the
phone keyboard, paste, and accessibility that a canvas UI cannot give
cheaply.

## D-21 · Ranking pages: static HTML/CSS/vanilla JS + JSON API — 2026-10-28 (planned, P13)

**Decision.** `/ranking/` pages are plain files built into the client
bundle; the game server serves read-only JSON from an in-memory
leaderboard snapshot.
**Why.** Loads in milliseconds on any phone without the WASM, can be
shared and indexed, costs no framework, and the snapshot makes reads
almost free for the server.
**Reopen if** the pages grow into an app (accounts management, social).

## D-22 · Medieval art direction on an extended code-built pipeline — 2026-10-28 (planned, P15)

**Decision.** The visual suite keeps D-07's code-built approach but
extends it: rig v2 (18 joints with knees and feet, spring-driven
secondary joints, prop sockets), eased clips with events, per-Archetype
clip sets, and the medieval style guide in `docs/ART-DIRECTION.md`.
**Why.** It already ships, stays light on WebGL2 phones (shared merged
meshes, one material), is reproducible by agents, and avoids the legacy
skinned pipeline's 8 % pass rate (ADR-007). Knees, feet, springs, and
better curves remove most of the "toy" stiffness.
**Fallback.** If the owner rejects the P15 style-lock Archer twice, spike
one Archer as a Blender-scripted (bpy, in-repo, reproducible) skinned
glTF with Bevy animation, measure WASM size and phone FPS, and let the
owner choose.

## D-23 · Stronghold styles are free cosmetics — 2026-10-28 (planned, P18)

**Decision.** Three styles, all free, identical in gameplay, stored on
the account (guests: on the device), sent to both players by the server.
**Why.** Personal expression without touching fairness (BR-U05, BR-C04).

## D-24 · Quality tiers remove cost, never information — 2026-10-28 (planned, P17/P30)

**Decision.** Low/Medium/High (+ Auto) tiers change shadows, ambient
animation, render scale, and effect counts only; team rings, HP bars,
statuses, and splash rings are always shown.
**Why.** "Fluid for every player" must not become "unfair on cheap phones".

## D-25 · Arcanas are a gated gameplay extra after P31 — 2026-10-28 (Proposed, P32–P38)

**Decision.** Finish all existing phases before beginning this optional
extra. Start with three Archetype-based trees, six points, six two-rank
options, additive HP/damage/healing and no supremes. GAME-DESIGN §9 owns
the proposed rules and values. Audit combat first, implement an offline
experiment, validate paired-seed matrices and adversarial presets, then
integrate authoritative selections, UI and durable account presets. Human,
performance, security and real-server checks gate public activation in P38.

**Why.** Five deep trees for eight units would duplicate Evolution/Synergy,
make a one-unit Support specialization Shop-dependent, and expose root,
cadence and Stronghold time-up risks. The small candidate has 141 complete
presets, so exhaustive preset screening is practical without an optimizer
or new dependencies. Flat mutually exclusive talents remain the fallback
if percentage choices fail to produce meaningful strategy.

**Boundary.** All power is free immediately, guests included. Preserve
Energy, Shop probabilities, SlotCount, existing identity/Elo/FIFO and the
three-crate architecture. Do not deploy the extra or rewrite historical
results while executing P32–P37. Writing specs is not balance approval.

**Reopen when** P32 reproduces a rule conflict, P34 finds a compulsory or
dominated choice, or human playtests find the system unclear or unfun.
Prefer revising/removing the candidate to adding trees, haste, range,
Stronghold bonuses or a generic modifier framework.
