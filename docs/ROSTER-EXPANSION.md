# Roster expansion — 15 new units (plan, Proposed)

> Planning document. Nothing here is active: every value is **Proposed**
> and enters the game only through the phases in §G, each gated by the
> balance lab (D-33) and switched online by the owner. Written 2026-10-29
> from the owner's drafts and an audit of the code at `d1151b5`, then
> rebased on `main` at `0154dd4` (D-45: a purchase redraws all five Shop
> offers). Measured numbers cite the command that produced them;
> everything else is labelled *model* or *hypothesis*.

## Contents

A. Diagnosis of the current game · B. Final cast · C. Unit sheets ·
D. Interaction matrix · E. Balance model · F. Architecture ·
G. Implementation phases · H. Competitive risks · I. Release criteria ·
J. Next action · Appendix: draft reconciliation, owner decisions

---

## A. Diagnosis of the current game

### A.1 What the code really does (confirmed in `tbg-core`)

| Topic | Implemented behavior | Where |
|-------|----------------------|-------|
| Tick, units | 20 Hz; positions in milli-tiles on a 16-tile lane; spawn 0.5 tile in front of the own Stronghold | `content.rs`, `intent.rs::spawn` |
| Movement | One axis. **No collision**: allies pass through allies (BR-B03). A unit walks until its target is within range, then stops | `combat.rs::advance_unit` |
| "Blocking" | Implicit for every unit: nobody walks past an enemy, because the nearest enemy in range stops it. There is no push, jump, teleport or knock-back in the game | `targeting.rs::enemy_in_range` |
| Targeting | Nearest enemy by \|dx\| (ties: lowest id); the enemy Stronghold only when strictly nearer; Guard: any enemy Rampart in range first (BR-U09). Re-chosen at every attack start | `targeting.rs` |
| Attacks | Cooldown 0 → 6-tick windup (`WINDUP`) → resolves on the target chosen at windup start if it is still alive, without a range re-check → cooldown `interval − windup`. Root cancels the windup | `combat.rs` |
| Support | Heal > 0: most damaged ally (lowest HP %) in range. Damage 0 and nothing to heal: holds when an enemy is in range. A hit aimed at an own-seat unit **is treated as a heal** | `targeting.rs::engage`, `hits.rs::resolve_hit` |
| Splash | Same damage to every other enemy within radius of the target's x, plus the enemy Stronghold if inside; no friendly fire; on-hit status and situational bonuses only on the primary | `hits.rs::splash` |
| Status | **One slot per unit**: `Slowed{bp}` (move *and* attack speed) or `Rooted` (no move, no attack). Root replaces anything; a stronger slow replaces, a weaker one refreshes; slow never replaces root | `hits.rs::apply_on_hit` |
| Damage pipeline | Augment + Arcana damage reduction (cap 20 %, BR-X05) → Second Wind → HP; lifesteal on primary hits on enemy conventional units (cap 12 %, BR-X06) | `hits.rs::damage` |
| Deaths and rewards | Dead units removed at the end of the tick; **every** conventional death is an eligible kill for the other seat (Bounty, Stone Mend), whatever caused it | `combat.rs::remove_dead` |
| Summons | Only Beetling. `is_summon()` is decided **by kind**; no Synergy, Arcana, Bounty, Evolution; lifetime 10 s; limits per Nester and per seat, hard bound 64 | `summon.rs`, `types.rs` |
| Stats | Recomputed every tick from kind, tier, branch, Synergy, Arcanas and Augments; the unit stores no other stat state | `ruleset.rs::stats_with`, `state.rs::stats_of` |
| Synergy | 2/4 living units: Humans +20/40 % HP, Undead +20/40 % attack speed, Machines +5/20 % damage, Plants 0.75/2 % HP per second | `synergy.rs` |
| Caps | Attack speed Σ ≤ +100 % (`MIN_INTERVAL` 5 ticks), move Σ ≤ +50 %, damage reduction ≤ 20 %, lifesteal ≤ 12 %, status duration ≤ +30 % | BR-X02…X07 |
| Economy | Start 5 Energy, 17 sub-energy per tick (≈ 0.57/s), Interest ×1.25 per full 5 held up to ×2.25, no cap, Augment round every 60 s of combat (battle paused) | `sim.rs`, D-41, D-43 |
| Shop | Five distinct offers; every purchase redraws all five uniformly from the whole roster, the bought unit included (D-45); no rotation, no reroll | `shop.rs::draw_offers`, BR-S04 |
| Determinism | Integer state, units sorted by id, PCG32 streams per purpose; `MatchState::hash()` hashes the **whole serialized state**; replays keyed by the selection clock | `sim.rs`, `replay.rs` |
| Protocol | v12; `UnitView.status` is a `u8` (0/1/2); `Event::Hit` has no source | `protocol.rs`, `view.rs` |

### A.2 Mechanics: implemented, documented-only, to build, undefined

| Mechanic | State |
|----------|-------|
| Single-slot slow/root, splash, heal, Guard, summons with lifetime, situational damage (Augments), heal cut, lifesteal, damage reduction, Second Wind | **Implemented** |
| Stronghold kinds, Fortifications, Shop Reserve, Bounty kinds | **Documented, not active** (§10.3–10.5) |
| Units referenced by the drafts: Grave Needle, Soulreaver, Duelist, Ramguard | **Do not exist** anywhere in code or docs; ignored by this plan |
| Stackable effects (poison, fish), per-unit growth, friendly fire, confusion / attack redirection, cleanse and ward, allied-movement rules (escort, formation), plates, self-sacrifice, HP transfer, random summons | **To build** (§F) |
| Who gets credit for a friendly-fire or self-inflicted death; poison/DoT vs Strongholds; whether a summon can be a roster kind | **Undefined today**; decided in §C/§F |

### A.3 Measured baseline (canon, no new unit)

`cargo build --release -p tbg-balance` then
`./target/release/tbg-balance gate --pairs 256` (seeds 1 000 000…, 88 064
games in 40 s on 16 threads, 2 184 games/s) and
`./target/release/tbg-balance duels`:

| Row | Measured | Target | |
|-----|----------|--------|--|
| Seat A, bot vs bot | 50.2 % [49.0–51.4] | 48.5–51.5 | pass |
| Median duration | 109 s | 150–240 s | **fail** |
| Faction majority Humans / Undead / Machines / Plants | 34.2 / 50.8 / **77.8** / 39.6 % | 45–55 % | **3 fail** |
| Strongest strategic style | `saver` 84.1 % | ≤ 60 % | **fail** |
| Nash support / largest weight | 1 policy / 100 % (`saver`) | ≥ 3 / ≤ 50 % | **fail** |
| `favor-siegewalker` vs bot | 41.8 % | median ± 10 pp (12.1–32.1) | **fail** |
| Duel base forms oppressive everywhere | Bone Brute, Siege Walker, Nester | 0 | **fail** |
| Duel base forms weak everywhere | Sawbot, Rootcaller | 0 | **fail** |
| Evolutions never worth their Energy | 47.5 % of forms | ≤ 25 % | **fail** |
| Comebacks | 23.0 % | ≥ 20 % | pass |

**12 of 39 rows fail before any new unit exists.** (Measured before
D-45 changed the Shop; P42 re-measures the baseline on current `main`.) The bot forgoes every
Augment round, so none of this measures Augments. Consequences for the
plan: (1) new units are judged *relative to this baseline* plus absolute
rows that the baseline already passes; (2) the old outliers are part of
the balance work (§E.6), because a new unit tuned against Bone Brute or
Siege Walker would be tuned against an error; (3) the banking dominance
(`saver`, D-41/D-43) is an economy problem no unit can fix and stays an
owner decision.

### A.4 Integration risks in one paragraph

The engine is small, integer and deterministic, and the lab can play
~2 000 full matches per second, so every new mechanic can be measured
cheaply. The risks are elsewhere: the single status slot cannot hold
stacks; "a hit on an ally is a heal" makes any redirection dangerous;
summons are recognised by kind; there is no event source for attribution;
`hash()` changes whenever `Unit` gains a field; the Shop pool is a
`const`, so units cannot be staged; and open-ended matches (D-41) mean a
stalling combination has no clock to end it.

---

## B. Final cast

### B.1 Which drafts are in this plan

| # | Draft file | Unit | Status in this plan |
|---|-----------|------|---------------------|
| 1 | `1` | Rotcap | in |
| 2 | `2` | Farwatch | in |
| 3 | `3` | Leechroot | in |
| 4 | `4` | Bombard | in |
| 5 | `5` | Old Finn | in |
| 6 | `6` | Drumguard | in |
| 7 | `7` | Longaxe | in |
| 8 | `8` | Oilpress | in |
| 9 | `9` | Antidote Brewer | in |
| 10 | `10` | Puppetmaster | in |
| 11 | `11` | Smuggler | in |
| 12 | `12` | Cargo Crab | in (owner decision O-R1, 2026-10-29) |
| — | none | Shatterback | in, designed from the one-line brief (no draft exists) |
| 13 | `13` | Blood Furnace | in (owner addition) |
| 14 | `14` | Bloodweaver | in (owner addition) |

The brief's twelve, Cargo Crab (owner decision O-R1: Shatterback *and*
Cargo Crab) and the two owner additions make **fifteen** new units; with
today's ten the Shop pool becomes **25**.

### B.2 Faction and class balance after the expansion

| Faction | Today | New | Total | Archetypes covered |
|---------|-------|-----|-------|--------------------|
| Humans | Shieldbearer, Archer | Farwatch, Old Finn, Drumguard, Longaxe, Smuggler | 7 | all five |
| Undead | Bone Brute, Hexer | Puppetmaster, Blood Furnace, Bloodweaver | 5 | Tank, DPS, Support, Control (no Ranged) |
| Machines | Sawbot, Siege Walker, Rampart | Bombard, Oilpress, Cargo Crab | 6 | Tank, DPS, Ranged (no Support, Control) |
| Plants | Bloomkeeper, Rootcaller, Nester | Rotcap, Leechroot, Antidote Brewer, Shatterback | 7 | all five |

| Archetype | Units | Count |
|-----------|-------|-------|
| Tank | Shieldbearer, Bone Brute, Rampart, Oilpress, Shatterback, Cargo Crab | 6 |
| DPS | Sawbot, Leechroot, Longaxe, Blood Furnace | 4 |
| Ranged | Archer, Siege Walker, Rotcap, Farwatch, Bombard | 5 |
| Support | Bloomkeeper, Nester, Drumguard, Antidote Brewer, Smuggler, Bloodweaver | 6 |
| Control | Hexer, Rootcaller, Old Finn, Puppetmaster | 4 |

**Is 7/5/6/7 a competitive problem?** Model: the probability that at
least one offer of a faction is on screen, five distinct cards from 25, is
83.9 % for a 7-unit faction, 78.1 % for 6 and 70.8 % for 5 (today, five of
ten: 91.7 % for 3 kinds vs 77.8 % for 2). The gap is smaller than today's and
Synergy needs living units, not distinct kinds, so duplicates count.
Without the two Undead additions, Undead would have had 3 kinds and
55.8 %; the owner's additions closed that gap, so **no faction change is
proposed.** The lab's faction-majority rows (§I) check it with evidence.

