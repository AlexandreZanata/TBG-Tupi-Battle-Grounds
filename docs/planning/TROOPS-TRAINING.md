# Troops training — player journeys, measurement and acceptance

> **Planning companion to [P62](../phases/P62-troops-training-lab.md).**
> This describes future behavior. No UI, bots, dummy, lore or combat change
> has been implemented by this branch. New values are **Proposed** in
> [GAME-DESIGN §16](../GAME-DESIGN.md#16-troops-and-local-training-p62-proposed).

## 1. The player's path

From either the helmet icon in the header or the lower Troops card, the
player enters the same catalog. Keep the medieval visual language in the
supplied screenshot: existing navigation, gold hierarchy, dark readable
panels and a prominent champion model. Avoid squeezing the entire interface
over the current battle HUD. Use a focused page with a roster/filter area,
model viewport and readable detail panel; expand long lore/abilities in
scrollable content that leaves essential actions visible.

The entry remains usable for a Guest. The page shows Playable entries by
default. Staged entries can be inspected only when real presentation data
exists, marked In development and not selectable for normal training.
Summons appear under their parent or a clearly labeled secondary view;
they are not independent purchasable champions.

### Journey A — inspect one champion

1. Open Troops and search by name or filter by Faction/Archetype.
2. Select a champion. Read identity, lore, gameplay role and abilities.
3. Switch base/A1/A2/B1/B2 where available. The model and all values update
   together; display what changes and what conditions apply.
4. Play Walk, Attack/Action or Dance; rotate/zoom the model. The detail panel
   never shows damage caused by preview animation as a combat measurement.
5. Select Add to training. A visible draft counter confirms the added unit.

### Journey B — exactly three against chosen champions

1. Add Archer, Shieldbearer and Bloomkeeper once each; confirm “3 units”.
2. Open the editor, select forms and default or edited positions.
3. Choose Champions as the opponent and add Bone Brute and Hexer, or any
   other valid combination. Duplicates on either side are allowed.
4. Review both compositions, arena, Synergy and modifier context. Start.
5. Observe automatic canonical combat, select an instance to inspect it,
   pause, step one simulation tick, resume and inspect actual results.
6. Reset with the same seed to repeat; return to setup to change an enemy.
   That change starts a fresh trial and clears the old trial's totals.

The three selected champions are three actual initial units, not a filter
that quietly fills the team with other kinds. Ability-created summons are
shown separately and retain canonical behavior. This route does not offer
manual steering or fabricated ability-cast buttons for an auto-combat game.

### Journey C — attack a dummy

1. Choose one or several allied champions and Dummy as target mode.
2. Choose finite or renewable HP, count and placement; read target eligibility.
3. Start; observe actual hit, splash and applicable status behavior. Finite
   targets report a kill time; renewable targets report each life and totals.
4. A support champion with no valid ally/action shows an explanation and an
   action to add an appropriate ally or switch to champion opponents.
5. Choose a real enemy composition to test lifesteal, feeding, armor-like
   defenses, betrayal, boarding or other conventional-unit interactions.

Use wording equivalent to “This inert target does not trigger effects that
require a conventional enemy.” This is a gameplay explanation, not an
implementation detail about its internal Crate representation.

### Journey D — challenge a bot

1. Open Practice vs Bot from Troops or the existing lobby shortcut.
2. Choose Easy, Medium, Hard or Impossible; read its concise behavior.
3. Pick the existing practice arena, Arcana and Stronghold options. Full
   practice starts with the normal Shop and match rules.
4. If Impossible assistance is approved, display the Energy regeneration
   advantage in setup, gameplay and results. Never hide it in a tooltip only.
5. Play, retry at the same difficulty, change difficulty, or return to Troops.
   Results explicitly say Practice and never update ranking/account records.

## 2. Content inventory to produce at Step 0

On the audited main baseline the active 25 kinds are Shieldbearer, Archer,
Bone Brute, Hexer, Sawbot, Siege Walker, Bloomkeeper, Rootcaller, Nester,
Rampart, Longaxe, Farwatch, Bombard, Rotcap, Old Finn, Antidote Brewer,
Shatterback, Blood Furnace, Bloodweaver, Oilpress, Drumguard, Leechroot,
Cargo Crab, Puppetmaster and Smuggler. Use core IDs as keys, not these
English names. Beetling and Crate are summon entries, not Shop choices.

The native snapshot additionally declares Bonehorn, Ravager, Acidmaw and
Whisperer with staged content. Their presence in enums/content tables is
not permission to train with unfinished mechanics. Later P59 changes can
add more kinds and forms: regenerate coverage from the reviewed active
manifest and record the actual integration fingerprint.

For **every active kind**, record: ID, playable status, display name keys,
Faction/Archetype, portrait/kit, each available form, lore keys and review
status, base/conditional ability coverage, stats coverage, animation/action
coverage and owner visual verdict. Fail the content gate on missing active
records. Do not fill absent lore with an unreviewed statement about the
world and mark it finished. Lore must describe the game's fictional world;
it must not invent historical claims about real Indigenous peoples.

Compile-time or focused coverage checks should discover newly playable
kinds and new ability/form variants. A future roster patch must add its
Troops material in the same change or explicitly keep the kind staged.
No per-frame JSON parsing or catalog-wide model rebuild is needed.

## 3. What a number means

### Catalog attributes

Source numeric values from the same definitions and integer helpers used
by combat. Format units only at the presentation edge:

- HP and damage/heal points; Energy with the core scale.
- Tick durations divided by `TICK_HZ`; movement/range in the project's
  actual lane units converted with existing display helpers.
- Basis points shown as percentages with consistent rounding. Display
  the rounded value while retaining the exact integer for comparisons.
- Show interval, range and theoretical base hit rate, but label any
  `damage × TICK_HZ / interval` estimate “Nominal, continuous basic attacks”.
  It excludes walking, windup alignment, effects, mitigation and target loss.
- Show conditional attack speed, splash share, status effects, auras,
  stacks, cooldowns, ally requirements and cap behavior in the relevant
  ability section. A missing/non-applicable field is “Not applicable”,
  not an invented zero-damage ability.

Catalog neutral stats, configured scenario stats and current live stats
are three explicit contexts. Do not present full Arcana/Synergy bonuses as
the neutral base. “All stats” means complete gameplay attributes and
mechanical conditions, not RNG internals or network diagnostics.

### Measured laboratory results

Let `N` be the number of active simulation ticks since Start/reset. Then
elapsed active seconds = `N / TICK_HZ`, and effective DPS is
`effective_hp_damage * TICK_HZ / N`. Before the first completed active tick,
show an em dash, not NaN/infinity. The display may format a decimal quotient
from integer counters. Pause time, setup, preview, UI rendering delays and
renewable-target respawn waiting outside active ticks do not add time.
The actual respawn tick is active and does count. Keep this denominator
identical for per-unit and team rows so sums can be reconciled.

Required counters and distinctions:

- Effective HP damage dealt/taken; direct, splash and periodic categories.
- Shield absorption and plate damage separately; never add them to HP DPS.
- Effective healing, overheal excluded; self-healing and ally healing distinct.
- Hit/action count and applied status duration. Per-effect duration can
  overlap; do not label its sum unique target-control time.
- Summon damage attributed to the summon, with an optional parent-group
  total clearly labeled; team totals count it once, not twice.
- Finite target kill time from Start to lethal tick, and optional first-hit
  to lethal duration with its own label. Renewable mode reports per-life
  kill times plus trial totals; no single infinite-target time-to-kill.
- Friendly damage/betrayal identified separately from hostile damage. A
  transformed/retargeted effect retains its recorded source/target semantics.
- Source-dead periodic damage remains attributed to its original source.
  Unsupported provenance is counted as unattributed, never dropped.

Use wide integer accumulators and bounded entity records. Include dead
instances in results. With the Proposed 256 living-entity ceiling, also cap
retained per-instance result rows at **4,096 per trial**; on that cap pause
and ask for Reset instead of losing attribution. This protects renewable
target and summon churn without silently corrupting totals.

A recording stores scenario identity and content fingerprint in memory;
this phase requires repeatable Reset, not durable replays or cloud sync.
Statistics are measurements of that local experiment, not ranked pick/win
rates, patch evidence by themselves or values to upload into P61.

## 4. Validation cases

These are future acceptance cases. Use focused regression tests for actual
contracts, not a unit test per UI label. Evidence includes configuration,
seed, expected behavior, actual result and the integration SHA.

### V01 — entry and access (R01)

Exercise the top icon and lower card in the browser and native shell used
by the integrated build. Guest and account reach the same catalog. Back
restores lobby state. Queued/playing/reconnecting/watching cannot start a
training session or discard the online session through this route.

### V02 — roster, lore and forms (R02–R03)

Compare catalog coverage to the active core pool and content manifest;
inspect all forms and summon relationships. New/staged enum entries do not
become active by iteration. Validate Faction filters as the count expands,
search results and zero-result state. Missing translations and missing
authored lore fail the release content inventory.

### V03 — values agree with the game (R03–R04)

For a representative basic attacker, healer, summoner and conditional
ability champion, compare every presented attribute against canonical
helpers for base, an A form and a B form. Exercise a real Synergy threshold,
neutral/selected Arcanas, an Augment, arena modifier and a temporary effect.
Check the exhaustive ability presenter covers all active ability variants.
Verify scale/rounding and distinguish nominal DPS from measured DPS.

### V04 — truthful animation (R05)

Play Idle/Walk/Attack or Action/Dance with different rigs and all available
forms. No wrong model, frozen limbs, detached gear, invalid target, camera
clipping or phantom damage. Passive supports demonstrate their real action.
Switch repeatedly and leave: one active figure/camera set, no accumulating
effects/sounds, no modified match state. Owner reviews style and identity.

### V05 — exact compositions (R06–R07)

The three-unit journey starts with exactly those three instances and the
chosen enemies; summons appear only through canonical behavior. Repeat a
kind twice and distinguish IDs/counters. Test minimum and Proposed maximum
sizes, malformed kind/form/position, unknown IDs, empty side, oversized
input and atomically rejected drafts. A three-kind selection never calls
Shop drawing on a pool shorter than `OFFERS`.

### V06 — dummy versus real enemies (R07)

Finite and renewable dummies do not move, attack, open cargo, expire,
earn Bounty, contribute Synergy or restore HP before damage measurement.
Statuses and periodic damage follow the declared target class. Renewable
respawn uses clean state and does not leave a stale windup targeting an
unrelated new entity. Real Smuggler Crates still open and release cargo.
Use selected conventional enemies to verify lifesteal/Leechroot feeding,
target defenses, conditional damage, multi-target effects and betrayal.

### V07 — lifecycle and determinism (R08)

Equal scenario, seed, command/tick sequence and content produce equal
snapshots, outcomes and integer totals. Pause is inert; one Step advances
exactly one simulation tick; Reset clears all unit effects, summons,
counters, RNG, local queues and dummy lives. Old commands are rejected by
session identity. Test elimination, draw, run limit, entity/row cap, focus
loss and exit while paused. Normal practice keeps its canonical outcome.

### V08 — measurement fixtures (R09)

- A hit larger than remaining HP: HP damage equals actual removed HP;
  damage prevented by shields/plates and overkill are not effective DPS.
- A partially injured ally: healing equals HP restored; overheal is excluded.
- A splash strike: each affected target contributes once; team and instance
  sums reconcile, including friendly damage where the rule permits it.
- Two periodic sources, same-kind units and dead source: source attribution
  survives replacement/stack refresh and reconciles with mutation records.
- Summons and parent grouping: no double counting, including parent death.
- Second Wind, conditional damage and kill timing: correct lethal/nonlethal
  outcome and denominator; no inference from `Event::Hit.amount` alone.
- Zero elapsed ticks, pause, same-seed reset and renewable targets: stable
  totals, no NaN, no wall-clock-dependent results and clean per-life IDs.

The totals must equal the sum of actual health mutations, with healing and
damage recorded separately even when both occur in one tick. End-to-end
net HP difference is insufficient to validate simultaneous effects.

### V09 — difficulty and fair information (R10)

Test each profile's legal actions, cadence and deterministic tie-breaks.
Changing an opponent's hidden offers/Energy while its public view stays
the same must not change the fair bot's action. Fair profiles receive no
stat/resource advantage. Hard evaluates a legal useful Augment, handles
an open council and does not spend while paused or on stale views.

Impossible's local bonus equals exactly its configured extra ordinary
regen, not Bounty, starting Energy, Interest counted twice or a fixed amount
per rendered frame. Countdown, council and pause grant none. Switching
difficulty requires a new practice match. Regular `Bot::new` behavior and
all other callers remain unchanged.

For challenge calibration, propose a named bounded experiment: **24 fixed
seeds, swapped seats, for each adjacent pair** (Easy/Medium, Medium/Hard,
Hard/Impossible) = **144 matches**, neutral shared presets and Forest,
plus owner playtests using actual Shop control. Authorize this workload
explicitly before running; no balance gate or million-match search.
Cap each experimental game at **6,000 simulation ticks** and classify
unfinished games separately, never as wins. Record versions, policies,
seed list, seats, duration, actions, outcomes, errors and truncations.

Evaluate paired results and uncertainty: a Proposed screening goal is
at least 60% score (win=1, draw=0.5) for the stronger profile, with a
pair-aware interval reported. This is a tuning signal, not proof of human
difficulty. Report resolved coverage and inconclusive pairs; changing the
seed suite until a profile wins is not validation. Repeat only affected
pairs after a policy change. Longer runs need separate authorization.

Owner playtests decide whether Easy is approachable and Impossible feels
exceptionally challenging. No numerical human win-rate promise is inferred
from bot-vs-bot results. A failed challenge goal leaves R10 partially open.

### V10 — isolation from competitive play (R11)

Trace all TrainingCommand paths: none serialize into `ClientMsg`, mutate a
server seat or arrive at a results writer. Local completion cannot reach
ranked persistence, P60/P61 collection, live registries or rewards. An
idle presence socket may remain; verify no training match/results traffic.
After exit, an ordinary online test has normal resources and controls.
Preserve protocol golden bytes, normal hashes and canonical bot fixtures.
Test ordinary matches in a build with training enabled, and validate the
client/server `lab` prohibition still holds under feature unification.

### V11 — bounded cost (R12)

Use the agreed desktop reference machine/build and compare to the same
baseline. Proposed feature budgets: one active preview figure, catalog
interaction p95 below 100 ms after assets are loaded, and frame-time p95
no more than 10% worse than the comparable existing preview/local scene.
Report cold-load time separately; no invented hardware-independent result.

Repeat open/close/reset **20 times** in a focused manual session: entity,
camera, render-target and session counts return to baseline, memory does
not grow with every reset after warm-up, and pending actions do not survive.
Exercise the configured maximum initial composition with summons and the
safety caps; show the pause reason and preserve results. No gallery-wide
stress run is implied. Fix leaking Ruleset allocations before acceptance.

### V12 — desktop usability and localization (R01, R12)

Check 1366×768 and 1920×1080 desktop windows, resizing and supported desktop
UI scaling, keyboard/mouse, readable contrast/focus, independent scrolling,
English/pt-BR long text, loading/failure/empty states and return navigation.
Test browser WebGL2 and the actual integrated native shell. Record native
OS coverage honestly; a Linux run does not certify Windows/macOS. Keep
existing phone code compiling but do not expand this phase into mobile QA.

## 5. Owner decisions and implementation order

The practical order is catalog/content → preview/Dance → safe training
kernel and dummy → team editor/results → bot profiles → consolidation.
This gives a useful catalog early while protecting the simulation from a
large combined UI/core change. Review each boundary before layering UI
onto inaccurate measurements or a misbehaving dummy.

Resolve P62 O01–O04 at the relevant implementation step. No answer is
needed to publish this plan. Recommended defaults and alternatives are
already concrete enough for a future executor to estimate and review.
Do not start a paid service or unrelated platform migration to deliver it.

## 6. Planning delivery report — 2026-10-31

- Scope: new phase specification, UX/measurement/test companion, Proposed
  canon/architecture/decisions and roadmap entry; documentation only.
- Audited: main `9008f117c4289122ddff978c5d25720d2bd79687`, native committed
  snapshot `c5c263dd253eaf2f8d40ce918036d71dfc9c56a6`, relevant live source,
  current local refs for decision reservations and the owner's screenshot.
- No production files, existing user work, installed game or account data
  were changed. No implementation or gameplay validation was performed.
- Documentation validation: a Python standard-library check passed for all
  16 newly added local links/anchors, balanced fenced blocks and coverage of
  12 requirement IDs, 12 validation cases and 12 Proposed canon rules.
  `git diff --check` and `git diff --cached --check` passed (exit 0), including
  new files. Both shell example blocks passed `bash -n` syntax checks without
  executing their commands. The staged scope check confirmed exactly six
  Markdown documents and no source/asset changes.
- Deferred: all Step 0–6 implementation, content authoring/approval,
  regression execution, browser/native playthroughs, metrics/difficulty
  measurements and release decisions. They require implementation first.
