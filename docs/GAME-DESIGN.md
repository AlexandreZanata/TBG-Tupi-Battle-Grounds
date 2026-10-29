# Game Design — Tupi Battle Grounds

> Authoritative game rules. Legacy rules are preserved from the archived
> design (sources S1–S5 below); MVP additions are marked **Proposed** and
> every number in §5 is a **new assumption** to be tuned by playtests.
> Technology lives in [ARCHITECTURE.md](ARCHITECTURE.md), never here.

## 1. Identity and pillars

**Tupi Battle Grounds (TBG)** is an open-source, browser-first, real-time
1v1 strategy game: automatic combat on a single lane, a continuous
five-offer shop, branching unit evolution, and faction synergies. No
micromanagement — depth comes from what you buy, when you deploy, how you
evolve, and which factions you combine.

| # | Pillar (confirmed legacy, S1) |
|---|-------------------------------|
| 1 | Live 1v1 PvP: both players act simultaneously; the board evolves in real time |
| 2 | One `BattleLine`: no lateral movement |
| 3 | Automation: units move and fight without player input |
| 4 | Continuous `Shop`: offers rotate during combat; no preparation phase |
| 5 | Branching `Evolution` of deployed units |
| 6 | Faction `Synergy` bonuses |
| 7 | Authoritative server owns every competitive outcome |
| 8 | Free-to-play, cosmetics only; never pay-to-win |

## 2. Glossary (ubiquitous language — use exactly)

| Term | Meaning | Code name |
|------|---------|-----------|
| TBG | Tupi Battle Grounds, the product | `TBG` |
| Match | One live session between two players, from start to outcome | `Match` |
| BattleLine | The single lane where units deploy and fight | `BattleLine` |
| Unit | Deployable character with `Archetype`, `Faction`, stats, `Energy` cost; not a skin | `Unit` |
| Archetype | Mechanical role: `Tank`, `DPS`, `Support`, `Ranged`, `Control` | `Archetype` |
| Faction | Visual language + synergy group: `Humans`, `Undead`, `Machines`, `Plants` | `Faction` |
| Energy | The only in-match resource; regenerates; not a currency | `Energy` |
| Shop | Per-player rotating offer of five distinct units during a match; not a store (**Proposed**, D-32) | `Shop` |
| Evolution | In-match branching upgrade of a deployed unit; not account progression | `Evolution` |
| Synergy | Bonus while enough units of one `Faction` (or a defined combo) are alive | `Synergy` |
| PlayerIntention | Client request (purchase, evolve, reposition); a request, never a state change | `PlayerIntention` |
| AuthoritativeServer | The process that owns combat, Energy, shop timers, outcomes | — |
| Stronghold | **Proposed (new):** each player's base at their end of the `BattleLine` | `Stronghold` |
| Cosmetic | Visual item with no gameplay effect | `Cosmetic` |
| Inventory | Account-bound collection of owned cosmetics | `Inventory` |
| Commander | Player-facing name for the player (TBD by brand voice) | `Player` |
| MVPUnitRoster | Launch target of 12 units; 2.0 ships the first 8 first | — |
| Account | **Proposed (P11):** a player identity — one `PlayerName` + one `AccountKey`; holds `Rating` and statistics | `Account` |
| PlayerName | **Proposed (P11):** unique public name chosen at account creation | `PlayerName` |
| AccountKey | **Proposed (P11):** the only credential: `PlayerName-XXXX-XXXX-XXXX-XXXX` | `AccountKey` |
| MatchMode | **Proposed (P12):** `Casual` (no rating change) or `Ranked` (accounts only, rating changes) | `MatchMode` |
| Rating | **Proposed (P12):** an account's ranked skill number (Elo) | `Rating` |
| Leaderboard | **Proposed (P13):** global ranking of accounts by `Rating` | `Leaderboard` |
| StrongholdStyle | **Proposed (P18):** cosmetic look of a `Stronghold`: Stone Keep, Timber Fort, Gatehouse | `StrongholdStyle` |
| Arcana | **Proposed (P32–P38, extra after P31):** a free pre-match choice that modifies a documented unit attribute | `ArcanaNode` |
| ArcanaPreset | **Proposed (P32–P38):** six bounded ranks chosen before matchmaking and frozen for a match | `ArcanaPreset` |
| RulesetVersion | **Proposed (P35):** compiled version of competitive rules and balance values, recorded with match results | `RulesetVersion` |
| StrongholdKind | **Proposed (P16-F3, not active):** a free pre-match choice of `Stronghold` with its own stats and passive; mechanical, unlike `StrongholdStyle` | `StrongholdKind` |
| Fortification | **Proposed (P16-F3, not active):** in-match branching upgrade of a player's own `Stronghold`, paid with `Energy`; never called `Evolution` | `Fortification` |
| Interest | **Proposed (P16-F1, not active):** periodic `Energy` paid on the lowest balance held since the previous payment | `Interest` |
| Bounty | **Proposed (P16-F1, not active):** `Energy` granted for an eligible kill, bounded per payment window | `Bounty` |
| Siege | **Proposed (P16-F1, not active):** the match phase after the running phase in which both `Stronghold`s suffer `Collapse`; replaces overtime | `Phase::Siege` |
| Collapse | **Proposed (P16-F1, not active):** symmetric structural loss of a percentage of each `Stronghold`'s max HP per second during `Siege` | `Collapse` |
| Reserve | **Proposed (P16-F2, not active):** one `Shop` offer a player holds out of rotation for a limited time | `Reserve` |
| Augment | **Proposed (P16-F5, not active):** an in-match bonus bought with `Energy` from an `AugmentRound`; not an `Arcana` (pre-match) and not a store item | `Augment` |
| AugmentRound | **Proposed (P16-F5, not active):** a timed offer of three `Augment`s, identical for both players | `AugmentRound` |
| Forgo | **Proposed (P16-F5, not active):** declining or letting an `AugmentRound` expire, compensated with `Energy` | `Forgo` |

Retired legacy names — never reuse for new concepts: `CharacterDefinition`,
`CharacterCatalog`, `CharacterStudio(Window)`, `ProceduralCharacterBuilder`,
`Mesh3DCharacterFactory`, `MaterialProfile`, `BehaviorProfile`,
`CharacterTextures`, `CharacterAssetMode`, `VisualGrammar`, `CharacterInstance`.