**Shop dilution (a real, global consequence).** Since D-45 every
purchase redraws five distinct units from the whole roster, so with 25
kinds a given unit is in each new set with probability 5/25 = 20 % (it
was 50 % with 10 kinds): a player who wants one specific unit waits
about 5 purchases on average, and a specific pair is in the same set
only 3.3 % of the time. Planned two-unit combinations stay possible but
slower; adapting to offers stays the skill, which also *lowers* combo
risk. It is measured (§I, "purchases until first offer"); if it removes
too much agency, the answer is an owner decision on the Shop, not a
new system in this plan.

**Roster health target.** GAME-DESIGN §6 asks every unit to have ≥ 5 %
of purchases; with 25 units a uniform share is 4.0 %, so the target is
impossible by construction. Proposed replacement: ≥ 2.5 % (about 60 % of
the uniform share at 25). Gate thresholds are product decisions (AGENTS.md);
the owner approved 2.5 % on 2026-10-29 (O-R2).

### B.3 Summary table (Proposed starting values)

Speeds in tiles/s (× 50 = milli-tiles per tick), times in seconds
(× 20 = ticks), cost in Energy (× 600 = sub-energy).

| Unit | Faction | Class | Cost | HP | Dmg | Every | Range | Speed | Identity in one line |
|------|---------|-------|------|----|-----|-------|-------|-------|----------------------|
| Rotcap | Plants | Ranged | 3 | 200 | 12 | 1.5 | 3.5 | 0.9 | Each hit adds a poison stack (6/s, 10 s, ≤ 6 per victim) that outlives it |
| Farwatch | Humans | Ranged | 3 | 150 | 70 | 4.0 | 12.0 | 0.9 | Shoots units across most of the lane; never targets Strongholds |
| Leechroot | Plants | DPS | 3 | 220 | 10 | 1.6 | 1.2 | 0.9 | Damage dealt to enemy units becomes permanent stacks, no maximum; lost on death |
| Bombard | Machines | Ranged | 5 | 1 000 | 24 | 2.2 | 4.0 | 0.5 | Blast radius 1.0 hits every unit, both sides and itself |
| Old Finn | Humans | Control | 3 | 240 | 8 | 2.0 | 3.5 | 0.9 | Fish: −15 % move each, ≤ 3, 5 s; never slows attacks |
| Drumguard | Humans | Support | 3 | 260 | — | — | 2.0 | 1.0 | Bond to the nearest ally: +25 % move, −15 % damage taken |
| Longaxe | Humans | DPS | 4 | 360 | 100 | 2.4 | 1.6 | 0.7 | Heavy melee that strikes over its own front line |
| Oilpress | Machines | Tank | 7 | 3 600 | 25 | 2.5 | 0.8 | 0.3 | Most HP in the game; while engaged, own units cannot walk past it |
| Antidote Brewer | Plants | Support | 3 | 260 | — | 4.0 | 3.0 | 0.9 | Cleanses the most afflicted ally and wards it 1 s |
| Puppetmaster | Undead | Control | 4 | 260 | — | 8.0 | 4.0 | 0.9 | The hardest-hitting enemy in range strikes its own ally once |
| Smuggler | Humans | Support | 6 | 320 | — | 20.0 | 5.0 | 0.6 | Every 20 s a crate opens into a random unit — or nothing |
| Shatterback | Plants | Tank | 4 | 560 + 4 × 100 plates | 30 | 1.5 | 0.8 | 0.55 | Each broken plate: permanently faster moves and attacks |
| Blood Furnace | Undead | DPS | 4 | 440 | 36 | 1.8 | 0.8 | 0.9 | Each hit burns 6 % max HP; damage grows up to ×4 as HP falls |
| Bloodweaver | Undead | Support | 3 | 300 | — | 5.0 | 3.0 | 0.9 | Moves up to 120 HP from the healthiest ally to the most wounded |
| Cargo Crab | Machines | Tank | 5 | 1 400 | 20 | 1.5 | 0.8 | 0.5 | Carries one non-Tank ally that single-target attacks cannot reach |

"—" in Dmg: the unit has no basic attack; "Every" is then its action
interval. Cost 3 is kept as the minimum (as today): a cheaper unit would
make redrawing the Shop for a combo cheaper (§B.2).

---

## C. Unit sheets

Conventions used by every sheet:

- **Stack list**: a fixed array of `(ticks_left, value)` per victim;
  every source shares the victim's cap; at the cap a new application
  replaces the entry with the least time left. Deterministic, no
  allocation.
- **No basic attack** (Drumguard, Antidote Brewer, Puppetmaster,
  Smuggler, Bloodweaver): the unit walks until an enemy is within its
  range, then holds; it is never assigned an enemy attack and never
  emits `Hit{amount: 0}`.
- **Action units** (Antidote Brewer, Puppetmaster, Bloodweaver) use the
  normal cooldown and windup, so the client gets `AttackStart` for free;
  when no valid target exists they keep the cooldown ready (nothing is
  wasted). Undead/Machines attack-speed Synergy and Augments shorten
  their interval like any attack.
- **Summon** below always means BR-U08: no Energy, Bounty, Synergy,
  Arcana, Evolution; Augments only if they name it.
- Every value is Proposed; every lab knob is a `Ruleset` field (D-33).

### C.1 Rotcap — Plants · Ranged — *Toxic Overload*

**Role.** Anti-Tank attrition that keeps working after it dies.
**Differs from** Archer (burst-free, ramps, diminishing with copies) and
Siege Walker (single target, no splash).

**Ability.** Every hit on an enemy *unit* deals 12 damage and adds one
poison stack: 6 damage per second for 10 s (10 pulses, aligned to the
existing 20-tick period so a stack always pulses exactly 10 times).
Each victim holds at most 6 stacks, shared by every Rotcap of that
side. The stack's damage is fixed when applied (Rotcap's static damage
bonus: Evolution, Synergy, Arcana, Augments), so later buffs do not
rewrite it.

