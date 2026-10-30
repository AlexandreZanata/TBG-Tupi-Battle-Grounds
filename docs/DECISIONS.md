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
**Amended by D-37 (Proposed, 2026-10-29):** styles stay cosmetic; the
mechanical `StrongholdKind` is a separate concept with its own rules.

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

## D-34 · Strategic expansion P16-F: mechanics enter as disabled ruleset data — 2026-10-29 (Proposed, owner request)

**Decision.** Plan a strategic expansion as phases P16-F1–F8 after P16-D
(balance; spec not written yet). P16-E is not used by this plan. Order:
economy and Siege (F1) → Shop Reserve (F2) → Stronghold kinds and
Fortifications offline (F3) and online (F4) → Augments core (F5), full
catalog (F6), online (F7) → competitive validation and release (F8).
Every new mechanic is a `Ruleset` field whose canonical value disables
it, so the online game does not change until the owner switches it on in
the phase that integrates it, after the lab gate (GAME-DESIGN §10.9).
Effects are a closed enum read at fixed hooks of `step()`; no effect
engine, event bus, trait hierarchy, or new dependency. Arcanas (P32–P38)
are not replaced: their additive formula is the same as BR-X01, with the
Arcana term empty until P33.
**Why.** P16-C showed banking (`saver`, 93 %), heavy spam (61.8 %) and
weak Evolutions dominate today; the expansion multiplies the ways to spend
Energy, so each mechanic must be measured against those findings before
players see it. Ruleset-gated mechanics reuse the lab, keep goldens
meaningful, and let a rejected mechanic be removed without a revert.
**Owner decisions pending** (asked in the phase that needs them):
O-1 scope, and whether F5–F8 interleave with the visual track (P16,
P22–P31) or run first; O-2 English and owner names of the kinds; O-3 the
default kind; O-4 Siege/Collapse (D-36); O-5 uncapped Energy with
Interest (D-35); O-6 identical Augment options for both players; O-7
three rounds at 0:30/2:00/3:30; O-8 the revised Augment catalog (D-38);
O-9 Fortification values; O-10 public equipped Augments; O-11 each canon
switch (F1, F2, F4, F7); O-12 Banner aura and the Ballista variant.
**Reopen when** P16-D changes the economy or the victory rule in a way
that invalidates F1, or a gate shows a mechanic cannot be balanced by
numbers (then remove it before adding another).

## D-35 · Uncapped Energy with low-balance Interest and bounded generation — 2026-10-29 (Proposed, P16-F1)

> **Revised by D-41 (owner revision, same day).** The rules below are now
> active as written, but they were switched on without the `banker-N`
> gate this entry required; that evidence is still missing.

**Decision.** Remove the 10-Energy cap (BR-E04). Pay Interest every 10 s
on the lowest balance held since the previous payment: 0.25 per full 5,
at most 1.25 (BR-E05–E07). Kill Bounties are capped per window and by a
1-Energy ceiling (BR-E08). No Augment, kind or Fortification touches base
regen, Interest or Shop probabilities (BR-E09).
**Why.** A cap punishes saving for a planned spike; Interest rewards it.
Reading the lowest balance, not the balance at the payment tick, makes
Interest pay for Energy actually held and resolves "a purchase on the
payment tick" without a special case. Max Interest is 22 % of base income,
paid only while 25 Energy (about seven units) sits off the field.
**Risk accepted only with evidence.** P16-C measured banking to the cap
(`saver`) at 93 % against the pool: deploying a stored army instantly at
home beats units that arrive one by one. Removing the cap and paying
Interest strengthen exactly that. F1 may not switch canon unless P16-D has
fixed or explained the `saver` result and the `banker-N` rows pass
(GAME-DESIGN §10.9). Alternatives considered: Interest read at the payment
tick (gameable by spending right after it); a soft cap with decay (an
artificial limit by another name); a growing Interest without maximum
(exponential, rejected).
**Reopen when** a `banker-N` policy exceeds its gate after tuning band
size and value, or playtests find passive play dominant.