## 3. Business rules

Status: **Confirmed** (legacy, unchanged) · **Adapted** (legacy concept,
2.0 wording) · **Proposed** (new for the MVP, needs playtest validation).
Sources: S1 ADR-001 · S2 ADR-002 · S3 infrastructure-overview · S4 legacy
GLOSSARY · S5 legacy unit schema (all under `archive/docs/` or
`archive/content/characters/definitions/`).

| ID | Rule | Status | Src |
|----|------|--------|-----|
| BR-M01 | A match is a live session between exactly two players | Confirmed | S1 |
| BR-M02 | No sub-rounds or preparation phases; shopping and combat overlap | Confirmed | S1 |
| BR-M03 | A disconnected player gets a reconnect window, then forfeits; the match keeps running meanwhile | Adapted | S1, S3 |
| BR-M04 | Each player has a `Stronghold`; destroying the enemy `Stronghold` wins | Proposed | — |
| BR-M05 | Fixed match clock: regulation, then overtime with faster `Energy`; tie after overtime is a draw (replaced by BR-M07–M11 when P16-F1 activates, §10) | Proposed | — |
| BR-M06 | A player may surrender while the match is in countdown, regulation, or overtime; it is a forfeit (BR-M03): the opponent wins and, in ranked, it is a loss (BR-R04) | Proposed | — |
| BR-B01 | Combat resolves on one lane; units never move laterally | Confirmed | S1, S4 |
| BR-B02 | A unit enters the lane only through a validated purchase-and-deploy intention; MVP deploys at the buyer's `Stronghold` | Adapted | S2, S3 |
| BR-B03 | Allied units pass through each other; a unit stops when an enemy is within its range | Proposed | — |
| BR-U01 | Deployed units move and fight automatically | Confirmed | S1 |
| BR-U02 | Every unit has one `Archetype`, one `Faction`, a stat block, and an `Energy` cost | Confirmed | S4, S5 |
| BR-U03 | ~~Every unit has a `SlotCount` of 1 or 2~~ | **Removed — D-32** | — |
| BR-U04 | Damage, deaths, and results are computed only by the authoritative simulation | Confirmed | S1, S2 |
| BR-U05 | Cosmetics never change stats, hitboxes, or behavior | Confirmed | S1 |
| BR-U06 | ~~A player's living units may occupy at most `SLOT_CAP` slots~~ | **Removed — D-32** | — |
| BR-U07 | There is no limit on how many conventional units a player may own; only `Energy` gates purchases | Proposed (D-32) | — |
| BR-U08 | A summon is a temporary unit created by a unit's ability: it costs no `Energy`, is never sold, never evolves, never summons, neither counts toward nor receives `Synergy`, is not part of the army count, and disappears when its lifetime ends | Proposed (D-32) | — |
| BR-U09 | Guard: an enemy unit or `Stronghold` whose attack range reaches a living Rampart must target the nearest such Rampart | Proposed (D-32) | — |
| BR-U10 | A server-protection entity limit, if ever needed, is documented as server protection with measured evidence (ARCHITECTURE), never as an army rule | Proposed (D-32) | — |
| BR-E01 | `Energy` is the only in-match resource, distinct from any currency | Confirmed | S4 |
| BR-E02 | `Energy` regenerates over time under server rules, up to a cap (the cap is removed by BR-E04 when P16-F1 activates, §10; owner decision D-35) | Confirmed | S2, S4 |
| BR-E03 | Purchases and evolutions exceeding available `Energy` are rejected, never queued or owed | Adapted | S1 |
| BR-S01 | Each player sees exactly five offers, five distinct units at any time | Proposed (D-32; was three) | S1, S4 |
| BR-S02 | Shop rotation timers come from the server match clock, identical for both players | Confirmed | S1, S4 |
| BR-S03 | The shop sells units only, never stats or advantage | Confirmed | S1 |
| BR-S04 | A purchased offer refills immediately; all offers rotate on a shared timer | Proposed | — |
| BR-S05 | Shop randomness is seeded server-side and never revealed to clients | Proposed | — |
| BR-V01 | Deployed units evolve along a branching tree; the tier-1 choice conditions tier 2 | Confirmed | S4 |
| BR-V02 | Evolution is paid with `Energy` in the match, never with out-of-match purchases | Adapted | S1, S4 |
| BR-V03 | Evolution never applies to cosmetics | Confirmed | S1 |
| BR-Y01 | Enough same-`Faction` units alive together grant a bonus | Confirmed | S2, S4 |
| BR-Y02 | Synergy rules are data owned by the simulation, identical for both players | Adapted | S2 |
| BR-A01 | Clients send only `PlayerIntention`s | Confirmed | S1, S3 |
| BR-A02 | The server validates every intention against Energy, shop, evolution, and match state | Confirmed | S2, S3 |
| BR-A03 | Clients may predict for responsiveness and must reconcile on rejection | Confirmed | S1 |
| BR-A04 | Client-reported damage, balances, and results are never trusted | Confirmed | S2, S3 |
| BR-C01 | Monetization is free-to-play with cosmetics only | Confirmed | S1 |
| BR-C02 | Persistent cosmetics are bound to an account `Inventory` | Confirmed | S1 |
| BR-C03 | Store purchases are granted only after server-side receipt validation | Adapted | S1, S2 |
| BR-C04 | `StrongholdStyle`s are free cosmetics; every style has identical gameplay (styles stay cosmetic; mechanical `StrongholdKind`s are separate, BR-K01–K08, D-37) | Proposed | — |
| BR-P01 | An `Account` is a unique `PlayerName` plus an `AccountKey`; names are 3–16 letters, digits or `_`, start with a letter, are unique ignoring case, and never change | Proposed | — |
| BR-P02 | The `AccountKey` is the only credential: shown once at creation, never shown to other players, not recoverable | Proposed | — |
| BR-P03 | One active session per account; a new login takes over the old one, including a running match | Proposed | — |
| BR-P04 | Guests may play practice, casual matches, and private rooms; ranked play needs an account | Proposed | — |
| BR-P05 | A public profile shows name, rating, rank, record, win rate, faction mix, and the last 10 ranked results; no other personal data is stored | Proposed | — |
| BR-R01 | Two queues: `Casual` (guests and accounts; private rooms are casual) and `Ranked` (accounts only) | Proposed | — |
| BR-R02 | Matchmaking pairs the two longest-waiting players of the same queue, regardless of rating | Proposed | — |
| BR-R03 | Every ranked match changes both players' `Rating` from both ratings (Elo, §5); win 1, draw ½, loss 0 | Proposed | — |
| BR-R04 | A forfeit (including staying disconnected past the window) is a ranked loss and a win for the opponent | Proposed | — |
| BR-R05 | New accounts start at 1000; a rating never goes below 100; the first 20 ranked games move faster | Proposed | — |
| BR-R06 | Ratings and results are computed and stored only by the authoritative server | Proposed | — |
| BR-R07 | The `Leaderboard` lists accounts with ≥ 1 ranked match, ordered by rating, then wins, then name | Proposed | — |

