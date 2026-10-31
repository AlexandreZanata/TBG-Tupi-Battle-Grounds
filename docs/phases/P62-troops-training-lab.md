# P62 — Troops catalog, training laboratory, and bot difficulty

> **Status: planning only — 2026-10-31.** No implementation, asset creation,
> gameplay verification, installation, merge, or deployment is included in
> this delivery. The owner requested another phase and publication of its
> plan on GitHub. All new rules and tuning below are **Proposed** until the
> implementation's product checkpoints are resolved.
>
> Planning branch: `codex/p62-troops-training-plan`.
> Suggested execution branch: `phase/p62-troops-training-lab`.
> Main baseline: `9008f117c4289122ddff978c5d25720d2bd79687`.

## 1. Outcome and boundaries

Both Troops entry points in the lobby open a useful, complete roster book.
Players inspect a champion's lore, abilities, forms, attributes and real
3D model, choose Idle, Walk, Attack/Action or Dance, and take selected
champions into a local training laboratory. A player can bring exactly
three champions, choose the opposing champions, or test against a wooden
training dummy. A separate practice setup offers Easy, Medium, Hard and
Impossible bots.

Deliver this as one phase with reviewable steps, not as a new game engine.
Reuse the canonical combat simulation, current champion kits, localization,
practice transport and arena. Desktop browser is the primary platform;
include the native desktop shell shown in the owner's screenshot when its
branch is integrated. No mobile layout or touch work is in scope.

The catalog, training controls and measurements are client/local features.
There is no new paid service, account requirement, matchmaking queue,
database, leaderboard, public result, reward or competitive balance patch.
Training cannot affect rating, ranked statistics, integrity evidence,
public spectating, match history, progression or account records.

### Read first

- [AGENTS.md](../../AGENTS.md); the owner's 2026-10-31 minimal-validation
  policy takes precedence over older phase commands. Read `docs/VALIDATION.md`
  if present on the eventual integration branch; it is absent on this base.
- [GAME-DESIGN](../GAME-DESIGN.md): glossary, combat, Shop, Evolution,
  Arcanas, Strongholds, Augments, arenas, and Proposed §16 below.
- [ARCHITECTURE](../ARCHITECTURE.md): crate boundaries and Proposed §18.
- [DECISIONS](../DECISIONS.md): D-33, D-50, D-80 and D-81.
- [ART-DIRECTION](../ART-DIRECTION.md): rig, motion, contact timing and
  desktop review; [P55](P55-champion-kits.md): existing kit ownership.
- [Training design and validation cases](../planning/TROOPS-TRAINING.md).

Read implementation files only for the step being executed. Do not read
or reuse `archive/`. This phase does not execute P60/P61 or depend on their
unmerged plans. P59 content and native shell integration are separate fronts.

## 2. Evidence from the current project

### 2.1 Two different source baselines must be reconciled

Main has protocol v17 and 25 purchasable kinds in `CANON.pool`/`FULL_POOL`,
plus Beetling and Crate in `ALL_KINDS`. `UNIT_KINDS` is the old ten-kind
pool, not the catalog source. The browser entry points are disabled in
`crates/tbg-client/index.html`; actions route through
`assets/lobby/lobby.js`, `src/ui/desktop_actions.rs` and `src/ui/actions.rs`.

The supplied image shows `TBG · 0.1.0-native.1`, with a disabled top Troops
tab and lower Troops card. The inspected native worktree's committed
baseline is `c5c263dd253eaf2f8d40ce918036d71dfc9c56a6` on
`codex/native-desktop-preview`: protocol v20, seven declared Factions and
four staged Monster kinds in a 31-row content table. Its active Shop pool
still contains 25 kinds; those four staged kinds are not playable merely
because an enum or model exists. Its relevant files are
`src/ui/shell/lobby/header.rs`, `cards.rs`, `shell/input.rs` and
`src/ui/actions.rs` under `crates/tbg-client`. That worktree also has
uncommitted unrelated work; this plan does not include or certify it.

P59's separate `phase/p59-complete-roster` branch reserves further content
and decisions through D-79. Recheck its actual merged status before coding;
do not hardcode either 25 or 31 as the future catalog size. This plan uses
D-80/D-81 and P62. GAME-DESIGN §14/§15 and ARCHITECTURE §17 are reserved by
the independently published P60/P61 plan; gaps here are intentional.

### 2.2 Existing pieces to reuse, with their limits