| Case | Rule |
|------|------|
| Rotcap dies | Stacks stay on the victim until they expire (the identity) |
| Victim dies | Stacks vanish; **no spreading** (the draft's open question): spreading would make a swarm wipe self-propagating, and branch B already gives spread under the player's control |
| Strongholds | Rotcap hits them for 12; no poison (on-hit effects only reach units, as today) |
| Summons | Poisoned normally |
| Pulse damage | Goes through `damage()`: damage reduction and Second Wind apply; it is **not** a hit: no on-hit effect, lifesteal, heal cut, Leechroot essence, Executioner or Bounty change |
| Credit | A poison death is a normal death (opponent's eligible kill) |
| Events | `StatusApplied`, `PoisonTick{unit, amount}` (not `Hit`) |

**Numbers (model).** One Rotcap on one target: first 10 s deal 84 direct
+ 225 poison ≈ 31 DPS; steady state 8 + 36 = 44 DPS (≈ Archer's 45).
Two Rotcaps on one target: 16 + 36 = 52 DPS, not 88 — the shared cap is
the anti-spam rule. Tail after death at full stacks ≈ 180–225 damage.

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Concentrated | Potent Toxin: stack 6 → 8/s | Deadly Venom: stack 10/s | kill one big target (steady 68 DPS) |
| B Contamination | Spore Split: every 3rd attack also stacks the 2nd-nearest enemy in range | Toxic Rain: every 2nd attack | pressure many targets, weaker on one |

**Balance.** Best with: anything that keeps targets in range for long
(Oilpress, Old Finn, Hexer). Counters:
Antidote Brewer, burst and dive (it is 200 HP), swarms (Beetlings and
cargo die before stacks matter), heals. Snowball: none (capped per
victim). Useless when: the enemy is all summons or all low-HP units.
Domination risk: low to medium (B branch vs clumped fronts — watched).

**Implementation: low–medium.** Needs the stack list (shared with Old
Finn), a pulse pass every 20 ticks, `echo` targets (shared with Old Finn
B). Tests: stacks persist after source death; cap shared across sources;
replacement of least-remaining; exact 10 pulses; no essence/lifesteal
from pulses; Second Wind; poison kill credit.

### C.2 Farwatch — Humans · Ranged — *Horizon Shot*

**Role.** Constant long-range chip and finisher from a safe spot.

**Ability.** Range 12 tiles (Siege Walker 6, Archer 4.5; the draft's
85–90 % = 13.6–14.4 is a dose-response point, §E.4). Targets the
nearest enemy **unit** (Guard still applies); **never targets a
Stronghold**: when the enemy Stronghold is the only thing in range it
holds. With no enemy units it therefore stands about 4 tiles from its
own Stronghold, inside its own Stronghold's cover (6) — exactly the
draft's "stays at home" picture, without a special position rule.

| Case | Rule |
|------|------|
| Enemy Rampart within 12 tiles | Guard forces Farwatch to shoot the Rampart (a natural counter to Farwatch spam) |
| Preparation | Normal 0.3 s windup; it never moves while it has a target (true of every unit) |
| Splash, statuses | None in the base form |
| Role invariant | GAME-DESIGN's "base range ≤ Stronghold range" exists to stop out-ranging bases; it is rewritten as "≤ Stronghold range **unless the unit never targets Strongholds**" and `knobs::valid` follows |

**Numbers.** 70 damage / 4 s = 17.5 DPS, 39 % of Archer's. Model of
compensation: it engages from spawn-to-contact (≈ 12 extra seconds per
enemy wave at 1 tile/s) and stands 7–8 tiles behind the front, so it is
usually the last unit hit.

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Sharpshooter | Steady Hands: interval 4.0 → 3.2 s | Finishing Shot: +100 % damage vs units below 35 % HP | finish wounded units |
| B Trapper | Crippling Arrow: hits slow 25 % for 1.5 s (existing status) | Deep Wound: hits also cut healing received by 50 % for 3 s | delay and anti-heal |

**Balance.** Best with: Oilpress, Old Finn (longer approach), any front
line. Counters: Rampart (Guard), fast dives (Sawbot), tanks that simply
walk through 17.5 DPS, Puppetmaster (a confused Farwatch shoots its own
ally across the map). Spam risk: high on paper (10 copies = 175 DPS from
home; model: 7 Bone Brutes still win the 30-Energy trade) → adversarial
policy `favor-farwatch` and a `farwatch-wall` comp in the gate.

**Implementation: low.** A `never_targets_strongholds` ability flag in
targeting; Finishing Shot reuses the Executioner condition; Deep Wound
reuses the heal-cut fields. Tests: holds instead of hitting a Stronghold;
Guard; range at both seats (mirrored x); finisher threshold integer math.

### C.3 Leechroot — Plants · DPS — *Endless Hunger*

**Role.** A protected carry that becomes the biggest threat on the field
if left alive.

**Ability.** HP removed by its hits from **enemy conventional units**
fills Essence (not Strongholds, not summons: no farming free units). The
n-th stack costs `40 + 2 × n` Essence; each stack gives +1 damage and +2 %
attack speed. Stacks never cap; the attack-speed part joins every other
source under BR-X02 (+100 %, reached at 50 stacks), after which growth
continues through damage only. All stacks are lost on death.

Why the rising stack cost: with a flat cost the feedback
`dE/dt = DPS(E)` is exponential (damage doubles every ~35 s once attack
speed is capped — a 4-minute Leechroot would end any match). With
`40 + 2n`, stacks grow **linearly in time** after the cap (≈ 0.6 stack/s),
so the threat is unbounded but predictable: the counter-player always
has time to respond.

| Engaged time (model, one target, no buffs) | Stacks | Damage / interval | DPS |
|-------------------------------------------|--------|-------------------|-----|
| 0 s | 0 | 10 / 1.6 s | 6 |
| ≈ 80 s | 25 | 35 / 1.07 s | 33 |
| ≈ 135 s | 50 | 60 / 0.8 s | 75 |
| ≈ 300 s | ≈ 150 | 160 / 0.8 s | 200 |
| ≈ 600 s | ≈ 340 | 350 / 0.8 s | 440 |

Range 1.2 (not "medium" 2.5 as drafted): it must stand close behind its
front, inside the radius of splash aimed at that front, so area damage is
always a counter. At 2.5 it would sit behind any tank out of every
splash and be untouchable (the Leechroot + Oilpress problem, §D).

| Case | Rule |
|------|------|
| Target switch | Stacks kept |
| Betrayed by Puppetmaster | The hit on its ally grants nothing |
| Cargo Leechroot (Smuggler) | Grows normally (its ability is its own) |
| Events | none new; the client reads a 0–7 growth stage from the view |

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Insatiable | Faster Feeding: stack cost `30 + 2n` | Endless Growth: +3 % attack speed per stack (cap at 34) | grow faster, still fragile |
| B Parasite | Life Drain: lifesteal 6 % | Blood Hunger: lifesteal 12 % (BR-X06 cap; War Leech adds nothing beyond it) | survive to keep growing |

**Balance.** Best with: Oilpress, Drumguard, Bloodweaver, healers,
Cargo Crab (safe from focus, exposed to every splash on the crab).
Counters: any splash (Siege Walker, Bombard, Reaper, Champion, Arrow
Rain, Hexer Plague, Longaxe B, Rotcap B), burst, Puppetmaster on the unit
in front of it, rushing the Stronghold before it matures. Snowball:
**yes, by design**, bounded to linear. Useless when: it dies early or
faces only summons. Domination risk: **the highest in the cast** (§H #1).

**Implementation: medium.** Two fields (`stacks: u16`, `progress: u32`)
and one stats hook in `stats_of`. Tests: stack cost loop bounded; no
essence from Strongholds, summons, betrayal, poison pulses; BR-X02 cap
shared with Synergy, Arcana and Augment attack speed; reset on death.

### C.4 Bombard — Machines · Ranged — *Friendly Fire*

**Role.** Armored anti-swarm artillery whose risk is your own formation.
**Differs from** Siege Walker: shorter range (4 vs 6), tiny damage per
target (24 vs 85), huge HP (1 000 vs 700), and friendly fire.

**Ability.** Targets enemies normally. The blast, centered on the
primary target's x with radius 1.0, deals 24 to **every unit of both
sides inside it, the Bombard included**, and to the enemy Stronghold if
inside (as splash does today); never to the own Stronghold. Situational
bonuses and statuses apply to the primary only. Same damage to allies,
no hidden reduction (draft rule); their own damage reduction applies.

| Case | Rule |
|------|------|
| Melee geometry | A friendly melee unit in contact (range ≤ 1.0) is inside the blast; Longaxe (1.6) and ranged units are not |
| Enemy reaches the Bombard | Its blasts hit itself: funny, survivable (1 000 HP) |
| Death credit | Unchanged code: an ally killed by your blast is the **opponent's** eligible kill (BR-E08 clarified: "whatever the source"). Misuse feeds the enemy's Plunder — a fair, zero-code punishment |
| Your own rewards | None: nothing in the game rewards killing your own units |
| Betrayed by Puppetmaster | One single-target bomb on its ally, no blast (§C.10) |

**Numbers (model).** Single target 10.9 DPS; vs a clump of 4 enemies
43.6 DPS; with 2 own melee units in the blast the net is 21.8. The
meaningful metric is the friendly-fire ratio (§E.3).

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Destruction | Bigger Blast: radius 1.5 | Heavy Ordnance: radius 2.0, +25 % damage | wipe swarms, demands an all-ranged army |
| B Iron Fortress | Reinforced Hull: +50 % HP | Relentless Shelling: +43 % attack speed (2.2 → 1.54 s) | a durable anchor that fits melee armies |

No branch removes friendly fire (draft rule kept).

**Balance.** Best with: Archers, Farwatch, Longaxe (reach), Old Finn.
Counters: dives that reach it (it then bombs itself), Puppetmaster is
weak here (by rule). Bad with: Sawbot, Shieldbearer fronts, own Nester
(Beetlings fight in melee). Snowball: none. Domination risk: medium vs
swarm-heavy metas (Nester, Smuggler cargo).

**Implementation: low–medium.** A `friendly` flag on the splash loop
(include own seat and self, exclude own Stronghold). Tests: self-hit when
in radius; allies take full damage; own Stronghold untouched; kill
credit; DR on allies.

### C.5 Old Finn — Humans · Control — *Fish Barrage*

**Role.** Progressive movement control that never stops attacks.
**Differs from** Hexer (one strong slow on move *and* attack) and
Rootcaller (full stop, short).

**Ability.** Each hit (8 damage) sticks a fish on the target: −15 %
**movement only**, max 3 fish per victim (all Finns share it), 5 s each
(stack list). Combined movement factor
`(1 − status slow) × (1 − Σ fish)`: multiplicative, so no combination
reaches zero and no global cap is needed (draft's concern); Hexer
Withering 60 % plus three Big Fish 60 % still leaves 16 % speed. Root
still stops completely.

| Case | Rule |
|------|------|
| Finn dies | Fish stay until they expire |
| Strongholds | Hit for 8, no fish |
| Ranged victims | Barely affected — they already stand still while shooting (counter by design) |
| Events | `StatusApplied`; the view carries the fish count (max 3 drawn) |

**Numbers (model).** One Finn sustains 2.5 fish on one target (37.5 %
slow); a melee unit crossing a 4-tile kill zone spends ×1.6 the time
under fire. Two Finns reach the 45 % cap.

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Big Catch | Big Fish: 20 % per fish (60 % at cap) | Giant Catfish: each hit adds 2 fish | lock one dangerous unit |
| B Fish Market | Double Catch: every 3rd attack also fishes the 2nd-nearest enemy in range | Fish Shower: every 3rd attack fishes 2 more enemies | slow a whole wave a little |

**Balance.** Best with: Rotcap, Farwatch, Longaxe, Oilpress, Blood
Furnace. Counters: ranged armies, Antidote Brewer, dives. Useless when:
the enemy is all ranged. Domination risk: low.

**Implementation: low** once the stack list exists (shared with Rotcap).

### C.6 Drumguard — Humans · Support — *Battle March*

**Role.** Escort buff for one ally; position decides who.

**Ability.** No attack. Each tick, bonds to the nearest allied
**conventional** unit within 2.0 tiles (not another Drumguard; ties
lowest id) and keeps the bond until it is dead, out of range, or another
candidate is at least 0.5 tile closer (hysteresis: the draft's
"stabilization", deterministic). The bonded ally gets +25 % movement and
−15 % damage taken. Several Drumguards on one ally: the best value of
each bonus applies, no stacking. **Drumguard never walks past its bonded
ally** (escort) and holds when an enemy is within 3.0 tiles. The bond
ends at once on Drumguard's death.

| Case | Rule |
|------|------|
| Caps | Move joins BR-X03 (+50 %), reduction joins BR-X05 (20 %) — e.g. with Shieldwall the ally is capped at 20 % |
| Fast allies | A bonded Sawbot (1.625 t/s) outruns a 1.0 t/s Drumguard, leaves the 2-tile range and loses the bond: readable, no special rule |
| Root | Not removed (draft) |
| Visual | A thin line to the bonded ally (state in the view, no event) |

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A War March | +35 % move | +45 % move (cap applies) | deliver a melee carry fast |
| B Iron Rhythm | −18 % damage taken | Unbreakable Rhythm: −18 % and the ally ignores slows and fish (not root) | keep one ally standing under control |

B2 adds a capability instead of a number above the 20 % cap, so it is
not wasted in stacked-reduction builds.

**Balance.** Best with: Longaxe, Leechroot, Blood Furnace, Oilpress
(+25 % on a 0.3 t/s wall matters). Counters: kill the drummer (260 HP);
AoE. Useless when: alone. Domination risk: low; watched in
`leech-fortress` (§D).

**Implementation: medium.** A bond pass before movement writing
`march_move_bp`/`march_dr_bp` on allies; escort clamp in movement.

### C.7 Longaxe — Humans · DPS — *Heavy Reach*

**Role.** Second-rank heavy melee.

**Ability (stats only).** Range 1.6 vs melee 0.6–1.0. Engine consequence
the drafts did not mention: with nearest-enemy targeting, Longaxe stops
1.6 tiles from the enemy while its own tank stands at 0.8, so it strikes
**over its own front line** and is rarely anyone's nearest target. That
is its real identity, so its HP stays below any Tank (draft rule) and its
speed is low.

| Numbers | 100 / 2.4 s = 41.7 DPS; overkill vs Beetlings (90 HP) and cargo rats wastes ~10–40 % per swing |

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Executioner | Deep Cut: +30 % damage | Crushing Edge: +30 % more vs units above 60 % HP | open tanks |
| B Polearm Master | Sweeping Arc: splash 0.8 at 40 % damage (enemies only) | Line Breaker: splash 1.2 at 60 % | clumped lines |

**Balance.** Best with: any Tank, Drumguard, Oilpress formation, Bombard
(outside the blast). Counters: swarms, Puppetmaster (its 100+ hit on its
own ally is the biggest swing in the game), ranged focus, control.
Domination risk: medium (second-rank immunity) → duel and favor rows.

**Implementation: low.** New form fields: splash damage share
(`splash_bp`) and a "target above X % HP" condition.

### C.8 Oilpress — Machines · Tank — *Total Blockade*

**Role.** The wall, and the formation anchor.

**Engine reality.** Every unit already stops every enemy (§A.1), so
"enemies cannot pass" would add nothing. What TBG lacks is the other half
of a wall: **your own back line runs ahead of slow tanks and dies first**
(allies pass through allies). Oilpress keeps the draft's fantasy and makes
it real in a single lane:

**Ability.** (1) Like every unit, enemies stop at it — it is simply the
toughest thing to stop at. (2) **Formation:** while an enemy unit is
within 4.0 tiles in front of a living Oilpress, allied units behind it
cannot move past it; they stop 0.2 tile behind. When no enemy is near,
allies pass freely, so a late Oilpress bought at home never stalls your
reinforcements.

| Case | Rule |
|------|------|
| Allies already ahead | Unaffected |
| Short melee allies held behind | May be out of range of the enemy (0.8 + 0.2 > 0.8) and idle — the composition cost of a wall (reach units and ranged shine) |
| Several Oilpresses | The nearest engaged one ahead of each ally applies |
| Enemy control on Oilpress | Normal (A2 removes it) |
| Strongholds | Never affected; no collision is created |

**Numbers (model).** 3 600 HP for 7 Energy = 514 HP/Energy (Rampart 250,
Bone Brute 325). Holds ~24 s against a 150-DPS army, ~12 s against 300;
3 Archers behind it convert those 24 s into ~3 200 damage. 10 DPS: it
almost never kills.

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Fortress Engine | Reinforced Hull: +35 % HP | Immovable Mass: +35 % HP, cannot be slowed, rooted or confused | the longest hold |
| B Tar Bastion | Oil Leak: hits slow 30 % for 2 s (existing status) | Black Mire: hits splash 1.0 (enemies only) and the slow reaches every unit splashed | turn the wall into a slowing zone |

**Balance.** Best with: Longaxe, Leechroot, Archer, Farwatch, Rotcap,
Bloodweaver (HP reservoir). Counters: Rotcap (poison on a huge HP pool),
Leechroot (feeds on it), Longaxe A2, Giant Slayer, raw DPS. Snowball:
no. Stall risk: medium with heavy healing (§H #4). Its clumped queue is
the best possible Bombard/Siege Walker target (natural counter).

**Implementation: medium.** One movement clamp; the "engaged" check is
one pass per tick per Oilpress. Touches the movement code every unit
uses → behavior goldens must stay identical for armies without Oilpress.

### C.9 Antidote Brewer — Plants · Support — *Purifying Brew*

**Role.** Anti-control and anti-poison specialist; deliberately weak
when the enemy applies nothing.

**Ability.** Every 4 s, throws at the ally in range (conventional or
summon) with the highest affliction score — Rooted or Confused 4,
+1 per poison stack, Slowed 2, +1 per fish; ties nearest, then lowest
id. The brew removes the slow/root status, Confused, up to 3 poison
stacks (strongest first) and all fish, then **wards** the ally for 1.0 s
(new slows, roots, confusion, poison stacks and fish are ignored; damage
is not). Nobody afflicted: it waits ready.

Why the ward (not in the draft): against a *live* applicator, removing
one stack every 5 s (the draft) is erased by the next hit 1.5 s later —
it would cleanse nothing in steady state. The ward makes the brew worth
something without nullifying the applicator.

| Steady state (model) | Effect |
|----------------------|--------|
| vs 1 Rotcap on one ally | average stacks 6 → ~4.4 (−27 % poison) |
| vs 2 Rotcaps | ~−15 % ("reduce, never nullify", draft) |
| vs Puppetmaster | Prevents the betrayal if it lands before the next attack, or blocks the cast during the ward |
| vs Rootcaller | Frees and briefly protects one ally |

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Master Antidote | Concentrated Cure: removes every poison stack; ward 2.0 s | Perfect Remedy: ward 3.5 s (≈ 88 % uptime on one ally) | keep one VIP clean |
| B Rapid Brewer | Quick Mixing: every 3.0 s | Double Flask: also cleanses the next most afflicted ally (no ward on it) | many allies, partially |

**Balance.** Overlap: the Vitality supreme *Purification* removes slows
through heals — keep both; Purification needs a healer and a 2-point
supreme. Counters: kill it (260 HP); armies without control. Useless vs
pure damage (accepted, cheap at 3). Domination risk: low.

**Implementation: medium** (needs §C.1/C.5 effects to exist).

### C.10 Puppetmaster — Undead · Control — *Friendly Betrayal*

**Role.** Punishes armies built on heavy hits.

**Ability.** Every 8 s, chooses among enemy conventional units in range
4.0 that have a basic attack (damage > 0, no heal), are not Confused and
not immune: **the highest static hit damage** (ties nearest, lowest id)
— the draft left the choice open; this makes the spell aim where it
matters. The target becomes **Confused** for up to 5 s: the next attack
it *starts* while Confused targets its nearest own ally within its own
range (not itself). If no such ally exists the attack is normal and the
confusion waits until it expires. A betrayal hit deals the attacker's
normal hit damage (static modifiers; Blood Furnace's furnace multiplier
and sacrifice included because they are its basic attack) through the
ally's damage reduction, with **no** splash, blast, on-hit status,
lifesteal, Essence, situational or Arcana conditional bonus. Then the
confusion ends and the unit is immune for 12 s.

| Case | Rule |
|------|------|
| Attack already winding up | Resolves normally; the next one is redirected (decided at windup start, so `AttackStart` already shows the betrayal) |
| Target or ally dies before resolution | Same as any attack: no hit |
| Strongholds | Never confused, never betrayal targets |
| Heals | Never redirected (healers are not eligible) |
| Resolver | **Explicit betrayal path**: today a hit on an own-seat unit is a heal; a `betray` flag on the windup forces damage |
| Rewards | A unit killed by betrayal is a normal death: the Puppetmaster's side gets the eligible kill (consistent with the friendly-fire rule) |
| Several Puppetmasters | Immunity (12 s) > cooldown (8 s): they rotate targets instead of locking one |
| Events | `StatusApplied`, `Betrayal{unit, ally}` |

**Numbers (model).** Each betrayal swings ≈ 2 × H (one enemy hit denied,
one landed on them). H ≈ 75 (Bone Brute) to 130+ (evolved Longaxe,
low-HP Blood Furnace): 19–33 DPS-equivalent for 4 Energy; near zero vs
Oilpress (25) or isolated ranged units.

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Cruel Strings | Painful Betrayal: +25 % betrayal damage | Stumble: +50 %, and the betrayer is slowed 30 % for 2 s | bigger single swings |
| B Twisted Chorus | Faster Strings: every 6 s | Double Puppet: confuses two distinct units per cast | disrupt wide armies |

**Balance.** Best vs: Longaxe, Blood Furnace, Bone Brute, Butcher,
melee stacks. Counters: Antidote, spacing (lone units), low-damage
armies, killing it. Domination risk: medium (B2 vs heavy armies).

**Implementation: medium–high (correctness risk).** Confusion fields,
target selection, betrayal flag through windup and resolver. Tests:
windup in progress, no ally in range, ally dies, immunity, Guard, Farwatch
range, Blood Furnace sacrifice under betrayal, reconnect/replay hash.

### C.11 Smuggler — Humans · Support — *Mystery Cargo*

**Role.** A gamble that turns Energy into a random stream of units.

**Ability.** No attack; holds 5 tiles behind the nearest enemy. Every
20 s (first crate after a full interval, draft) drops a **Crate** at its
x: a summon kind with 120 HP, no attack, no movement, targetable (the
draft's counter-window). After 2 s the crate opens and the Smuggler
seat's own cargo RNG stream (new, hidden, BR-S05) draws:

| Outcome | p | Content (a summon of a **roster kind**: no new art) |
|---------|---|-----------------------------------------------------|
| Empty | 20 % | nothing — "the crate was empty" |
| Common | 42 % | random base form of a cost-3 unit (9 kinds) |
| Strong | 28 % | random base form of a cost 4–6 unit (13 kinds) |
| Elite | 8 % | random tier-1 form (random branch) of any eligible unit |
| Legendary | 2 % | random tier-2 form of any eligible unit |

Eligible: every Shop unit except Smuggler and Nester (summons never
summon, BR-U08). Cargo is a permanent summon: no lifetime, no Energy,
Bounty, Synergy, Arcana, Augments or Evolution; never feeds Leechroot.
A destroyed crate releases nothing. Crates already placed still open if
the Smuggler dies. The draft's five custom units (rat, shooter…) are
replaced by roster units: zero new art, instantly readable, and the
"extremely bad" outcome survives as Empty.

**Model.** Value of a cargo unit ≈ 0.8 × its Energy cost (no Synergy,
Arcana, Augments). EV per crate = 0.42 × 2.4 + 0.28 × 3.51 + 0.08 × 4.7
+ 0.02 × 7.9 ≈ **2.52 Energy**, SD ≈ 1.58 per crate; over 6 crates
(2 min) 15.1 ± 3.9. Yield 0.126 Energy/s → payback ≈ 48 s (2.4 crates),
the same order as Interest from banking 6 Energy (0.14/s) but on the
board and killable. Spam: *n* Smugglers yield *n* × 0.126/s with no
Interest-like ceiling → `smuggler-economy` adversarial policy (§E.5);
lever = interval and Empty share, never a count limit.

**Evolutions.**

| Branch | Tier 1 | Tier 2 | EV gain (model) |
|--------|--------|--------|-----------------|
| A Lucky Dealer | Better Connections: 10/40/34/12/4 % | Black Market: 6/28/38/20/8 % | +42 % per crate, lower variance at the bottom |
| B Mass Smuggling | Faster Deliveries: every 18 s | Double Shipment: every 3rd crate holds two draws | +48 % per minute, more entities |

**Balance.** Best with: time (protected back line). Counters: dives,
Farwatch (shoots crates and the Smuggler across the lane), Bombard vs
the cargo swarm, rushing. Variance is the identity: measured, never
"smoothed" by a pity rule (draft).

**Implementation: medium–high.** Needs the unit-level summon flag
(§F.2), a cargo RNG stream, `Crate` kind, `CrateOpened` event, and every
other unit to exist (cargo draws from the whole roster) → last wave.

### C.12 Shatterback — Plants · Tank — *Shed the Shell* (no draft)

Designed from the brief only: "loses defensive plates and permanently
gains speed". Owner decision O-R1 (2026-10-29) keeps it alongside Cargo Crab.

**Ability.** 560 HP plus 4 plates of 100 HP each. Damage (after damage
reduction) empties the current plate before HP. Each broken plate
permanently gives +12.5 % movement and +10 % attack speed (4 plates =
+50 % move, BR-X03, and +40 % attack speed). Plates never regenerate:
heals, regen and transfers restore HP only.

| Case | Rule |
|------|------|
| Friendly fire (Bombard) | Breaks plates like any damage: legal, slow (17 bombs for 4 plates) and it costs the effective HP — not an exploit |
| HP transfer as donor | Takes HP, not plates |
| Second Wind | Only on HP |
| Events | `PlateBroken{unit}`; plates left in the view |

**Numbers (model).** 960 effective HP, 20 DPS → 30 DPS when bare.

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Thornplates | plates 150 HP | each breaking plate deals 50 damage to enemies within 1.0 | stay a tank |
| B Unleashed | each broken plate also +8 % damage | Rampage: with no plates left, hits splash 0.8 (enemies only) | become a bruiser |

A2's shrapnel is processed from a per-tick queue after the hit, never
recursively; a chain between Shatterbacks terminates because plates only
decrease.

**Balance.** Anti-synergy worth keeping: Drumguard's damage reduction
slows plate breaking (good for A, bad for B). Oilpress formation keeps
it behind the wall where plates never break. Domination risk: low–medium.

**Implementation: low–medium.** Two fields and a branch in `damage()`.

### C.13 Blood Furnace — Undead · DPS — *Life for Damage* (owner addition)

**Role.** High-risk melee whose most dangerous hit can be its last.
**Opposite of** Leechroot: Leechroot must survive to scale; Blood Furnace
must approach death.

**Ability.** When a hit **resolves** (never at windup start: a root or a
dead target cancels it at no cost — draft rule), Blood Furnace burns 6 %
of its current max HP (rounded down, min 1), ignoring damage reduction,
shields, plates and Second Wind. Damage is then
`base × (1 + 3 × lost)` where `lost = 1 − hp_after / max_hp`, in basis
points with one truncation. If the burn takes it to 0 HP the hit still
resolves at the maximum multiplier and it dies at the end of the tick
(Deathblow; two units can kill each other on the same tick). Healing
lowers the multiplier (draft). Max HP changes (Synergy, Evolution) keep
the HP ratio (existing BR-X08), so the multiplier is unchanged by them.

| Case | Rule |
|------|------|
| vs Strongholds | The multiplier does not apply (base damage, burn still paid): heal-sustained low-HP Furnaces cannot become base erasers (draft rule 5) |
| Equilibrium with heals | Healing ≥ burn holds it at a fixed HP %; the multiplier at that % is the price of being one hit from death — self-balancing, measured in the lab |
| Its own death | A normal death (opponent's eligible kill), no lifesteal or heal after dying |
| Betrayed | Uses its multiplier and pays the burn — the biggest Puppetmaster swing |

| Hit (untouched, model) | HP after | Damage |
|------------------------|----------|--------|
| 1 | 414 | 42 |
| 8 | 232 | 87 |
| 16 | 24 | 138 |
| 17 (Deathblow) | 0 | 144 |

≈ 1 590 damage over ≈ 31 s if nobody hits it (52 DPS average).

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Infernal Core | Overheated Blood: up to ×4.5 | Dying Sun: up to ×5.5 | spike, short life |
| B Controlled Burn | Efficient Combustion: burn 5 % | Eternal Embers: burn 4 % | more hits, slower ramp |

**Balance.** Best with: Drumguard (its reduction cuts enemy damage, never
the burn), Old Finn, Cargo Crab (the draft's combo: no focus fire, the
burn still kills it). Anti-synergy: behind an engaged Oilpress its 0.8
range rarely reaches the enemy, so it wants to be bought before the wall
or without one. Counters: ranged, control (lost time is
lost value), AoE, swarms (wasted big hits), Puppetmaster. Undead Synergy
(+40 % attack speed) burns it faster too. Domination risk: medium
(heal-locked equilibrium, measured).

**Implementation: low.** Burn + multiplier at resolve.

### C.14 Bloodweaver — Undead · Support — *Blood Transfer* (owner addition)

**Role.** Redistributes the army's existing HP; creates none.

**Ability.** No attack. Every 5 s, among allied **conventional** units
within 3.0 tiles (itself included; summons excluded as donors *and*
receivers — free units can never be a reservoir), the donor is the
highest HP % and the receiver the lowest HP % below full; they are two
different units and the donor's % must be higher. Amount =
`min(120, donor.hp − 1, receiver missing HP)`: donors (itself included)
never die from a transfer (draft). The donor loses it directly (no
damage reduction, plates or Second Wind); the receiver gains it **minus
heal cut** (Grievous Wounds, Deep Wound), so heal cut turns transfers
into a net loss (draft). It is **not a heal**: no Arcana/Augment heal
bonus, no Purification; not damage: no on-hit, lifesteal, Essence. No
valid pair → it waits ready.

**Evolutions.**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Blood Sacrifice | Deep Transfer: 156 per cast | Last Offering: ×2 when the receiver is below 20 % | save the dying carry |
| B Blood Circulation | Rapid Circulation: every 4 s | Shared Blood: a second transfer between the next donor and the next receiver (four distinct units) | keep many units up |

B2 doubles throughput only in a larger army (four units): a real
alternative to A2 instead of the draft's split of the same amount.

**Balance.** Best with: Oilpress (reservoir), Leechroot, Rampart,
Plants regen on donors. Anti-synergy: Blood Furnace is always the
receiver, so it lowers its own carry's damage (draft). Counters: AoE that
drops everyone together, heal cut, killing it. Domination risk: medium
(strengthens Leechroot + Oilpress).

**Implementation: low–medium.** One selection pass and a `Transfer
{from, to, amount}` event.

### C.15 Cargo Crab — Machines · Tank — *Armored Taxi* (owner decision O-R1)

**Role.** Personal armor for one valuable unit. **Differs from**
Drumguard (a buff on whoever is nearest) and Oilpress (protects a whole
line): it protects exactly one passenger, completely from single-target
attacks and not at all from area damage.

**Ability.** When the crab is within 0.5 tile of an eligible ally —
conventional, not a Tank, not carried, not under a re-board lock — the
nearest one (ties lowest id) boards. In practice the passenger is **the
next unit you buy after the crab** (both spawn at the same point), so the
player chooses it through purchase order, with no manual control. While
carried:

| Topic | Rule |
|-------|------|
| Position | The passenger's x is the crab's x (synced before the passenger acts and after every unit has moved); it never walks on its own; its range is measured from that x |
| Movement | Only the crab moves, with Tank rules (stops at its own 0.8 range), so the passenger fights **at the front line** |
| Actions | The passenger attacks, heals, casts, bonds, drops crates and grows normally |
| Targeting | **No targeted effect can choose a carried unit**: enemy attacks, Stronghold fire, Rotcap/Old Finn echoes, Puppetmaster's betrayal target, Farwatch. An attack already winding up on it when it boards hits the crab instead |
| Area | Splash, Bombard blasts, shrapnel and poison pulses reach it normally — at distance 0 from the crab, it is inside **every** splash aimed at the crab (the draft's main counter, made automatic by geometry) |
| Statuses | Applied normally when they come from area effects; cleanse, ward, transfers and bonds work on it |
| Crab dies | The passenger dismounts 0.2 tile behind (toward its own base) with a 3 s re-board lock (no ping-pong between crabs); no free invulnerability |
| Passenger dies | The crab can board again at once |
| Several crabs | One passenger each |
| Events | `Boarded{crab, unit}`, `Dismounted{unit}`; the view packs a "carried" bit |

**Numbers (model).** 1 400 HP for 5 Energy = 280 HP/Energy (Rampart
250, Oilpress 514: the Machines "Tank leads HP per Energy" invariant
still holds through Oilpress). 13.3 DPS. Its value is the passenger's
damage kept alive: an Archer (220 HP) normally dies to the first focus;
carried, it lives until the crab's 1 400 HP are gone or an area attack
kills it.

**Evolutions (draft kept).**

| Branch | Tier 1 | Tier 2 | Choice |
|--------|--------|--------|--------|
| A Armored Carrier | Reinforced Shell: −15 % damage taken while carrying (joins the 20 % cap) | Emergency Shield: when the crab dies, the passenger gets a shield of 20 % of its max HP for 6 s | protect a carry longer |
| B Turbo Carrier | Faster Legs: +20 % move while carrying | Overdrive: on boarding, +40 % move for 4 s (15 s cooldown) | deliver the passenger fast |

The A2 shield uses the same absorb path as Shatterback's plates (one
temporary plate without bonuses), so it adds no new damage rule.

**Balance.** Best with: Archer, Rotcap, Old Finn, Antidote Brewer,
Bloodweaver, Blood Furnace (draft), Leechroot. Counters: any area damage
(Bombard, Siege Walker, Reaper, Champion, Arrow Rain, Hexer Plague,
Longaxe B, Shatterback A2), Rotcap on the crab, killing the crab first
(it is the nearest target). Bad with: Oilpress formation (the crab is
held behind the wall like any ally). Domination risk: medium, highest
with Leechroot (§D, `comp-leechroot-cargocrab`).

**Implementation: medium–high.** Carrier/passenger fields, one filter in
every target-selection function, a position sync, a windup redirect and
a dismount rule. Tests: boarding by purchase order and ties; no targeted
effect ever picks a carried unit (enemy units, Strongholds, echoes,
betrayal, Farwatch); splash and blast reach the passenger; windup
redirect at boarding; dismount position and lock; passenger death frees
the crab; two crabs never swap a passenger; replay hash.

---

## D. Interaction matrix

Verdicts: **L** legitimate (keep), **W** watch (lab policy + threshold),
**F** fixed by a rule in §C, **X** none found.

### D.1 The combinations the brief asked about

| Pair | What actually happens in this engine | Verdict |
|------|---------------------------------------|---------|
| Leechroot + Oilpress | Formation holds Leechroot 0.2 behind the wall; at range 1.2 it still reaches enemy melee, and splash on the wall (radius ≥ 0.2) reaches it. Splash, poison echo, rush and raw DPS on the wall remain counters. With Bloodweaver + Drumguard this is the strongest legal carry setup | **W** (`leech-fortress`) |
| Leechroot + Drumguard | −15 % damage, +25 % move; no growth change | L |
| Leechroot + Cargo Crab | No single-target attack can reach it, but it rides at the front line at distance 0 from the crab, so every splash on the crab hits it — more exposed to area damage than behind Oilpress, less to focus fire. Its growth then depends on the enemy having area damage | **W** (`comp-leechroot-cargocrab`) |
| Rotcap + Old Finn | Fish do not affect poison; they lengthen melee approaches under Rotcap fire | L |
| Bombard + Shatterback | 17 own bombs to strip 4 plates; costs 400 effective HP for +50 % move | L (inefficient by numbers) |
| Bombard + fragile units | Melee allies in contact take full blasts; deaths feed the enemy's Bounty | L (risk is the identity); measured by the friendly-fire ratio |
| Puppetmaster + Longaxe | 100–170 damage on the Longaxe's own ally; Antidote and ward counter | L |
| Puppetmaster + Bombard | One single-target bomb, no blast | **F** |
| Antidote + Rotcap | −27 % vs one Rotcap, −15 % vs two | L |
| Smuggler + Synergy | Cargo is a summon: no Synergy, Bounty, Arcana, Augments, Essence | **F** |
| Farwatch + Oilpress | Long chip behind a wall; Farwatch cannot hit Strongholds, so this pair alone never wins — a stall needs heals on both sides | **W** (stagnation row) |
| Oilpress + movement control | Multiplicative slows never reach 0; root uptime ≤ ~50 %; Oilpress A2 is immune itself. No permanent freeze exists | L |
| Shatterback + heals/buffs | Heals restore HP, never plates; reduction slows plate breaking | L |

### D.2 Other combinations found

| Pair | Effect | Verdict |
|------|--------|---------|
| Farwatch vs enemy Rampart | Guard forces every Farwatch within 12 tiles onto the Rampart | L (counter) |
| Confused Farwatch | Shoots its own ally up to 12 tiles away | L (spectacle, low damage) |
| Blood Furnace + Bloodweaver | Furnace is always the receiver → loses multiplier | L (anti-synergy by design) |
| Blood Furnace + heals | Holds it at a fixed HP % with a fixed multiplier | **W** (`furnace-heal` comp) |
| Oilpress formation + short melee (Sawbot, Blood Furnace, Shatterback) | Held 0.2 behind, may be out of range | L (composition cost) |
| Oilpress formation + Bombard A | The queue edge is inside a 1.5–2.0 blast | L (choose B) |
| Drumguard + Bombard | Reduction also cuts friendly fire on the bonded ally | L |
| Antidote ward vs Puppetmaster | Blocks the cast or clears the confusion | L |
| Nester + own Bombard | Beetlings fight in melee inside the blast | L |
| Smuggler cargo + Bombard (enemy) | Cargo swarms are the best blast targets | L |
| Leechroot + Puppetmaster (enemy) | The unit in front is confused into hitting Leechroot | L |
| Rotcap poison + Second Wind | One survival at 1 HP, then the next pulse | L |
| Shatterback A2 + Shatterback A2 (enemy) | Shrapnel chains terminate (plates only decrease); processed from a queue | **F** |
| Cargo Crab + Blood Furnace | The draft's combo: protected from focus while it burns; still dies to its own burn and to area damage | L |
| Cargo Crab + Archer/Rotcap | A ranged unit carried to the front keeps shooting and cannot be focused | L |
| Cargo Crab + Smuggler passenger | Crates drop at the front line, exposed to the enemy | L (bad by geometry) |
| Cargo Crab vs Bombard / Siege Walker | Every blast on the crab hits both | L (the counter) |
| Cargo Crab + Oilpress formation | The crab is an ally behind the wall and is held there | L (anti-synergy) |
| Two Cargo Crabs | A dismounted passenger has a 3 s lock, so crabs never pass it back and forth | **F** |
| Confused passenger | Betrayal cannot target carried units; the passenger's betrayal hits its nearest ally in range, usually its own crab | L |
| Undead Synergy + Puppetmaster | Faster casts (8 → 5.7 s at level 2); immunity still 12 s | L |
| Grievous Wounds / Deep Wound + Bloodweaver | Transfers become net HP losses | L |
| Bloodweaver + Plants regen donors | Regen is generated on the donor, then redistributed | L |
| Rotcap + Antidote + Bloomkeeper vs Oilpress turtle | Long matches | **W** (stagnation) |

### D.3 Systematic search answers

| Question | Answer |
|----------|--------|
| Strong but legitimate | Leechroot + Oilpress + Bloodweaver/Drumguard; Leechroot or Blood Furnace on a Cargo Crab; Rotcap B + Old Finn B vs clumps; Bombard A + all-ranged |
| Too strong to counter | None identified by model; Leechroot fortress is the candidate → adversarial policy, threshold ≤ 60 % vs pool |
| Cheap with excessive return | Smuggler (unbounded income with copies); Farwatch (cheap uptime) |
| Infinite activation loops | None: poison and transfers are not hits; betrayal triggers nothing; summons never summon; cargo cannot be Smuggler/Nester; shrapnel queue terminates |
| Matches that cannot end | Inherited: an army with no damage dealer (or both players idle) never damages a Stronghold (true today, D-41). New: Farwatch, Drumguard, Antidote, Puppetmaster, Smuggler, Bloodweaver deal no Stronghold damage; heals + walls can stall → stagnation row; no clock is added (owner rule BR-M07) |
| Dominance by copies | Farwatch, Smuggler: watched. Rotcap/Old Finn: shared caps give diminishing returns. Puppetmaster: immunity rotates targets. Drumguard: one bond each, no stacking |
| Units useless vs a style | Antidote (no control), Puppetmaster (low damage, spread units), Old Finn (all ranged), Leechroot (burst/AoE), Bombard (with own short melee) |
| Fun emergent behavior to keep | Betrayals, Deathblow trades, crates, Farwatch from home, the Oilpress queue, Shatterback speeding up |

---

## E. Balance model

### E.1 Method (reuses P16-C/P16-D, adds no harness)

1. **Static seed** (this document): a "combat index"
   `CI = sqrt(HP × DPS) / cost` places a first number. On today's roster
   it reproduces the measured duel classification: Shieldbearer 31.9,
   Archer 33.2, Rampart 31.2, Siege Walker 30.9 (+ splash and range 6),
   Sawbot 27.5 (*weak* in duels), Rootcaller 15.3 + root (*weak*),
   Bone Brute 63.7 and Nester ≈ 40 (*oppressive*). So the plain-unit
   frontier is ≈ 31–33. **CI only seeds values**: P16-D measured a 0.073
   rank correlation between static formulas and full-match value.
2. **Energy-equal duels** (`tbg-balance duels`): every form of every unit;
   units with no basic attack are exempt from duel rows and judged in
   matches only.
3. **One-sided value** (`tbg-balance value`): PVE of each new unit and
   each identity parameter with `favor-<unit>` vs `avoid-<unit>`, paired
   seeds, against the Energy shadow price λ.
4. **Full matches** (`gate`, `meta`): pool of strategic styles, favor and
   avoid policies, combination and adversarial policies (§E.5), Nash mix.
5. **Dose-response** for each identity parameter (P40 Deadeye method).
6. **Confirmation** on held-out seeds; one cycle changes ≤ 3 numbers.

### E.2 Static seed of the new units

| Unit | CI (model) | Reading |
|------|-----------|---------|
| Longaxe | 30.6 | frontier, plus second-rank protection |
| Rotcap | 31.3 steady, ramp first | frontier, diminishing with copies |
| Shatterback | 34.6 → bruiser | slightly above, loses plates permanently |
| Bombard | 20.9 single / 36.2 vs 3 | below on one target, above vs clumps |
| Oilpress | 27.1 | below; value is time (§E.3) |
| Blood Furnace | ≈ 27 effective (burn halves EHP) | below until low HP |
| Leechroot | 12.4 → 43 at 50 stacks | starts far below |
| Farwatch | 17.1 | half, paid by uptime and safety |
| Old Finn | 10.3 + fish | Control |
| Cargo Crab | 27.3 | below; value is the passenger kept alive (§E.3) |
| Drumguard, Antidote, Puppetmaster, Smuggler, Bloodweaver | n/a | utility models below |

### E.3 Models for non-DPS value

| Unit | Model | Measured as |
|------|-------|-------------|
| Leechroot | Stack cost `40 + 2n`: `ds/dt = DPS(s) / (40 + 2s)` → linear after the attack-speed cap; table §C.3 | max stacks distribution, damage per life, `leech-fortress` score |
| Smuggler | EV 2.52 Energy/crate, SD 1.58; payback ≈ 48 s | per-crate outcome tally (χ² against the table), cargo damage share, `smuggler-economy` score |
| Oilpress | Value = held time × own damage behind it; held time = 3 600 / enemy DPS | seconds the front line stays at the wall, damage dealt by its allies while engaged |
| Puppetmaster | Swing per cast ≈ 2 × H when an ally of the target is in its range | betrayal count, damage redirected, damage denied |
| Antidote | Afflicted-time removed; stacks removed; ward blocks | cleanses, blocked applications, poison damage prevented |
| Drumguard | 15 % of damage taken by the bonded ally + approach time saved | bond uptime, damage prevented |
| Bloodweaver | HP moved from units that survive to units that would have died | HP transferred, deaths prevented (receiver alive 5 s later) |
| Cargo Crab | Passenger lifetime × passenger DPS gained vs the same unit uncarried | carried time, damage dealt by passengers, passenger deaths by area damage |
| Old Finn | Extra time enemies spend in own fire zones | fish uptime, average enemy speed in own half |
| Bombard | Friendly-fire ratio = enemy damage / ally damage | per match; median ≥ 2 : 1 for `favor-bombard` is *expected*, ≤ 1 : 1 in `bombard-melee` proves the risk is real |
| Blood Furnace | Damage curve §C.13; heal equilibrium | average multiplier at hit, Deathblow count, `furnace-heal` |
| Farwatch | Damage dealt before contact; survival time | damage share pre-contact, lifetime |

### E.4 Dose-response studies (one per identity)

| Parameter | Points |
|-----------|--------|
| Farwatch range | 8 / 10 / 12 / 14 tiles |
| Leechroot stack cost step | 0 (flat) / 1 / 2 / 4 |
| Smuggler interval | 15 / 20 / 25 s |
| Oilpress HP | 2 800 / 3 600 / 4 400 |
| Cargo Crab HP | 1 000 / 1 400 / 1 800 |
| Puppetmaster cast | 6 / 8 / 10 / 14 s |
| Rotcap stack cap | 4 / 6 / 8 |
| Blood Furnace max multiplier | ×3 / ×4 / ×5 |
| Bombard radius | 0.8 / 1.0 / 1.2 |

Pick the point that keeps the unit's favor row inside the band and its
adversarial policy ≤ 60 %, preferring the one closest to the draft.

### E.5 New lab policies (one generic mechanism)

`comp-<a>-<b>[-<c>]`: the shipped bot that buys any of the named units
when offered and otherwise cycles the cheapest card (the D-44 behavior).
It covers every combination below with no per-unit code:

| Policy | Probes |
|--------|--------|
| `comp-leechroot-oilpress-bloodweaver`, `…-drumguard` | carry fortress |
| `comp-farwatch-oilpress`, `favor-farwatch` | long-range wall / spam |
| `favor-smuggler` (= smuggler economy) | unbounded income |
| `comp-bombard-sawbot-shieldbearer` | friendly fire as harm |
| `comp-bombard-archer-farwatch` | friendly fire done right |
| `comp-rotcap-oldfinn-hexer` | control lock + poison |
| `comp-oilpress-bloomkeeper-antidote` | turtle / stagnation |
| `comp-bloodfurnace-bloomkeeper` | heal-locked Furnace |
| `comp-leechroot-cargocrab`, `comp-bloodfurnace-cargocrab` | protected carry |
| `comp-puppetmaster-*` vs `favor-longaxe` | heavy hitters punished |

### E.6 The whole cast, not only the new units

The baseline's outliers (§A.3) are balanced in the same passes, by
numbers only, with lab evidence: Bone Brute, Siege Walker and Nester
(oppressive), Sawbot and Rootcaller (weak), the 47.5 % of Evolutions
never worth their Energy. Order: fix outliers first in P50's first cycle
so new units are not tuned against them. The banking dominance (`saver`)
is an economy rule (D-41/D-43) and stays with the owner.

### E.7 Iteration loop

1. Run `gate` + `duels` + `value` on discovery seeds.
2. Rank failing rows by severity (§H) and pick at most three numbers.
3. Apply with `--apply`, rerun discovery; keep only changes that move the
   failing row without breaking another.
4. Confirm on held-out seeds (`confirm`); write the numbers into
   `content.rs` and GAME-DESIGN (**Proposed**) in one commit, contracts
   updated in the same commit.
5. Repeat until §I passes or a row needs a mechanic change (then stop and
   ask the owner).

---

## F. Architecture

Constraints kept: closed enums read at fixed hooks of `step()` (D-34
style), no trait or effect engine, no new dependency, integer math,
`Vec` in id order, every number in `Ruleset`.

### F.1 Data: one closed ability enum, absolute per form

```rust
// content.rs — Proposed shape; fields are integers in simulation units.
pub enum Ability {
    None,
    Hatch,                                   // Nester (moved here, values unchanged)
    Guard,                                   // Rampart (moved here)
    Poison { dps: u16, pulses: u8, cap: u8 },
    Fish { bp: u16, ticks: u16, cap: u8, per_hit: u8 },
    Echo { every: u8, extra: u8 },           // Rotcap B, Old Finn B (combined with the above)
    Blast { friendly: bool },                // Bombard
    NoStronghold,                            // Farwatch
    Essence { base: u16, step: u16, dmg: u16, atk_bp: u16 },
    Formation { engage: i32, gap: i32 },     // Oilpress
    March { reach: i32, move_bp: u16, dr_bp: u16, hysteresis: i32, ignore_slow: bool },
    Brew { strip_poison: u8, ward: u16, second: bool },
    Puppet { window: u16, immune: u16, bonus_bp: u16, targets: u8, stumble: bool },
    Cargo { every: u16, open: u16, table: [u16; 5], double_every: u8 },
    Plates { count: u8, hp: u16, move_bp: u16, atk_bp: u16, dmg_bp: u16, shrapnel: u16, rampage: i32 },
    Carry { board: i32, lock: u16, dr_bp: u16, move_bp: u16, shield_bp: u16, surge_bp: u16 },
    Furnace { burn_bp: u16, max_bp: u16 },
    Transfer { amount: u16, low_bp: u16, second: bool },
}
// UnitDef gains `abilities: [Ability; 5]` (base, A1, A2, B1, B2), absolute.
```

`FormDef` keeps stat modifiers and gains only shared fields:
`splash_bp` (splash damage share, default 10 000), `heal_cut` (on-hit),
`vs_below_bp`/`vs_above_bp` (+ thresholds) for conditional damage. A unit
uses at most two variants (Rotcap: Poison + Echo), so the per-form entry
is a fixed pair `[Ability; 2]` if needed — decided in P43, not before.

### F.2 State

- **`Unit::is_summon()`** (unit-level: `summoner.is_some()` or a summon
  kind) replaces the 35 `kind.is_summon()` call sites where a unit is at
  hand, so cargo can be a roster kind. Highest-risk refactor of the plan;
  done first, alone, with goldens.
- **`UnitFx`**, a compact `Copy` block inside `Unit` (~60 bytes):

```rust
pub struct Stack { pub left: u16, pub value: u16 }
pub struct UnitFx {
    pub poison: [Stack; 6], pub fish: [Stack; 3],
    pub confused_until: Tick, pub immune_until: Tick, pub ward_until: Tick,
    pub betray: bool, pub attacks: u8,          // echo counter
    pub essence: u16, pub essence_progress: u32,
    pub plates: u8, pub plate_hp: i16,
    pub march_move_bp: u16, pub march_dr_bp: u16, // rewritten every tick
    pub bond: Option<UnitId>,                     // Drumguard's ally
    pub carrier: Option<UnitId>, pub passenger: Option<UnitId>, // Cargo Crab
    pub board_lock: Tick, pub shield: u16, pub shield_until: Tick,
}
```

  `hatch_cd` becomes a general `action_cd` (Hatch and Cargo). Memory
  model: 200 units ≈ 12 KB today → ≈ 25 KB. `PlayerState` gains
  `cargo_rng: Pcg32` (new streams, never in views).

### F.3 Tick order (only additions)

```
synergy::update
combat::tick_timers        + fish/ward/confusion countdown; poison pulses every 20 ticks
summon::expire
summon::hatch              + cargo crates: place, open, draw (same module: summons)
support::bonds (new)       Drumguard bonds → march fields; Cargo Crab boarding
combat::run_units          + Hold for no-attack units, formation/escort clamps,
                             betrayal at windup start, action units (brew, puppet, transfer),
                             passenger x synced to its crab before it acts and after all moved
combat::stronghold_fire
combat::remove_dead        + shrapnel queue before removal
```

New modules, one responsibility each: `afflictions.rs` (stack lists,
pulses, cleanse, ward), `support.rs` (bonds, brew, transfer, puppet
selection, Cargo Crab boarding and dismount). Every target-selection
function in `targeting.rs` skips carried units through one shared filter.
`hits.rs` gains the blast, betrayal, essence, plate and
furnace branches; if it grows past its cohesion, plates and furnace move
to `afflictions.rs`'s sibling `vitals.rs` (decided at review).

### F.4 Protocol, view, events (v13)

- `UnitView.fx: u16` packed (poison 3 bits, fish 2, confused 1, ward 1,
  bonded 1, plates 3, growth stage 3, carried 1, 1 spare): postcard varint makes it
  1 byte for most units (+~40 B per view at 40 units, < 1 % of budget).
- Events: `PoisonTick{unit, amount}`, `Betrayal{unit, ally}`,
  `Cleansed{unit}`, `PlateBroken{unit}`, `CrateOpened{crate, unit:
  Option<UnitId>}`, `Transfer{from, to, amount}`, `Boarded{crab, unit}`,
  `Dismounted{unit}`. Bonds and growth are
  view state, not events.
- `PROTOCOL_VERSION` 13; the client renders unknown kinds as a blockout.

### F.5 Staging and rollback: the Shop pool becomes ruleset data

`Ruleset.pool: &'static [UnitKind]` replaces the `UNIT_KINDS` constant in
`shop.rs`, Smuggler draws and the lab. `CANON.pool` stays the current ten
until the owner switches the release (P53); the lab and a
**Practice-only preview flag** on the dev/test server use the full pool.
Removing a unit from the game = removing it from the pool; no revert.

### F.6 Instrumentation (lab only)

Behind the existing `lab` feature (never in server or client): per seat ×
kind counters for damage dealt/taken (direct, splash, blast, poison,
betrayal, friendly), healing and transfers, cleanses, control ticks
applied, bond uptime, crates by outcome, max Essence, plates broken;
per match: duration percentiles and a **stagnation detector** (60 s of
combat with no Stronghold damage and no conventional death). Attribution
needs a `source: Option<UnitId>` argument to `damage()`; under the
feature only.

### F.7 Risks of the architecture

| Risk | Mitigation |
|------|------------|
| `hash()` changes when `Unit` grows (goldens hash the whole state) | P43 updates golden hashes **once**, proving equal behavior by: identical tick/winner/reason per golden and a byte-identical `gate` CSV on the same seeds before/after |
| Unit-level summon refactor | done alone, first, with the lab CSV comparison |
| A carried unit picked by some targeting path | one shared filter; a test enumerates every selection function (enemy units, Strongholds, echoes, betrayal, Farwatch) |
| Movement rules touch every unit | goldens without Oilpress/Drumguard must not change |
| Betrayal through the heal path | explicit flag; tests for every case in §C.10 |
| Old clients | protocol v13 handshake rejects them (existing behavior) |
| Bandwidth | `fx` packed; delta views stay the P30-B lever |
| CPU | stacks are fixed arrays; bonds and formation are O(n·k) with k = Drumguards/Oilpresses; P16-B measured 45 µs at 200 units → budget ≤ 100 µs |

---

## G. Implementation phases

Order is by dependency and risk: measurement before change, shared
systems before units, units that need other units last. Every phase ends
with the AGENTS.md verification (fmt, clippy, tests, WASM check), a lab
comparison against P42's baseline, and a report; each unit wave is its
own branch and PR (main is protected: PR + green `check`).

| Phase | Goal | Units | Depends on |
|-------|------|-------|------------|
| P42 | Canon rows, lab instrumentation, baseline | — | — |
| P43 | Shared foundations, no behavior change | — | P42 |
| P44 | Wave A — direct damage | Longaxe, Farwatch, Bombard | P43 |
| P45 | Wave B — afflictions | Rotcap, Old Finn, Antidote Brewer | P43 |
| P46 | Wave C — HP as a resource | Shatterback, Blood Furnace, Bloodweaver | P43 |
| P47 | Wave D — army shape | Oilpress, Drumguard | P43 |
| P48-A | Wave E1 — protected carries | Leechroot, Cargo Crab | P44–P47 |
| P48-B | Wave E2 — rule benders | Puppetmaster, Smuggler | P48-A |
| P49 | Cross-roster interactions and adversarial search | all | P48-B |
| P50 | Balance cycles (whole cast) | all 25 | P49 |
| P51 | Readability and minimal visuals | all | per wave, polish after P50 |
| P52 | Performance and determinism validation | all | P50 |
| P53 | Release review and canon switch | all | P51, P52, owner |

P44–P47 touch different modules only partly (`hits.rs` and `combat.rs`
are shared), so they run **in sequence**, never by two agents at once.

### P42 — Spec, instrumentation, baseline (offline; gameplay unchanged)

- **Systems:** §F.6 counters, stagnation detector, duration percentiles,
  `comp-*` policy, duel exemption for no-attack units, role-invariant
  exception for "never targets Strongholds".
- **Files:** `docs/GAME-DESIGN.md` (§5 planned-units table, rules
  BR-U11… marked **Proposed, not active**), `docs/DECISIONS.md` (D-46),
  `crates/tbg-core/src/{hits,state}.rs` (feature-gated), `crates/tbg-balance/src/{metrics,policy,gate,duels,knobs}.rs`.
- **Tests:** counters sum to `Hit`/`Heal` events on 100 seeds; stagnation
  detector on a synthetic stall; contract tests unchanged.
- **Done when:** `gate --pairs 1024` baseline report with the new rows is
  committed under `docs/media/roster/`; online build byte-identical in
  behavior (goldens untouched — feature-gated code only).
- **Risk:** low. **Rollback:** feature-gated.

### P43 — Shared foundations (behavior must not change)

- **Systems:** `Unit::is_summon()`; `Ability` enum with Hatch and Guard
  migrated; `UnitFx`; `action_cd`; Hold rule for no-attack units;
  `Ruleset.pool`; packed `fx` in views; protocol v13 with the new events
  (unused yet); Practice preview flag.
- **Files:** `types.rs`, `content.rs`, `ruleset.rs`, `state.rs`,
  `combat.rs`, `targeting.rs`, `summon.rs`, `shop.rs`, `view.rs`,
  `protocol.rs`, `intent.rs`; client `sync/` for v13; lab pool.
- **Tests:** goldens: same tick/winner/reason, new hashes recorded once;
  `gate` CSV identical to P42 on the same seeds; protocol contract v13.
- **Done when:** the above, plus WASM check and a two-client online match.
- **Risk:** medium (wide refactor). **Rollback:** single PR revert; no
  new unit is reachable.

### P44 — Wave A: Longaxe, Farwatch, Bombard

- **Systems:** `splash_bp`, conditional damage (above/below HP), on-hit
  heal cut, `NoStronghold` targeting, friendly blast.
- **Tests (cite rules):** Farwatch holds instead of hitting a Stronghold
  at both seats; Guard vs Farwatch; Bombard self-hit, ally full damage,
  own Stronghold untouched, kill credit; Longaxe strikes over own tank
  (micro-scenario with `lab_spawn_at`); splash share math.
- **Done when:** duels and favor rows for the three units reported;
  no row that passed at P42 fails; placeholder figurines (blockout
  recolors), names and tips in both P41 catalogs; Practice preview plays.
- **Risk:** low.

### P45 — Wave B: Rotcap, Old Finn, Antidote Brewer

- **Systems:** stack lists, pulses, echo, multiplicative movement factor,
  cleanse, ward.
- **Tests:** §C.1, §C.5, §C.9 edge cases; stacks shared and capped;
  exact pulse count; fish never change attack interval; ward blocks every
  listed effect and not damage; Antidote waits when nothing to cleanse.
- **Done when:** as P44, plus `comp-rotcap-oldfinn-hexer` reported.

### P46 — Wave C: Shatterback, Blood Furnace, Bloodweaver

- **Systems:** plates in `damage()`, shrapnel queue, burn and multiplier
  at resolve, transfer selection and event.
- **Tests:** plates before HP after reduction; plates never restored;
  shrapnel chain terminates; burn only at resolve (root cancels at no
  cost); Deathblow at 0 HP and simultaneous deaths; multiplier off vs
  Strongholds; transfer bounds (donor ≥ 1 HP, heal cut, summons
  excluded, ties).
- **Done when:** as P44, plus `comp-bloodfurnace-bloomkeeper` reported.

### P47 — Wave D: Oilpress, Drumguard

- **Systems:** formation clamp, bonds with hysteresis, escort clamp,
  march bonuses into the caps.
- **Tests:** formation only while engaged; allies ahead unaffected; no
  effect on enemies or Strongholds; bond hysteresis deterministic; best
  bonus with two drummers; caps; goldens without these units unchanged.
- **Done when:** as P44, plus stagnation row for `comp-oilpress-*`.
- **Risk:** medium (movement).

### P48-A — Wave E1: Leechroot, Cargo Crab

- **Why together and late:** the two most dangerous carry setups
  (Leechroot behind Oilpress with Bloodweaver/Drumguard, Leechroot on a
  crab) need every other wave present to be measured; Cargo Crab's
  targeting filter must cover every targeting path, so all of them must
  already exist.
- **Systems:** Essence; carrier/passenger fields, shared targeting
  filter, position sync, windup redirect, dismount, A2 shield on the
  plate absorb path.
- **Tests:** §C.3 and §C.15 edge cases; the targeting-filter test that
  enumerates every selection function.
- **Done when:** as P44, plus `leech-fortress`,
  `comp-leechroot-cargocrab`, `comp-bloodfurnace-cargocrab` rows.
- **Risk:** high (snowball, wide targeting change).

### P48-B — Wave E2: Puppetmaster, Smuggler

- **Why last:** Puppetmaster's value needs Longaxe and Blood Furnace and
  its counter Antidote, and its target rules must respect carried units;
  Smuggler's cargo draws from the whole roster, Cargo Crab included.
- **Systems:** confusion, betrayal flag, target choice; crates, cargo
  stream, roster-kind summons.
- **Tests:** §C.10, §C.11 edge cases; cargo table χ² over 100 000
  crates within 1 %; replay of a match with crates gives the same hash;
  betrayal never heals and never picks a carried unit.
- **Done when:** as P44, plus `smuggler-economy` row.
- **Risk:** high (randomness, correctness).

### P49 — Interactions and adversarial search

- One deterministic micro-scenario test per row of §D marked L/F/W
  (`tbg-balance/tests/roster_interactions.rs`, generic helpers, no
  harness).
- Search over 2- and 3-unit `comp-*` policies (the search module's
  (1+λ) over unit sets) for any composition above 60 % vs the pool.
- **Done when:** every §D row has a test or a lab row; search report
  committed; new findings added to §H.

### P50 — Balance cycles

- §E.7 loop, outliers first (§E.6), then new units; dose-response
  studies (§E.4). Each cycle ≤ 3 numbers, confirmed on held-out seeds,
  GAME-DESIGN **Proposed** in the same commit.
- **Done when:** §I lab rows pass or the remaining failures are owner
  decisions written in the report.

### P51 — Readability and minimal visuals

- Desktop only (owner rule). Pooled, cheap cues: poison pips (≤ 6),
  fish (≤ 3), confusion strings, ward glint, bond line, plates count,
  Leechroot glow by growth stage, Blood Furnace emissive by HP %, crate
  box + open puff, blast ring, transfer thread. No new particle system.
- Placeholder figurines become blockouts per ART-DIRECTION; final art is
  per-character phases like P22–P29, after the owner's look sign-off.
- **Done when:** gallery screenshots of all 15 at desktop size; a 40-unit
  match recorded where a tester can name each effect; FPS unchanged
  within noise on the P30-C harness.

### P52 — Performance and determinism

- `examples/army_load.rs` with all abilities at 10–100 per side: step
  mean ≤ 100 µs at 200 units; view bytes ≤ +15 % vs P16-B.
- A real server match with every new unit replayed headless: same hash.
- 100 000 lab matches: 0 hits of the 6-hour safety bound.
- WASM release build size within the ARCHITECTURE budget.

### P53 — Release review

- Owner switches `CANON.pool` to 25 after: §I, a two-device online match
  with reconnect, and the playtest (≥ 8 people, ≥ 24 matches).
- GAME-DESIGN, ARCHITECTURE, DECISIONS, ROADMAP updated; values stay
  **Proposed** until the owner's balance sign-off.

---

## H. Competitive risks (most severe first)

| # | Risk | Severity | Why | Mitigation |
|---|------|----------|-----|-----------|
| 1 | Protected Leechroot (Oilpress + Bloodweaver + Drumguard + heals, or riding a Cargo Crab) | Critical | Unbounded growth while single-target damage cannot reach it | Linear stack cost, range 1.2 inside splash, a carried unit sits inside every splash on its crab, summon/Stronghold exclusions; `leech-fortress` and `comp-leechroot-cargocrab` ≤ 60 % |
| 2 | Stalls in open-ended matches (walls, heals, cleanse, no-Stronghold damage) | High | No clock exists (BR-M07) | Stagnation row ≤ 1 %; 0 safety-bound hits; if it fails, an owner decision on a mechanism, not a silent fix |
| 3 | Smuggler economy with copies | High | Income without an Interest-like ceiling | Interval/Empty-share levers; cargo excluded from every economy and Synergy rule |
| 4 | Inherited baseline failures (`saver`, Machines 78 %, outliers) | High | New units would be tuned against errors | P42 baseline, outliers first in P50, economy stays with the owner |
| 5 | Farwatch spam from home | High | Cheap, safe, uncapped | Rampart Guard, Stronghold immunity, range dose-response, `favor-farwatch` row |
| 6 | Betrayal correctness (heal path, windup, deaths) | Medium–high | Silent wrong outcomes are worse than imbalance | Explicit betrayal flag, exhaustive tests, replay check |
| 7 | Heal-locked Blood Furnace | Medium | A fixed high multiplier sustained by healers | No multiplier vs Strongholds, `comp-bloodfurnace-bloomkeeper` row, heal cut counters |
| 8 | Shop dilution to 25 kinds | Medium | Less agency to build a plan (a wanted unit in 20 % of sets instead of 50 %) | "Purchases until first offer" metric; an owner decision on the Shop if needed |
| 9 | Bombard frustration (friendly fire, enemy Bounty) | Medium | Novices lose to their own unit | Clear blast ring, tip text, friendly-fire ratio row; identity kept |
| 10 | Control lock on slow units (Old Finn + Hexer + Rootcaller) | Medium–low | Long immobility | Multiplicative movement, fish never slow attacks, Antidote, Drumguard B2 |

---

## I. Release criteria (all objective)

Lab rows run with `gate --pairs 1024` on confirmation seeds, 95 %
cluster-bootstrap intervals.

| Row | Threshold |
|-----|-----------|
| Every row that passed at the P42 baseline | still passes |
| Each new unit `favor-<unit>` vs bot | median of all favor rows ± 10 pp |
| Each `comp-*` and adversarial policy of §E.5 vs the pool | ≤ 60 % |
| Strongest strategic style | not above the P42 value + 3 pp |
| Duels: new base forms ≥ 70 % average at every budget | none (required) |
| Duels: new forms never worth their Energy | ≤ 25 % of new forms; no branch never better than its sibling |
| Faction-majority rows | no faction further from 50 % than at P42 |
| Roster health | ≥ 2.5 % purchase share per unit (owner decision O-R2) |
| Stagnation (60 s without Stronghold damage or conventional death) | ≤ 1 % of matches |
| Duration | p99 ≤ 1.5 × P42 p99; 0 matches reach the 6-hour bound in 100 000 |
| Comebacks | ≥ 20 % |
| Smuggler outcomes | table matched within 1 % over 100 000 crates |
| Bombard friendly-fire ratio | ≥ 2 : 1 median in `favor-bombard` and ≤ 1 : 1 in `comp-bombard-sawbot-shieldbearer` (risk real both ways) |
| Determinism | goldens, replay of a real server match, same hash |
| Performance | ≤ 100 µs step at 200 units; view bytes ≤ +15 % |
| Build | fmt, clippy, tests, WASM check, `trunk build --release`, two clients finish a match with a reconnect |
| Humans | ≥ 8 players, ≥ 24 matches; ≥ 4 of 5 testers explain a loss that involved a new unit; owner signs off look and fun |

Not a criterion: every unit at 50 %. Bands follow the existing gate
philosophy (necessary, never sufficient).

---

## J. Next action

**Start with P42.** It changes no gameplay, takes the least effort, and
everything after it depends on it: the baseline already fails 12 of 39
rows, so without per-unit counters, duration percentiles and a
stagnation detector no later phase could tell whether a new unit caused
a failure or inherited it. It also writes the canon rows and asks the
owner the two open questions while P43 is being prepared.

---

## Appendix 1 — Draft reconciliation

| Draft said | This plan | Why |
|------------|-----------|-----|
| Oilpress: enemies cannot pass | Kept and noted as universal; added **formation** for own units | Every unit already stops enemies in this engine |
| Leechroot: +1 dmg, +2 % per 50 damage, unbounded | Kept unbounded; stack cost rises `40 + 2n`; range 1.2 | Flat cost is exponential; 2.5 range would be out of every splash |
| Farwatch: 85–90 % of the lane | 12 tiles (75 %), dose-response 8–14 | Same identity, measured choice |
| Antidote: one stack per 5 s | Full cleanse of one ally + 1 s ward every 4 s | One stack is erased by the next hit |
| Puppetmaster: 14 s, target unspecified | 8 s, highest-damage enemy in range | Value per cast is 2 × H; aim where it matters |
| Smuggler: five custom units, 7 000 Energy | Roster units as summons, Empty outcome, cost 6 | No new art, readable, EV computed |
| Drumguard: −15 %/−20 %/−25 % | −15 %/−18 %/−18 % + slow immunity | 25 % exceeds the 20 % cap (BR-X05) |
| Rotcap: spread on death? | No spread | Self-propagating swarm wipes; branch B covers spread |
| Bloodweaver B2: split one amount | Second independent transfer (four units) | The split added nothing |
| Cargo Crab: hits aimed at the passenger hit the crab | Carried units cannot be chosen by any targeted effect; boarding by distance (purchase order) | One rule instead of a redirect per hit; deterministic |
| Grave Needle, Soulreaver, Duelist, Ramguard | Ignored | Not in the game or its docs |

## Appendix 2 — Cargo Crab (included by owner decision O-R1)

The plan first recommended deferring Cargo Crab (draft `12`) because it
adds a unit relation that touches every targeting rule and because a
carried Leechroot cannot be focused. The owner chose to include it with
Shatterback (2026-10-29). The plan absorbs that risk in three ways:

- **Geometry is the counter.** The passenger sits at distance 0 from the
  crab, at the front line, so every splash or blast aimed at the crab
  also hits it (§C.15); a carried carry is safer from focus and *less*
  safe from area damage.
- **One filter, one test.** Carried units are excluded from targeting by
  a single shared filter in `targeting.rs`, and one test enumerates every
  selection function (§F.7), so a new targeting rule cannot forget it.
- **Measured late.** It ships in P48-A together with Leechroot, after
  every other unit and targeting rule exists, with two dedicated
  adversarial rows (§E.5, §I).

Draft changes: boarding is decided by distance (0.5 tile), which in
practice means purchase order; "attacks aimed at the passenger hit the
crab" became "no targeted effect can choose a carried unit" (one rule
instead of a redirection at every hit), except for an attack already
winding up at the moment of boarding.

## Appendix 3 — Owner decisions (both answered)

| ID | Question | Default if not answered |
|----|----------|-------------------------|
| O-R1 | Shatterback (no draft, designed here) or Cargo Crab? | **Decided 2026-10-29: both** (15 new units, Appendix 2) |
| O-R2 | Roster-health threshold for 25 units (5 % is impossible: a perfectly even share is 4.0 %) | **Decided 2026-10-29: ≥ 2.5 %** (about 60 % of an even share); written into GAME-DESIGN §6 in P42 |