## 4. How a match plays (MVP)

1. **Start.** Two players are paired. 3-second countdown; both start with
   the same `Energy` and their own five shop offers.
2. **Buy = deploy.** Tapping an affordable offer sends a `Purchase`
   intention. If valid, the unit spawns in front of the buyer's
   `Stronghold` and the offer refills. Rejections: insufficient `Energy`,
   stale offer, match not running. There is no army cap (BR-U07).
3. **Automatic combat.** Units walk toward the enemy `Stronghold`. A unit
   stops when its target is in range: the nearest enemy ahead (unit or
   `Stronghold`; ties broken by lowest unit id). It attacks on its
   cooldown. `Stronghold`s shoot the nearest enemy in range.
4. **Roles.** `Tank`: high HP, short range. `DPS`: high damage, fragile.
   `Ranged`: long range. `Support`: heals the lowest-HP% ally in range
   instead of attacking. `Control`: hits apply a status — *slow* (move
   and attack speed reduced) or *root* (cannot move or attack). Statuses
   do not stack; a new one replaces the old if stronger, else refreshes.
   *Splash* damages every enemy within a radius of the primary target.
5. **Evolve.** Tapping one of your living units opens its evolution
   choices. Tier 1 offers two branches; tier 2 offers the single upgrade
   that follows the chosen branch. Evolution is instant, costs `Energy`,
   and keeps the unit's current HP percentage.
6. **Synergy.** Each faction has bonuses at 2 and 4 living units of that
   faction (every conventional unit counts as one; summons do not count). Bonuses apply to that
   faction's units only, recomputed every tick, preserving HP percentage.
7. **End.** Destroying the enemy `Stronghold` wins immediately. When
   regulation ends, the higher `Stronghold` HP wins; if tied, overtime
   starts with doubled `Energy` regeneration. A tie after overtime is a
   draw. A player who stays disconnected past the reconnect window, or
   who surrenders (two taps in the HUD), forfeits. The result screen
   offers **Play again**.

## 5. MVP starting values (all **new assumptions**, tune via playtests)

Bot-harness balance pass of 2026-10-28 (P7) is applied below; human
playtests decide the final values (decision D-14).

Distances in tiles, times in seconds. The simulation stores them as
integers (see ARCHITECTURE.md §4).

| Constant | Value | | Constant | Value |
|----------|-------|-|----------|-------|
| BattleLine length (Stronghold to Stronghold) | 16 | | Countdown | 3 |
| Regulation | 180 | | Overtime | 60 |
| Energy start / cap | 5 / 10 | | Energy regen | 1 per ~1.76 s (17 sub-energy/tick; overtime 34) |
| Shop offers / full rotation | 5 distinct / every 10 (**Proposed**) | | Army cap | none (BR-U07) |
| Stronghold HP | 4500 | | Stronghold attack | 65 dmg / 1.0 s, range 6 |
| Evolution cost tier 1 / tier 2 | 3 / 4 | | Reconnect window | 30 |

**Roster (10 units + 1 summon, working names).** Every archetype appears.
Bone Brute, Siege Walker, and Rampart are drawn with the large body plan
(visual only since D-32). Nester, Rampart, and the Beetling are **Proposed**.

| Unit | Faction | Archetype | Cost | HP | Dmg | Every (s) | Range | Speed | Special |
|------|---------|-----------|------|----|-----|-----------|-------|-------|---------|
| Shieldbearer | Humans | Tank | 3 | 650 | 30 | 1.2 | 0.8 | 0.8 | — |
| Archer | Humans | Ranged | 3 | 220 | 45 | 1.0 | 4.5 | 1.0 | — |
| Bone Brute | Undead | Tank | 4 | 1300 | 75 | 1.5 | 1.0 | 0.6 | — |
| Hexer | Undead | Control | 4 | 300 | 30 | 1.4 | 3.5 | 1.0 | slow 40% for 2 s |
| Sawbot | Machines | DPS | 3 | 270 | 55 | 1.05 | 0.8 | 1.3 | — |
| Siege Walker | Machines | Ranged | 5 | 700 | 85 | 2.5 | 6.0 | 0.6 | splash 1.5 |
| Bloomkeeper | Plants | Support | 3 | 280 | heal 40 | 1.5 | 3.0 | 0.9 | heals instead of attacking |
| Rootcaller | Plants | Control | 4 | 300 | 20 | 1.6 | 3.0 | 0.9 | root 0.8 s |
| Nester | Plants | Support | 4 | 300 | 15 | 1.5 | 2.5 | 0.8 | Hatch: every 4.0 s a Beetling; ≤ 2 alive per Nester, ≤ 6 per player |
| Rampart | Machines | Tank | 5 | 1500 | 35 | 1.5 | 0.8 | 0.5 | Guard (BR-U09) |
| *Beetling (summon)* | Plants | DPS | — | 90 | 18 | 0.8 | 0.6 | 1.4 | lifetime 10 s; inherits the Nester's tier and branch at hatch |

**Display names (Proposed, ART-DIRECTION §5):** `Sawbot` is shown as
*Sawknight* and `SiegeWalker` as *Ogre Archer*; code ids and stats are
unchanged.

**Evolution trees** (`T1 → T2` per branch; modifiers stack on the base).