- `tbg-core/src/bot.rs`: a seeded view-based heuristic, not four difficulty
  levels. It prioritizes a tank, buying, Evolution and Fortification; it
  forgoes Augment rounds. Its current wait gives roughly 0.4–0.9 seconds
  between decisions when fed ten views per second. Preserve this baseline
  for existing balance measurements and online fallback users.
- `tbg-client/src/local.rs` and `link.rs`: local practice already steps
  `MatchState`, accepts `PlayerIntention`s and renders normal match views.
  Its debug army-fill loop mutates Shop offers/Energy; it is not a validated
  player-facing team builder. `with_preview` calls a leaking Ruleset
  constructor; repeated lab resets must not repeat that allocation.
- `tbg-core/src/shop.rs`: five distinct offers require a pool of at least
  five kinds. A selection of three champions must not replace the Shop
  pool with a three-element slice.
- `content.rs`, `ruleset.rs`, `state.rs`: existing definitions, forms,
  effective stats and live modifiers are authoritative data sources.
  Display names and localized content keys are presentation, not IDs.
- `gallery.rs`, `art/clips.rs`, `art/kits/`: models, rigs, aim, Walk and
  Attack/Action already exist. Gallery is a developer review scene; it is
  not a production catalog. There is Victory, but no shared Dance clip.
  Passive support kits may have `Attack::None` and need an honest Action
  demonstration instead of a fabricated attack.
- `tbg-core/src/lab.rs` is the **offline balance lab**. C10 in
  `tests/contract_lab.rs` forbids enabling `lab` in client/server and rejects
  custom Ruleset construction there. Do not weaken that protection to
  build this feature.
- `Event::Hit { to, amount }` carries neither attacker attribution nor
  necessarily effective HP removed. Summing it would overcount overkill
  and misattribute poison, splash, shields and support effects. Existing
  `LabStats` is per kind/seat, gated to offline work, not a complete
  player-facing per-instance meter.

## 3. Required product contracts

Requirement IDs are used by steps and tests; every one must have an
implementation path and recorded evidence before P62 is marked complete.

| ID | Required behavior | Validation cases |
|---|---|---|
| R01 | Both Troops entry points open the same catalog for guests/accounts | V01, V12 |
| R02 | List all currently playable champions; label staged and summon entries accurately | V02 |
| R03 | Lore, role, Faction, abilities, counters/conditions and all available Evolution forms | V02, V03 |
| R04 | All canonical numeric attributes with units and context, without copied balance constants | V03 |
| R05 | Rotate/zoom a real model; Idle, Walk, Attack/Action and Dance, with form switching | V04 |
| R06 | Add/remove exact units; choose one, three or other bounded team sizes and duplicates | V05 |
| R07 | Choose an opposing champion composition independently, or use training dummies | V05, V06 |
| R08 | Start, pause, resume, one-tick step, reset, edit setup and exit; repeat a seed | V07 |
| R09 | Accurate damage/healing/status measurements per unit and team; finite-target time to kill | V08 |
| R10 | Easy, Medium, Hard and Impossible with visible difficulty rules | V09 |
| R11 | No training effects reach online matches or account/ranking data | V10 |
| R12 | Desktop UI, English/pt-BR presentation, bounded resources, complete cleanup | V11, V12 |

The requested feature set is accepted as planning scope. Particular caps,
handicaps, bot weights, lore and visual choreography remain Proposed;
publishing this document does not approve them for production.

### 3.1 Three routes within Troops

1. **Catalog:** browse, inspect and animate champions. “Add to training”
   opens or updates a draft composition without starting a match.
2. **Combat laboratory:** exact pre-placed allies versus chosen enemies or
   dummies; no Shop, Energy spending, Fortification or timed Augment picks.
   Automatic movement/combat remains the normal simulation. Lab controls
   configure the experiment; this is not a new manual champion-control game.
3. **Practice vs Bot:** a full canonical match with Shop, Energy, Evolution,
   Strongholds and Augments; select bot difficulty and existing practice
   presets. Preserve the lobby's Practice shortcut and route it to this setup.

Explicit enemy champion selection belongs to the combat laboratory. Bot
difficulty belongs to full practice. The UI must explain this distinction:
the scripted combat of two pre-placed compositions has no purchasing bot
to make “Hard” different. Do not silently ignore the selected team or
pretend a fixed squad is a canonical five-offer Shop.

