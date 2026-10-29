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

## D-26 · 100 MB asset ceiling, memory stays lean — 2026-10-28 (Proposed, owner request; P16-A)

**Decision.** The total compressed download (WASM + every asset, lazy ones
included) may grow to **100 MB** so the art can be richer. First load stays
small: WASM ≤ 4 MB brotli (ceiling 6 MB) and ≤ 1 MB of other first-load
assets, so time-to-menu does not move; everything else loads lazily after
the menu is interactive. Triangle budgets get headroom: ≤ 2 500 per
one-slot unit form, ≤ 5 000 per two-slot form, ≤ 60 000 for the arena.
Browser memory stays ≤ 300 MB and shared merged meshes with one material
stay the rule.
**Why.** The owner wants a more beautiful, professional look and accepts a
bigger game, but not a slower start or a heavier runtime.
**Reopen when** P30 measures phone FPS below budget with the new triangle
counts, or real players report slow loads.

## D-27 · Desktop lobby is an HTML/CSS layer over the canvas — 2026-10-28 (Proposed, P16-A)

**Decision.** On landscape desktop (≥ 1280×720) the lobby is plain HTML and
CSS over the live Bevy arena, extending D-20's overlay bridge: no
framework, no CDN, self-hosted OFL fonts; buttons call existing
`UiAction`s. Smaller or portrait screens keep the Bevy menu until P20.
**Why.** The owner's reference needs crisp typography, layered panels,
blur, gradients, and illustrations that HTML/CSS gives for free and Bevy UI
would need much custom code for.
**Reopen when** the overlay causes input or focus bugs on the canvas, or
P20 decides one lobby for every ratio.

## D-28 · Draw calls are a diagnostic, not a visual ceiling — 2026-10-28 (owner-approved)

**Decision.** Remove the fixed maximum draw-call count from the architecture,
art direction, and visual phase acceptance checks. More draw calls are allowed
when they improve the game's appearance. Keep recording counts so regressions
can be investigated; a count alone must not reject a visual improvement.
**Why.** The owner explicitly prioritizes a more beautiful game over an arbitrary
render-submission ceiling during P16-A implementation.
**Still applies.** Readability, measured FPS targets, triangle and download
budgets remain in force. Performance is judged by the game running on the
actual target devices. Do not remove visible detail or shadows solely to meet
a draw-call number.

## D-29 · Desktop battle HUD uses the existing HTML bridge — 2026-10-28 (owner request)

**Decision.** After P16-A was finalized, the owner supplied a desktop battle
reference and requested its Stronghold HP bars, Energy panel, and Shop cards.
The desktop battle HUD extends D-27's HTML/CSS presentation layer. It reads
`LatestView` and `MatchInfo`, shows numeric current/max Stronghold HP in the
colored bars, omits the Energy lightning icon, and uses screenshots of the
actual base unit models in the Shop cards. Smaller viewports and native builds
retain the Bevy HUD.
**Authority.** Purchase, inspect, and surrender use the existing UiAction
queue. The server's seat, HP, Energy, SlotCount, phase, and offers remain the
source of truth. The reference does not authorize copying its sample numbers
into the simulation or changing the two-step surrender contract.
**Art.** A gallery portrait tool captures each base form against a dark studio
background. Portraits are cached, local WebP UI assets; the 3D scene remains
vertex-colored. The supplied reference stays in ignored `.local/references/`.

## D-30 · Triangles are measured, not capped — 2026-10-28 (owner decision)

**Decision.** Remove the fixed triangle maximums for unit forms, Strongholds,
and the arena (supersedes the triangle part of D-26). Every visual phase
still logs and reports triangle counts per form and per arena layer.
**Why.** The owner wants the most beautiful, professional game the hardware
allows; a fixed number rejected visible quality that the frame rate could
afford.
**Still applies.** 60 FPS desktop and ≥ 30 FPS on a mid-range phone with 30
units, browser memory ≤ 300 MB, the download budgets of D-26, shared merged
meshes with one material. Detail is cut only when a measured FPS or memory
target fails, starting with what players notice least.

## D-31 · Visual track narrowed to characters and performance — 2026-10-28 (owner decision)

**Decision.** Archive P17 (arena for every ratio), P18 (Stronghold styles),
P19 (UI kit and cards), P20 (lobby restyle), and P21 (effects kit). The
track after P16-A is P16 (class animation sets), P22–P29 (each character and
its Evolutions), P30 (client fluidity and performance), P30-B (server
capacity on minimal hardware), and P31 (consistency, playtest, release).
Character phases build their effects and sounds on the existing `vfx` and
`audio` modules; P21's pooling rule (no per-frame allocation, pooled
effects) moves into P30. Every Evolution must read clearly at match
distance (ART-DIRECTION §5, Proposed).
**Why.** The desktop lobby and arena (P16-A) already cover what the owner
needed from P17–P20; the remaining value is in competitive performance on
minimal hardware and in each character's quality and Evolution readability.
**Reopen when** the owner asks for phone-specific lobby/arena work,
Stronghold styles, or a shared effects kit.

## D-32 · Open army, five offers, Nester and Rampart — 2026-10-28 (owner decision, planned P16-B)