| Unit | Branch A | Branch B |
|------|----------|----------|
| Shieldbearer | Bulwark +50% HP → Fortress +50% HP, hits slow 30% 1.5 s | Vanguard +40% dmg, +20% speed → Champion +40% dmg, splash 1.0 |
| Archer | Longbow +1.5 range, +20% dmg → Marksman +60% dmg | Volley +40% attack speed → Arrow Rain splash 1.0 |
| Bone Brute | Ossified +50% HP → Bone Wall +50% HP, hits slow 30% 1.5 s | Butcher +50% dmg → Reaper +30% dmg, splash 1.2 |
| Hexer | Withering slow becomes 60% → Doom +60% dmg | Plague splash 1.0 → Pestilence splash 1.5, +40% dmg |
| Sawbot | Overclock +40% attack speed → Shredder +40% attack speed, +30% speed | Plating +60% HP → Juggernaut +60% HP, splash 0.8 |
| Siege Walker | Longshot +2 range → Artillery +50% dmg | Shrapnel splash +1.0 → Concussion hits root 0.8 s |
| Bloomkeeper | Blossom +50% heal → Grove heal splash 1.5 | Bramble +50% HP, gains 25 dmg attack → Thornwarden hits root 0.6 s |
| Rootcaller | Deep Roots root +0.6 s → Strangle +80% dmg | Spores splash 1.0 → Overgrowth +1 range, root +0.4 s |
| Nester | Brood ≤ 3 alive per Nester → Hive hatch every 3.0 s | Venom Beetling hits slow 30% 1.0 s → Blight Beetlings +50% HP, +50% dmg |
| Rampart | Bastion +50% HP → Citadel +50% HP, hits slow 30% 1.5 s | Spiked +60% dmg → Siegebreaker +40% dmg, splash 1.0 |

**Synergies** (living units of the faction: 2 / 4).

| Faction | Bonus to that faction's units |
|---------|-------------------------------|
| Humans | +20% / +40% max HP |
| Undead | +20% / +40% attack speed |
| Machines | +5% / +20% damage |
| Plants | regenerate 0.75% / 2% max HP per second |

**Ranked rating (Proposed, P12; Elo).** Expected score
`E = 1 / (1 + 10^((R_opponent − R_own) / 400))`; change
`round(K × (S − E))` with S = 1 / ½ / 0; a win is at least +1 and a loss at
most −1. Each player uses their own K.

| Constant | Value | | Constant | Value |
|----------|-------|-|----------|-------|
| Start rating | 1000 | | Rating floor | 100 |
| K, first 20 ranked games | 40 | | K afterwards | 24 |

## 6. Is it fun? (measured, not claimed)

Collected from bot-vs-bot runs (balance harness), match records, and
human playtests. Targets are starting hypotheses.

| Quality | Measure | Target |
|---------|---------|--------|
| Match length | Median duration | 2.5–4 min |
| Decisive play | Matches decided by Stronghold kill / overtime / draw | ≥ 60% / ≤ 30% / ≤ 2% |
| Decision frequency | Intentions per player per match | ≥ 25 (one every 4–7 s) |
| Strategic variety | Win rate of any faction-majority army (harness + humans) | 45–55% |
| Roster health | Purchase share of each unit | ≥ 5% |
| Counterplay | Every unit has a documented counter in the roster | 100% |
| Comebacks | Wins by the player behind in Stronghold HP at 2:00 | ≥ 20% |
| Readability | Testers can explain why they lost | ≥ 4 of 5 testers |
| Replayability | Testers pressing Play again after a match | ≥ 50% |

### Balance lab targets (**Proposed**, P16-C, D-33)

Measured by `tbg-balance gate` over seed pairs (both seat orders, same
participants), 95 % cluster-bootstrap intervals, against a pool of
seeded policies that see only their own `MatchView`: `bot` (the shipped
bot), `evolver`, `saver`, `faction-<f>` for every faction (the *strategic*
styles), the probes `heavy` and `cheap` (P16-B spam) and `favor-<unit>`
for every shop unit (the bot, but always buying that unit). Non-bot
policies decide every 5 views (500 ms). §6 rows above are computed over
games between strategic styles. Bots are not humans: a passing gate is
necessary, never sufficient; the owner's playtest decides.

| Row | Target | Required for release |
|-----|--------|----------------------|
| Seat A score, `bot` vs `bot` | 48.5–51.5 % | yes |
| `heavy` vs `bot`, `cheap` vs `bot` | ≤ 55 % each | yes |
| Strongest strategic style, average against the others | ≤ 60 % | — |
| Each `favor-<unit>` vs `bot` | median of all favor rows ± 10 pp | — |
| Energy-equal duels: base forms averaging ≥ 70 % against the other base forms at every budget (20/40/60 Energy) | none | yes |
| Duels: base forms averaging ≤ 30 % everywhere (Support exempt) | none | — |
| Duels: forms that win on average and that nothing beats ≥ 60 % | none | — |
| Duels: Evolved forms losing to their own base form at every budget | ≤ 25 % of forms | — |
| Duels: Evolution branches never better than their sibling | none | — |
| Nash equilibrium of the full pool: support / largest weight | ≥ 3 policies / ≤ 50 % | yes |

**Role invariants (Proposed).** Searched rulesets keep: base range ≤
Stronghold range; cost between 1 Energy and the Energy cap; interval ≥
the minimum; in a faction with a Tank, a Tank has the most HP per Energy;
in a faction with a Ranged unit, a Ranged unit has the longest range; a
zero stays zero (no new ability). Untuned by the lab: Synergy thresholds,
Hatch limits, slow/root effects, and every structural constant.

## 7. Deferred (rules kept, not in the MVP)

- `Reposition` intention: legacy names it but defines no semantics; with
  buy-to-deploy and automatic movement nothing is repositionable yet.
- Buying into a bench and deploying later (MVP merges buy and deploy).
- Shop reroll, offer weighting, duplicate rules.
- Cross-faction synergy combos (allowed by BR-Y01, none defined).
- Remaining 4 units of the 12-unit `MVPUnitRoster`.
- `Inventory`, `Cosmetic` store, receipt validation (BR-C01–C03 stay
  binding: nothing may sell power). Accounts and ranking are planned in
  P11–P13.
- Rematch, spectating, replays, emotes; rating-based matchmaking, seasons,
  and rating decay (revisit with enough players, D-18).

## 8. Retired technology decisions (game rules kept)