## D-36 · Siege and Collapse replace the time-up decision — 2026-10-29 (Proposed, P16-F1)

> **Superseded by D-41 (owner revision, same day).** No time-up decision
> remains, but neither do `Siege`, `Collapse` nor the `ServerLimit` draw:
> matches are open-ended. Kept for history.

**Decision.** No result is decided by comparing HP at a time (BR-M07). At
4:00 of running time the phase becomes `Siege` (today's overtime slot):
regen doubles and both Strongholds lose a growing percentage of their own
max HP per second (`Collapse`, BR-M08–M10). A server-protection limit at
10:00 ends a live match as a draw (`ServerLimit`, BR-M11); contracts prove
it unreachable. `Phase::Overtime` keeps its wire index renamed `Siege`;
`EndReason::TimeUp` stays for old records and is never produced again.
**Why.** Without a clock decision, uncapped Energy, Interest, Redoubt,
Walls, healing, lifesteal and Stone Mend can make two defensive players
trade forever, and an endless match costs server time and players'
patience. Options compared: damage-taken multipliers over time and
bonuses against Strongholds (no guarantee if nothing reaches the
Stronghold); absolute HP decay (favors high-HP kinds); weakening
Stronghold fire (no guarantee); an absolute time limit with an HP
comparison (what the owner wants removed). Percentage Collapse is the
only option that guarantees an end, treats every kind and Fortification
alike, and still leaves 1–2 minutes of play in which the side behind can
come back.
**Consequences.** `headless`/`replay` stop bounds, the trace goldens, the
`TimeUp` gate row, the client clock and result text, and the server's
shutdown `DRAIN_LIMIT` (300 s, shorter than the new worst case) change in
F1.
**Reopen when** Collapse decides more than 15 % of matches, or players
report the late game as a countdown rather than a fight.

## D-37 · Mechanical Stronghold kinds and a shared Fortification tree — 2026-10-29 (Proposed, P16-F3/F4)

**Decision.** Four free `StrongholdKind`s (Treasury, Redoubt, Ballista,
Banner), defined as basis points of the canonical Stronghold, chosen
while idle and frozen at queue or room entry (BR-K01–K08). One shared
`Fortification` tree, Walls or Arms, 6 then 10 Energy, whose tier 2 adds a
capability (slow or splash) instead of a second invisible +5 % (BR-F01–F05).
`StrongholdStyle` (D-23, archived P18) stays cosmetic and separate.
**Why.** Kinds give a pre-match identity that the Shop cannot take away;
percentages survive P16-D tuning. Raw tower power (HP × damage, canon =
1.00) is 0.86 Treasury, 0.98 Redoubt, 1.09 Ballista, 0.88 Banner: the two
utility kinds pay for their passives, which the lab must confirm. Tower
HP costs 3–4× more Energy per HP than a Tank and tower damage far more
than an Archer, so +10 % tiers would rarely be bought; +15 % and a tier-2
capability make the investment a real alternative. Kind-exclusive tiers
are deferred: four kinds × two branches already give eight end states to
balance and draw.
**Reopen when** a kind leaves 45–55 % against the field after tuning, or
players cannot tell the kinds apart at a glance.

## D-38 · Augments: three mirrored rounds and a closed effect list — 2026-10-29 (Proposed, P16-F5–F7)

> **Superseded by D-41 (owner revision, same day)** for rounds, options,
> limits and timing: a round every 60 s of combat, forever, four options,
> unlimited copies, and a battle pause per round. The revised 40-effect
> catalog and the closed effect list stand. Kept for history.

**Decision.** Three `AugmentRound`s at 0:30, 2:00 and 3:30; both players
see the same three options; one purchase or Forgo (+2) per round; at most
three Augments, public, permanent; no sell, swap, reroll or fourth round
(BR-AG01–AG11). The owner's 40-Augment list is revised to 40 distinct
effects (GAME-DESIGN §10.8; mapping in P16-F6): duplicates of the same
effect at a higher rarity are merged, invisible values (+4 % tower damage,
+6 % Control damage on 20–30 base damage) are raised or replaced, and new
counters (Grievous Wounds vs healing, Giant Slayer vs Tanks, Executioner,
Second Wind vs burst) fill the gaps. Fourteen ship first (F5); the rest
only after F5 passes its gate.
**Why.** Identical options remove seat and luck asymmetry from the most
random system in the game; the remaining difference is how well each
player adapts their army, which is skill. Three rounds fit the 2.5–4
minute target and Siege at 4:00, so the three-Augment limit needs no
replacement UI. Offers every 90 s would give most matches one round
(1:30); 0:30 lets every player use the system, at a cost (3–6 Energy) the
start economy affords within the 30-s window.
**Reopen when** a single Augment's value exceeds its gate after tuning, the
Forgo rate exceeds 50 % in a round (options too weak) or falls below 5 %
(Forgo pointless), or playtests find the window too busy on phones.

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
`ServerMsg` variants after the v8 Arcana messages and a
`MatchMode::Challenge` value; clients older than v9 are refused. Challenge and rematch state is lost on server restart, like
guest sessions. A player who closes the tab on the result screen keeps the
rematch ticket for its TTL so a reload can recover it; an explicit
Menu/Cancel declines and notifies the other player. Spectators, chat,
accounts, and persistent invitations remain out of scope.

**Reopen when** playtests show challenge links being shared beyond two
players (then a room with spectators, not a bigger invite), or rematch
abandonment makes the 120-second ticket wasteful.

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

## D-41 · Owner revision: open-ended matches, compounding Interest, infinite Augment rounds that pause the battle — 2026-10-29 (owner decision)

**Decision.** The owner switched the P16-F economy and Augments on now,
in every mode, ahead of the lab gates of D-34:
- Matches have no clock and no time-based result (BR-M07). `Phase` is
  `Countdown`/`Running`/`Ended`; `EndReason` is `StrongholdDestroyed`/
  `Forfeit`. Siege, Collapse and `ServerLimit` are dropped, not deferred.
- `Energy` is uncapped (`Ruleset.energy_cap = None`) and Interest compounds
  on the low-water mark every 10 s of combat (BR-E04–E08, D-35 numbers).
- An Augment round opens every 60 s of combat, forever, with four mirrored
  options from a reshuffled deck of all 40; one purchase or Forgo per
  round; copies unlimited and stacking (BR-AG01–AG11).
- Every round **pauses the battle for both players** (BR-AG12–AG14). The
  combat tick freezes; a separate selection clock (`MatchState::clock`,
  one per server step) closes the 20-s window (first built as 60 s; the owner shortened it the same day). The round resolves as soon
  as both confirmed, or at the deadline. Picks are paid on confirmation
  but stay hidden until both outcomes apply on the same step.
**How.** No new task, timer or sync system: the existing match task keeps
calling `step()` at 20 Hz; a paused `step()` only advances the selection
clock and checks the close condition, so the server, the practice match,
`headless` and `replay` all pause identically. Views go out on the
selection clock (10 Hz) so countdowns and reconnects work while the
combat tick is frozen. Replay ops are keyed by the selection clock, so a
choice made late in a pause replays at the same step. `ServerMsg::View`
boxes the now larger `MatchView`. The practice bot forgoes every round so
a human never waits a full minute on it.
**Why.** The owner wants a long-form economic game where saving and
adapting matter and a pause makes each Augment choice a deliberate,
simultaneous decision instead of a distraction mid-fight; hiding picks
removes any advantage from confirming second, and the separate clock means
a frozen battle can never freeze the selection.
**Accepted risks.** Balance is not measured: P16-C's `saver` finding
(banking dominates) is likely stronger with Interest; unlimited copies
grow HP and damage linearly; open-ended matches have no server-side time
bound (a match ends only by Stronghold, forfeit or reconnect expiry). The
match ruleset fingerprint (`ruleset_fp`) is not recorded yet.
**Open owner questions.** Forgo value (canon 0, lab +2); whether to keep
the 6-hour offline safety bound; Shop Reserve, Stronghold kinds and
Fortifications (F2–F4) remain unscheduled.
**Reopen when** the lab or playtests show a dominant banking or
single-Augment strategy, pauses feel too frequent, or match lengths
threaten server capacity.

## D-42 · Owner revision: fifteen Arcanas, twelve points and three supremes — 2026-10-29 (owner decision)

**Decision.** The owner expanded the live Arcana system (D-40) from six
nodes and six points to three five-node families and twelve points.
Every family keeps its two original nodes unchanged (indices 0–5), adds
two one-point children and one two-point supreme, for 15 nodes and 8
points to complete a family. A player may finish one family plus 4
points elsewhere, or build 4/4/4 without a supreme. All options remain
free and available to every account and guest; no progression, account
gate or payment is introduced.
- **Effects.** War adds Shieldwall, Battle Rush and Combined Arms;
  Precision adds Hunter's Focus, Quickdraw and Deadeye; Vitality adds
  Unshaken, Emergency Care and Purification. The conditional values are
  Proposed and tunable through `Ruleset.arcana`; the original per-rank
  values stay in `Ruleset.arcana_bp`.
- **Prerequisites.** A child needs its parent at rank 1; a supreme needs
  both originals at rank 2 and both children. Removing a rank that would
  invalidate a held dependent is refused and names the dependent; the
  editor and the server use the same table.
- **Schema and wire.** `ArcanaPreset` is 15 rank bytes and validates the
  real point cost (supremes cost 2). Protocol v11 changes `SetArcanas`
  and `ArcanasSelected`; older peers are rejected at the handshake.
  Both frozen presets still travel in `MatchStart`, match records and
  ranked rows.
- **Migration.** Account rows and browser storage accept the old
  six-byte/rank document. The six original ranks are preserved; the six
  new points raise the original nodes to rank 2 in index order. No child
  or supreme is selected automatically, so migration cannot grant a
  behavior the player did not choose. Match and ranked history rows are
  not rewritten. The complete default is
  `[2,2,2,2,2,2,0,0,0,0,0,0,0,0,0]`.
- **Simulation.** Conditional army checks use per-tick living role
  counts, decremented on death, instead of scanning the army per hit.
  Deadeye changes only conventional Ranged range; Quickdraw only the
  post-hit cooldown; Unshaken only incoming duration; Purification only
  after a real heal. No new event bus, trait framework or per-tick
  allocation is added.

**Why.** The six small percentage nodes produced shallow choices. The
owner wants War, Precision and Vitality to change how an army is built
and fights, while keeping the existing art, editor, authority, freeze
and persistence infrastructure.

**Guard rails.** Values stay Proposed. Summons remain excluded. The
twelve-point build must be validated by the P16-C/P16-D lab (named builds
and the complete-preset screen) before release claims; a supreme may not
become universally mandatory and a hybrid build must remain viable.
Historical presets, results and Rating are never reinterpreted.

**Reopen when** the lab shows a dominant or never-used node, a supreme
has no viable counter, or the editor/database schema needs another
version.

**Audit addendum (2026-10-29, `phase/p40-review`).** A review of the
executor's work re-measured the lab on 3 072 paired seeds with a null
control and held-out confirmation: the first numbers (512 pairs, ±4 pp,
duplicated builds) were noise. Deadeye moves from +0.1 to **+0.25 tile**
(null 49.4 %, +0.1 51.6 %, +0.25 53.7 %, +0.5 57.0 %); Quickdraw no longer
exceeds the +100 % attack-speed cap (BR-X02). Not fixed, by evidence: the
default 4/4/4 beats every build that swaps flat originals for War or
Vitality conditions (war-full 47.7 %, vitality 47.2 %, war-vitality
44.2 %), and Shieldwall and Combined Arms have a DPS ally in only 7.5 % and
5.7 % of bot seat-ticks. Those values need policies that play each family
before they are retuned. Evidence: `docs/media/p40/`.

## D-43 · Interest becomes a regen boost on the Energy held — 2026-10-29 (owner decision)

**Decision.** The owner replaced the 10-s Interest payment (D-35, D-41)
with a continuous boost: every full 5 `Energy` held adds 25 % of the base
regeneration, counting at most 25 `Energy` (×1.25 at 5, ×1.50 at 10, ×1.75
at 15, ×2.00 at 20, ×2.25 at 25 or more). `Energy` itself stays uncapped.
`MatchView::regen_bp` replaces the payment fields, `Event::InterestPaid`,
`PlayerState::low` and `interest_paid` are gone, protocol v12, and the
Bounty window keeps its own 10-s reset (`BOUNTY_WINDOW_TICKS`).
**Why.** One rule that is visible every second (the HUD shows ×1.50 and
how many of the 5 steps are filled) instead of a lump every ten seconds,
and no low-water mark to explain. Reading the balance at the start of each
tick lets a spend lower the rate at once, so nothing needs a special case.
**Cost accepted.** The base regen of 17 sub-energy per tick (about 0.57
`Energy` per second) is unchanged; the owner described the base as "1 per
second", which is read as the unboosted rate, not as a new base. At 25 held
the player regenerates about 1.28 per second, 125 % more than idle, and the
lab has not measured the effect (P16-C's `saver` finding is likely
stronger).
**Reopen when** the lab or playtests show banking dominates, or the owner
wants an absolute base of 1 `Energy` per second.

## D-44 · The lab's planted-imbalance contract (C7) under a purchase-only Shop — 2026-10-29

**Decision.** BR-S04 (offers change only when bought) made a unit appear
rarely, so the shipped-bot match row of contract C7 stopped detecting a
planted ×2 HP and damage Shieldbearer (+0.009; it passed before the Shop
change, on `4f332ed`). `favor-<unit>` now buys the cheapest card to bring
new ones when its unit is not offered, as a player must under BR-S04, and
C7 adds one more disjunct: the same planted imbalance measured against the
cheap-spam policy. No threshold moved (±0.10 on 256 pairs, 20 points on a
duel row).
**Why.** A contract that goes red because the game changed, not because a
unit is unbalanced, would hide real regressions; the thresholds are a
product decision and stay. The duel rows alone cannot carry C7 (nine
opponents make one duel 0.111, under the 0.20 step).
**Reopen when** the Shop gains another way to refresh offers (Reserve,
F2), or a planted imbalance is missed again.

## D-45 · A purchase redraws all five Shop offers — 2026-10-29 (owner decision)

**Decision.** Every purchase now redraws all five offers at random from
the whole roster, as five distinct units (BR-S01 holds): the unit just
bought can come back, about half the time. It replaces the earlier
same-day one-slot refill (BR-S04). There is still no timed rotation.
`shop::draw_offers` serves the first offers and every redraw; `refill` is
gone. The player's own RNG stream drives it (BR-S05), so replays and
hashes stay deterministic; the contract goldens are refrozen.
**Why.** Choosing a card now reshuffles what the player can reach next,
so a plan can no longer be broken by one unlucky slot, and a wanted unit
is available about half the time instead of one time in five.
**Read as.** "Five random cards, the bought one allowed" is read as five
*distinct* units, keeping BR-S01; independent draws that may repeat a
unit inside one set are not implemented. Say if duplicates inside a set
are wanted.
**D-44 still holds.** With the full redraw the original C7 still missed a
planted ×3-cost Siege Walker, so the cheap-spam disjunct and the
refresh-by-buying `favor` policy stay (both are lab-only).
**Reopen when** the lab shows the Shop makes banking or one unit
dominant, or the owner wants duplicates inside an offer set.

## D-46 · Four Stronghold kinds with kind-exclusive Fortification — 2026-10-29 (owner decision)

**Decision.** The owner's product spec of 2026-10-29 makes Stronghold
kinds and Fortification active (GAME-DESIGN §10.4–10.5, BR-SK01–SK09):
Treasury, Bastion, Artillery and Command, chosen free before the match,
each with its own base, passive and two exclusive branches (tier 1
3 Energy, tier 2 4 Energy). It supersedes the earlier proposal
(Treasury, Redoubt, Ballista, Banner on a shared Walls/Arms tree,
BR-K01–K08 and BR-F01–F05, D-37's shared tree) and the unmerged
`phase/p17a-strategic-strongholds` plan, whose code it reuses only as a
reference. Default kind: Bastion.
**How.** `tbg-core::stronghold` derives the tower's stats from
`STRONGHOLD_KINDS` and the owned tiers; the real shot, the Treasury Spoils
(paid from the authoritative death, per Bounty window) and the War Banner
(read at hit time, one more term in BR-X01's bp sum) use it. The kind is
chosen like the Arcanas: `SetStronghold` while idle, frozen at pairing,
saved per account (schema v5 `accounts.stronghold`), recorded in
replays; `Fortify{branch}` is a `PlayerIntention`; `MatchView.strongholds`
makes both towers public. Protocol v13.
**Classic stays** as the reference tower: every plain `MatchState`, the
tests and the knob search keep it, so the lab's baselines and the
contract matches are unchanged (same winners and end ticks; only the
state hash moved with the new fields). The bot fortifies only a player
kind.
**Values are Proposed.** Treasury and Command keep the product
hypotheses' ratios on today's 4 500 HP / 53 damage tower; Artillery
(−1 500 HP / +1 000 damage bp) and Bastion (−1 000 damage bp) moved after
the first `strongholds` lab matrix showed Artillery at 61–67 % against
every kind for a near-tower army (`docs/media/p17/lab/`).
**Reopen when** the matrix or live play shows a kind above 55 % against
the field, or a branch that is never worth buying.
## D-54 · View wire: derive four fields and split static unit identity (A + C) — 2026-10-29 (Proposed, P30-A; numbered D-41 on its branch, renumbered at consolidation)

**Decision.** P30-A measured the full-snapshot `MatchView` and chose to
(A) stop sending what the client computes exactly (`max_hp`,
`phase_ticks_left`, `next_rotation_in`, `you`) and (C) send unit static
identity (`seat`, `kind`, `tier`, `branch`) only on spawn, evolution, and
full views (reconnect), with a complete dynamic list (`id`, `x`, `hp`,
`status`) every view. Measured payload: −41.6 % natural, −44.7 % at 20
units/side, −42.5 % at 100, with build+encode CPU not higher. Delta views,
compression, 5/8 Hz, and a client rewrite were measured and rejected or
deferred. P30-B applies it as the next `PROTOCOL_VERSION` (v10 unless P17-A
takes it first); that commit changes the append-only protocol contract
tests and must name this decision.
**Why.** Static identity plus `max_hp` is 35 % of every view and is
repeated 10×/s; the split keeps one lost view from leaving a stale
position or HP and needs no keyframe state machine.
**Guard rails.** Server authority is unchanged (the client derives
presentation values, never outcomes); a decode error or an unknown static
id forces the existing reconnect; events stay presentation-only; 10 Hz and
the 20 Hz simulation are unchanged.
**Reopen when** the VPS shows bandwidth, not CPU, is still the first limit
after C (then delta, with a ≥15 pp measured gain over C at 100 units), or
C fails the 100-seed reconstruction or the 50-match reconnect chaos.

## D-48 · Roster expansion: fifteen units, staged by ruleset pool — 2026-10-29 (owner decision)

**Decision.** Fifteen new units enter the game in stages through phases
P42–P53, as planned in `docs/ROSTER-EXPANSION.md` (Rotcap, Farwatch,
Leechroot, Bombard, Old Finn, Drumguard, Longaxe, Oilpress, Antidote
Brewer, Puppetmaster, Smuggler, Shatterback, Blood Furnace, Bloodweaver,
Cargo Crab): both Shatterback (designed here, no draft exists) and Cargo
Crab are in (O-R1); the roster-health gate becomes ≥ 2.5 % purchase share
per unit (O-R2); no new unit reaches the `Shop` before its wave, and the
pool switch is an owner decision in P53.
**Why.** The current game already fails 12 of 39 lab rows before any new
unit exists (median duration, three faction rows, the strongest strategic
style, the Nash mix, duel outliers and weak Evolutions), so new units are
judged relative to a frozen P42 baseline plus absolute rows the baseline
already passes, with the old outliers rebalanced by numbers first; with 25
kinds a 5 % per-unit share is impossible (even share 4.0 %) and offer odds
per unit fall from 50 % to 20 % of sets, so the threshold moves to about
60 % of the even share and the lab watches dilution.
**Reopen when** a wave shows a unit that numbers cannot balance.

## D-49 · P43 shared foundations: ability pairs, staged pool, preview path — 2026-10-29 (executor decision, pending review)

**Decision.** `UnitDef.abilities` is `[[Ability; 2]; 5]` (one pair per
form slot: base, A1, A2, B1, B2), so Rotcap's Poison + Echo fits without a
later reshape; Hatch and Guard migrate to ability dispatch with values
unchanged; `Unit::is_summon()` (unit-level) replaces kind checks where a
unit is at hand; `hatch_cd` becomes `action_cd`; the Shop draws from
`Ruleset.pool` (`CANON.pool` stays ten until the P53 owner switch);
`Ruleset::preview()` names the full staging pool and Practice uses it;
`UnitFx`/`Stack` land zeroed (hashes updated once); eight staged events
and `UnitView.fx` ship under protocol v13 with the client rendering staged
kinds as a gray silhouette. Thresholds for the conditional-damage fields
stay pending for P44.
**Why.** ROSTER-EXPANSION §F delegates the pair shape to P43; pairs are
required by a known P45 unit, not speculation. Everything else is the
cheapest shape that keeps P43 behavior-identical (gate CSV byte-identical
to the P42 baseline on the same seeds).
**Reopen when** a wave needs a third ability on one form, or the pool
switch needs more than a field assignment.

## D-50 · All fifteen new units go online before P50 tuning — 2026-10-30 (owner decision)

**Decision.** The owner moves the P53 pool switch forward: `CANON.pool`
is `FULL_POOL` (25 kinds), so online, Practice and the lab draw the same
`Shop`. Rules from ROSTER-EXPANSION are active; every number stays
**Proposed**. Shop portraits exist only for the ten original kinds, so
the fifteen show an empty card until owner art lands.
**Why.** The owner wants the full roster playable on the test server now
and accepts untuned values; P50 tunes them with `tbg-balance` evidence.
The P42 lab baseline on the ten-kind pool stays the reference for D-48.
**Reopen when** live play or the lab shows a new unit that breaks
matches (e.g. a stall or a share outside the D-48 gate), or the owner
wants the pool back to ten.

## D-51 · Per-unit art kits; distance-driven gait; aiming Strongholds — 2026-10-30 (owner request; approved)

**Decision.** Each character owns a code-built *art kit*
(`crates/tbg-client/src/art/kits/<unit>/`): its five-form figure on rig v2,
a signature pose layer over the shared clip library, its gait, its aim
and its attack style — its own projectile, muzzle effect and impact, or
its own melee impact. Kits are plain data and function pointers in a
static registry (`kits::kit(UnitKind)`), like the Stronghold `PartDef`s;
no trait, no engine layer, no assets. First kits: Siege Walker, Bone
Brute, Rootcaller (P54). Units without a kit keep the shared library and
the generic projectile until their own phase.
**Gait v2 (all units).** Walk cadence is the distance the figure actually
travels divided by its stride, so feet never slide at any speed, slow or
Evolution bonus; planted feet lock in world X and Z; every figure
(native or adapted) walks on two-bone legs; walk/idle has a 0.15 s stop
hysteresis. Animation runs in `PostUpdate` after all movement.
**Aim.** A unit that attacks keeps its target for one interval plus
0.7 s; kits turn the body (an off-center weapon is lined up exactly) and
raise the weapon to the launch tangent of their own shot, which leaves
the real muzzle. Strongholds aim their gun (Artillery carriage and
barrel, Sentinel ballista) the same way and fire their own cannonball or
bolt from it on the render-delayed `AttackStart`; gunless towers loose a
crossbow bolt from their top.
**Unchanged.** All of this is cosmetic: no simulation value, event,
timing or protocol changed; ART-DIRECTION §4's 0.30 / 0.18 / 0.22 s event
times hold (kits only lengthen recoveries). The Siege Walker is drawn
with a shoulder bombard ("bazooka", owner 2026-10-30), not the P24
arbalest; its "Ogre Archer" display name is an open owner decision.
**Reopen when** a kit's pose cost shows up in the 30-unit stress
measurement, or skinned glTF replaces code-built figures (D-22).
**Approved (owner, 2026-10-30).** After reviewing the build at
`localhost` (gallery forms, `&range`, `&evolve`, walking, aiming towers)
the owner approved the three kits, gait v2 and the Artillery's aim and
projectiles as they stand; the Artillery's animation is frozen. The
Siege Walker's display name stays an open owner decision.

## D-52 · Defenders and lobby-faithful dress for the gunless Strongholds — 2026-10-30 (owner request)

**Decision.** Every Stronghold kind now shows who fires its shot, animated
for its role, and is dressed after its lobby portrait
(`assets/stronghold/<kind>.webp`, GPT-generated, D-46) in the same
low-poly vertex-color style as the kits:
- **Classic:** a crossbowman in a team tabard and kettle hat walks round
  the crown to face the target, aims, recoils (`gunners.rs`); steps,
  lintel, door studs, arrow slits and a gold band.
- **Bastion:** a gun port in the wall facing the enemy; its shutter flies
  open on every shot and a swivel falconet runs out through it, fires and
  runs back (Sentinel's ballista keeps firing on Royal Sentinel); an
  arched gate up a flight of steps, four round corner turrets with
  conical roofs and team pennants, a crenellated keep with its flag.
- **Treasury:** a stone dormer on an eight-sided terracotta roof whose
  trapdoors open for a brass mortar that lobs a purse of gold (its own
  `ShotKind::Coins` and gold impact); a gold dome with a lantern spire,
  gold finials and team pennants on the pillars, team banners beside the
  vault, entry steps, gold-framed windows, striped market awnings.
- **Command:** an archer on the observation platform turns, draws and
  looses; cloth panels on the rail, a heraldic banner over an arched
  door, masonry courses and entry steps.
**How.** Defenders are ordinary tower pieces: `Motion::Swivel` (walks
round the tower's axis with the aim), `Motion::Mounted` (a carried
weapon: orbits, aims, recoils), `Motion::Kick` (a fixed gun: aims,
recoils; negative recoil runs it out of a port), `Motion::Hatch` (swings
open on a shot, then shuts). `aim::Turret` gained `orbit` (aim from the
tower axis) and `arc` (a port's yaw limit). A tower whose defender fell
with its damage state still looses a bolt from its top.
**Unchanged.** The Artillery (approved, D-51), all simulation values and
the render-delayed shot timing.
**Reopen when** a defender reads poorly at match distance or the owner
wants a different role animation.

## D-53 · Every champion on its own art kit — 2026-10-30 (owner request)

**Decision.** The P54 kits (D-51) and the Stronghold defenders (D-52) are
the visual standard for every champion. Twenty-three champions that still
used the shared library now have their own kit (figure with five distinct
forms, motion for every clip, own projectile or melee impact): the
fifteen new champions and the Smuggler's Crate, and the originals
Shieldbearer, Hexer, Sawbot, Bloomkeeper, Nester, Beetling and Rampart.
Kits share one library (`art/kits/common/`: anatomy, dress, gear, motion
blocks on the fixed event times). Support actions with an `AttackStart`
are shot kits; the crate toss, the blood transfer stream and broken
plates are cued from their own events (`sync/actions.rs`).
**Unchanged.** The Siege Walker, Bone Brute and Rootcaller kits, the
towers, the Archer's approved v2 figure, all simulation values, event
times and the protocol.
**Reopen when** the owner rejects a kit's look, a kit reads poorly at
match distance, or the view packs the carrier and plate bits (then the
Cargo Crab can seat its passenger and the Shatterback can shed plates).
