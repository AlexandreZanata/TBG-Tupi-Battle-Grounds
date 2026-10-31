# P58 — Technical character and future low-poly catalog

**21 text-derived purchases; planning only.** The owner explicitly limits
all PNGs to visual reference. No image name, statistic, ability, evolution,
flight or apparent physical action defines gameplay. Sources T/V refer to
the [inventory](P58-AUDIT-AND-INTEGRATION.md#1-source-authority-and-inventory).
Every stat, ability parameter and Evolution value is an initial **Proposed**
balance hypothesis in [GAME-DESIGN §13](../GAME-DESIGN.md#13-p58-roster-expansion--proposed-inactive).
Those tables are part of each sheet; they are not duplicated here.

All sheets inherit the exact [competitive contracts](P58-AUDIT-AND-INTEGRATION.md#3-shared-competitive-contracts-all-proposed-inactive),
which specify timing, target loss, interruption, rounding, status stacking,
summons, Strongholds, heal interaction, events and reconnect behavior.
Secondary roles below are descriptive: BR-U02 still gives one Archetype.

## Reading the art and animation specifications

Each character needs base/A2/A3/B2/B3 visual forms (stored base/A1/A2/B1/B2).
Apply the existing ART-DIRECTION Evolution progression: larger silhouette,
one large color/material change and one new silhouette piece per step,
then a branch artifact at the last step. Team blue/red panels plus solid/
notched ground rings stay readable regardless of Faction color. Use shared
vertex-color material, merge parts by moving joint and cache form geometry.
No new per-unit light, shader, cloth solver or texture dependency.

The motion tables below define **individual poses**, not one generic attack
with different weapons. Shared evaluation/rig machinery can be reused.
All approximate durations are presentation hypotheses. Each sheet's named
parts move; other parts hold their rest pose unless secondary motion is
specified. The common timing and event mapping applies to every row:

| Motion | Approximate duration and trigger | Reuse / cost / reduced quality |
|---|---|---|
| I: Idle | 2.4 s loop when not moving or acting; phase offset from UnitId | Existing pose sampler; low-amplitude torso plus named prop; disable secondary prop movement on Low |
| W: Walk | 0.8 s reference cycle, retimed to authoritative x speed | Existing planted-foot sampler for humanoids; local appendage tracks for other bodies; no root displacement from feet |
| P: Anticipation | First 0.30 s after AttackStart or new special-start cue | Existing joints; holds character-specific windup silhouette; no client delay to simulation |
| A: Basic strike | 0.50 s main clip, contact at 0.30 s; ranged release at 0.18 s | Main clip sampler supports this. If reviewed P54/P55 integrates, reuse its 0.62 s recovery with the same contact times |
| H: Hit/impact | 0.15 s on authoritative Hit, no override of an active strike | Existing tint/Hit cue; no simulated stagger or attack cancellation |
| D: Death | 0.60 s collapse plus current corpse retention, then disappear | Existing Death cue; no debris entities, gameplay corpses or ragdoll |
| S: Special | 0.60 s gesture on special-start/resolution; effect at authoritative resolution | New per-kind visual profile only; timer/buff overlays can persist until their view expiry, never infer damage from particles |
| E: Evolution | 0.60 s on Evolved, swap form with a readable pose | Existing cue; change scale/gear at form swap, no invulnerability or extra action |

Ranged release-to-impact presentation is shortened/held to match the real
Hit rather than delaying damage to a cosmetic projectile. Instant procs
reuse the current strike's contact and add a cue; they do not add a second
competitive windup. A death or Root cancels a pending special's visual cue
when the authoritative view says so. An already applied DoT/mark/shield
continues for its documented lifetime after source death.

Each sheet names a visual extension when required. Its cost is O(animated
parts) per visible unit, with shared meshes/materials and no extra world
entities for companions. For dozens/hundreds of units: reduce spring/wing/
smoke movement first, retain character silhouette, attack contact, team,
HP, shield and persistent effect indicators. Measure actual node count,
triangles, draws and frame time; none of the following is a performance
measurement or an asset-generation instruction.

## R01 — Bastion

**Source:** T01 / V01. **Proposed:** Machines, Tank; secondary role line
holder. **Shop text:** “An armored walker that braces to endure the front.”
Its mobile-fortress fantasy is about a resistance window, not Rampart's
forced-target Guard. T01 supplies both branches, +HP/+damage and Slow,
but is missing its base sheet; the entire base/trigger is a clearly marked
completion in the registry. V01 supplies only stone-and-strap appearance.

**Ability — Muralha Viva (B).** Ready with a nearby enemy unit, begin the
shared special windup; on resolution anchor at current x and apply the
self-reduction window. It may still attack normally while anchored; Root
stops those attacks normally, without extending protection. No ally aura,
taunt, collision wall or HP-threshold trigger. Stronghold attacks can hurt
it through the same capped reduction. B3 basic Slow uses ordinary on-hit
rules; it does not turn area damage into a Guard mechanic.

**Choice and matchup.** A buys survival/window uptime, B buys modest
offense and later control. Compared with Rampart, it trades away Guard
and baseline damage for a timed stance; compared with Bone Brute it has
much lower damage per Energy. Pair with Archer/Bloomkeeper; Siege Walker
and focused damage remain counters. Its biggest risk is a shield/healing
stall; it is not a compulsory replacement for Rampart. Approve the missing
base before executing this unit.

**Low-poly forms:** broad stone dome over compact humanoid legs, visible
face/tool, iron bands, oak straps and brass Machines fasteners. Use a
large team-colored cloth panel on the shell. A2 adds shell buttresses and
warmer stone; A3 adds a crenellated back crest. B2 exposes iron forearms
and a larger tool; B3 adds a flat stone impact shoe. Reuse Large anatomy,
stone/metal palette and weapon socket; no walkable Stronghold on its back.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Head peers from shell; chest slow breath | Hips rock under shell, short planted shin/foot steps | Both forearms lower tool, chest braces; compact downward tool blow | Head retracts; knees fold and shell settles | Feet spread, shell lowers to brace; E lifts shell and reveals new supports |

No additional rig nodes beyond a shell/back attachment; Low drops strap
oscillation. Visual bracing is tied to the actual stance expiry.

## R02 — Demolidor

**Source:** T02 / V02. Machines, Tank; secondary area pressure.
**Shop text:** “Hydraulic fists punish packed enemies at close range.”
Stats/branches preserve the text. Its apparent launch is explicitly visual
in T02: do not implement knockback, airborne status or cancelled attacks.

**Esmagamento em Área (B):** acquire an enemy unit in basic reach, resolve
one caster-centered area hit after the shared windup. No allied or
Stronghold collateral. The base hit has no control status; B3 adds Slow
to ordinary primary attacks, as the text's “golpes” is interpreted, rather
than secretly giving a stun to the impact. A favors larger/harder impact;
B favors frequency and eventual attack control.

It matches Bone Brute's base DPS at greater Energy cost and has less HP;
its added value is delayed group damage. Against three enemies its
perfect-uptime special model adds 36 raw DPS total, requiring arrival and
survival. Rampart protects access; Rootcaller/Hexer deny it; Siege Walker
is the main ranged area comparator, Bombard† the future one. This is a
close-range area option, not a superior ranged bomber. Watch support-
protected stacking of many impacts.

**Art:** squat broad humanoid, fists almost ground-sized, two piston cuffs
and a compact pressure tank; iron/brass/rust, team cloth at waist/back.
A2 thickens fist plates, A3 adds broad hammer palms; B2 exposes cyan-free
forge-orange valve guards, B3 adds asymmetric shock coils. Reuse Large
limbs and mechanical mesh parts; valves are rigid props.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Forearm pistons settle one after the other | Heavy shoulder counter-swing and short heel plant | Opposite fist draws back; forward hydraulic punch | Chest compresses; pistons buckle as torso kneels | Both fists overhead then grounded together; E pumps fists once |

Optional paired piston presentation nodes only. Impact ring is shared
geometry; never animate enemies flying, since no movement/interruption is
simulated. Low removes pressure puffs, retains the two-fist strike.

## R03 — Crowcaller

**Source:** T03; no confidently associated image. Undead, Ranged;
secondary anti-sustain control. **Shop text:** “A plague bolt marks one
foe for slower movement, weaker healing and precise follow-up.”

**Marca do Corvo (B):** readiness arms the next completed enemy-unit
primary hit. Apply the mark after that hit's damage, so the triggering
shot gains no mark bonus unless its previous own mark was still active.
Use the standard Slow slot plus ability heal-cut; caster owns one bonus
target/deadline. Recasts/two Crowcallers cannot accumulate a team damage
amplifier. Strongholds receive ordinary shots only, never the mark/cut.
No summoned raven, remote vision, stealth reveal or support-priority
targeting. A improves anti-heal/window; B improves only its own offense.

Archer offers higher immediate DPS for less Energy; Hexer offers stronger
standard Slow; Crowcaller pays for mixed, timed anti-sustain. Bloomkeeper
and large healed Tanks are useful matchups, not an excuse to bypass Guard.
Fast Sawbot and ranged pressure counter it. Rotcap† differs by stacked
poison, Farwatch† by extreme range. Measure whether cut/control makes
Crowcaller compulsory, especially with Crematumba.

**Art:** narrow plague-mask silhouette, torn coat, short ritual crossbow,
one shoulder raven and relic pouch; charcoal, bone, tarnished bronze and
muted plague green with team lining. A2 adds feather mantle and green
vial panel, A3 a longer beak/censer. B2 adds rigid crossbow arms and pale
sleeves, B3 a large ruined sight. Raven is a shoulder/back prop, not Unit.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Head and shoulder raven glance in opposite directions | Narrow stalking gait, coat trails | Hand draws crossbow latch, head aligns beak; sharp bolt release | Mask turns away; coat collapses around folded knees | Raven wing opens at marked shot; E turns cloak then re-aims |

One optional raven wing node; Low fixes the bird in a recognizable perch.
Mark badge belongs to target view state, not a free-flying raven entity.

## R04 — Gatilho

**Source:** T04 / V04. Humans, Ranged; secondary small-group pressure.
**Shop text:** “A nimble feline mercenary adds an occasional second shot.”
Use **Tiro Cruzado**, not the image's hunt mark; Human allegiance is
explicit in the text. There is no Creatures Faction or execution passive.

**Tiro Cruzado (B):** the fourth completed enemy-unit shot also hits the
nearest distinct eligible enemy within the primary-centered search
radius; ties distance from primary then UnitId. Secondary damage is the
floored ratio of current static basic damage. No second candidate means
the proc is spent with one shot only. Guard governs primary; transported
passengers† remain untargetable by the secondary. No recursive on-hit,
Stronghold secondary or same-target double damage. A offers cadence/
mobility; B improves group coverage without improving single-target DPS.

Archer has better range and base DPS at lower cost; Gatilho buys mobility
and split pressure, with little HP. Rampart/Bone Brute absorb his primary,
Siege Walker punishes clumps; Esporax must remain the slower multi-target
alternative. Pair with a Tank, not a fictitious flanking system. Measure
whether A's extra cadence overwhelms B's conditional value.

**Art:** small orange feline, huge hat, two revolvers, booted legs and
curved tail; leather/oak/iron and Human gold, team neckerchief. A2 raises
hat brim and adds blue/red long coat, A3 swept tail guard/spurs. B2 adds
wide brass barrel collars and dark vest, B3 twin angular sights. Keep
weapons in this supplied mercenary fantasy; no imported protected asset.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| One pistol rolls at wrist, ears tilt | Quick heel-toe steps, tail counters hips | One arm levels, other covers; alternating recoil | Hat dips; knees skid into seated fall | Both forearms fire at the fourth-shot contact; E tips enlarged hat |

One tail/hat secondary track; Low freezes tail flourish, keeps two distinct
muzzles. No root translation, dash or target switching from cosmetic aim.

## R05 — Matraca & Tico

**Source:** T05 / V05. Humans, Tank; secondary periodic area control.
**Shop text:** “One lumbering bruiser, one noisy passenger, one shared fate.”
Tico is part of one model and one HP pool, not a summon/passenger system.
There is no separate 30-damage attack, stun or anti-heal from the image.

**Bagunça em Dobro (B):** Tico's special timer runs independently of the
club attack. Primary is the ordinary nearest enemy unit, area centered
there; apply damage and Slow once per affected enemy. A dead primary
cancels the bomb, including when the club killed it first that tick.
No allied/Stronghold splash and no physics projectile. A invests in
Matraca's HP and club; B in Tico's cast frequency/coverage.

Compared with Bone Brute it pays more for less base durability/DPS; its
timed bomb must earn that gap. Bloomkeeper and Drumguard† help delivery;
Rootcaller and Siege Walker counter it. Demolidor has the larger impact,
Matraca the more frequent smaller group pressure. Track repeated shielded
bombs and avoid making both brothers separate targets or rewards.

**Art:** square heavy torso, oversized nailed club, small thin brother
above one shoulder, bomb backpack; leather/linen/iron with broad Human
team sash. A2 expands chest straps and club head, A3 iron shoulder guard.
B2 enlarges Tico's satchel and striped coat, B3 adds visible bomb rack.
Do not reuse Cargo Crab boarding or add another Unit.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Matraca scowls while Tico points silently | Hips stomp, Tico lags behind chest | Matraca twists both shoulders; heavy club arc | Matraca hunches, Tico grabs straps; both tumble as one | Tico leans and throws while club may swing; E Matraca flexes, Tico waves |

Local Tico torso/arm nodes are necessary presentation extensions; his
independent gesture cannot be one generic Tank clip. Low keeps Tico's
throw and suppresses idle pointing; no additional per-tick targeting.

## R06 — Capivardo

**Source:** T06 / V06. Plants, Tank; secondary defensive area control.
**Shop text:** “A calm giant buys time with a heavy belly impact.”
Capivardo is the text's capybara, not the PNG title Tamanduá; neither
Creatures nor damage reduction during every base animation is adopted.

**Barrigada do Brejo (B):** acquire a close enemy unit and resolve a
caster-centered area hit plus ordinary Slow. No allied or Stronghold
special damage. Base has no damage reduction; A3 earns a capped window
after impact, independent of cosmetic airtime. Root cancels the pending
cast; existing applied protection ends normally. A offers survival, B
offers more frequent/wider control, not equivalent raw damage upgrades.

Bone Brute supplies much more DPS; Rampart supplies Guard. Capivardo's
value is Plants regeneration plus defensive time. Bloomkeeper/Rootcaller
and ranged allies benefit; Hexer/Rootcaller/ranged damage deny approach.
Oilpress† has a different formation wall, Shatterback† a permanent plate
loss tradeoff. Watch HP/regen/shield loops; keep the low damage identity.

**Art:** huge rounded belly, short thick limbs, capybara muzzle/ears,
leaf necklace and fiber skirt; warm ochre hide, bark/moss, team cloth on
pack. A2 stone-like belly guard and darker flanks, A3 broad leaf gorget.
B2 visible belly bands and pale fiber skirt, B3 a wide bark waist disk.
Rounded low-poly volumes can reuse Large torso/limb primitives.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Heavy belly breath and sleepy head tilt | Short rolling hip steps | Forearms sweep low, chest leans; small palm/body bump | Belly compresses; rolls onto side then settles | Knees tuck and torso belly-flops locally; E proud belly pat |

Local Y/rotation only: no simulated jump or enemy launch. Low removes
leaf wobble and dust, retains belly pose and a simple impact ring.

## R07 — Tridente

**Source:** T07 / V07. Plants, Tank; secondary multi-target attrition.
**Shop text:** “A shark warrior wears groups down with bleeding cuts.”
No Creatures Faction or armor/defense reduction; text bleed coexists with
healing and does not itself apply anti-heal.

**Rasgo Sangrento (B):** start only with a unit inside basic reach; hit
enemy units around the locked primary's current x, apply one bleed slot
per surviving victim. The circular x-interval is the text's explicit
radius rule, not a new directional cone. No allied/Stronghold bleed or
special collateral. Refresh cannot stack sources or delay the next pulse.
A buys HP/bleed strength, B frequency/coverage. Already-applied bleed
does not upgrade retroactively when Tridente evolves.

It pays more than Bone Brute for lower basic damage, so its group bleed
must be evaluated separately. Plants sustain and Bloomkeeper support it;
range/control counter it. Demolidor offers immediate burst, Tridente
slower damage. Rotcap† and Vespira provide distinct ranged poison, not
extra bleed stacks. Test its low solo Stronghold pressure and mixed DoTs.

**Art:** broad shark head, dorsal fin, muscular legs, rusted trident and
large chain shapes; slate hide, bone teeth, rust/bark and team sash.
A2 shell cuirass and pale belly, A3 large dorsal armor; B2 wider trident
and red-brown wrap, B3 crescent side tines. Chains merge into torso mesh.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Jaw opens slightly, trident rests | Torso pitches with fin counterbalance | Hands slide shaft back; short forward thrust | Head twists, shaft guards chest; knees buckle sideways | Both hands drive a broad low sweep; E plants trident and shows fin |

No water simulation. A shared short arc and persistent bleed indicator
carry the effect; Low drops loose-chain motion, not the damage status.

## R08 — Grumak

**Source:** T08 / V08 shared material reference only. Monsters, Tank.
**Shop text:** “A primordial quadruped hardens under sustained punishment.”
The text's rhinoceros-like head, tusks and bone armor are required; it is
not the image's crocodilian Troncão and does not inherit its ground slam.

**Couraça Primordial (B):** only positive HP removed by a damage call
builds a stack, after that call. The triggering hit gains no new reduction;
zero damage, shield absorption, healing and HP transfers do not build it.
Later friendly damage can build it if the same damage path actually
removes HP; no free fully-shielded stacking. Every positive receipt refreshes
the shared inactivity expiry; no source list or indefinite permanent armor.
A improves HP/expiry; B improves basic damage against the defined printed
short-range targets, not an invented armor category.

Rampart trades higher control of enemy targeting for Grumak's ramping
survival; Bone Brute remains much more offensive. Pair with low-cost
ranged units, counter with burst/range/control and pauses between hits.
Healing plus Sentinel is a critical stall case. Monstrous armor never
exceeds the global reduction cap or receives AoE immunity.

**Art:** low, wide quadruped, heavy rhino muzzle, oversized tusks, separate
bone dorsal plates; charcoal hide, ivory, turquoise eye grooves and team
cloth across flank. A2 adds overlapping pale shoulder plates, A3 broad
back ridge. B2 darkens neck and lengthens tusks, B3 adds an outer jaw guard.
No direct one-to-one V08 panel match is asserted.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Head lowers, ribcage breathes | Diagonal four-leg heavy gait | Neck coils, front legs brace; upward tusk shove | Shoulder plates compress; front knees fold then rear | Plates tighten once at stack gain; E neck rises under new bone crest |

Four-leg mesh/track mapping is a local visual extension; no collision
volume. Low fixes small plates and removes turquoise flicker.

## R09 — Rasgarr

**Source:** T08 / V08 palette only. Monsters, Dps.
**Shop text:** “A six-limbed predator becomes lethal if its prey cannot escape.”
Preserve six legs/large front scythes; no spear, invisibility or poison.

**Frenesi Predatório (B):** first hit has no bonus; subsequent valid
primary hits on the same enemy ramp to the registry cap. Target change,
death or inactivity clears the count; a cancelled windup does not add one.
Stronghold hits have ordinary base damage and reset the prey sequence.
A closes faster/attacks faster; B reaches a slightly higher bounded ramp
and tolerates a longer gap. No growth survives death or transfers to allies.

Sawbot has greater basic DPS at equal cost; Rasgarr buys HP and sustained
same-target ramp. Louva spends a discrete fourth-hit burst instead of
maintaining ramp, preventing identical roles. Rootcaller, Hexer and splash
are clear counters; Grumak buys it time. Test focus switching in summon
waves and compare full-cycle damage at equal Energy, not only peak DPS.

**Art:** low head, six planted limbs, massive anterior scythe limbs and
lean abdomen; dark hide/bone with turquoise eyes and team dorsal sash.
A2 swept lighter leg guards, A3 long trailing crest; B2 thick scythes and
dark chest, B3 serrated jaw shield. No Sombrajaré image anatomy import.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Front scythes scrape together | Alternating tripod crawl | One front limb hooks inward; opposite scythe cuts | Body flattens; limbs curl beneath abdomen | Jaw/scythes close progressively with ramp; E spreads all six supports |

Extra leg tracks required; merge static claws into limbs. Low retains
coarse tripod cadence and drops individual toe articulation.

## R10 — Esporax

**Source:** T08 / V08 surface language only. Monsters, Ranged.
**Shop text:** “A split-jawed spitter splashes a few nearby foes.”
Its acidic visual is not poison, armor shred or a damage-over-time effect.

**Cuspe Corrosivo (B):** third completed enemy-unit shot deals ordinary
primary damage, then flat damage to the closest distinct eligible
secondaries around it. Selection is one bounded best-k pass, no Stronghold
secondary, no recursion. Primary damage does not become 18 or 24; those
are auxiliary values. A improves single-target reach/damage; B improves
small-group coverage, with at most the registry's additional targets.

Archer offers much higher baseline DPS and better reach at lower Energy;
Siege Walker has unlimited-radius-target splash, while Gatilho is faster
and closer-ranged. Esporax therefore needs measured secondary value to
justify its price. Tanks protect it, Sawbot and distant shooters punish
its low HP. Avoid making its higher target count dominate Gatilho B.

**Art:** tall thin neck, four-part jaw, glowing sac at throat and slender
legs; muted hide, bone braces, turquoise sacs, team shoulder cloth. A2
elongates throat guard and lightens neck, A3 long sight-like horn. B2
widens sac shell and darkens torso, B3 three-lobed throat collar.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Throat sac breathes, jaw plates twitch | High-kneed narrow strides | Neck recoils, jaw opens four ways; sharp spit snap | Neck retracts; sac deflates into folded body | Sac swells at third shot then contracts; E jaw fans once |

Jaw/sac presentation nodes only; Low uses one sac scale track and fixed
jaw petals. No poison particles or misleading persistent acid puddles.

## R11 — Murmúrio

**Source:** T08 / V08 colors only. Monsters, Control; descriptive mental
support. **Shop text:** “A drifting eye disrupts one enemy's rhythm.”
The text is a floating jellyfish-like creature with bone mask/short
tentacles. It is not image-only Xamágua or a healing swamp shaman.

**Pulsação do Medo (B):** ordinary nearest/Guard-eligible unit gets the
standard Slow; no fear movement, attack redirection or extra special hit.
Root takes priority; duration benefits remain bounded. Basic damage still
exists, so it can attack Strongholds at its low printed output. A offers
reach/duration, B uptime/basic damage. Visual floating never changes
targetability or movement coordinate.

Hexer is cheaper, has much stronger continuous Slow and higher DPS: this
is an obvious underpower risk to investigate, not a balanced price claim.
Murmúrio might buy intermittent control in Monsters compositions; it
does not require a unique new mechanic merely to justify cost. Test
Rasgarr protection and reject/tune the hypothesis if no useful matchup
emerges. Range and focused fire are counters.

**Art:** floating bell shape, one luminous eye/bone mask and short tapered
tentacles; charcoal/ivory/turquoise, team ribbon underneath. A2 taller
eye crest and pale bell, A3 wide mask rim; B2 dark segmented bell and
paired forehead ridges, B3 prominent eye socket. Reuse primitive lathe.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Bell compresses, eye tracks forward | Bell rolls above grounded x marker | Tentacles fold inward; eye emits a short pulse | Bell flattens; sinks with closed eye | Tentacles spread into a radial pulse; E opens new crest |

Local tentacle nodes; Low fixes outer tentacles and keeps eye/contact.
No actual flight, vertical hitbox or destination displacement.

## R12 — Sentinela Solar

**Source:** T09 / V09 first panel. Celestials, Tank; descriptive Support.
**Shop text:** “A solar guardian lends a brief shield to a wounded ally.”
The text selects lowest HP fraction, not the nearest healthy ally; there
is no taunt or self shield from the picture.

**Barreira Solar (B on the reaudited base; grant path needed):** shared windup targets the lowest-fraction
other allied unit in special range, including allied summons; ties
distance then id. If all other allies are full HP, remain ready (completion
to avoid random idle shielding). At resolution add/replace its temporary
shield under the shared strongest/equal/weak rule. No healing, overheal,
max-HP change or Stronghold shielding. B2 shields a second distinct ally
from the same resolution selection, at its own smaller amount; it cannot
double-stack on one recipient. A improves own HP/main shield, B coverage.

Its shield's perfect-consumption model is 12.5 HP/s, not guaranteed heal
output. With Bloomkeeper it can protect recovery; Arqueira is the explicit
shield counter, while sustained damage consumes it. Rampart's Guard is
still unique; Sentinel is not a better armored Rampart. Multiple Sentinels
with Oráculo create the critical defense-loop test.

**Art:** upright spear/shield guardian, oversized sun disk, ivory/gold
plates, dark blue cloth and team tabard separate from gold. A2 broad pale
breastplate, A3 taller disk shield crest. B2 paired shoulder disks/darker
cloth, B3 wide halo with side panels. Reuse humanoid spear/shield parts.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Shield slowly rises with breath | Deliberate shield-leading steps | Spear draws near hip; direct controlled thrust | Shield tilts to cover face; kneels behind it | Shield arm extends toward recipient, sun disk opens; E spear held upright |

One halo prop track; Low keeps a simple shield bar/bubble outline and
removes floating fragments. Shield view state must survive reconnect.

## R13 — Oráculo Luminar

**Source:** T09 / V09 second panel. Celestials, Control; descriptive
Support. **Shop text:** “A measured light pulse weakens one foe's attacks.”
The authoritative text gives damage weakening only. “Luz Restauradora”
is a branch name, not permission for healing or a protective aura.

**Pulso Harmônico (B):** select an enemy unit under shared offensive
targeting, apply one weakening slot after windup. It reduces future basic
attack damage, including already-winding attacks when they resolve,
not healing or auxiliary DoT/cast values. No stacking between Oráculos;
ward/cleanse policy follows shared contracts. A extends duration, B reduces
period, with intermediate steps proposed because source branches are not
fully tiered. No multiple target pulse.

Low basic output and modest debuff suggest an underpowered isolated unit:
against a 50-DPS target, ideal average prevented damage is only 2.25 HP/s
before timing effects. Its case is protecting a meaningful ally, not
winning a duel. Compare Hexer/Bloomkeeper utility and require composition
ablations. Enemy range/focus counters it; Sentinel loops are the risk.

**Art:** long robe, staff sun-ring, hooded empty face and geometric cape;
ivory/gold/navy, team lining. A2 wider pale stole, A3 tall staff ring.
B2 short dark over-robe with side fins, B3 paired staff-ring struts.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Head inclines, staff ring rotates slightly | Small robe-hidden planted steps | Hands align staff and chest; small staff spark | Torso turns inside cape; staff lowers as knees fold | Ring faces target, offhand closes to weaken; E ring lifts above hood |

One ring/cape secondary track. Cosmetic robe bob does not imply flight;
Low removes ring rotation and stray shards, not weakening badge/expiry.

## R14 — Arqueira Celeste

**Source:** T09 / V09 third panel. Celestials, Ranged.
**Shop text:** “A precise archer breaks temporary shields with a charged shot.”
No line piercing, multiple targets or team damage-amplification mark from
the image. The text's shield counter is the strategic differentiator.

**Flecha da Purificação (B; reads R12's shield field):** every fourth valid
enemy-unit primary shot strips that unit's temporary shield **before**
replacing ordinary shot damage with the empowered amount. Does not remove
plates, HP, max HP, Synergy, Evolution, statuses or Stronghold shields.
No shield present still yields empowered damage. All ordinary shots
against Strongholds use basic damage without incrementing the proc.
A raises ordinary damage, which scales the empowered shot by the same
static ratio; B increases proc frequency and later empowered value.

Archer has greater basic DPS/range for less Energy, so Arqueira must
earn a niche through shield counterplay. She should not become mandatory
when no shield unit is present; test both ablations. Frontline protection
helps her, fast melee and Siege Walker counter her. No “holy damage”
resistance bypass. Shield stripping uses the existing `UnitFx.shield` field, which R12 must make grantable first.

**Art:** slim armored archer, angular gold bow, rigid feather wing motifs
and narrow halo; ivory/navy, team quiver sash. A2 long bow/ivory sleeves,
A3 large sight-halo. B2 pronged bow/dark cuirass, B3 three-point bow crown.
Wings are short shoulder ornaments, not flight machinery.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Bow vertical, wing plates settle | Light balanced steps | Shoulder blades align, hand draws; precise upright release | Bow tucks, torso turns; drops to knee then side | Halo/bow tips flare on empowered release; E raises bow across halo |

Reuse Archer grip/foot mechanics with a distinct upright pose and recovery;
one wing pair track. Low freezes wings, keeps empowered-arrow cue.

## R15 — Justiçar

**Source:** T09 / V09 fourth panel. Celestials, Control; descriptive area.
**Shop text:** “A heavy celestial punishes a tightly packed enemy line.”
One text-defined damage pulse, not three/four meteor explosions.

**Chuva Celeste (B):** locked enemy-unit primary, single area at its
current x, one damage application per living enemy. No ally/Stronghold
collateral. B3 explicitly adds standard Slow to each affected enemy; no
root, stun, burning zone or global target selection. A favors damage/
durability; B coverage/control. His ordinary basic hit remains available.

Siege Walker at lower Energy has the same base HP, substantially higher
DPS/range and regular splash; Justiçar's initial hypothesis is likely weak.
Test whether controlled timed pressure adds a useful Celestial role; do
not copy image damage to fix that. Pair with Tanks and test into Nester
clumps; range/fast pressure counter him. Multiple overlapping pulses must
not multiply events for one victim.

**Art:** broad segmented construct, compact gold chest sun, floating-looking
but rigid connected forearms, ivory stone/gold/navy and team shoulder panel.
A2 wide bright chest plates, A3 large sun collar; B2 dark segmented arms,
B3 angular side fins. Visual gaps use short hidden rigid supports.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Chest sun expands slightly | Deliberate flat-footed steps | One segmented forearm cocks; short hammer strike | Plates compress; torso powers down into crouch | Both palms lower a geometric light stamp; E chest plates open then lock |

One chest/forearm track beyond shared rig. Low replaces fragments with
one impact disk; no multiple delayed simulation explosions.

## R16 — Mandíbulo

**Source:** T10 / V10 first panel. Insects, Tank; descriptive self-defense.
**Shop text:** “A giant beetle times its carapace against incoming damage.”
Text protection is self-only; no group damage-reduction aura or Guard.

**Carapaça Erguida (B):** require a reachable enemy, wind up, then apply
capped self-reduction. It can keep attacking; Root cancels a pending
activation but does not remove an already applied window. B2 arms one
bonus charge when protection expires; next completed enemy-unit primary
consumes it, never Stronghold/secondary/DoT. An unspent charge cannot
accumulate. A improves HP/window, B retaliation/basic damage.

Rampart gives Guard, Grumak ramps from actual damage, Mandíbulo gives a
predictable periodic window. This is a deliberate overlap to measure, not
a reason to invent armor physics. Insects movement helps arrival; ranged
focus and Root still work. Compare its low HP/DPS per Energy against
Bone Brute and require useful faction/composition evidence.

**Art:** wide beetle shield shell, prominent mandibles and horn, stout
limbs; moss/amber/black with team side bands. A2 pale layered carapace,
A3 large back shield ridge. B2 darker head and longer mandibles, B3 outer
amber jaw blades. Reuse Large stance with local mandible props.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Mandibles click softly, shell breathes | Broad low steps | Head retracts, one jaw pulls back; horizontal bite/shove | Shell tilts; legs fold under shield | Shell lowers over shoulders; on B retaliation jaws open wide; E unfolds crest |

At most paired mandible tracks; Low freezes decorative plates and keeps
the defensive pose and active-window indicator.

## R17 — Louva

**Source:** T10 / V10 second panel. Insects, Dps.
**Shop text:** “A mantis earns a decisive fourth cut against the same foe.”
No dash, stealth, kill-trigger haste or enemy damage debuff from the PNG.

**Golpe Preciso (B):** count consecutive valid hits on one enemy unit;
fourth adds the bonus and resets to zero. Target switch/death resets it,
cancelled windups do not advance it; no arbitrary timeout is added where
the text gives none. Stronghold attacks do not gain/count the bonus.
A offers cadence/delivery, B a stronger fourth-hit payoff. Distinguish
from Rasgarr, whose persistent ramp loses value on an inactivity timer.

Sawbot gives greater ordinary DPS; Louva's fourth-hit cycle compensates
only if she survives and maintains contact. Mandíbulo/Fiandeira help;
Rootcaller, Hexer, Siege Walker and high-HP Tanks remain counters. Test
bonus damage into Second Wind and shifting summon targets so one branch
does not become strictly superior in every composition.

**Art:** tall narrow mantis, folded scythe forearms, triangular face and
antennae; moss/amber/black, team abdomen band. A2 swept pale leg guards,
A3 long antenna crest; B2 dark thick blades, B3 broad serrated elbows.
Humanoid joints can drive bent insect legs without adding six limbs.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Forearms folded in prayer, antennae tilt | Springy but planted high knees | One blade unfolds against chest; fast opposing cuts | Head recoils; folds into crossed blades | Fourth cut crosses both blades at contact; E unfolds to full height |

One antenna secondary track; Low fixes it and retains crossed fourth-cut
pose. No root-motion leap or simulation invulnerability.

## R18 — Vespira

**Source:** T10 / V10 third panel. Insects, Ranged; descriptive attrition.
**Shop text:** “A wasp's measured stings leave one persistent venom effect.”
Text poison is not image AoE/Slow. Flight appearance grants no immunity.

**Ferrão Venenoso (B):** third valid enemy-unit primary applies the single
Vespira poison slot. Shared by all Vespiras per victim, refresh-only with
preserved pulse phase, source death does not stop it. Distinct from
Rotcap† stacks; Brew† can cleanse/ward it by the explicit new contract.
Stronghold shots are ordinary, never poison. A improves pulse damage;
B cadence/range. No unlimited swarm entities or targets.

Very low HP and modest damage at her Energy cost make performance against
unshielded Archer an essential underpower test. Mandíbulo buys time;
Rootcaller/range/area attacks deny it. Compare total poison delivered and
wasted refreshes, not a fabricated sum from every source. Colony movement
must not become stealth/flight access to protected targets.

**Art:** slender wasp abdomen, stinger-like ranged weapon, opaque faceted
wing pairs and antennae; amber/black/moss with team abdomen plate. A2
green venom sacs and pale thorax, A3 long stinger casing. B2 swept wings/
dark thorax, B3 tall fin pair. Use painted thin geometry, no transparency
or wing shadow light.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Abdomen tilts, wing pair hums slowly | Light planted steps with body bob | Abdomen coils, arm/stinger aligns; sharp sting-bolt release | Wings tuck; abdomen lowers then tips | Venom sac flashes on third shot; E spreads wing silhouette |

Wing/abdomen presentation tracks only. Low holds wings at diagonal rest,
retains venom status icon and ordinary ground/team marker.

## R19 — Fiandeira

**Source:** T10 / V10 fourth panel. Insects, Control; descriptive Support.
**Shop text:** “A spider pins one enemy long enough for allies to capitalize.”
No image-derived second root target, allied shield or general web zone.

**Teia Adesiva (B):** windup then single primary damage plus ordinary
Root. B3 queues one follow-up Slow when Root truly ends; if another Root
extends it, wait; if cleansing removes Root, discard the queue. Pending
Slow obeys ward/standard priority on transition, never overwrites a new
Root. This bounded extra state exists only for B3. A improves uptime,
B reach/post-root control. Ordinary attacks are low damage; no healing.

Rootcaller is cheaper in control cadence terms and much stronger as a
continuous Root source: Fiandeira's sparse special must justify its reach
and branch shape without a fabricated stun. Louva profits from the window;
Siege Walker and ranged pressure punish her. Test multi-Fiandeira refresh,
Purification and Antidote† interactions to prevent permanent locks.

**Art:** large abdomen, eight clearly grouped legs, low head with green
eyes, web-spinner nozzle; moss/amber/dark chitin, team back plate. A2 pale
abdomen bands and wider spinner, A3 broad web-shaped back crest. B2 dark
body/longer forelegs, B3 angular nozzle fork. No simulated web geometry.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Front legs feel ground, abdomen breathes | Alternating four-leg groups | Forelegs brace, abdomen raises; small fang/spinner flick | Legs recoil inward; abdomen settles in curled legs | Spinner points and shoots one short thread; E raises abdomen crest |

Local leg/spinner tracks are required; shared animator still evaluates
them. Low groups leg movement and removes thread curls, retains Root cue.

## R20 — Crematumba

**Source:** T11; no supplied associated image. Undead, proposed Tank;
secondary anti-heal area damage. **Shop text:** “Profane flame burns every
unit ahead, including your own.” Missing base values remain explicit
completion hypotheses. Friendly fire is the immutable creative identity.

**Fogo Profano (B with reviewed friendly-damage reuse):** flame replaces
normal attack output, with the basic cadence/windup. Engage an ordinary
enemy/Stronghold in range; at resolution scan all other living units in
the forward one-dimensional segment, including same-x units and allied/
enemy summons. Never distinguish teams in damage/cut strength. Self is
excluded; no lateral cone math or target cap. Stronghold receives only
the proposed primary basic hit, never collateral/cut. Damage/cut apply
independently to surviving victims; no lifesteal, Essence or on-hit procs
from flame. Friendly conventional deaths credit the opponent under the
existing death-owner rule, never the caster. No free Bounty farming.

A increases HP/range and therefore friendly-fire reach; B increases damage/
cut on **both** sides. Neither removes friendly fire. Bloomkeeper behind
it may support it; a Tank placed ahead is endangered. Compare Hexer/
Siege Walker, Bombard† and Blood Furnace† are all on the integrated base.
Automatic positioning makes this a serious fun/product test; the client
must show the actual forward danger segment. No manual facing/reposition
system is introduced to make the concept easier to play.

**Art from text:** hunched funerary armored burner, coffin furnace pack,
wide nozzle and rigid hose; bone/rusted iron/charcoal, restrained green
flame and team straps. A2 large coffin breastplate and pale shoulder,
A3 long nozzle/more visible furnace rim. B2 cracked dark canister and
green valve guards, B3 prominent triple nozzle shroud. No image invented.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Furnace shoulder rises, nozzle lowers | Hunched short dragging steps | Both hands brace nozzle, chest inhales; short repeated flame pulse | Pack tilts, nozzle dips; knees and furnace fold | S is the flame pulse itself, no second ability; E valves lock into new shroud |

No fluid/fire simulation. Reuse one cheap tapered flame mesh with short
segments, fixed to authoritative facing; Low drops smoke/sparks while
retaining the exact danger segment and friendly-hit feedback. Dense
emitter armies require CPU and event-bandwidth benchmarks before release.

## R21 — Brocatanque

**Source:** T12; no supplied associated image. Machines, Ranged;
descriptive summoner. **Shop text:** “A tracked factory launches temporary
drills that fuse into stronger machines.” No direct tank damage; no
zero-damage Hit event. The three named drills are one bounded summon
family, not three new Shop purchases or purchasable Evolutions.

**Ninho de Brocas (C):** at production readiness, require an enemy unit or
enemy Stronghold in printed production range under ordinary Guard-first
selection. Spawn after a six-tick production windup; if target disappears,
cancel without replacement. Spawn one Microbroca at producer x. Full cap
at start or resolution leaves it ready, spawns nothing, and never queues
a burst. Producer holds while engaged; drills use ordinary melee movement/
Guard/attack resolution and may damage Strongholds normally. No homing
projectile, collision body or summon-generated summon. Its lifetime and
upgrades snapshot at birth, independently of later producer death.

**Fusion contract:** after movement/hits and before new production is
resolved, collect living unexpired drills by seat, at most the family cap.
Visit eligible same-rank pairs in `(lowest id, next id)` order within the
registry distance. Consume each parent at most once per tick; output may
merge again next tick, never recursively now. New larger id, midpoint x
rounded down in milli-tiles, same seat, next rank. Megabroca never merges.
Output HP is `floor((parent HP sum) × output max HP / parent max-HP sum)`,
minimum one: preserve weighted fraction, never restore full health.
Use the lower-id parent's recorded producer/branch/tier profile, so cross-
producer merges cannot pick whichever upgrade is better in each field.
Parent cooldown maximum is retained, clamped to the output interval;
cancel parent windups with no free output attack on merge tick. Rank TTL
restarts as in the text, but only two promotions are possible.

**Effect inheritance:** no merge cleanse. Preserve the stronger Root/Slow
with maximum remaining duration; inherited Root cancels movement/attacks.
Merge poison/fish stack lists from the reviewed baseline deterministically
by strength descending, remaining duration descending, then parent id/
slot, retaining each family's existing cap. For each new one-slot poison/
bleed/cut/weakening use highest strength, earliest next pulse, longest
remaining duration/pulse count; no summing. Carry the larger existing
shield amount and its own expiry, never add them; equal shields take
later expiry. Discard positive haste/bond/carrier links: drills are not
eligible for them under this proposal. Any effect introduced later must
define inheritance before fusion is allowed with it. Merge removal is not
Death, grants no Bounty/on-kill/shrapnel, and replacement does not consume
another cap slot. Brew† cleanses normally before the merge phase.

A favors production and a small post-merge shield, B higher-rank output
against printed high-HP units, never Stronghold damage bonus. Eight-cap
is per seat across all producers/ranks, independent of Nester's cap and
unaffected by Beetling-only Augments. Do not apply Arcanas/Synergy/Bounty/
Evolution to drills. Existing shields are assigned before A3's smaller
shield under the strongest-wins rule. A new producer does not multiply
the family cap. Source death leaves drills until TTL, no unbounded scaling.

**Matchups:** Nester already offers temporary numbers, so fusion must
create the distinctive payoff; a no-fusion alternative needs owner
approval. Siege Walker/Bombard† and timely control counter the small
drills; shields/support cannot turn merging into free HP or status removal.
At rank cap, continued production competes for space rather than feeding
infinite promotions. Measure cap-blocked time, merges/rank distribution,
damage before expiry and actual investment recovery.

**Art:** low tracked hull, industrial honeycomb launcher, thick treads,
one cannon/nozzle and forge vents; iron/brass/oak and team hull panels.
A2 widens hatch rack and lightens side armor, A3 prominent twin feed
housing. B2 thickens drill barrel/dark armor, B3 broad siege shroud.
Microbroca is a small pointed mechanical tick, Duobroca two fused shells,
Megabroca broad multi-bladed body; rank reads by silhouette, not scale alone.

| I | W | P / A | H / D | S / E |
|---|---|---|---|---|
| Hull rocks as launcher valve settles | Rigid treads visually cycle, hull stays on x | Hatch opens and launcher compresses; launch gesture instead of a damage shot | Hull jolts; nozzle sags and treads stop | Drill fusion shells close/open on new-rank cue; E hatch rack unfolds |

Drill I/W use low body bob and drill rotation; P/A use a forward bit
brace/thrust at normal contact; H/D are chassis tilt/bit stop; merge is a
short shell transition, never a physics explosion. Local tread/hatch/bit
nodes are required. Low replaces tread cycling with hull bob and freezes
idle drill rotation, preserving movement, rank, TTL and HP indicators.

## Comparative value worksheet (models, not measured balance)

These arithmetic ratios use §13 base stats only, without effective HP,
Synergy, DR, target uptime, summons, specials or modifiers. They expose
weak hypotheses and are not a universal balance formula. “DPS/E” is raw
basic DPS per Energy; R20 is per victim and R21 has no direct DPS.

| ID | HP/E | DPS/E | Reference and reason the price must be tested |
|---|---:|---:|---|
| R01 | 233.3 | 2.50 | Rampart 250 / 3.89 with Guard; stance alone may not justify price |
| R02 | 208.3 | 8.33 | Bone Brute 325 / 12.50; delayed area is the investment |
| R03 | 130.0 | 8.53 | Archer 73.3 / 15.00; mixed mark utility pays the gap |
| R04 | 57.5 | 7.35 | Archer cheaper and longer reach; mobility/split shots matter |
| R05 | 180.0 | 6.32 | Bone Brute stronger raw frontline; independent bomb is value |
| R06 | 260.0 | 3.80 | Plants sustain plus control, not damage efficiency |
| R07 | 208.3 | 4.58 | Multi-target bleed must survive approach and counter-healing |
| R08 | 266.7 | 3.33 | Rampart Guard versus reactive self-defense |
| R09 | 77.5 | 10.00 | Sawbot 57.8 / 13.10; same-target ramp and extra HP |
| R10 | 65.0 | 5.83 | Low ordinary DPS; secondary count must earn its cost |
| R11 | 76.0 | 1.50 | Hexer 75 / 5.18 with frequent stronger Slow; likely weak |
| R12 | 210.0 | 3.50 | Shield ceiling is prevention, not guaranteed healing |
| R13 | 75.0 | 1.50 | Modest basic-damage prevention; likely weak outside compositions |
| R14 | 52.5 | 6.88 | More expensive than Archer; shield counter is conditional |
| R15 | 116.7 | 2.54 | Siege Walker 140 / 6.80 plus ordinary area; likely weak |
| R16 | 233.3 | 3.75 | Self-window cannot replace Guard in every matchup |
| R17 | 70.0 | 10.42 | Fourth-hit bonus versus Sawbot's ordinary DPS |
| R18 | 52.0 | 4.67 | Refresh-only DoT wastes overlapping sources; likely weak |
| R19 | 87.5 | 1.50 | Rootcaller 75 / 3.13 with ordinary Root; sparse burst utility |
| R20 | 133.3 | 4.00 | Per victim; subtract full allied harm and cut externalities |
| R21 | 180.0 | 0.00 | Unit output is capped, expires, and cannot be priced as infinite DPS |

Utility “payback” must use the existing value lab's Energy shadow price
and measured additional HP removed/prevented or wins, including time
alive and banked-Interest opportunity cost. No kill-energy refund is
assumed. Compare each branch against same-Energy base purchases and
the other branch, with support/Tank/summon/control contexts. If a proposal
has no distinct useful matchup, tune locally using evidence and return
product identity conflicts to the owner; do not discard a character or
invent another mechanic to make a weak sheet look complete.
