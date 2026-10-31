# P63 — Arena victory celebration and individual champion dances

> **Planning only — 2026-10-31.** Requested after P62: after victory,
> show the winner's characters in the arena, facing the player, including
> those who died, performing their individual celebration animations.
> Future champions can each have their own signature dance.
> No game implementation or gameplay validation is included in this branch.
> New behavior and numbers are **Proposed** in GAME-DESIGN §17.

Planning branch: `codex/p63-victory-celebration-plan`, based on the P62
planning commit `0249d01422005139cc0a7eb150243023b02cbdde`.
Suggested execution branch: `phase/p63-victory-celebration`.

## 1. Read first and dependencies

Read [AGENTS.md](../../AGENTS.md), the current owner's minimal-validation
policy and `docs/VALIDATION.md` when present on the integration base, then:

- [GAME-DESIGN §17](../GAME-DESIGN.md#17-arena-victory-celebration-p63-proposed).
- [DECISIONS D-82](../DECISIONS.md#d-82--post-match-cast-and-reusable-celebration-motion--2026-10-31-proposed).
- [ARCHITECTURE §19](../ARCHITECTURE.md#19-victory-celebration-p63-proposed).
- [ART-DIRECTION](../ART-DIRECTION.md), rig/motion/contact and visual review.
- [P62](P62-troops-training-lab.md), preview and shared animation ownership.

Freeze the actual integrated desktop/browser/native base before coding.
The P62 planning branch does not contain its implementation. P63 can reuse
existing Victory clips without waiting for the entire training laboratory;
if P62's Dance interface has landed, share it. Neither phase owns a second
copy of the animation library. Coordinate edits to art/animation modules
sequentially, with one executor per shared file. Do not touch `archive/`.

## 2. Current evidence and the missing behavior

Inspected source includes the main-derived client and the desktop work
front; recheck the corresponding paths on the chosen implementation SHA.

- `crates/tbg-client/src/art/clips.rs` already defines looping `Clip::Victory`.
- `art/kits/common/moves.rs` implements Raise, Both and Dance celebrations.
  Many champion kits specialize `Clip::Victory`; some share the same motion.
  Existing Victory is not evidence that every champion has a unique dance.
- `sync/messages.rs` handles `MatchEnd` by storing the authoritative Outcome,
  playing victory/defeat audio and transitioning directly to `GameState::Result`.
- `sync.rs` clears unit entities on leaving InMatch; dead visuals are also
  removed during the match. They cannot simply be rotated after Result.
- `ui/result.rs` draws the result panel, rating updates, retry/menu and
  challenge rematch controls. `RematchOpen` can request Result again.
- `tbg-core/src/view.rs` exposes currently represented units and their forms,
  not a complete history of every unit ever deployed. A cache of received
  views misses units born/dying between views and units before reconnect.
- `tbg-server/src/match_task.rs` sends MatchEnd; its spectator, reconnect,
  botclient and result tests must be inspected before changing the end wire.

Therefore this is a result presentation feature plus a small reliable
cast-summary contract, not resurrection of simulation entities. The result,
combat, rating and rewards are already decided before the dancing begins.

## 3. Player-facing contract

### 3.1 Who celebrates

The winning player sees the **winning army's full participating cast** in
the match arena, facing the result camera. Include dead and surviving
instances; dead characters appear as intact cosmetic figures, not corpses.
Repeated deployments remain repeated figures, rather than one figure per
kind. Use each instance's last attained Evolution form: death does not
reset it, and an evolved unit is counted once rather than at every tier.

Recommended scope, Proposed: include purchased champions and animated combat
summons, including Beetlings and animated Smuggler cargo. Exclude inert
Crates, Strongholds, scenery and training dummies. Confirm this eligibility
rule at O01; “all characters” must not silently become “only survivors” or
“one of each champion”. A winner with no eligible deployments gets a valid
empty-cast result, without fabricated units.

The scene is shown on the winner's client. Proposed initial behavior for
the losing client and spectators is their ordinary result presentation;
do not make the loser watch a mandatory taunt or imply a spectator owns the
winning army. Draw, canceled/unknown result and disconnected-without-outcome
do not trigger victory. A confirmed win by surrender/timeout may celebrate
the real deployed cast. Local full practice can use its actual Outcome;
P62 controlled drills/preview scenes do not become ranked victory screens.

### 3.2 Arena, formation and frontal view

Keep the match's real arena. Reframe a clear central presentation area,
disable battle-only HUD/effects, and place cosmetic figures in deterministic
staggered rows. Larger silhouettes need larger spacing; weapons, wings,
plants and carried/multi-part rigs must not intersect their neighbors.
Each figure faces the camera's horizontal direction using its kit's actual
forward axis. Test seat A and mirrored seat B: a global fixed yaw is not a
valid solution. Avoid a camera directly above the figures or showing backs.

Frame the whole visible formation using bounds and desktop aspect ratio,
with safe space for result text/actions. Keep the lowest/front row out of
the panel area. The victory title and controls remain readable without
covering faces. A short settle/crossfade is acceptable; no mandatory cinematic.

Army size is not globally capped. Proposed performance strategy: show all
eligible instances in one formation when the cast has at most **64 figures**;
larger casts use deterministic pages of at most 64, clearly labeled
“Showing X–Y of N”, with Previous/Next controls and automatic framing.
Every cast member is reachable; no silent truncation, top-performer-only
selection or loss of duplicates. Resolve O02 before implementation: if the
owner requires every instance simultaneously, replace paging with a measured
LOD/instancing plan and obtain visual acceptance before claiming feasibility.
This presentation limit never limits the real army or changes results.

### 3.3 Individual dances and future extension

Immediately reuse each kit's real Victory motion as its current celebration.
Where a kit currently raises its arms, show that honest current celebration;
record that it still needs its signature Dance. Do not label a repeated
shared animation “a unique dance for every champion”.

Use one shared client resolver for victory and P62's Dance preview:
`celebration_for(kind, form)`, returning the current clip/motion, duration,
loop mode and optional kit override. Prefer the existing kit dispatch and
small data/functions to a new trait framework or asset service. A champion
with no override gets a rig-compatible shared celebration. Add an explicit
Dance identity only when P62 or this phase actually introduces it; preserve
existing `Clip::Victory` callers until migrated intentionally.

This phase must include a coverage inventory for every active kind/form,
a working per-champion override seam, and at least one signature-dance pilot
approved by the owner. Authoring a new unique dance for the entire roster
is a future content pass, not a hidden prerequisite. The pilot's character
and choreography are chosen at O03. Existing distinct kit motions remain.

All figures loop until the player leaves the result. Use stable per-slot
phase offsets to avoid rigid synchronized clones; no unseeded core RNG.
Root motion stays within each formation slot. Reset death, hit-stop, aim,
target, passenger attachment and combat actions for presentation figures.
Carried champions stand independently unless a reviewed kit requires a
visual companion. Cosmetic props cannot spawn real passengers or summons.

Dance animation events must not fire attack sounds, damage, projectiles,
footstep spam or combat effects through the normal event pipeline. One
existing victory sting plays once per completed match. Respect sound and
reduced-motion preferences; reduced motion uses a stable celebration pose
with a static camera, preserving cast access and result controls.

### 3.4 Result interaction and lifecycle

Recommended architecture: remain in `GameState::Result` and attach a
`ResultPresentation` scene/controller. Do not add a blocking intermediate
state that delays rating messages, rematch offers or user navigation.
Menu, Play again and Request rematch remain available immediately. Allow
Hide celebration to restore the regular result scene without leaving the
match result; preference persistence is optional and not needed for MVP.

MatchEnd duplication, late RatingChanged and RematchOpen must not restart
the dance, replay audio, clear rematch state or allocate another cast.
The cosmetic roster is immutable once a completed result is accepted.
Exit/retry/rematch destroys all celebration-owned entities, effects,
lights/camera overrides and counters, then restores normal camera/HUD state.
Stale async messages from the old session cannot recreate its scene.

## 4. Reliable cast data: preferred minimal architecture

### 4.1 Aggregated lifetime roster

Prefer a bounded **runtime-only aggregate** recorded at authoritative spawn
and successful Evolution, indexed by seat, kind, available form and summon
eligibility. Each entry is a count of historical participants in their last
form. It includes dead units without retaining their complete simulation
states, poses, positions or per-instance event history.

On spawn increment the exact initial-form bucket. On successful Evolution
decrement that participant's previous-form bucket and increment its new
bucket. Death does not decrement participation. An expired summon remains
a participant. Existing live units supply previous form on Evolution, so
the collector needs no unbounded per-ID graveyard. Count a reused/carrying
actor once; moving a unit between visual attachments is not a deployment.
Use checked wide counters, validate underflow and reject inconsistent final
summaries. Do not silently saturate and claim an exact roster.

Keep this metadata out of ordinary state serialization/hash and combat RNG.
Bound bucket count by the content/form manifest, not match duration or
number of deaths. Observe all spawn paths: purchases, Nester hatching,
Smuggler cargo, pre-placed practice units and debug fixtures where relevant.
A shared core observer/counter helper is acceptable if it has no I/O and
cannot change simulation behavior. Do not implement several divergent
collectors based solely on lossy public view events.

### 4.2 Online delivery and reconnect

Deliver the finalized summary with the authoritative match-end result,
using the existing end-message route rather than a new HTTP service.
Proposed conceptual schema: `CelebrationCast { content_version, winner,
entries: [(kind, tier, branch, summon_class, count)] }`. This is a spec,
not an existing type. A finalized session/result identifier must bind it
to the correct match; reuse an existing suitable identifier if available.

This is a **wire change** if the current end contract has no equivalent
data. Allocate the next available protocol version only after inspecting
the integration base: main v17, native v20 and other planned versions are
not a safe version reservation. Update client/server/local practice,
handshake rejection, fixtures, exhaustive botclient matches and any stored
reconnect end payloads together. Follow the current protocol naming/types.
Spectators need no private roster in the proposed initial viewer policy.

The server retains this compact final summary for the existing reconnect/
result delivery window, including the case where the client was absent
before the final unit died. Do not introduce indefinite database history,
a replay service or a new match-result TTL just for cosmetics. If current
reconnect cannot deliver a completed result, document that gap and add the
narrow result replay route with reviewed lifetime semantics before claiming
reconnect coverage. Do not infer complete history from the reconnect view.

Send only winner identity and public presentation kind/form/count data;
no opponent Shop, Energy, private choices, seed or hidden combat state.
Bound decoding entries by manifest capacity; validate enum/form, duplicate
buckets, counts, sum overflow, seat/result consistency and frame-size limits.
Never allocate `count` figures while decoding. Paginate logical slots from
counts and allocate only the visible page. Missing/unusable summary falls
back to the ordinary result with a clear diagnostic; it must not replace
the winner, disconnect the player or create a partial cast labeled complete.

### 4.3 Presentation modules and ownership

Add focused client modules such as `celebration/{cast,layout,scene,motion}`.
`cast` validates and indexes logical slots; `layout` frames figures;
`scene` owns lifecycle; `motion` integrates existing kits and shared preview.
Keep result actions in the existing result UI, with browser/native adapters
only where the integrated UI requires them. New local scene entities have
a dedicated marker and never enter `UnitEntities`, targeting, local MatchState,
match metrics, purchase counts or network intention handling.

Reuse shared meshes/materials from FigureLibrary and rig-aware model bounds.
Animation continues in Result, but no gameplay stepping is enabled by it.
Use presentation clock time for dance loops, never to modify core ticks.
Catalog preview, ordinary victory, training and gallery must retain their
own lifecycles without leaking one scene's camera or animation state.

## 5. Execution steps

**Step 0 — integration and decisions.** Record SHA, protocol, current result
surfaces, animation resolver/P62 status and all spawn/Evolution/end/reconnect
paths. Resolve O01–O03. Produce active cast/form inventory and a protocol
compatibility plan. No merge of unrelated fronts is implied.

**Step 1 — exact roster contract.** Implement aggregate lifetime tracking,
local practice summary and focused tests for births, deaths, Evolution,
summons and counters. Prove ordinary simulation events/hash/outcomes remain
unchanged. No UI reconstruction from survivors.

**Step 2 — authoritative result delivery.** Integrate the summary into the
end/reconnect route, allocate the actual next wire version and update all
affected consumers/contracts. Enforce bounded decode and lazy expansion.
Update GAME-DESIGN/DECISIONS alongside any guarded contract change.

**Step 3 — result scene and formation.** Implement winner gating, cosmetic
figures, frontal camera/spacing, arena reuse, page controls if approved,
reduced motion and immediate result actions. Test both seats and empty cast.

**Step 4 — shared motion and pilot.** Reuse kit Victory, add the signature
override pilot, inventory fallback coverage and integrate with P62 Dance
when available. Review root bounds, multi-part rigs and animation-event
isolation. Record the future per-champion content backlog explicitly.

**Step 5 — targeted consolidation and owner review.** One reviewer runs the
union of affected focused tests once, desktop browser/native journeys and
owner visual sign-off. Record actual coverage, deviations and remaining
acceptance. Do not claim the full unique-dance roster is authored.

Commit one logical change at a time on the execution branch. Formatting/
cheap scoped checks per front follow the current validation policy. Write
regressions on fronts; queue their exact commands for consolidation.

## 6. Acceptance and validation plan

- **A01 Complete cast:** an early-dead unit, an evolved-then-dead unit,
  survivor, duplicate champion, summon and a unit born/dying between view
  ticks are all represented correctly. Evolution changes one bucket rather
  than duplicating an instance; dead participants remain. Reconnect sees
  the same final aggregate as a continuously connected client.
- **A02 Gating:** winning seat A/B, practice win and confirmed surrender win
  celebrate correctly. Loss, draw, spectator, unknown/canceled result and
  P62 controlled drills follow the declared policy. Empty cast is valid.
- **A03 Frontal layout:** humanoid, plant, heavy machine, wings/large rigs
  and carried units stand correctly, face the camera and fit without
  blocking controls at 1366×768 and 1920×1080 desktop windows and resizing.
- **A04 Motion:** existing per-kit Victory works, pilot Dance is distinct,
  fallback inventory is complete, future override registration requires no
  result-flow rewrite, and P62 preview uses the same resolver where present.
  No corpse pose, frozen hit-stop, residual aim or escaped root motion.
- **A05 Interaction:** Menu, retry and rematch work during celebration;
  duplicate MatchEnd/RematchOpen and late rating messages are idempotent.
  Connection polling continues; no mandatory wait or repeated victory sound.
- **A06 Wire and isolation:** malformed summaries, old protocol, wrong
  result identity, unknown kinds/forms, overflow and oversize payloads fail
  safely. Original result/rating are preserved. No presentation entity sends
  intentions, causes combat effects or changes normal state hashes/events.
- **A07 Scale:** test empty, 1, 3, 64 and 65 logical figures plus a very
  large count without eager expansion; every logical participant is reachable
  under the approved strategy. Counts match all pages, with no duplicates
  or missing indices across page boundaries.
- **A08 Lifecycle/accessibility:** 20 result/open/exit/page cycles return
  entities, cameras and audio to baseline after warm-up; no growing history
  buffer. Reduced motion, muted audio, keyboard navigation and English/pt-BR
  work. Exit/rematch cannot revive a previous match's scene.

Proposed performance budget: at most 64 visible figures under the paging
proposal, one celebration camera override, p95 frame-time within +10% of
the comparable same-arena/same-cast reference scene on the agreed desktop
machine. Measure cold load separately. Report real device, target, build,
cast and frame data; a native build does not prove WebGL2 compatibility.

Future focused targets **to create**: `tbg-core/tests/celebration_cast.rs`,
server tests filtered by `celebration`, and client tests filtered by
`celebration` for validation/layout/lifecycle. At consolidation run only
these plus affected existing protocol/determinism/result/rematch/reconnect
contracts; inventory the exact test names on the actual base. Example:

```bash
# Planned target, not present or executed by this planning branch.
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-core --test celebration_cast
```

All compiling validation uses the shared nonblocking lock and at most two
jobs. Busy lock/cache unavailable means defer. No default full workspace
suite, benchmarks, balance gates, full galleries, packaging or release.
Native OS/browser coverage and owner look approval are recorded separately.
Contract tests remain append-only under the repository's canon-change rule.

## 7. Owner checkpoints, rollout and follow-ups

- **O01:** recommended winning player's complete animated cast, including
  animated summons, excluding inert props; current result for losers/watchers.
- **O02:** recommended all figures simultaneously through 64, then clearly
  labeled pages with every participant accessible. Confirm paging or require
  a measured all-at-once LOD alternative; never silently drop participants.
- **O03:** choose and approve one signature-dance pilot plus camera/formation
  look. Unique dances for every character are a later content pass.

Enable after targeted validation and owner visual review. A cosmetic failure
falls back to the ordinary result immediately. Disable the presentation
entry to roll back visuals; retain compatible end-payload parsing until a
coordinated protocol release. Do not downgrade only one endpoint or revert
rating/result data. Normal revert commits preserve history.

Future content can add signature dances one champion at a time, with form
overrides and motion review, reusing the same preview/result resolver.
Dance purchases, emote chat, taunts, loot, unlock progression and replay
services are separate product requests and are unnecessary for this phase.

## Execution amendment (owner, 2026-10-31)

O01 and O02 recommendations are approved. O03 now requires a shared standard
with existing Victory motions and future per-character overrides; the owner
explicitly defers an exclusive pilot. Apply this amendment to Steps 0/4 and
A04 rather than inventing an unapproved signature dance. See
[P63-execution.md](P63-execution.md) for actual implementation and evidence.

## 8. Planning report

Documentation only. The plan records existing Victory support, result-unit
cleanup, full-cast/reconnect gaps, bounded summary design, camera/motion,
execution order and acceptance. No Rust/assets, gameplay rules in production,
installed client or server configuration changed. Implementation, targeted
tests, measurements, protocol allocation and O01–O03 remain pending.

Documentation checks passed: six phase links/anchors resolve, fenced blocks
are balanced, shell examples pass `bash -n` without execution, A01–A08 are
present, and `git diff --check` passes. The staged documentation is checked
again before its commit. No Cargo, browser or bot validation was run.

Executor report must record: integration SHA, resolved decisions, modified
modules, exact commands/results, deferred checks and reasons, cast/coverage
counts, protocol compatibility, hardware/frame measurements and owner review.