| Retired | Replaced by |
|---------|-------------|
| Unity C# client, native app stores | Rust/Bevy browser client (WASM/WebGL2) |
| Nakama + TypeScript match module | Rust authoritative server running `tbg-core` |
| PostgreSQL via Nakama | SQLite for MVP match records |
| Per-character MB-Lab/Mixamo rigging (ADR-003–006; ~8% auto pass rate, ADR-007) | Code-built rigid-part figurines with a shared animation library |
| StoreKit / Play Billing IAP | Deferred; cosmetics-only principle unchanged |

## 9. Arcanas extra — Proposed, not active (P32–P38)

This is an optional extension **after P31**, not a change to the current
MVP or a blocker for any existing phase. Values below are experimental
starting hypotheses. P32 refreshes the evidence on the then-current HEAD;
P34 must validate them before online integration. P38 gates public release.

| ID | Proposed rule |
|----|---------------|
| BR-AR01 | Every player, including a new guest, has all six points and all options immediately; payments, account age, and playtime never grant power |
| BR-AR02 | A complete preset spends exactly 6 points; each of 6 nodes has rank 0–2, costs 1 point per rank, and has no prerequisites; each two-node tree therefore holds at most 4 points; there are no supremes or unlock levels |
| BR-AR03 | Eligibility uses the unit's base `Archetype`, remains unchanged by `Evolution`, and grants no new behavior or missing attribute |
| BR-AR04 | Arcana percentages add to existing percentages on the same base attribute, use wide integer intermediates, and truncate once; preserve current HP percentage when max HP changes |
| BR-AR05 | Preset editing is allowed only while idle; joining either queue or creating/joining a room freezes selection until cancellation or match end; reconnect and account takeover keep the match's original preset |
| BR-AR06 | The server validates ranks, budget, schema, ruleset, and session state; clients send selections, never final bonuses or a claimed seat |
| BR-AR07 | Record both applied presets, their schema, and the RulesetVersion with every result; later tuning never rewrites historical outcomes or Rating |
| BR-AR08 | Provide one active preset per account, recovered across devices, and local storage for guests; a complete default requires no setup; account ownership never changes available power |
| BR-AR09 | Casual and Ranked use identical released Arcana rules; existing Elo, FIFO pairing, account identity, Energy, Shop randomness, Evolution costs, and Synergy thresholds do not change |
| BR-AR10 | The first version changes unit max HP, normal damage, and Bloomkeeper healing only; no Stronghold, cadence, movement, range, slow/root, regeneration-rate, or splash-radius bonuses |

**Node order is a serialization contract**, not alphabetical order.
Names below are English UI names. `DPS` is `Archetype::Dps` in code.

| Index | Tree / node | Base Archetypes | Per rank | Rank-2 maximum |
|-------|-------------|-----------------|----------|----------------|
| 0 | War / Guard | Tank, DPS | +4% base max HP | +8% |
| 1 | War / Assault | Tank, DPS | +3% base damage | +6% |
| 2 | Precision / Piercing | Ranged | +3% base damage | +6% |
| 3 | Precision / Endurance | Ranged | +4% base max HP | +8% |
| 4 | Vitality / Resilience | Support, Control | +4% base max HP | +8% |
| 5 | Vitality / Renewal | Support with an existing heal | +5% base direct/splash heal | +10% |

The candidate default is `[2, 0, 2, 0, 2, 0]`. There are 141 complete
rank vectors. Zero/partial presets are diagnostic or editor states, not
valid online selections. Vitality does not raise Plants regeneration's
percentage; a larger max HP can still change its integer healing amount.
Bloomkeeper's attacking forms keep their healing priority and eligibility.
Zero damage/healing stays zero. Splash inherits the modified damage or
heal amount; on-hit status still affects the primary target only.

For each affected attribute, add percentages by source before scaling:

```text
damage = floor((base_damage + Evolution_flat_damage)
               * (10000 + Evolution_bp + Synergy_bp + Arcana_bp) / 10000)
max_hp = floor(base_hp * (10000 + Evolution_bp + Synergy_bp + Arcana_bp) / 10000)
heal = floor(base_heal * (10000 + Evolution_heal_bp + Arcana_heal_bp) / 10000)
```

**Acceptance hypotheses, not claims of balance.** Against other complete
presets, investigate aggregate scores above 55% or individual matchups
above 65%; report neutral controls separately. Require at least three
competitive, non-dominated presets and a default within 3 percentage
points of the best generalist reference. Use held-out paired seeds,
95% confidence intervals, and the existing §6 fun targets. An unresolved
counterless advantage blocks release; a 50% aggregate alone proves nothing.
Human validation needs at least 8 participants and 24 completed matches;
at least 80% must explain their preset and a relevant reason for defeat.

**Conditional combat proposal, not approved or active.** P32 must first
reproduce root lock and resolve the documented durations. If needed, test
a maximum continuous root episode of 32 ticks (1.6 s), including refreshes,
followed by 8 root-immune ticks (0.4 s). This changes the existing refresh
rule and needs an explicit owner decision backed by measurements. A
refresh must never postpone immunity indefinitely. No root bonus is added.

**Deferred alternatives.** Five deep trees, ten points, supremes, Fortress,
Domination, haste, mobility, and range remain outside this extra. If
Stronghold max HP ever varies, first propose fixed per-match maxima and
compare remaining ratios with `i64(hp_a) * i64(max_b)` versus
`i64(hp_b) * i64(max_a)`, including full-health ties and healing cases.
Do not implement that unused subsystem in P32–P38. Reopen progression only
if the small system fails the strategic-choice/playtest gates.

## 10. Strategic expansion — Proposed, not active (P16-F1–F8)

Planned after P16-D (balance). Nothing in this section is active: each
mechanic enters the `Ruleset` disabled, the balance lab measures it, and
the owner switches it on in its phase (D-34). Values are starting
hypotheses. Existing rules in §3–§6 stay binding until the phase that
replaces them is reviewed. Rule IDs are stable; never reuse one.

**Four properties that are not the same thing** (D-36):

| Property | Meaning | This design |
|----------|---------|-------------|
| No time-up decision | No result is ever decided by comparing HP at a clock time | yes (BR-M07) |
| Ends only by unit damage | Only units and Strongholds can reduce Stronghold HP | **no** — `Collapse` also does during `Siege` |
| Progressive structural attrition | Both Strongholds lose HP over time, symmetrically | yes, from 4:00 (BR-M08) |
| Mathematical termination | Every match ends before a bound computable from the rules | yes (BR-M10), plus a server-protection limit that the contracts prove unreachable (BR-M11) |

