# P58 — Source inventory, engine audit and integration contracts

**Planning only.** Audited on 2026-10-30 against remote `main`
`6ef155bc9540527c99086aee0d9726908ca6a041`; **reaudited on 2026-10-30 against
`origin/main` `e3b977f`** (this branch, `phase/roster-expansion`, is based on
`phase/units-fixes` `e2cc2b1`, which contains `origin/main`). Sections 2, 3, 4,
5 and 7 were reaudited; section 2.2 lists what changed. Nothing in this document
authorizes implementation. Read with the [phase](../phases/P58-roster-expansion-planning.md),
[catalog](P58-CATALOG.md) and inactive [numerical registry](../GAME-DESIGN.md#13-p58-roster-expansion--proposed-inactive).

## 1. Source authority and inventory

The owner's clarification on 2026-10-30 is binding: **text controls names,
Faction, Archetype, functions, abilities and numbers; images are visual
reference only for a future 3D phase, in every case.** Draft prose is design
input, not authorization to execute its embedded suggestions. All twelve
named text files and all nine named PNGs were accessible and read/inspected.
No original image is imported into the game or this PR.

The source manifest below records SHA-256 prefixes; these identify files,
not Git objects. External paths are provenance, not portable repository links.

| Source | External text file | SHA-256 prefix | Characters |
|---|---|---|---|
| T01 | `/data/downloads/p-1-imagem` | `ff6cd8354fcddaf5` | Bastion; fragment, not a complete sheet |
| T02 | `/home/iiii/p2-imagem` | `b88f877ac069f927` | Demolidor |
| T03 | `/home/iiii/p3` | `939d06da0ab73d12` | Crowcaller |
| T04 | `/home/iiii/p4-imagem.txt` | `3ab0041deb929f6d` | Gatilho |
| T05 | `/home/iiii/p5-imagem` | `9a4829610ba050e7` | Matraca & Tico; one purchased unit |
| T06 | `/home/iiii/p6-imagem` | `4a1f2145c82066ca` | Capivardo |
| T07 | `/home/iiii/p7-imagem` | `8682f7fa6c6eb211` | Tridente |
| T08 | `/home/iiii/p8-4-persoagens-juntos` | `cdb10e9cbdbcaa8d` | Grumak, Rasgarr, Esporax, Murmúrio |
| T09 | `/home/iiii/p9-4-personagens juntos` | `d151a8c0b9b41435` | Sentinela Solar, Oráculo Luminar, Arqueira Celeste, Justiçar |
| T10 | `/home/iiii/p10-4personagens-juntos` | `bc55e75bd03e5ad6` | Mandíbulo, Louva, Vespira, Fiandeira |
| T11 | `/data/downloads/personagem-novo-1.txt` | `631b1b3fbb00c04e` | Crematumba |
| T12 | `/home/iiii/personagem-novo-2.txt` | `f19ec6250b1c1e10` | Brocatanque; Microbroca, Duobroca and Megabroca are summons, not three additional purchases |

| Image | External PNG under `/data/downloads/` | SHA-256 prefix | Association and limits |
|---|---|---|---|
| V01 | `Bastion_ Guardião da Muralha.png` | `54aff61ace400186` | Bastion: stone shell, straps, heavy tool; ignore all printed gameplay |
| V02 | `Demolidor_ Ficha do Colosso Mecânico.png` | `da1f5aad099a5710` | Demolidor: hydraulic fists and backpack |
| V04 | `Gatilho_ O Fora da Lei.png` | `9f8228755ba29654` | Gatilho: feline gunslinger; its printed Faction and ability do not apply |
| V05 | `Matraca_ O Caos em Dobro.png` | `007117ad2a6c5c1` | Matraca & Tico: two brothers, one body hierarchy |
| V06 | `Tamanduá_ Guardião da Floresta.png` | `27f62490d5fc98bb` | Capivardo: capybara anatomy and belly; neither the name Tamanduá nor Creatures is adopted |
| V07 | `Tridente_ Shark Warrior Character Sheet.png` | `6d47f1cdeb2e35c0` | Tridente: shark head, dorsal fin, trident |
| V08 | `Monstros_ Swamp Warrior Faction Lineup.png` | `bd5e59e7e02071cf` | Shared hide, bone, weathering and green light only; not four new characters |
| V09 | `Celestial Faction Character Roster.png` | `63b7796ea443287b` | Corresponding four text names, ivory/gold geometry and role silhouettes |
| V10 | `Insetos_ Heróis da Colônia.png` | `ef8d28562ac55c32` | Corresponding four text names, chitin, mandibles and wings |

**Reconciliation.** There are **21 purchased character concepts**, three
new Factions and three drill summon forms. Bastion also names an existing
Stronghold kind and a Rampart Evolution: use a distinct future unit id and
qualify the UI as “Bastion — unit” where context is insufficient. Capivardo
and Tamanduá are one concept; Matraca and Tico are one unit. Creatures is
not a new Faction: the texts explicitly put Gatilho in Humans and Capivardo
and Tridente in Plants. The V08 labels Troncão, Lançafúria, Sombraj aré
(printed Som brajaré/Sombrajaré) and Xamágua are **image-only labels**, not
aliases or extra units. No ability, statistic or crocodilian anatomy from
those panels overrides Grumak's quadruped, Rasgarr's six limbs, Esporax's
split jaw or Murmúrio's floating jellyfish body.

**Missing information, not missing characters.** T01 starts at its Evolution
table: no base Faction, cost, stats or complete Muralha Viva rule survives
in that fragment. Its Machines/Tank placement and base ability below are
explicit completion proposals, requiring owner confirmation. T11 lacks
cost, HP, movement, basic-attack treatment and numerical base forms; T12
lacks precise merge resolution, summon melee range and a complete two-step
Evolution specification. Several group drafts give only one benefit per
branch; the registry assigns it to steps and labels completions. No named
image for Crematumba, Crowcaller or Brocatanque was supplied; T03 mentions
an image, but it cannot be confidently associated with any of the nine
PNGs. Their visual briefs come from text. Unnamed download images were not
guessed into associations. A future artist needs these external references
available again or an owner-approved portable reference location; this PR
does not silently vendor multi-megabyte PNGs.

## 2. Main audit: behavior actually present

Paths below are relative to the reaudited `origin/main` (`e3b977f`), unless
labelled branch. Rows changed by the 2026-10-30 reaudit are listed in §2.2.

| System | Confirmed implementation and practical consequence | Evidence |
|---|---|---|
| Determinism | Integer simulation, 20 Hz, 16,000 milli-tile BattleLine, ascending UnitId, PCG streams; no 2D collision or physics | `crates/tbg-core/src/{content,types,rng,state,sim}.rs` |
| Movement | Speed is milli-tiles **per tick**; allies overlap. A nearby enemy causes the unit to stop, not circle or flank | `combat.rs::advance_unit`, `targeting.rs::engage` |
| Targeting | Guard prioritizes Rampart in range, then nearest enemy by absolute x distance/id; Stronghold only when strictly nearer. “Priority hunter” does not mean backline selection | `targeting.rs::enemy_in_range` |
| Attacks | Six-tick windup, locked target, alive check at resolution without another range check. Root clears windup and prevents movement/attacks | `combat.rs::advance_unit` |
| Status | One Slow/Root slot. Slow affects movement **and attack cadence**; Root has priority. There is no independent stun or airborne status. Old Finn fish is a movement-only slow stack list in `UnitFx` (`combat.rs`), outside this slot | `types.rs::Status`, `hits.rs::apply_on_hit`, `combat.rs` |
| Area damage | Radius on x around primary; ordinary splash hits enemy units and potentially the Stronghold. Secondary targets do not automatically receive primary on-hit status or conditional bonuses | `hits.rs::splash` |
| Healing | Lowest injured HP fraction in range, including self/summons; friendly target in `resolve_hit` means healing, not damage. Direct heals may purify Slow/Root via Arcana | `targeting.rs::most_damaged_ally`, `hits.rs::{resolve_hit,heal}` |
| Damage | Damage reduction from Augments/Arcanas capped at 20% (`hits.rs` `.min(2_000)`, `augment.rs` `bp.min(2_000)`), then Second Wind then HP. Unit absorption: `UnitFx.shield`/`shield_until` exist and `hits.rs::absorb_shield` consumes them, but **no code writes them**, so no unit shield is ever granted. Stronghold shield exists (`PlayerState.shield`, LastStand) | `hits.rs::{damage,absorb_shield}`, `augment.rs`, `state.rs::{UnitFx,PlayerState}` |
| Heal reduction | One per-victim heal-cut pair (`Unit.heal_cut_until`/`heal_cut_bp`), written by the Augment heal cut (`hits.rs::apply_heal_cut`) and the Deep Wound form (`apply_form_heal_cut`), read by heal paths (`augment.rs`, `support.rs`) and Plants regeneration (`synergy.rs`). Afflictions now exist: poison stacks (`UnitFx.poison: [Stack; 6]`, Rotcap) and cleanse/brew (`afflictions.rs::cleanse`). No bleed slot, no temporary attack-weakening slot, no per-caster mark | `hits.rs`, `augment.rs`, `synergy.rs`, `afflictions.rs`, `state.rs` |
| Death attribution | Every conventional death feeds the opposite seat's eligible-kill hooks; no killer/source on Hit. Friendly damage needs deliberate reward rules | `combat.rs::remove_dead`, `state.rs::Event` |
| Summons | Two summon kinds: Beetling (hatched by a Nester; per-Nester and per-seat caps; lifetime) and Crate (Smuggler cargo; uncapped; BR-U08). `UnitKind::is_summon` covers both; `FX_SUMMON` marks them in views. No drill, merge or summon-family code exists (no `drill` or `fusion` symbols in `tbg-core`) | `summon.rs`, `types.rs::is_summon`, `view.rs` |
| Evolution | Base is stored tier 0; two branch-locked purchases become tiers 1/2, preserving HP fraction. Forms accumulate base-relative stat deltas | `intent.rs::evolve`, `ruleset.rs::form_of`, `state.rs::refresh_max_hp` |
| Factions | Exactly four enum values and fixed arrays of four. Two/four living conventional units, duplicates count; faction-only bonuses | `types.rs`, `synergy.rs`, `ruleset.rs`, `view.rs` |
| Economy | Start 5 Energy; 17/600 Energy per tick; +25% regeneration per full 5 banked, max multiplier 2.25; no gameplay cap, no clock-decided victory | `content.rs`, `sim.rs::regen_energy`, D-41/D-43 |
| Shop | 25 purchasable kinds: 10 original plus 15 expansion kinds (D-50); `CANON.pool = &FULL_POOL`. Five distinct uniform offers (`content.rs` `OFFERS = 5`); each purchase redraws all five, including the bought kind (BR-S04). `UNIT_KINDS` (10 kinds) is a legacy constant still used by tests, lab knobs and the client gallery, not by the Shop. No timed rotation or active Reserve | `types.rs::{FULL_POOL,UNIT_KINDS}`, `ruleset.rs`, `shop.rs`, `intent.rs::purchase` |
| Augments | Forty definitions, repeated 60-second rounds, paused combat, four options; bounded stat formulas, factions/archetypes/kinds as filters | `augment.rs`, `content.rs`, `sim.rs::step` |
| Arcanas | Fifteen nodes, twelve-point preset; conditional roles and hit-time bonuses; summons excluded | `arcana.rs`, `ruleset.rs`, `state.rs::stats_of` |
| Strongholds | Classic reference plus four selectable kinds (Treasury, Bastion, Artillery, Command; default Bastion), individual Fortifications and Command aura. **Name collision:** the Stronghold kind `Bastion` (D-46) and planned unit R01 Bastion share a name; see the owner decisions in the phase | `stronghold.rs`, `content.rs`, `types.rs`, `intent.rs::Fortify` |
| UnitFx (integrated #16) | Per-unit `Copy` block of fixed arrays, no allocation: `poison: [Stack; 6]`, `fish: [Stack; 3]`, `confused_until`, `immune_until`, `ward_until`, `betray`, `attacks: u8`, `essence`, `essence_progress`, `plates`, `plate_hp`, `march_move_bp`, `march_dr_bp`, `march_ignore_slow`, `surge_until`, `betray_bp`, `betray_stumble`, `bond`, `carrier`, `passenger`, `board_lock`, `shield: u16`, `shield_until`. Stored in `Unit.fx`; views expose only the `FX_CARRIED` and `FX_SUMMON` bits | `state.rs` (`UnitFx` at line 66, `Unit.fx` at line 136), `view.rs` |
| Arenas (D-55) | `Arena` enum: Forest (neutral), Tundra, Volcanic, Ruins, Storm (`types.rs`); drawn by the server at pairing; `MatchView.evolve_costs`; replay stores the arena (`replay.rs`) | `types.rs`, `replay.rs`, `protocol.rs` (v15) |
| Cargo Crab (D-56) | `Ability::Carry { board, lock, dr_bp, move_bp, shield_bp, surge_bp }` (`content.rs`, branch rows near line 744); carrier and passenger in `UnitFx`; carried units are not targetable by targeted effects (`targeting.rs`); `FX_CARRIED` (bit 14) in `UnitView.fx`; base carry speed boost 2 000 bp and Overdrive surge (`support.rs::carry_move_bp`). `shield_bp` is parsed by `support.rs::carry_of` and **ignored by both callers**. The carry bonus rule is D-60 (in progress, another branch) | `support.rs`, `content.rs`, `targeting.rs`, `view.rs` |
| Authority and network | `PROTOCOL_VERSION` 16 (`protocol.rs`; v15 arenas, v16 `FX_CARRIED`). Postcard over WebSocket binary frames; `MAX_CLIENT_MSG_BYTES` 1,024 (`protocol.rs`) set as the ws max message and frame size (`tbg-server/src/ws.rs`); seat stamped by connection. Per-seat 10 Hz views (`match_task.rs`) plus acknowledgments and reconnect catch-up | `protocol.rs`, `tbg-server/src/ws.rs`, `match_task.rs` |
| Wire/state | `UnitView`: id, seat, kind, tier and branch (form), x, hp, max_hp, a 0/1/2 status byte, and `fx: u16` (only `FX_CARRIED` bit 14 and `FX_SUMMON` bit 15 are set). No unit shield amount, poison, fish or bond in views. `Event::Hit` has no source. `MatchView` has fixed four-Faction arrays (`synergy: [[u8; 4]; 2]`). Full state is hashed by `sim.rs::hash` | `view.rs`, `state.rs::Event`, `sim.rs::hash` |
| Replay | Seed, presets, Stronghold kinds, arena (`replay.rs`) and intentions keyed by selection clock, not paused combat tick; runs on compiled CANON, not an arbitrary historical ruleset | `replay.rs` |
| Rendering | Bevy 0.20 WebGL2; code-built low-poly parts, vertex colors, shared material, mesh per joint, Humanoid/Large plans. No enabled glTF/skeletal animation import pipeline | `client/Cargo.toml`, `art/{mesh,rig,figure,roster}.rs` |
| Motion and effects | Seventeen body joints plus secondary/prop nodes, procedural pose sampling and clips. Main cues choose shared Archetype clips, with special Archer tuning; AttackStart/Hit/Heal/Evolved/Death drive cues and simple projectiles | `art/{rig,anim,clips,archer_v2_clips}.rs`, `visuals/cues.rs` |
| Lab | Ruleset variants, duels, policy matches, paired seat swaps, meta/search/confirm, value model, Arcana/Stronghold studies. Purchase-share gate: 2.5 % per kind (`gate.rs`, O-R2). Not human fun evidence | `tbg-balance/src/{cli,duels,runner,gate,search,value}*` |
| Tests/benchmarks | Main has rules, protocol, determinism, economy, Augment/Arcana/Stronghold, open-army and lab contracts; `examples/army_load.rs` and `scripts/capacity.sh` exist | `core/tests/`, `balance/tests/contract.rs`, `.github/workflows/ci.yml` |

**Document drift is real.** Main's glossary still calls several live
Augment features inactive, describes a fixed twelve-unit MVP target, and
contains retired Siege language. The later rule sections and executable
code above govern this audit. This PR appends inactive proposals; it does
not repair unrelated canon history or reinterpret obsolete numbers.

### 2.1 Work outside remote main

The initial workspace was clean on `phase/p30d-server-performance`
`bf9ba40`, which includes local merge commits. That is **not** the remote
main baseline. An isolated worktree was created from freshly fetched main;
no existing worktree or branch was overwritten.

| State | Inspected evidence | Consequence |
|---|---|---|
| Integrated roster (was non-integrated) | `release/roster-online` at `d06279aa5287b67efb24e224c4710db55a8ca739` (PR #16) is an ancestor of `origin/main` `e3b977f` (checked 2026-10-30). Local `main` `077abe7` does not contain it | Fifteen purchased kinds, Ability pairs, UnitFx, poison/fish, friendly blasts, formation, bonds, cleanse, transport, crates and transfers are on the reaudited base. The review and re-audit gate is still open (see the dependency below) |
| Earlier plan | `docs/ROSTER-EXPANSION.md` on that same ref; P42–P48B reports | Reuse its foundations after integration/review; P49–P53 are reserved follow-up milestones, not fresh numbers |
| Troop polish | `phase/a1-troops-port` at `e820ebc`, open [PR #11](https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/pull/11) | Directional aim/death styles are not baseline assumptions |
| Art kits | P54 is marked done and owner-approved (2026-10-30) in ROADMAP. The old branch `phase/p54-unit-art-kits` and PR #15 were not re-checked | Per-kind kits are in the tree (`tbg-client/src/art/kits/`) |
| Champion kits | P55 is marked done and owner-approved (2026-10-30) in ROADMAP; the local branch `phase/p55-champion-kits` was not re-checked | Champion kits share the kit library (`tbg-client/src/art/kits/`) |
| Other planning | P41 i18n and P47 spectating PRs; local themed-arena branch | Preserve their work. Existing P47 numbering collision is outside this change; no new P42–P55 allocation |

**Chosen dependency:** implementation starts only after the roster
foundations in #16 are reviewed and re-audited on the implementation base.
Reaudit finding: #16 is already in `origin/main` (`e3b977f`), so the
integration step is done; the review and the implementation-base re-audit
are not. Do not reimplement its Ability/UnitFx/Shop-pool/poison/cleanse
systems in parallel. If the reviewed #16 code later changes or is reverted,
stop before implementation and revise this dependency; the task does not
authorize porting it. P54/P55 integration gates their reuse for final art,
not independent offline combat work.

### 2.2 Reaudit changes (2026-10-30)

| Item | Old (6ef155b audit) | Reaudited fact | Evidence |
|---|---|---|---|
| Shop kinds | Ten purchased kinds | 25 (`CANON.pool = &FULL_POOL`); the 10-kind `UNIT_KINDS` is legacy | `ruleset.rs`, `types.rs` |
| Protocol | Version 13 | `PROTOCOL_VERSION` 16; the lead's Part A (D-61) bumps it before Part C; no further number reserved | `protocol.rs` |
| Unit shield | Does not exist on main | Field and absorb path exist; no writer; Cargo Crab `shield_bp` ignored | `state.rs`, `hits.rs`, `support.rs` |
| PR #16 | Open, not integrated | In `origin/main` | `git merge-base --is-ancestor` |
| Fixed-size-4 Faction arrays | Not listed | Ten sites, listed in §5 | grep of `tbg-core` and `tbg-client` |
| Summon kinds | Beetling only | Beetling and Crate | `types.rs::is_summon` |
| Heal cut | Augment-only fields | Augment and Deep Wound write one pair; plus Plants regeneration reads it | `hits.rs`, `synergy.rs` |
| Status | No movement-only slow | Old Finn fish is one (`UnitFx`) | `combat.rs` |
| UnitView | kind/form/x/HP/status | Adds tier, branch, max_hp, fx bits 14 and 15 | `view.rs` |
| Arenas and Cargo Crab | Absent | D-55 and D-56 present; D-60 (in progress) on another branch | `types.rs`, `content.rs` |

Rows not listed were checked by grep for the named symbols and are
unchanged. Movement, Attacks, Area damage and Death attribution were not
re-read line by line. PR states for #11 and #15 were not re-checked (no
network check in this docs-only task).

## 3. Shared competitive contracts (all Proposed, inactive)

These close ambiguities for future execution; they do not change main.
Every numerical hypothesis lives in GAME-DESIGN §13.

### 3.1 Units, timing and target selection

- Store integer HP/damage, milli-tile distances, speed per tick, basis
  points and combat ticks. Convert draft speed by rounding `tiles/s × 50`
  to nearest integer, halves upward; registry lists the resulting value.
  All arithmetic products widen to i64, floor positive division, clamp
  before narrowing, and saturate timers/id allocation rather than wrap.
- One mechanical Archetype per unit. Secondary roles in the catalog are
  descriptive, never a second Arcana/Augment eligibility. Display “Tier 2”
  means stored tier 1; “Tier 3” means stored tier 2. There are five forms,
  not a new third purchase. Costs remain the existing 2 then 4 Energy,
  with existing discounts and minimum cost. Evolution carries counters,
  target locks and remaining cooldowns unchanged; new parameters affect
  future resolutions. Existing DoT/shield values stay as applied.
- Basic targeting/windup/Guard stay as main; no backline bypass, kiting,
  dash, teleport, flight immunity or knockback. Special offensive primary
  selection uses the same Guard-first candidate order but requires an
  enemy **unit**, never automatically selects a Stronghold. Area and
  secondary selection have their own explicit restrictions below.
- Timer abilities use a separate fixed cooldown and pending action, not a
  general skill scheduler. Initially wait a full period after spawn.
  Once ready, require a legal target and no pending special, start a
  six-tick windup, and set next readiness to start + period. No target:
  remain ready. Root cancels pending special; an already consumed period
  is not refunded. Dead/removed primary at resolution cancels it without
  retarget or refund. A living locked primary is not range-checked again.
  Existing basic attacks continue independently, as Tico requires.
- Resolve basic then special for each living unit in ascending UnitId.
  Area centers read current primary x at resolution, or caster x where
  specified. Enumerate living affected units in UnitId order. Units killed
  earlier that tick cannot cast or receive another effect. New summons
  begin acting next tick, not recursively in their birth loop.
- Counted attacks count completed primary hits on enemy units (including
  summons), not cancelled windups, heals, friendly hits, Stronghold shots
  or secondary effects. Same-target sequences reset on target change or
  target disappearance; a Root cancellation adds no hit. Ready-on-next-hit
  marks wait indefinitely for that valid hit and consume readiness once.

### 3.2 Damage, debuffs, healing and presentation

- Basic hits retain their existing modifiers. Auxiliary damage uses its
  registry amount scaled by the caster's static damage modifiers once;
  victim damage reduction/shield/Second Wind still apply. Auxiliary
  damage triggers no lifesteal, on-hit, counters, Essence, Bounty beyond
  normal death accounting, or recursively generated casts. Damage is
  never expressed as a friendly healing target.
- Timer/secondary area attacks hit enemy summons and conventional units,
  not allies or Strongholds unless a sheet explicitly says otherwise.
  Normal basic attacks still damage Strongholds. Area damage includes a
  transported passenger (integrated #16 rule); single-target secondaries
  respect `targetable` (`targeting.rs`), which excludes carried units.
- Slow/Root reuse the existing single-slot priority, duration modifiers,
  Unshaken/Purification and control immunity once available. Every Slow
  below affects attack cadence too. No animation cancels enemy attacks.
  Debuff immunity/ward from the integrated baseline is checked for each
  new hostile status; raw damage is still taken.
- Added heal-cut uses its own bounded ability slot alongside the existing
  single heal-cut pair (`Unit.heal_cut_*`, written by the Augment and Deep
  Wound effects): take the highest **active** cut, never sum them. Across
  applications of that ability slot, stronger replaces and restarts its
  timer; equal refreshes; weaker is ignored until expiry. Never extend a
  strong cut by repeatedly applying a weaker one. It affects direct heals,
  regeneration and HP transfer only through the reviewed existing heal
  path; it does not reduce temporary shields or resurrect. New slots are
  cleansed/warded by explicit policy in §4, not silently ignored.
- Temporary attack weakening is one separate slot: same stronger/equal/
  weaker rule; applies to basic attack damage at hit resolution, not heal
  amount, poison/bleed already applied, or special damage. Sum reductions
  before applying the existing damage formula, with the registry bound.
- A non-stacking DoT family has one victim slot (value, next pulse, pulses
  remaining, source seat). First pulse is one second after application;
  exactly the registry pulse count. Refresh active equal/stronger effects
  without resetting next pulse, reset remaining count, and take strongest
  value; weaker applications do not refresh. Survives source death. Poison
  and bleed have separate slots and never take Rotcap's stack capacity.
  Pulse before decrement/removal of its last due pulse; no on-hit effects,
  lifesteal or Stronghold damage. Healing coexists normally, subject only
  to an explicitly applied heal-cut. `afflictions.rs::cleanse` already strips
  existing poison stacks; cleansing the new one-slot DoT before ordinary
  poison stacks is a Proposed contract that needs a new test.
- Per-caster Crowcaller marks store one target and deadline on the caster,
  preventing unbounded source lists on victims. Only that caster gains
  its bonus; expired/dead target clears the mark. Casting on another unit
  transfers its own bonus, without stripping existing victim Slow/cut.
- Timed self-reduction adds to existing reduction and shares its 20% cap;
  no separate multiplication or bypass for area damage. Healing does not
  reset cooldowns or resistance stacks. Same-source duration uses combat
  time; timers, DoTs, summons and fusion freeze in Augment selection.
- Full reconnect views must show current shield amount, relevant effect
  flags/expiries, special readiness, drill rank and life remaining. Events
  alone cannot reconstruct persistent effects after reconnect. Plan an
  appended ability-start/resolution cue carrying caster, primary/center,
  ability discriminator and combat tick, plus shield/fusion cues where
  applicable. Existing Hit/Heal/Death/Evolved remain; the client never
  derives a competitive hit from animation timing. `PROTOCOL_VERSION` is 16
  (`protocol.rs`); the lead's Part A (D-61) bumps it before Part C, and this
  plan reserves no number beyond that.

## 4. Integration matrix

Category is for the **chosen primary ability relative to the reaudited base (`origin/main` `e3b977f`)**:
A = existing data only; B = bounded, localized hooks/state using existing
primitives; C = structural dependency. New Faction infrastructure is an
additional shared dependency, not counted as an extra ability. Thus **0 A,
20 B, 1 C**. Reaudit: R12 and R14 move from C to B, because the unit shield
fields and absorb path are already in the tree (only the grant path, expiry
and view field remain). R21 stays C. The 18 B / 3 C count of the 6ef155b audit
is superseded. All 21 reuse some primitives;
that overlap is not another disjoint category count.

Module names below are future destinations; they do not exist in the reaudited base:
`roster_actions.rs` (timers/counters), `roster_marks.rs` (marks/cuts),
`drill.rs` (summon fusion). Add only those responsibilities actually needed,
splitting if cohesion requires; do not build a generic ability framework.

| ID / unit | Cat. | Reuse / smallest required extension | State / scans / wire | Required scenario; main risk |
|---|---|---|---|---|
| R01 Bastion | B | Normal Tank attack + capped reduction; anchored self-window | cooldown/pending/expiry; O(1) window check | No Guard substitution; stationary window vs heal-heavy defense |
| R02 Demolidor | B | Damage + Slow; explicit unit-only area resolver | timer; one O(n) scan per cast | Primary dies; no launch/cancel/Stronghold splash |
| R03 Crowcaller | B | Basic hit, Slow, heal-cut; source-specific mark | caster target/deadline + victim cut; no mark-source list | Two casters/cuts/Root; anti-heal becoming mandatory |
| R04 Gatilho | B | Primary attack + one deterministic secondary | bounded attack counter; O(n) only on fourth hit | One enemy/Guard/passenger; overlap with Esporax |
| R05 Matraca & Tico | B | Independent timer area + Slow, one Unit | timer/pending; O(n) per bomb; visual extra torso | Club kills bomb target; no Tico entity or phantom stun |
| R06 Capivardo | B | Unit-only damage + Slow + capped self-window | timer/expiry; O(n) per cast | Plants regeneration + defensive Augments stall |
| R07 Tridente | B | Area hit; separate one-slot bleed | timer + bounded victim DoT; O(n) pulses/casts | Refresh cannot defer final pulse forever; no armor debuff |
| R08 Grumak | B | Post-HP-damage hook + capped reduction | stack count/deadline, O(1) per incoming hit | Zero/absorbed hit, stack next-hit timing; healing wall |
| R09 Rasgarr | B | Primary-hit damage modifier | last target/count/last-hit tick; O(1) | Fourth-hit cap and target reset; damage growth bounded |
| R10 Esporax | B | Counted secondary strikes | count, fixed best-three selector O(n) on proc | Tie and radius; not a cheaper Siege Walker |
| R11 Murmúrio | B | Ordinary Slow | special timer; standard target scan | Slow refresh vs Root; control saturation |
| R12 Sentinela Solar | B | Allied target selection; grant path for the existing `UnitFx.shield` field (absorb path exists, no writer) | shield/expiry and timer; O(n) select; shield amount needs a view field | No shield accumulation, transfer or healing classification |
| R13 Oráculo Luminar | B | Enemy target + damage-debuff hit hook | victim weakening slot + timer | Overlap/expiry/cancel; output debuff + shields stall |
| R14 Arqueira Celeste | B | Counted empowered primary; the strip reads the existing `UnitFx.shield`, so it needs R12's grant path | counter, inspect shield once | Strip before hit; never strip HP/plates/Stronghold shield |
| R15 Justiçar | B | Single area pulse, not simulated meteors | timer; O(n) per cast | One hit per victim; no triple image explosion |
| R16 Mandíbulo | B | Capped self-reduction + optional empowered next hit | timer/expiry/one ready flag; O(1) | Cap shared with Arcana; no protective aura from image |
| R17 Louva | B | Same-target counted bonus | target/count; O(1) | Fourth-hit reset vs Rasgarr's continuing ramp |
| R18 Vespira | B | Counted hit + non-stacking poison; reuse integrated poison stacks and cleanse order | counter + poison slot; O(n) pulse pass | No Rotcap-stack multiplication; Brew counter |
| R19 Fiandeira | B | Root + damage; post-Root Slow only on B3 | timer + one queued Slow per victim | Queued Slow clears on cleanse; longer Root delays transition |
| R20 Crematumba | B | Direct damage/cut; reuse the integrated friendly-damage path | normal attack cadence; O(n) each pulse | Same-damage friendly fire, no heal dispatch/reward exploit |
| R21 Brocatanque | C | Summon ownership/TTL; new deterministic capped rank-fusion lifecycle | per-owner fixed eight-id candidate list, rank/kind, merge cues | Full-cap production ordering, damaged merge, no recursive summon |

**Shared files:** `core/src/{content,ruleset,types,state,sim,combat,hits,
targeting,summon,synergy,view,protocol,replay}.rs`, integrated afflictions
and support modules, `balance/src/{policy,metrics,knobs,duels,gate}*`,
`client/src/{art,visuals,sync,ui}*`, server handshake and replay/result
metadata tests. This is an impact map, not permission to edit them now.

### 4.1 Structural alternatives and decisions

- **Unit shields (R12/R14):** the integrated #16 `UnitFx.shield` and
  `shield_until` fields and `hits.rs::absorb_shield` exist, so the resource
  is no longer new. Nothing grants a unit shield yet: the grant path, the
  expiry sweep and the view field are the missing pieces. The Cargo Crab
  `shield_bp` (`content.rs`, P48-A design) is parsed and ignored. Planned
  semantics: one amount and expiry per victim, strongest replaces, equal
  refreshes, weaker ignored; expires without HP loss, consumes before HP,
  cannot revive and does not count as healing. Shielding a lower shield with
  a stronger shield replaces the amount, never adds it. Sentinel A/B
  shielding and Arqueira stripping use this same field once. The timed
  damage-reduction substitute from the 6ef155b audit is no longer needed for
  the resource and stays unselected. **Owner decision:** confirm these
  semantics, and whether the dormant Cargo Crab `shield_bp` is granted by the
  same path or removed.
- **Brocatanque:** fusion is its central fantasy; a no-fusion capped
  temporary drill producer is the small alternative, but removes that
  identity. Retain the structural proposal for a separately authorized
  final wave; do not ship a substitute as if approved. Fusion details are
  in the catalog, shared summon exclusions remain BR-U08.
- Image-only airborne hits, taunts, chain piercing, invisibility, true
  flight, armor reduction, healing pools and large protective auras are
  **not proposals** in this plan. The text already gives viable abilities;
  no physics, area persistence, global armor system or stealth framework
  is needed to recreate the pictures.

## 5. Factions, arrays and compatibility

Append Monsters, Celestials and Insects **after** Humans/Undead/Machines/
Plants, preserving existing discriminants. Seven Factions require an audit
of every `Faction::index`, exhaustive match and `[...; 4]` that actually
means Factions (do not replace four Shop/Augment option arrays). Update
FACTIONS, Synergy tables, Ruleset, PlayerState, MatchView, client icons/
labels/selection, lab faction-policy parsing and results, Augment filters,
Convergence and serialization fixtures together in the future contract
change. One derived count from FACTIONS is enough; no registry framework.

**Reaudit (2026-10-30) of the arrays that W1 must generalize.** Grep of
`crates/tbg-core/src` and `crates/tbg-client/src` for fixed arrays of four:

- Faction arrays: `types.rs` `FACTIONS: [Faction; 4]` (line 213);
  `synergy.rs` `counts = [0u32; 4]` (line 13); `content.rs`
  `SYNERGIES: [(SynergyEffect, [i32; 2]); 4]` (line 271); `ruleset.rs`
  `Ruleset.synergies` (line 69); `state.rs` `PlayerState.synergy: [u8; 4]`
  (line 156); `augment.rs` `unit_bonus` parameter `synergy: [u8; 4]` (line 288);
  `view.rs` `MatchView.synergy: [[u8; 4]; 2]` (line 97); `sim.rs` synergy
  initializer `[0; 4]` (line 68); client `visuals/assets.rs`
  `FACTION_COLORS: [[u8; 3]; 4]` (line 25); client `art/palette.rs`
  `FACTIONS: [[Color; 8]; 4]` (line 87).
- Not Faction arrays, leave as they are: `AugmentRule.costs` and
  `AUGMENT_COST: [u32; 4]` (Augment prices, `ruleset.rs` line 47,
  `content.rs` line 38); `StrongholdKind::SELECTABLE` (`types.rs`);
  `forms: [FormDef; 4]` (`content.rs` lines 458 and 491); the
  `profiling::snapshot` result; the Synergy level thresholds
  `synergy_thresholds: [u8; 2]`.
- `SynergyEffect` has four variants, `MaxHp`, `AttackSpeed`, `Damage` and
  `Regen` (`content.rs` line 262). Insects' movement Synergy needs a new
  variant. Movement is already capped at 150 % of base speed (`state.rs`).
- Magnitudes differ. Existing level 1/2 values are MaxHp 20/40 %,
  AttackSpeed 20/40 %, Damage 5/20 % and Regen 75/200 bp. The planned
  Monsters damage, Celestials max HP and Insects movement use 5/10 %; that
  is the owner decision in the phase doc.
- `tbg-balance/src/knobs.rs` imports `FACTIONS` for faction-policy parsing.

| Proposed Faction | Narrative / visual | Archetype distribution | Proposed Synergy and constraints |
|---|---|---|---|
| Monsters | Primordial kin, bone armor, dark hide, turquoise organs; adaptation without mimicry | Tank / Dps / Ranged / Control | Damage 5/10% at 2/4 alive; a completion hypothesis, not in T08. No regen or reactive army-wide stacking |
| Celestials | Order/protection, ivory armor, geometric gold halos, dark blue cloth | Tank / Control / Ranged / Control; Oráculo has descriptive Support role | Max HP 5/10%, as T09; replaces first bonus, not cumulative. Shields are finite; no passive healing |
| Insects | Colony discipline, moss/amber chitin, asymmetry of mandibles/wings/web | Tank / Dps / Ranged / Control; Fiandeira has descriptive Support role | Movement 5/10%, as T10; extend only the existing Synergy effect list/formula, sharing +50% move cap; never grant flight or summon Synergy |

Monsters need time on a target and lose to control/focus fire; Celestials
lose damage tempo to their protection and lose shields to Arqueira;
Insects reach combat quickly but remain vulnerable to area damage/control.
Mixed-Faction units receive their own Synergy only. All conventional
counts still include duplicates; new Factions can activate Convergence,
but never multiply its bonus per Faction. Existing faction-specific
Augments do not automatically apply to the three new Factions. No new
Augment/Arcana catalog is proposed. Review the lack of faction-specific
Augments in the lab instead of inventing parity bonuses.

Appending enums alone is not wire compatibility. The future release must
bump the then-current protocol, reject older handshakes, deploy matching
client/server, validate kinds/forms and preserve private per-seat data.
Record rules/content version and replay format where the integrated base
supports them; old replays must run with their original binary/rules, not
be claimed compatible because enum order survived. New state fields change
serialized hashes; compare repeated runs within a version and semantic
old-roster traces across versions, not identical old hash bytes.

## 6. Interaction matrix and exploitation scenarios

“Main” below names available opponents; † opponents require reviewed #16.
Every new unit also receives the common duel/support/Tank/summon/control/
Evolution/Arcanas/performance matrix in the phase.

| Combination / scenario | Required resolution or counterplay | Risk / future lab case |
|---|---|---|
| Bastion + Bloomkeeper + Sentinel | Anchoring never grants Guard; finite shield/DR; enemy Siege Walker still attacks normally | High stall; `bastion-wall` |
| Demolidor / Capivardo / Matraca vs Nester | Damage and Slow enumerated once per living victim; no displaced x or real stun | Group-value explosion; `pulse-summons` |
| Crowcaller + Hexer + Rootcaller | One Slow/Root slot; mark bonus/cut continue until expiry, Root itself not overwritten by Slow | Control/anti-heal density; `crow-control` |
| Crowcaller + Crematumba + Grievous Wounds | Highest active cut, no addition; weaker cut cannot preserve stronger indefinitely | Heal negation; `cut-overlap` |
| Gatilho + Esporax vs Rampart | Primary honors Guard; secondary is a bounded distinct unit; protected Cargo passenger† excluded | Backline bypass by accident; `secondary-guard` |
| Tridente + Rotcap† + Vespira | Three bounded independent DoT families; no shared stack theft or proc chains | Burst memory/event count; `dot-mix` |
| Vespira / Tridente vs Antidote Brewer† | New one-slot DoT cleanse order and ward explicit; periodic damage already dealt is not undone | Counter contract drift; `brew-new-dots` |
| Grumak + Mandíbulo + Shieldwall | Each self-reduction joins 20% cap; area attacks do not bypass it | No indestructible wall; `dr-cap` |
| Rasgarr vs Louva against Bone Brute | Rasgarr ramps and maintains; Louva expends fourth-hit burst then resets | Avoid one strictly dominating; `same-target-melee` |
| Murmúrio + Fiandeira | Root priority, no queued Slow during cleanse; status-duration Augments/Arcanas bounded | Root-lock and long games; `web-fear` |
| Sentinela + Sentinela + Oráculo | Shields replace, never add; weakening is bounded; no resource becomes permanent HP | Critical stall; `celestial-loop` |
| Arqueira vs Sentinel / Cargo shield† / Stronghold shield | Strip only temporary Unit shield before empowered hit; not plates, max HP, Synergy, Stronghold shield or passengers she cannot target | Unintended universal dispel; `purification-boundary` |
| Justiçar vs packed Tanks and supports | One pulse per cast at primary x; no image-derived multiple meteors | Area value scaling; `justicar-clump` |
| Fiandeira + Old Finn† + Hexer | Main Slow affects attacks; fish† movement-only remains separate; cleansing clears queued web Slow too | Permanent slow by transitions; `control-chain` |
| Crematumba + Bloomkeeper / Bloodweaver† | Friendly HP damage and cut are real, including allied summons; no negative healing, lifesteal or Essence from friendly hits | Critical reward/HP-transfer exploit; `crema-friendly` |
| Crematumba + Bombard† + Shatterback† | Both friendly blasts can break plates; process existing shrapnel once, through non-recursive queue | Trigger explosion; `friendly-plates` |
| Brocatanque + Nester / Smuggler† | Drill cap separate from Beetling cap; drills never summon, evolve, count for Synergy or Bounty | Wrong kind-based summon classification; `summon-families` |
| Brocatanque + shields/DoTs + merge | No HP-fraction recovery, cleanse, cap evasion or timer reset outside catalog merge contract | Critical scaling; `merge-afflictions` |
| Every new Dps/Tank + conditional Arcanas | Primary Archetype alone governs eligibility; descriptive dual role grants nothing extra | Hidden double bonuses; `role-eligibility` |
| Seven Factions + Convergence | Boolean two-or-more condition, not a bonus per new Faction | Global stat spike; `seven-factions` |
| Any new unit reconnecting during Augment pause | Timers/fusion frozen; full public effects reconstructed, no opponent offers/Energy/RNG | State divergence/privacy; `reconnect-effects` |

## 7. Cost and performance reasoning

Main targeting is already O(n) per acting unit, so worst-case combat can
be O(n²); do not describe the existing engine as linear. New ordinary
timers/counters add O(n) storage/timer work; area procs scan O(n) only on
their event. Select at most three secondaries with a fixed best-k buffer
in one pass, not a sorted heap/Vec per proc. DoT pulses use a single bounded
victim pass, not a source-by-victim cross product. New per-unit state must
stay fixed size; measure actual `size_of`, serialized hash bytes, view
bytes, allocations and memory rather than claiming a guessed byte count.

Crematumba is the hottest uncapped ability: each half-second emitter
visits every candidate; dense all-Crematumba armies can add O(n²) work.
Measure before authorization of its online wave. Use the current scan
first; optimize only a measured bottleneck, preserving victim order.
Fusion considers at most eight drills per seat, at most 28 pairs, plus
one O(n) collection pass. Never compare every unit pair or allocate a new
world search per drill. Caps apply to the family, not each producer.

Rendering cost is per animated joint/prop, not per nominal character.
Merge static detail into existing joint meshes, share material handles,
cache five form meshes once, and avoid lights/particle emitters per unit.
Extra spider legs, Tico's torso and drill rotation need local presentation
nodes, not simulation entities. Low quality may drop smoke, spring motion,
halo fragments and secondary wing movement; never hide team shape, HP,
shield, target, flame danger interval or drill rank. Desktop checks only,
following the current owner platform rule. No fixed triangle/draw-call
ceiling overrides D-28/D-30; measure those diagnostics and enforce existing
frame/memory/download contracts.

Reaudit note (2026-10-30): `UnitFx` is a `Copy` block of fixed arrays and
scalars (`state.rs`), so its size is fixed by construction. Its size, the
serialized hash bytes and the view bytes were not measured in this
docs-only task (no cargo run); the measurement rule above still applies
before any wave is authorized.

No new performance measurement in this plan proves the unimplemented
roster fast or balanced. The phase specifies baseline and before/after
workloads, including hundreds of units and maximum procs, before release.