### 3.2 Catalog and preview

Use a shared catalog presentation model fed from core IDs and definitions.
Create authored lore separately from numeric content. Each playable kind
needs a short identity/lore paragraph, role, strengths, limitations and
mechanical ability text for each available form; all numerical tokens come
from core. Do not advertise an unimplemented ability or lore as approved.

Show cost, HP, damage per hit, interval/attacks per second, range, movement
speed, splash radius/share, healing, on-hit status duration/magnitude,
ability-specific cooldown/threshold/cap, Evolution costs and form changes.
Explain Synergy, Arcana, Augment, arena and temporary modifiers. Separate
neutral catalog values from “current in this experiment”; conditional
effects show their trigger, not a single misleading permanent number.

Use the five available form slots where supported (base, A1, A2, B1, B2).
Unavailable staged forms remain visibly unavailable. Match displayed model,
stats and ability form. Filters use Faction, Archetype and name; keyboard
focus, search, empty results and long localized text are part of delivery.

The preview owns one active figure, its camera, lights and effects. Reuse
kit selection, aim and contact timing. Do not spawn the entire roster or
advance combat to play a cosmetic animation. Add a preview Dance motion
with rig-specific overrides, including non-humanoid bodies; do not relabel
Victory as Dance. Attack contact/cast effects in the preview produce no
damage. Support-only champions show their real action and suitable prop
or ally. Missing assets produce an explicit loading/error state, not a
silent substitute champion. Desktop reduced-motion and sound preferences
apply; keep rotation/zoom bounded and do not hijack text-entry shortcuts.

### 3.3 Exact compositions and experiment context

**Proposed initial limits:** 1–12 initially selected units per side; a dummy
scenario has 1–5 dummies on the target side. Duplicates count as separate
instances. Summons created by abilities do not consume these selection
slots; reuse existing summon rules and enforce a separate local safety
ceiling of 256 living entities. On that ceiling, pause with an explanation,
not silent despawning. These are lab safety caps, not online army limits.

Choose kind, available form and lane position within valid bounds for
each instance, with deterministic default spacing. Position editing applies
in setup, not as a new online `Reposition` intention. Controls need buttons
or numeric alternatives to dragging. Draft edits require Reset to apply;
they never silently mutate a running measurement. Validate the complete
draft atomically before allocating a session.

Default to neutral Arcanas/Augments, Forest arena, automatic Synergy from
the chosen champions, and no tower fire or Stronghold auras in the combat
laboratory. Expose presets for Arcanas, arena and valid Augments as explicit
advanced context, using existing validation and supported rules. Show the
full context next to results. Stronghold combat and economy are practiced
in the full-match route. Do not describe stripped laboratory conditions
as a prediction of ranked win rate.

Finite combat stops at side elimination; a mutual elimination is a draw.
An inert target alone does not win a match. A **Proposed 300 active-second**
lab run limit pauses the experiment and preserves results; no fabricated
winner. Pause, focus loss and preview time are excluded from measurements.
The initial speed is 1x; time scaling and scenario sharing are follow-ups.

### 3.4 Dummy behavior: honest and implementable

Provide **Finite** and **Renewable** wooden dummies. Proposed HP presets
are 1,000 and 10,000, with zero innate damage reduction, no attacks,
movement, Faction contribution, loot, Bounty, cargo, Evolution or expiry.
Finite is removed on lethal damage and reports time to kill. Renewable
uses finite HP, dies normally, then respawns at the next simulation tick
with clean effects and a new instance ID; cumulative trial totals continue.
Never implement immortality by undoing damage before it is measured.
In Renewable mode, an empty target side awaiting respawn does not end the
trial; elimination of the attacking team still ends it. Clear windups to
dead IDs and perform the scheduled respawn before the next active combat
pass. Finite mode never respawns a target.

Recommended minimal representation: a runtime-tagged inert Crate carrier
inside `TrainingSession`, with a dedicated dummy renderer and HP override.
This reuses targetability and summon exclusion without adding a wire enum.
It **requires an explicit guard in the crate hatch/open path**; a zero
timer opens a normal Crate immediately. Real Smuggler crates keep normal
behavior. Audit `stats_of`, max HP display, targeting, hatch, expiry, damage,
death and view rendering before accepting this representation.