### 10.1 Economy (P16-F1)

| ID | Proposed rule |
|----|---------------|
| BR-E04 | `Energy` has no gameplay cap; every *generating* source has a documented bound per window instead (regen per tick, Interest per payment, Bounty per window, Forgo per round) |
| BR-E05 | Interest is paid every 10 s of running time (the Shop rotation ticks), to both players on the same tick |
| BR-E06 | Interest = `min(low / band, max_bands) × per_band`, where `low` is the lowest balance the player held since the previous payment (spending lowers it; gains never raise it); after the payment `low` becomes the current balance |
| BR-E07 | Interest, Bounty and Forgo Energy are added with saturating `u32` arithmetic in sub-energy and integer division (never rounded up); `low` starts at the start balance and the first payment is at 0:10 of running time |
| BR-E08 | An eligible kill is the death by damage of an enemy conventional unit (BR-U08: summons and expiries are never eligible); every Bounty source of a player sums per kill and is capped by the sum of their caps and by a per-player ceiling of 1 Energy per 10-s window (windows reset on Interest ticks) |
| BR-E09 | No `Augment`, `StrongholdKind` or `Fortification` changes base regen, Interest bands, or Shop probabilities |
| BR-E10 | Purchases, Evolutions, Fortifications and Augments are still rejected when `Energy` is short (BR-E03) |

| Constant | Value | Sub-energy (600 = 1) |
|----------|-------|----------------------|
| Start | 5 (unchanged unless P16-D changes it) | 3 000 |
| Base regen running / `Siege` | unchanged / doubled (today 17 / 34 per tick) | — |
| Interest period | 10 s (200 ticks), on the rotation ticks | — |
| Interest band / value per band / max bands | 5 / 0.25 / 5 (max 1.25 per payment at a low of 25) | 3 000 / 150 / 5 |
| Bounty ceiling | 1.0 per player per window | 600 |

Tick order addition: intentions (they lower `low`), then regen, then
Interest on payment ticks, then the window reset.

### 10.2 Victory, Siege and Collapse (P16-F1)

| ID | Proposed rule |
|----|---------------|
| BR-M07 | A match is never decided by comparing Stronghold HP at a time; it ends when a Stronghold reaches 0 HP, by forfeit (BR-M03, BR-M06), or by the server-protection limit (BR-M11) |
| BR-M08 | At 4:00 of running time the phase becomes `Siege`: base Energy regen doubles and every second each Stronghold loses `floor(max_hp × rate / 10 000)` HP, `rate` = 10 bp × (1 + full 10-s steps elapsed in `Siege`) |
| BR-M09 | `Collapse` ignores shields and damage reduction, is never a kill (no Bounty, Mend or Fury), and resolves before units act; if it brings a Stronghold to 0 the match ends that tick with reason `Collapse`; both at 0 on the same tick is a draw |
| BR-M10 | Stronghold healing is bounded per window (BR-X10), so Collapse always outgrows it: with no damage at all, a full Stronghold falls within 140 s of `Siege` start; every match ends before 6:40 of running time (contract test, P16-F1) |
| BR-M11 | Server protection, not a game rule: a match still live at 10:00 of running time ends as a draw with reason `ServerLimit`; a contract test proves the rules cannot reach it |

Collapse is a percentage of each Stronghold's own max HP, so every
`StrongholdKind` and `Fortification` falls at the same pace from the same
HP fraction: endurance builds are rewarded for surviving damage, never for
outlasting a clock. Forfeit, surrender, reconnect windows, ranked Elo
(draw ½) and result persistence are unchanged.

### 10.3 Shop Reserve (P16-F2)

| ID | Proposed rule |
|----|---------------|
| BR-S06 | A player may hold one visible offer (`Reserve{offer, kind}`), free, for 20 s (400 ticks); `kind` must match the offer or the intention is rejected as stale |
| BR-S07 | A held offer is skipped by rotation; refills and rotations never show a duplicate of it (BR-S01 still holds for all five) |
| BR-S08 | Buying the held offer consumes the hold and refills the slot normally; `Release` ends the hold early |
| BR-S09 | When a hold ends (expiry or `Release`) the card stays as a normal offer until the next rotation; an offer that has been held cannot be held again until its slot is redrawn |
| BR-S10 | Same-tick order: intentions first (a purchase or hold at an expiry or rotation tick sees the offers before the Shop step), then hold expiry, then rotation |

### 10.4 Stronghold kinds (P16-F3/F4)

| ID | Proposed rule |
|----|---------------|
| BR-K01 | Every player, guests included, chooses one of four `StrongholdKind`s for free; account age, rating, payments and playtime never unlock or change them |
| BR-K02 | The choice can change only while idle; joining a queue or creating/joining a room freezes it until cancel or match end; reconnect and account takeover keep the match's kinds |
| BR-K03 | Both kinds are public from `MatchStart` (name, icon, silhouette, stats) |
| BR-K04 | A player who never chose gets the default kind (Redoubt, pending owner decision O-3) |
| BR-K05 | Practice: the bot's kind is drawn uniformly from the match seed and shown at start |
| BR-K06 | A kind changes Stronghold max HP, damage, and at most one passive; never range, attack interval, unit costs, Shop, Energy start, regen or Interest |
| BR-K07 | Max HP is per seat and changes only with `Fortification`; current HP never exceeds max; a max change keeps the HP fraction (`hp' = floor(hp × max' / max)` in `i64`, at least 1 if hp ≥ 1) and never heals the fraction |
| BR-K08 | `StrongholdStyle` (cosmetic, BR-C04) never hides or imitates a kind's silhouette cues |

Kinds are percentages of the canonical Stronghold (today 4 500 HP, 65
damage per 1.0 s, range 6), so P16-D tuning carries over. `Classic` (the
canonical Stronghold) stays in the lab as the neutral control and as the
online kind until P16-F4 activates selection.

| Kind (code) | Owner name | HP bp | Damage bp | Today | Passive |
|-------------|------------|-------|-----------|-------|---------|
| `Treasury` | Torre do Tesouro | −700 | −800 | 4 185 / 59 | Bounty 0.15 Energy per eligible kill, cap 0.6 per window |
| `Redoubt` | Torre Bastião | +1 500 | −1 500 | 5 175 / 55 | none: endurance |
| `Ballista` | Torre de Artilharia | −1 100 | +2 300 | 4 005 / 79 | none: firepower (lab variant B: +800 bp damage and splash 1.0 tile) |
| `Banner` | Torre do Comando | −400 | −800 | 4 320 / 59 | own conventional units within 4 tiles of this Stronghold: +10 % damage |