**Decision.** Remove the army cap entirely: `SLOT_CAP` and `SlotCount` leave
the game, and no other number replaces them; `Energy` alone gates purchases.
The Shop shows five offers. Two units join the roster: Nester (Plants
Support summoner, owner name *Ninhador*) and Rampart (Machines Tank with
Guard, owner name *Baluarte*). Summon limits belong to the Hatch ability,
not to the army. There is no per-type limit on Ramparts.
**Why.** The owner wants strategic freedom, large battles, and varied
compositions; the eight-slot ceiling decided armies more than choices did.
**Guard rails.** Unit values are **Proposed** and are re-checked with a
before/after harness run and adversarial spam policies; performance is
measured from 10 to 100 units per side; a server-protection entity limit is
allowed only with measured evidence and is documented apart from game rules.
**Reopen when** the harness or playtests show a dominant spam strategy that
cost/stat tuning cannot fix, or large armies miss the frame-rate or server
targets after optimization.

## D-33 · Balance lab: headless matches at full speed, ruleset as data — 2026-10-29 (owner request, planned P16-C)

**Decision.** Balance is measured by an offline lab (`crates/tbg-balance`)
that plays millions of complete matches with the same `step()` the server
runs, back to back on every core instead of every 50 ms. The tick length is
never scaled. Every tunable number becomes a `Ruleset` value; the online
game only ever uses the canonical ruleset built from `content.rs`. The lab
measures many opponent policies, Energy-equal unit duels, paired seeds with
bootstrap intervals, and a Nash meta-game, then searches for small,
confirmed number changes.
**Why.** One heuristic bot and a few thousand matches (P7, P16-B) measure
the bot more than the game; the open army (D-32) made balance harder and
the heavy-spam finding needs evidence, not guesses.
**Guard rails.** Equality with 1× is proven by trace goldens and a real
server match replayed headless; lab contract tests are generic over the
roster and append-only; confirmation seeds are never used for tuning;
values stay **Proposed** until the owner's playtest; no mechanic change
without its own owner decision.
**Reopen when** lab predictions and real match records disagree
persistently, or the lab's cost to maintain exceeds its value.

## D-40 · Arcanas live in every mode before P31 — 2026-10-29 (owner decision)

**Decision.** The owner asked for Arcanas to be fully integrated and live
on the test server now, in Practice, Casual, Ranked, and private rooms,
ahead of the P31 gate that P32–P38 assumed. The GAME-DESIGN §9 values
(six nodes, six points, rank ≤ 2, default `[2,0,2,0,2,0]`) apply as
**Proposed**. Built on P16-C: per-rank values are `Ruleset` data and lab
knobs. Protocol v8 adds `SetArcanas`, `ArcanasSelected/Rejected`, and both
presets in `MatchStart`; the server validates while idle and freezes the
preset at pairing; accounts keep one preset (schema v4); match and ranked
rows record both applied presets. Summons get no Arcana bonus (as with
Synergy, BR-U08). The editor is the Codex desktop editor (D-40 replaces its
"preview" status); phones use the saved or default preset until the P36
phone editor exists.
**Why.** The owner wants players to choose Arcanas before matches now.
**Guard rails.** Server authority (clients send ranks, never bonuses);
no preset may score outside 45–55 % against the default in the lab (it
does not today, `docs/media/arcanas/`); values stay Proposed; the
RulesetVersion field of P35 is replaced by the protocol version for now.
**Reopen when** the full P34 matrix, human playtests, or ranked data show
a dominant preset or a compulsory node.

## D-39 · Quick Challenge by link and rematch reuse the lobby and match task — 2026-10-29 (Proposed, P39)

**Decision.** Add a direct Quick Challenge flow beside the existing private
room. The server issues a 128-bit single-use `InviteToken` (32 hex chars in
`?challenge=`), independent of the session token; it expires after 10
minutes and pairs the inviting session with exactly one different session.
Challenge matches are `MatchMode::Challenge`: the same simulation, pairing,
reconnect, and result path as Casual, but no queue and no rating change.
After a challenge match, a 120-second in-memory `RematchTicket` lets the two
sessions request a new match; it starts only when both request and are
connected, with a fresh seed, and never enters a public queue. Private rooms keep their 5-character
code, `?room=CODE`, TTL, and pairing path. Invitations and rematch tickets
are lobby memory only, cleaned by the existing 10-second sweep; no
simulation starts before two sessions are paired.

**Why.** It reuses every existing session, pairing, match-task, reconnect,
and result mechanism; the only new state is two small invitation records in
the lobby. A 5-character room code (~25 bits) is fine for a manually shared
private room but not for a public one-click invite; 128 bits removes
guessing as a concern while the per-IP invite limiter and the WebSocket
token bucket bound automated attempts. A separate quick-challenge queue or
server would duplicate matchmaking and combat for no gain.

**Consequences.** `PROTOCOL_VERSION` 9 appends three `ClientMsg` and five
`ServerMsg` variants after the v8 Arcanas variants, preserving their indices,
and a `MatchMode::Challenge` value; clients older than
v9 are refused. Challenge and rematch state is lost on server restart, like
guest sessions. A player who closes the tab on the result screen keeps the
rematch ticket for its TTL so a reload can recover it; an explicit
Menu/Cancel declines and notifies the other player. Spectators, chat,
accounts, and persistent invitations remain out of scope.

**Reopen when** playtests show challenge links being shared beyond two
players (then a room with spectators, not a bigger invite), or rematch
abandonment makes the 120-second ticket wasteful.