The initial dummy intentionally has summon-like target eligibility:
effects that require a conventional enemy (e.g. lifesteal or Leechroot
feeding) do not trigger on it. Show “Inert target — use a champion target
for conventional-unit interactions” before starting and in results. Use
chosen enemy champions to test those abilities and their real defenses.
Poison/status effects apply only as they normally apply to this target
class. Do not fake a universal human opponent by silently changing core
eligibility. A conventional-unit dummy requires a later explicit rule and
target-model extension if the owner wants it; do not substitute that
extension for the working initial dummy without revising this spec.

### 3.5 Difficulty proposal and fairness

Keep `Bot::new` and all existing non-practice call sites behaviorally
unchanged. Add `PracticeBot`/`BotProfile` for this route, consuming only
the bot seat's legal `MatchView` and returning ordinary `PlayerIntention`s.
No opponent Shop/Energy, hidden choice, future seed draw or private state
may reach a fair bot. Use seeded integer choices and fixed decision budgets.

**Proposed profiles**, measured in the existing 10 Hz decision cadence:

| Level | Decision spacing | Policy and resources |
|---|---|---|
| Easy | 15–25 view steps (1.5–2.5 s) | Seeded legal buys; simple frontline/support mix; infrequent Evolution; no lookahead; normal resources |
| Medium | Current baseline, roughly 4–9 steps | Delegate to existing Bot behavior initially; normal resources; label its current Augment limitations |
| Hard | 2–4 steps | Score affordable legal buys, Evolution, Reserve/Release, Fortify and Augment choices by visible composition, defense, Synergy and cost; normal resources |
| Impossible | 1–2 steps | Hard policy with a disclosed practice-only Energy regeneration advantage; normal HP, damage, Shop randomness and visibility |

Hard's initial scorer is a small deterministic heuristic, not an LLM or
unbounded search. Enumerate at most the existing legal offers/actions;
stable tie-break by action type and ID, with any exploration drawn from
its own seeded RNG. Prioritize survival, then useful composition/ability
coverage, then efficient Evolution/economy. Version and tune weights using
the paired-seed evaluation below; do not claim optimality from the table.

**Recommended Impossible default, Proposed:** 2x the bot's normal per-tick
regeneration, including its ordinary arena/Interest result, implemented as
a bounded local credit of that tick's normal regen gain. Starting Energy,
purchase prices and combat stats stay canonical; Bounty and other credits
are not doubled. No bonus during Countdown, Augment selection or pause.
Display “Impossible · Bot Energy regeneration ×2 · No ranked rewards” on
setup and throughout the match. Reset/regenerate its fractional accounting
deterministically. This assistance is owned by the local session, never
by `CANON`, generic `Bot`, online `MatchState::step` or a network message.

The owner may choose a fair-only strongest profile instead. Resolve O03
before enabling the assisted mode. “Impossible” is a difficulty label,
not a promise of an unbeatable solver. Require hard playtest evidence;
if it is easily beaten, revise the Proposed profile, not secretly grant
extra information, invulnerability or arbitrary wins.

## 4. Technical contracts and file ownership

These are proposed interfaces to implement, not APIs that exist today.
Keep concerns split; do not grow `ui/actions.rs` or `local.rs` into a
catalog, scenario editor, bot engine and telemetry file.

### 4.1 Core: narrow deterministic training support

Create `tbg-core/src/training/` with `scenario`, `session`, `dummy` and
`metrics` responsibilities, and a separate `practice_bot` module. Introduce
an optional `training` feature, enabled by client only; `lab` remains
offline-only. Cargo feature unification is not a security boundary: every
mutation also requires an explicitly constructed local `TrainingSession`.
Default matches have no active training runtime, even in a build that
contains this feature. The server must not import or construct it.

- `TrainingScenario`: stable schema version, seed, content/rules fingerprint,
  two bounded compositions, arena/preset context, target mode and run limit.
- `TrainingUnitSpec`: kind ID, validated tier/branch and clamped/validated
  integer lane coordinate; quantity expands to distinct IDs on construction.
- `TrainingCommand`: Start, Pause, Resume, Step, Reset and ReplaceDraft,
  local-only, with typed errors for stale session IDs or invalid states.
- `TrainingSession`: owns private canonical `MatchState`, runtime dummy tags,
  lifecycle, bounded per-instance counters, and seed. No mutable state
  escape hatch for the client. Expose a read-only presentation snapshot.
- `TrainingMetrics`: integer totals and simulation tick durations; separately
  identify direct, periodic, splash, summon, heal and unattributed sources.
  Presentation may format decimals; simulation does not use floats.