English names avoid existing Evolution names (`Bastion`, `Artillery`,
`Fortress`) and the glossary's `Commander`; the owner may rename (O-2).

### 10.5 Fortification (P16-F3/F4)

| ID | Proposed rule |
|----|---------------|
| BR-F01 | Each Stronghold starts at tier 0; `Fortify{branch}` at tier 0 picks Walls (0) or Arms (1) at tier 1; at tier 1 only the same branch reaches tier 2 |
| BR-F02 | No duplicate, out-of-order, or branch-changing Fortification; the server validates cost, tier and branch (`Rejection::MaxFortification`, `WrongBranch`, `NotEnoughEnergy`) |
| BR-F03 | Allowed in Running and Siege; effects apply from the next step; max HP follows BR-K07 |
| BR-F04 | Tier and branch are public, visible on the model and in the HUD, and kept across reconnect |
| BR-F05 | All four kinds share the same tree (kind-exclusive tiers are deferred, D-37) |

| Branch | Tier 1 (6 Energy) | Tier 2 (10 Energy) |
|--------|-------------------|--------------------|
| Walls | Reinforced Walls: +15 % max HP | Curtain Wall: +15 % max HP more; Stronghold hits slow 20 % for 1.0 s |
| Arms | Heavy Bolts: +15 % damage | War Engines: +15 % damage more; Stronghold hits splash 1.0 tile |

Owner's starting proposal was +10 % / +10 % + 5 %; the capability at tier
2 replaces the invisible +5 % (D-37).

### 10.6 Augments (P16-F5–F7)

| ID | Proposed rule |
|----|---------------|
| BR-AG01 | Three `AugmentRound`s open at 0:30, 2:00 and 3:30 of running time; each stays open 30 s while combat, Shop, Evolution and Fortification continue |
| BR-AG02 | Rarity profiles: round 1 Bronze, Bronze, Silver; round 2 Bronze, Silver, Gold; round 3 Silver, Gold, Diamond |
| BR-AG03 | Both players receive the same three options, drawn by the server from a match stream (seed stream 3) that clients never see (pending owner decision O-6) |
| BR-AG04 | Draw: for each slot in profile order, uniform among that rarity's Augments never offered earlier in this match and not already in this offer; if none of the three is Universal, the first slot (in profile order) whose rarity still has an eligible Universal Augment is redrawn from those; if no rarity has one, the draw stands |
| BR-AG05 | A player buys at most one option per round (`ChooseAugment{round, option}`), or declines (`Forgo{round}`); the round then closes for that player only |
| BR-AG06 | Declining, or letting the 30 s expire without a purchase, grants Forgo +2 Energy once per round; disconnecting and reconnecting changes nothing; nothing is ever assigned automatically |
| BR-AG07 | A player short of Energy may wait inside the window; a match that ends with an open round grants nothing |
| BR-AG08 | A player holds at most three Augments (one per round), kept until the match ends; there is no sell, swap or reroll |
| BR-AG09 | Each player's equipped Augments are public to both players |
| BR-AG10 | An Augment applies to summons only if its text names Beetlings; it never makes a summon count for Synergy or Bounty |
| BR-AG11 | Augment effects are computed only by the simulation; clients send a round and an option index, never an effect |

| Rarity | Cost | Forgo |
|--------|------|-------|
| Bronze | 3 | +2 for any round |
| Silver | 6 | |
| Gold | 10 | |
| Diamond | 15 | |

### 10.7 Modifier stacking contract (all P16-F phases)

| ID | Proposed rule |
|----|---------------|
| BR-X01 | Percentage bonuses to one attribute add in basis points across all sources (Evolution, Synergy, Arcana when it exists, Augment, kind, Fortification, Banner aura, conditional effects) and apply once: `floor((base + flat) × (10 000 + Σbp) / 10 000)` with `i64` intermediates |
| BR-X02 | Attack interval: `max(MIN_INTERVAL, floor(base × 10 000 / (10 000 + Σatk_bp)))`, Σatk_bp ≤ +10 000 |
| BR-X03 | Move speed Σbp ≤ +5 000; slow strength is never modified by Augments |
| BR-X04 | Situational damage (vs Strongholds, vs Tanks, vs low-HP targets, comeback) joins Σbp at hit time; one truncation |
| BR-X05 | Damage reduction from all sources ≤ 2 000 bp, applied after BR-X04: `floor(dmg × (10 000 − Σdr) / 10 000)`, at least 1 when dmg ≥ 1; Collapse is never reduced |
| BR-X06 | Lifesteal Σ ≤ 1 200 bp of the HP actually removed by a primary hit on an enemy conventional unit (never splash, Strongholds or summons); never above max HP |
| BR-X07 | Status duration Σ ≤ +3 000 bp; any root contract adopted by P32 still bounds root |
| BR-X08 | Max HP changes keep the HP fraction (BR-K07 rule, also for units, as today) |
| BR-X09 | Each source is finite (one kind, two Fortification tiers, two Evolution tiers, two Synergy levels, three Augments) and every timed effect has a cap or a cooldown: no bonus stacks without bound |
| BR-X10 | Stronghold healing exists only through named effects with a per-window cap and never exceeds max HP; at most one Stronghold shield exists at a time and it absorbs unit and Stronghold damage only |
| BR-X11 | Conditional effects are evaluated once per tick, after the Shop step, from that tick's state, in fixed seat and unit-id order |

### 10.8 Augment catalog (40, Proposed)

Index order is a serialization contract (P16-F5). **U** = Universal
(useful to any army); otherwise the tag names who benefits. Percentages
use BR-X01–X07. Per-Augment analysis (interactions, counters, exploits,
tests, hypotheses) lives in [P16-F6](phases/P16F6-augment-catalog.md); the
first 14 marked ★ ship first in P16-F5.