Use the canonical constructor, a narrow validated spawn/form initializer,
and a shared combat-step path. Factor the existing step into small helpers
only as needed so the laboratory can omit economy, councils, tower fire,
tower auras and match-end processing while retaining the exact order of
timers, statuses, summons, targeting, movement, hits and deaths. Do not copy
the combat algorithm into training. Disabled training hooks must preserve
normal state hashes, events, RNG draws and outcomes on matching inputs.

Any new runtime-only metadata must be omitted from network serialization
and ordinary state hashes; a training replay identity includes scenario,
seed, command sequence and content fingerprint separately. Do not claim a
normal `MatchState::hash()` alone proves a whole training replay identical.
Do not introduce a new `ClientMsg`, server endpoint, `MatchMode`, rating
record or protocol version for cosmetic/local controls. If implementation
finds a wire change necessary, stop that dependent step and revise the
plan with compatibility evidence before proceeding.

Keep canonical Ruleset access borrowed/static. Do not construct and leak
a Ruleset on each reset; remove the redundant preview allocation from the
affected practice path only after proving its pool is equivalent on the
chosen integration base. Preserve separately staged preview behavior if
that equality no longer holds.

### 4.2 Measurement hooks, without changing combat results

Record at the actual mutation points, not from rendered floating numbers.
`hits.rs` reports effective HP removed after reduction, shields, plates,
Second Wind and overkill clipping; `heal` reports actual HP restored after
all limits. Keep shield absorption and plate damage separate from HP damage.
Attribute via attacker ID plus effect kind at the call site; periodic
effects need runtime provenance captured when applied, including refresh,
replacement, death of the source and multiple sources of the same kind.
Use the same attribution rule for splash, summons, betrayal and secondary
effects. Do not increase or reorder online event streams for these counters.

Map attribution points through `hits.rs`, `afflictions.rs`, `support.rs`,
`summon.rs` and `combat.rs`. Use a bounded sidecar keyed by IDs/effect slot
when source is absent from serialized effect state. An unknown source
must be shown as unattributed and included in team reconciliation; never
guess from nearby AttackStart events. Completion requires zero unexplained
damage in the documented controlled fixtures, not merely attractive charts.

### 4.3 Client: shared data, small surface adapters

Create `src/troops/{catalog,details,preview,content}` and
`src/training/{setup,controls,results}` or equivalent focused modules.
Use the existing i18n catalogs (`assets/i18n/en.json`, `pt-BR.json`) and
generated resolver; avoid a second translation framework. English source
and documentation remain required; pt-BR strings are product localization.

Add a typed local training variant/controller alongside `MatchLink::Local`.
The normal net sender must have no overload accepting `TrainingCommand`.
A distinct local session context gates debug controls and result handling;
do not rely on `MatchMode::Casual` to identify training. Ordinary online
intentions still get the seat from the server connection.

On the native integration base, add Troops actions to the shared shell
header/cards and a dedicated page module. On the browser base, wire both
disabled HTML buttons and existing bridge to the same Rust catalog/setup
state. Prefer the already-integrated shared Bevy shell if it serves both
targets; do not introduce a second full UI solely to match an obsolete
branch. Step 0 records which real surface each target uses.

Entry requires an idle player. During queue, match, reconnect or spectating,
disable launching training and explain how to return to idle; never silently
abandon a match. An existing idle presence socket need not be closed, but
must not report a training match or receive training controls/results.
Exit tears down figures, camera/render targets, metrics, entities, effects,
audio, selected units, intent sequence and pending local actions. Restore
lobby camera, focus, prior filters and normal HUD routing. A pending command
from the old session cannot affect a new online match.

## 5. Execution sequence and acceptance

Every step ends with a focused diff and a report entry. One executor owns
shared files at a time. No implementation is performed by this document.

### Step 0 — Freeze the integration base and content inventory

Record target commit, active browser/native shells, actual roster pools,
protocol, kit coverage, localization coverage and P59 status. Resolve O01–O04
or retain explicitly disabled optional modes. Audit every dummy/measurement
hook above, especially Crate opening and per-effect provenance. Select the
bounded architecture and record tests that will prove no online change.

Acceptance: approved requirement mapping, module ownership, content inventory
with actual IDs/forms, and exact target commands. No branch-specific native
or staged content silently assumed merged. Reusing a three-kind Shop pool,
enabling `lab` in client, or unbounded Ruleset allocation is rejected here.