| # | Id | Name | Tag | Effect |
|---|----|------|-----|--------|
| 0 | `forced_march` | Forced March | U | Units you buy move +40 % for 5 s after spawning |
| 1 | `field_kit` ★ | Field Kit | U | Conventional units +4 % max HP |
| 2 | `sentry` ★ | Sentry | U | Your Stronghold +10 % damage |
| 3 | `steady_aim` ★ | Steady Aim | Ranged | Ranged +8 % damage |
| 4 | `reinforced_shields` ★ | Reinforced Shields | Tank | Tanks +8 % max HP |
| 5 | `first_aid` | First Aid | Support | +10 % direct and splash heal |
| 6 | `drilled_ranks` | Drilled Ranks | Humans | Humans +8 % damage |
| 7 | `frenzied_bones` | Frenzied Bones | Undead | Undead +8 % attack speed |
| 8 | `tuned_gears` | Tuned Gears | Machines | Machines +8 % attack speed |
| 9 | `hardy_sap` | Hardy Sap | Plants | Plants +8 % max HP |
| 10 | `plunder` ★ | Plunder | U | Bounty 0.15 Energy per eligible kill, cap 0.45 per window |
| 11 | `stone_mend` | Stone Mend | U | Stronghold heals 50 HP per eligible kill, cap 150 HP per 30 s |
| 12 | `field_promotion` ★ | Field Promotion | U | Evolutions cost 1 Energy less (minimum 1) |
| 13 | `veterans` ★ | Veterans | U | Evolved units +8 % max HP, +4 % damage |
| 14 | `shield_wall` | Shield Wall | Rampart | Your conventional units within 1.5 tiles of your living Rampart take −8 % damage |
| 15 | `siege_arrows` ★ | Siege Arrows | Ranged | Ranged +15 % damage to Strongholds |
| 16 | `armored_brood` | Armored Brood | Beetlings | Beetlings +25 % max HP |
| 17 | `triage` | Triage | Support | +25 % direct heal on allies below 40 % HP |
| 18 | `lingering_hex` | Lingering Hex | Control | Slow and root you apply last +15 % |
| 19 | `shield_step` | Shield Step | Tank | Tanks +12 % move, +5 % max HP |
| 20 | `war_leech` ★ | War Leech | U | Lifesteal 8 % (BR-X06) |
| 21 | `conquerors` ★ | Conquerors | U | Conventional units +12 % damage to Strongholds |
| 22 | `tower_fury` | Tower Fury | U | Each kill by your Stronghold: +5 % Stronghold damage, max 3 stacks, all reset 12 s after the last kill |
| 23 | `entrenched` | Entrenched | U | While your Stronghold is below 50 % HP, your units within 5 tiles of it take −10 % damage |
| 24 | `grievous_wounds` | Grievous Wounds | U | Your hits reduce healing the target receives by 40 % for 2 s (no stacking) |
| 25 | `giant_slayer` ★ | Giant Slayer | U | +15 % damage to enemy Tanks |
| 26 | `executioner` | Executioner | U | +20 % damage to targets below 35 % HP |
| 27 | `hunters_instinct` | Hunter's Instinct | DPS | DPS +10 % attack speed, +8 % move |
| 28 | `iron_vanguard` | Iron Vanguard | Tank | Tanks +15 % max HP, +10 % move |
| 29 | `spellweaver` | Spellweaver | Control | Control +20 % attack speed |
| 30 | `last_bastion` ★ | Last Bastion | U | Once, when your Stronghold first falls below 25 % HP: shield of 12 % max HP for 15 s |
| 31 | `desperate_assault` ★ | Desperate Assault | U | While your Stronghold HP fraction is below the enemy's: +15 % damage to Strongholds |
| 32 | `legacy` ★ | Legacy | U | Evolved units +15 % max HP, +10 % damage |
| 33 | `second_wind` | Second Wind | U | Each conventional unit survives its first lethal hit with 1 HP (once per unit) |
| 34 | `convergence` | Convergence | Two factions | With Synergy active for two Factions: conventional units +8 % max HP, +8 % damage |
| 35 | `royal_banner` | Royal Banner | Humans | Humans +12 % max HP, +10 % damage |
| 36 | `deathless_legion` | Deathless Legion | Undead | Undead +15 % attack speed, +10 % max HP |
| 37 | `iron_legion` | Iron Legion | Machines | Machines +15 % damage, +10 % attack speed |
| 38 | `bountiful_harvest` | Bountiful Harvest | Plants | Plants +10 % max HP, +15 % direct heal |
| 39 | `hive_awakening` | Hive Awakening | Beetlings | Beetlings +25 % damage; your summon limit +1 (6 → 7) |

Rarity by index: 0–9 Bronze, 10–19 Silver, 20–29 Gold, 30–39 Diamond.
Universal per rarity: 3 / 4 / 7 / 4, so BR-AG04 can always place one
(★ subset: 2 / 3 / 3 / 3).

### 10.9 Balance targets for the expansion (Proposed, P16-F gates)

Measured with the P16-C lab (paired seeds, both seat orders, 95 %
cluster-bootstrap intervals). Bots are not humans: passing is necessary,
never sufficient (§6).

| Row | Target |
|-----|--------|
| Best `banker-N` (N ∈ 10, 15, 20, 25, 30, 40) against the strategic pool | ≤ P16-D's `saver` score + 3 pp, and ≤ 60 % |
| `rush` and `eco` against the strategic pool | 40–60 % each |
| Median / p90 / p99 / max running time | 150–240 s / ≤ 330 s / ≤ 390 s / < 400 s |
| Matches reaching `Siege` / ended by `Collapse` | ≤ 35 % / ≤ 15 % |
| Siege stretches of 60 s with no Stronghold damage from units | ≤ 1 % of matches |
| Each kind against the field of kinds (pool × kinds) | 45–55 % |
| Any kind head-to-head | ≤ 60 % |
| Each Fortification branch | has a matchup where buying it beats not buying it; `fortify-always` ≤ 55 % |
| Each Augment's value Δ = score(bot + forced Augment) − score(bot + Forgo), same seeds | Bronze/Silver ≥ 0 pp; every Augment ≤ +12 pp |
| Pick rate of one Augment by the greedy policy when offered | ≤ 60 % |
| Best Augment triple found by adversarial search, against the pool | ≤ 65 % |
| Comebacks (behind at 2:00, then won) | not below the P16-D canon; target ≥ 20 % |
| Nash support / largest weight | ≥ 3 / ≤ 50 % |