### Step 1 — Catalog entry, complete content and stat presentation

Implement R01–R04. Unlock both Troops entry points only when they reach the
working catalog. Build complete content records for each active kind,
summon relationships, staged labels and actual form coverage. Author lore
and translations with owner review; derive numeric tokens from core.
Use exhaustive mappings/coverage checks so future roster additions fail
visibly when presentation is missing. Keep authoring independently reviewable.

Acceptance: V01–V03; no placeholder lore counted as finished, no missing
active champion, and no future champion enabled by the catalog.

### Step 2 — Real model viewer and Dance

Implement R05 with existing art kits and a single preview scene lifecycle.
Add form controls, camera bounds, clip selection, rig-specific Dance and
real support-action previews. Retain motion/contact rules; no combat events
from cosmetics. Capture representative humanoid, plant, heavy machine,
carried/multi-part and passive support cases, then inspect remaining forms.

Acceptance: V04 and owner look review. A generic animation that visibly
breaks some rigs is incomplete, even if all buttons respond.

### Step 3 — Training kernel and actual dummy

Implement the narrow `training` feature, validated scenario builder,
canonical spawn/form setup, shared combat step, finite/renewable dummy
and deterministic reset. Add focused regressions before UI controls. Keep
non-training baseline hashes/events and C10 intact. The dummy must never
open cargo, earn rewards, contribute Synergy or alter real Crate behavior.

Acceptance: V05–V07 and V10 at the core boundary; exact three-unit fixture,
enemy selection, status behavior, invalid configs and repeated reset work.

### Step 4 — Team editor, lifecycle and measurements

Implement R06–R09 and R12. Connect catalog selection to the editor, target
selection to canonical combat and metrics hooks to read-only results.
Add controls, units, context labels, reset semantics and safe session exit.
Distinguish practice results from the online end screen.

Acceptance: V05–V08, V11–V12. A player can complete the entire three-champion
journey without URL debug flags or dev tools. Totals reconcile with actual
HP changes; missing attribution is exposed and resolved for required fixtures.

### Step 5 — Practice difficulty and scoped calibration

Implement R10 with `PracticeBot`, preserving old bot entry points. Add Easy,
Medium and Hard; validate hidden-information independence. After O03,
implement the visible Impossible local assistance and its exact regen
accounting. Existing Practice shortcut opens this setup; replays/reset
preserve the selected level. Store no performance claim in marketing text
until measured.

Acceptance: V09–V10; normal resources for fair levels, disclosed assistance
only for Impossible, deterministic policies, measured ordering and owner
challenge review. Report infeasible or inconclusive difficulty targets
honestly; no ranked balancing changes or mass simulation runs as a shortcut.

### Step 6 — Consolidation and delivery

One reviewer runs the deduplicated affected tests once and focused desktop
journeys, fixes regressions, records owner approvals and unresolved checks,
then updates roadmap status. Exercise switching from lab to ordinary practice
and to an authorized online test session with no leftover controls/effects.
Evaluate resource cleanup and load measurements on the target integrated
tree. Merge/deployment/installation require their own authorization; none
is implied by completion of the planning branch.

Acceptance: every R01–R12 has evidence or a clearly recorded blocker;
applicable deferred acceptance prevents a “fully verified” claim. Catalog
may be released separately after its gates, but P62 stays incomplete until
the lab and difficulty requirements are delivered.

## 6. Validation policy and proposed commands

### Planning delivery (this branch)

Only inspect documentation, references, numbering, schema examples and the
diff; run `git diff --check`. No Cargo, Trunk, browser build, balance run,
bot match or screenshot generation is needed to validate a plan.

### Future work-front mode

Follow the owner's 2026-10-31 policy. Format only the affected package and,
if cached/inexpensive, one narrowed `cargo check`; do not routinely check
both targets. Write meaningful tests below and queue them for consolidation.
Cold Bevy checks and unavailable cache are deferred, not retried with a new
target directory. Every compilation uses the shared lock and at most two
Cargo jobs; a busy lock (75) means defer, never poll or wait.

Example for the core implementation front, **not run by this plan**:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo check -p tbg-core --lib --features training
```

### Future consolidation queue

The test targets below are **to be created** by Steps 1–5. Current repository
targets are named separately. The reviewer adjusts paths if the integrated
base changes, records the exact final commands, and deduplicates them.

```bash
# Planned integration test targets; training feature does not exist yet.
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-core --features training \
  --test training_scenarios --test training_metrics --test practice_bot

# Existing contracts on this base; append cases rather than rewriting goldens.
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-core \
  --test contract_determinism --test contract_lab --test contract_protocol

# Planned focused client test module on the eventual native-capable base.
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client troops_training
```

If the integrated client needs target-specific features, use its actual
native manifest options instead of guessing. Record the browser compatibility
check once at consolidation and a focused browser run of V01/V05/V08/V10/V12;
a native check does not prove WASM compatibility. A native screenshot does
not prove the HTML lobby or its bridge works. Test C10 with combined
`lab,training` core features if shared hooks change, and separately verify
that client/server manifests still do not enable `lab`; feature unification
must not activate training behavior on ordinary matches.

Contract tests are append-only; any legitimate contract change accompanies
the GAME-DESIGN/DECISIONS update in the same commit and names it. Normal
hash or wire golden changes are a regression to investigate, not expected
output to bless merely because training was added.

No automatic workspace suite, cold rebuild, full gallery, balance gate,
benchmark or mass bot evaluation. The bounded difficulty experiment in the
companion document requires explicit authorization for that named workload;
manual owner playtesting is separately recorded. These future commands and
cases are a queue, not a claim that any check has passed today.

## 7. Open decisions, risks and suggested improvements

- **O01 — integration base:** recommended integrate against the reviewed
  shared desktop shell and real active roster. If unavailable, deliver the
  browser adapter on main and keep native parity explicitly pending.
- **O02 — training caps/target semantics:** recommended 12 starting units
  per side, finite/renewable inert dummy, conventional-enemy interactions
  through selected champions. Approve limits and labels before enabling.
- **O03 — Impossible:** recommended transparent 2x regen assistance; a
  strongest fair-only bot is an alternative. Do not invent owner approval.
- **O04 — authored material:** owner reviews lore, role descriptions and
  Dance identity for every active kind, then reviews the full desktop flow.

Priority improvements are part of the necessary work: one catalog inventory
instead of copying Shop arrays; one stats presenter instead of duplicated
balance numbers; local-only typed controls; attributed integer metrics;
reusable preview lifecycle; elimination of reset allocations; preserving
the old bot as a reproducible baseline. These directly reduce wrong stats,
missing champions, online leakage and performance regressions.

Potential later improvements, not blockers for the requested feature:
save/load local scenario presets, seed links with a versioned bounded
parser, speed controls, conventional-unit dummies, side-by-side champion
comparison, training lessons and advanced constrained-purchase bot drills.
No replay service, cloud scenario database, ML training farm, account sync,
additional ranked queue or universal bot framework is warranted for this MVP.

### Rollout and rollback

Integrate the reviewed catalog/preview first, then laboratory controls and
measurements, then difficulty profiles. Enable each player-facing entry
only after its own acceptance; keep unavailable actions explicit. An
incomplete feature is not completed by hiding its failure behind Locked.
Owner approval of look/lore/challenge and the actual target evidence remain
release gates, separate from authorization to publish this plan.

If integration regresses a surface, disable only its new training/preview
entry and restore the existing Practice path. If bot assistance misbehaves,
disable Impossible and retain the unchanged baseline Bot. Discard only the
current in-memory trial; no account migration or ranking repair is needed.
A core equivalence failure blocks release of the affected core change and
requires a focused fix or normal revert commit, never rewritten history.
Document any disabled requirement as incomplete and keep its evidence gap
visible in the roadmap. No production rollback is executed in this task.

## 8. Execution report template

The executor fills this during implementation; leave checks unclaimed here.

- Integration SHA, target surfaces, active roster and rules fingerprint:
- Requirement IDs completed / partial / blocked, with file and evidence:
- Product decisions O01–O04 and owner review evidence:
- Commands actually run, exit codes, scoped results and lock state:
- Deferred commands/cases and exact reason; consolidation owner:
- Measurements: stats reconciliation, difficulty sample, time/memory/frame
  data, hardware/build profile and scenario identity:
- Unchanged normal-play evidence, protocol/C10/feature isolation:
- Deviations, compatibility concerns and rollback behavior:
- Validation level: docs only / work-front / targeted consolidation:

For the planning delivery, see the actual report at the end of
[TROOPS-TRAINING.md](../planning/TROOPS-TRAINING.md).
