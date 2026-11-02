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
| 4 | Continuous `Shop`: every purchase during combat brings five new offers; no preparation phase |
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
| Shop | Per-player offer of five distinct units during a match, redrawn in full on every purchase; not a store (**Proposed**, D-32) | `Shop` |
| Evolution | In-match branching upgrade of a deployed unit; not account progression | `Evolution` |
| Synergy | Bonus while enough units of one `Faction` (or a defined combo) are alive | `Synergy` |
| PlayerIntention | Client request (purchase, evolve, reposition); a request, never a state change | `PlayerIntention` |
| AuthoritativeServer | The process that owns combat, Energy, shop timers, outcomes | — |
| Stronghold | **Proposed (new):** each player's base at their end of the `BattleLine` | `Stronghold` |
| Arena | **Proposed (ARENAS-G, D-55):** the battlefield of one match, one of five; the server draws it and its one small modifier applies to both sides | `Arena` |
| Cosmetic | Visual item with no gameplay effect | `Cosmetic` |
| Inventory | Account-bound collection of owned cosmetics | `Inventory` |
| Commander | Player-facing name for the player (TBD by brand voice) | `Player` |
| MVPUnitRoster | Launch target of 12 units; 2.0 ships the first 8 first | — |
| Account | **Proposed (P11):** a player identity — one `PlayerName` + one `AccountKey`; holds `Rating` and statistics | `Account` |
| PlayerName | **Proposed (P11):** unique public name chosen at account creation | `PlayerName` |
| AccountKey | **Proposed (P11/P68-A):** the long-lived manual credential: `PlayerName-XXXX-XXXX-XXXX-XXXX`; delegated session tokens are defined in §21 | `AccountKey` |
| MatchMode | **Proposed (P12, P39):** `Casual` (no rating change), `Ranked` (accounts only, rating changes), or `Challenge` (a direct invitation match; casual rules, no rating change) | `MatchMode` |
| Quick Challenge | **Proposed (P39):** a direct invitation match created in one click; one different session joins by link; casual rules, no rating, no queue | `Challenge` |
| Rematch | **Proposed (P39):** a new Challenge match between the same two sessions after both request it within the expiry | `RematchTicket` |
| Rating | **Proposed (P12):** an account's ranked skill number (Elo) | `Rating` |
| Leaderboard | **Proposed (P13):** global ranking of accounts by `Rating` | `Leaderboard` |
| StrongholdStyle | **Proposed (P18):** cosmetic look of a `Stronghold`: Stone Keep, Timber Fort, Gatehouse | `StrongholdStyle` |
| Arcana | **Proposed (P40, active):** a free pre-match choice in one of three five-node families that modifies a documented unit attribute or conditional combat behavior | `ArcanaNode` |
| ArcanaPreset | **Proposed (P40, active):** fifteen bounded ranks costing exactly twelve points, chosen before matchmaking and frozen for a match | `ArcanaPreset` |
| RulesetVersion | **Proposed (P35):** compiled version of competitive rules and balance values, recorded with match results | `RulesetVersion` |
| StrongholdKind | A free pre-match choice of `Stronghold` (Treasury, Bastion, Artillery, Command) with its own stats, passive and Fortification branches (D-46); mechanical, unlike `StrongholdStyle` | `StrongholdKind` |
| Fortification | In-match branching upgrade of a player's own `Stronghold`, paid with `Energy`, two kind-exclusive branches (D-46); never called `Evolution` | `Fortification` |
| Interest | A boost to `Energy` regeneration that grows with the `Energy` held: +25 % of the base rate for every full 5 `Energy`, counting at most 25 (BR-E05–E07, owner revision 2026-10-29) | `Interest` |
| Bounty | **Proposed (P16-F1, not active):** `Energy` granted for an eligible kill, bounded per payment window | `Bounty` |
| Siege | **Proposed (P16-F1, not active):** the match phase after the running phase in which both `Stronghold`s suffer `Collapse`; replaces overtime | `Phase::Siege` |
| Collapse | **Proposed (P16-F1, not active):** symmetric structural loss of a percentage of each `Stronghold`'s max HP per second during `Siege` | `Collapse` |
| Reserve | **Proposed (P16-F2, not active):** one `Shop` offer a player holds out of rotation for a limited time | `Reserve` |
| Augment | **Proposed (P16-F5, not active):** an in-match bonus bought with `Energy` from an `AugmentRound`; not an `Arcana` (pre-match) and not a store item | `Augment` |
| AugmentRound | **Proposed (P16-F5, not active):** a timed offer of three `Augment`s, identical for both players | `AugmentRound` |
| Forgo | **Proposed (P16-F5, not active):** declining or letting an `AugmentRound` expire, compensated with `Energy` | `Forgo` |
| RecoveryEmail | **Proposed (P75, inactive):** the private, verified address that lets an `Account` replace a lost `AccountKey` in place (§22) | `recovery_identity` |
| Operator | **Proposed (P75, inactive):** an administrator identity (Owner or Moderator) separate from every `Account`; signs in to the console with an `AdminKey` | `Operator` |
| AdminKey | **Proposed (P75, inactive):** an Operator's long-lived console credential `TBGADM-…`, claimed and recovered only by email | `AdminKey` |
| Suspension | **Proposed (P75, inactive):** an Operator's temporary or permanent ban of an `Account` from playing; history is never rewritten | `account_moderation` |
| Report | **Proposed (P75, inactive):** a player's private complaint about another `Account`, with server-captured evidence | `Report` |
| MatchChat | **Proposed (P75, inactive):** short plain-text messages between the two seats of one online match; never part of the simulation | `ChatSend`/`Chat` |
| Emoji | **Proposed (P75, inactive):** a picture a seat sends during a match from its three `EmojiSlot`s; standard or custom; purely cosmetic | `EmojiRef` |
| EmojiSlot | **Proposed (P75, inactive):** one of an account's three equipped emojis, frozen at pairing | `account_emoji_slots` |
| CustomEmoji | **Proposed (P75, inactive):** a player-uploaded image/GIF usable by its owner only after an Operator approved it | `emoji_assets` |

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
| BR-M02 | No sub-rounds or preparation phases; shopping and combat overlap. The only exception is the Augment selection pause (BR-AG12, owner revision 2026-10-29) | Confirmed | S1 |
| BR-M03 | A disconnected player who has been connected first pauses the match for both players: combat tick, countdown and Augment selection clock stand still and intentions are refused (`Paused`) while that player's connection-pause budget lasts (**Proposed, P71**; 120 s per player per match). Then the reconnect window runs with the match running, then the player forfeits. A seat that never connected gets only the window | Adapted; pause Proposed (P71) | S1, S3 |
| BR-M04 | Each player has a `Stronghold`; destroying the enemy `Stronghold` wins | Proposed | — |
| BR-M05 | ~~Fixed match clock: regulation, then overtime with faster `Energy`; tie after overtime is a draw~~ | **Removed — owner revision 2026-10-29 (D-41)**: matches are open-ended, BR-M07 | — |
| BR-M06 | A player may surrender while the match is in countdown or running (an Augment pause included); it is a forfeit (BR-M03): the opponent wins and, in ranked, it is a loss (BR-R04) | Proposed | — |
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
| BR-U08a | Summoners (Nester, Smuggler and future summoners) act on their own timers, independently of visible enemies, range or enemies being present in the lane | Confirmed (owner 2026-10-31) | — |
| BR-U08 | A summon is a temporary unit created by a unit's ability: it costs no `Energy`, is never sold, never evolves, never summons, neither counts toward nor receives `Synergy`, is not part of the army count, and disappears when its lifetime ends | Proposed (D-32) | — |
| BR-U09 | Guard: an enemy unit or `Stronghold` whose attack range reaches a living Rampart must target the nearest such Rampart | Proposed (D-32) | — |
| BR-U10 | A server-protection entity limit, if ever needed, is documented as server protection with measured evidence (ARCHITECTURE), never as an army rule | Proposed (D-32) | — |
| BR-E01 | `Energy` is the only in-match resource, distinct from any currency | Confirmed | S4 |
| BR-E02 | `Energy` regenerates over time under server rules; there is no gameplay cap (BR-E04, owner revision 2026-10-29, D-41) | Confirmed | S2, S4 |
| BR-E03 | Purchases and evolutions exceeding available `Energy` are rejected, never queued or owed | Adapted | S1 |
| BR-S01 | Each player sees exactly five offers, five distinct units at any time | Proposed (D-32; was three) | S1, S4 |
| BR-S02 | ~~Shop rotation timers come from the server match clock, identical for both players~~ | **Removed — owner revision 2026-10-29**: offers never rotate on a timer (BR-S04) | S1, S4 |
| BR-S03 | The shop sells units only, never stats or advantage | Confirmed | S1 |
| BR-S04 | Offers change only when bought: every purchase redraws all five offers at once, uniformly at random from the whole roster as five distinct units (BR-S01), so the unit just bought may appear again (about half the time). There is no timed rotation and no other redraw | Proposed (owner revision 2026-10-29, second revision the same day: was one refilled slot) | — |
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
| BR-C04 | `StrongholdStyle`s are free cosmetics; every style has identical gameplay (styles stay cosmetic; mechanical `StrongholdKind`s are separate, BR-SK01–SK09, D-46) | Proposed | — |
| BR-P01 | An `Account` is a unique `PlayerName` plus an `AccountKey`; P75 requires a private email for new accounts, while existing accounts keep playing; names are 3–16 letters, digits or `_`, start with a letter, are unique ignoring case, and never change | Proposed | — |
| BR-P02 | The `AccountKey` is the long-lived manual credential: shown once at creation or in-place recovery, never shown to other players; P75 recovery requires the verified RecoveryEmail (§22). P68-A delegates runtime authentication to expiring tokens (§21) without changing existing keys | Proposed (P11/P68-A) | — |
| BR-P03 | One active account controller; explicit login/takeover can replace the old controller, including its match. P68-A allows separate persistent device families, but passive refresh/startup never takes over automatically (§21) | Proposed (P11/P68-A) | — |
| BR-P04 | Guests may play practice, casual matches, and private rooms; ranked play needs an account | Proposed | — |
| BR-P05 | A public profile shows name, rating, rank, record, win rate and faction mix. P73 makes the last 10 ranked results and detailed history subject to the account's Public/Only me choice (BR-AP11); private history exposes no recent rows. P68-A additionally stores minimal private session-security metadata (§21), never exposing tokens/device identifiers on public profiles | Proposed (P11/P68-A/P73) | — |
| BR-R01 | Two queues: `Casual` (guests and accounts; private rooms are casual) and `Ranked` (accounts only) | Proposed | — |
| BR-R02 | Matchmaking pairs the two longest-waiting players of the same queue, regardless of rating | Proposed | — |
| BR-R03 | Every ranked match changes both players' `Rating` from both ratings (Elo, §5); win 1, draw ½, loss 0 | Proposed | — |
| BR-R04 | A forfeit (including staying disconnected past the window) is a ranked loss and a win for the opponent | Proposed | — |
| BR-R05 | New accounts start at 1000; a rating never goes below 100; the first 20 ranked games move faster | Proposed | — |
| BR-R06 | Ratings and results are computed and stored only by the authoritative server | Proposed | — |
| BR-R07 | The `Leaderboard` lists accounts with ≥ 1 ranked match, ordered by rating, then wins, then name | Proposed | — |
| BR-R08 | A Quick Challenge is a direct invitation, not a queue; it pairs the inviting session and exactly one different session, is casual (no rating change), and uses the same simulation and reconnect rules | Proposed (P39) | — |
| BR-R09 | A rematch is a new Challenge match between the same two sessions; it requires both requests within the expiry while both sessions are connected, uses a fresh seed, and never enters a queue or changes rating | Proposed (P39) | — |

## 4. How a match plays (MVP)

1. **Start.** Two players are paired. 3-second countdown; both start with
   the same `Energy` and their own five shop offers.
2. **Buy = deploy.** Tapping an affordable offer sends a `Purchase`
   intention. If valid, the unit spawns in front of the buyer's
   `Stronghold` and all five offers are redrawn. Rejections: insufficient `Energy`,
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
7. **Interest.** Every full 5 `Energy` a player holds adds 25 % to their
   `Energy` regeneration, counting at most 25: 5 held regenerates ×1.25,
   10 ×1.50, 15 ×1.75, 20 ×2.00 and 25 or more ×2.25 (BR-E05–E07).
   Saving compounds, and spending lowers the rate at once.
8. **War Council.** Every 60 s of combat the whole battle pauses for both
   players and the same four `Augment`s are offered. Each player buys one
   or forgoes; a confirmed choice is final and hidden until the round
   resolves. The battle resumes when both have confirmed or after 20 s of
   selection (BR-AG01–AG14).
9. **End.** Matches are open-ended (BR-M07): destroying the enemy
   `Stronghold` wins immediately, and nothing is ever decided by a clock.
   A player who stays disconnected past the reconnect window, or who
   surrenders (two taps in the HUD), forfeits. Both Strongholds falling on
   the same tick is a draw. The result screen offers **Play again**.

## 5. MVP starting values (all **new assumptions**, tune via playtests)

Bot-harness balance pass of 2026-10-28 (P7) is applied below; human
playtests decide the final values (decision D-14).

**P16-C balance-lab pass, 2026-10-29 (Proposed, D-33).** Eight numbers
changed on lab evidence confirmed on held-out seeds
(`docs/media/p16c/confirm.txt`): Evolution tier 1 cost 3 → 2, Stronghold
attack 65 → 53, Shieldbearer cost 3 → 4, Hexer damage 30 → 29, Sawbot
cost 3 → 4 and HP 270 → 231, Siege Walker speed 0.6 → 0.66, Rampart cost
5 → 6. They narrow the faction spread and Evolution's cost gap; they do
not remove banked-wave dominance or heavy spam (P16-C report). The
owner's playtest decides.

Distances in tiles, times in seconds. The simulation stores them as
integers (see ARCHITECTURE.md §4).

| Constant | Value | | Constant | Value |
|----------|-------|-|----------|-------|
| BattleLine length (Stronghold to Stronghold) | 16 | | Countdown | 3 |
| Match length | open-ended, no clock (BR-M07) | | Augment round | every 60 s of combat; 20 s selection pause |
| Energy start / cap | 5 / none (BR-E04) | | Energy regen | 1 per ~1.76 s (17 sub-energy/tick) |
| Interest | regen +25 % per full 5 Energy held, up to ×2.25 at 25 (BR-E06) | | Augment cost Bronze / Silver / Gold / Diamond | 3 / 6 / 10 / 15 |
| Shop offers / rotation | 5 distinct / none: a purchase redraws all five (BR-S04) | | Army cap | none (BR-U07) |
| Stronghold HP | 4500 | | Stronghold attack | 53 dmg / 1.0 s, range 6 |
| Evolution cost tier 1 / tier 2 | 2 / 4 | | Reconnect window | 30 |
| | | | Connection-pause budget (BR-M03) | 120 s per player per match (**Proposed**, P71) |

**Roster (10 units + 1 summon, working names).** Every archetype appears.
Bone Brute, Siege Walker, and Rampart are drawn with the large body plan
(visual only since D-32). Nester, Rampart, and the Beetling are **Proposed**.

| Unit | Faction | Archetype | Cost | HP | Dmg | Every (s) | Range | Speed | Special |
|------|---------|-----------|------|----|-----|-----------|-------|-------|---------|
| Shieldbearer | Humans | Tank | 4 | 650 | 30 | 1.2 | 0.8 | 0.8 | — |
| Archer | Humans | Ranged | 3 | 220 | 45 | 1.0 | 4.5 | 1.0 | — |
| Bone Brute | Undead | Tank | 4 | 1300 | 75 | 1.5 | 1.0 | 0.6 | — |
| Hexer | Undead | Control | 4 | 300 | 29 | 1.4 | 3.5 | 1.0 | slow 40% for 2 s |
| Sawbot | Machines | DPS | 4 | 231 | 55 | 1.05 | 0.8 | 1.3 | — |
| Siege Walker | Machines | Ranged | 5 | 700 | 85 | 2.5 | 6.0 | 0.66 | splash 1.5 |
| Bloomkeeper | Plants | Support | 3 | 280 | heal 40 | 1.5 | 3.0 | 0.9 | heals instead of attacking |
| Rootcaller | Plants | Control | 4 | 300 | 20 | 1.6 | 3.0 | 0.9 | root 0.8 s |
| Nester | Plants | Support | 4 | 300 | 15 | 1.5 | 3.0 | 0.8 | Hatch: every 4.0 s a Beetling; ≤ 2 alive per Nester, ≤ 6 per player |
| Rampart | Machines | Tank | 6 | 1500 | 35 | 1.5 | 0.8 | 0.5 | Guard (BR-U09) |
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
| Roster health | Purchase share of each unit | ≥ 62.5% of uniform share: 0.625 / N of purchases for each active conventional kind (owner D-48 amendment, 2026-10-31; 2.5% at N=25, approximately 1.36% at N=46) |
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
Stronghold range, except units that never target Strongholds and Farwatch's
registered canonical reach (**Proposed P71 owner rule**, clarified by D-102).
P71 allows Farwatch to target Strongholds at its existing 12.0-tile base range;
this exception does not authorize a greater experimental range or exempt another
Stronghold attacker. If the experimental Stronghold range is larger, the ordinary
cap still applies. Cost stays between 1 Energy and the Energy cap; interval ≥
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
- Spectating, replays, emotes; rating-based matchmaking, seasons, and
  rating decay (revisit with enough players, D-18).

## 8. Retired technology decisions (game rules kept)

| Retired | Replaced by |
|---------|-------------|
| Unity C# client, native app stores | Rust/Bevy browser client (WASM/WebGL2) |
| Nakama + TypeScript match module | Rust authoritative server running `tbg-core` |
| PostgreSQL via Nakama | SQLite for MVP match records |
| Per-character MB-Lab/Mixamo rigging (ADR-003–006; ~8% auto pass rate, ADR-007) | Code-built rigid-part figurines with a shared animation library |
| StoreKit / Play Billing IAP | Deferred; cosmetics-only principle unchanged |

## 9. Arcanas extra — Proposed values, active in every mode (D-40/P40)

The owner moved this extension ahead of P31 (D-40) and expanded it to
three five-node families on 2026-10-29 (D-42): Arcanas are live in
Practice, Casual, Ranked, private rooms, and Quick Challenge with the
values below, which stay **Proposed** until playtests. Summons (Beetlings)
receive no Arcana bonus and never count for a conditional Arcana, like
Synergy (BR-U08). The new effects are conditional and should be measured
in the P34/P40 lab before any value is treated as final.

| ID | Proposed rule |
|----|---------------|
| BR-AR01 | Every player, including a new guest, has all twelve points and all fifteen options immediately; payments, account age, and playtime never grant power |
| BR-AR02 | A complete preset spends exactly 12 points over 15 nodes. Indices 0–5 and the six children have rank 0–2 (original) or 0–1 (child) and cost 1 point per rank; indices 8, 11 and 14 are supremes with rank 0–1 and cost 2 points. Node order is a serialization contract |
| BR-AR03 | A child needs its parent at rank 1. A supreme needs both original nodes at rank 2 and both children at rank 1. The editor and the server enforce the same table; a removal that would invalidate a held dependent is refused with the dependent named |
| BR-AR04 | Eligibility uses the unit's base `Archetype`, remains unchanged by `Evolution`, and grants no new behavior or missing attribute; summons are excluded from every Arcana |
| BR-AR05 | Percentage bonuses add in basis points with Evolution, Synergy, Arcana and Augment sources on the same attribute and truncate once. Conditional bonuses are evaluated at the attack, movement, status or heal resolution point named below, never pre-applied |
| BR-AR06 | Conditional army checks count living conventional units only. A condition activates once no matter how many units satisfy it, is rebuilt from the living army each combat tick, and is decremented when a unit dies |
| BR-AR07 | Unshaken changes only the duration of an incoming slow or root on Support and Control; slow intensity is untouched and a duration never falls below one tick |
| BR-AR08 | Emergency Care compares HP before the heal with integer arithmetic (`hp × 10 000 < 40 % × max_hp`); overheal is never converted into another resource and the bonus never creates healing |
| BR-AR09 | Purification requires a real heal greater than zero on a conventional ally, removes an active slow, and halves the remaining root duration (minimum one tick). It cannot trigger another heal and never removes other control |
| BR-AR10 | Deadeye adds 0.25 tile (250 milli-tiles) of attack range to conventional Ranged units only; Strongholds, summons and non-Ranged units are unchanged, and no special targeting is added. Dose-response on 3 072 paired seeds (Precision build against the default): +0.1 tile 51.6 %, +0.25 tile 53.7 %, +0.5 tile 57.0 %, none 49.1 % (noise floor 49.4 %); +0.25 keeps the supreme visible and inside the 55 % investigation line (P40 audit, `docs/media/p40/deadeye-dose.txt`) |
| BR-AR11 | Preset editing is allowed only while idle; joining either queue or creating/joining a room or Quick Challenge freezes selection until cancellation or match end; reconnect and account takeover keep the match's original preset |
| BR-AR12 | The server validates ranks, budget, prerequisites, schema, ruleset, and session state; clients send selections, never final bonuses or a claimed seat |
| BR-AR13 | One active preset per account, recovered across devices, and local storage for guests. A schema-1 six-rank preset keeps its ranks; the six new points raise the original nodes to rank 2 in index order, never selecting a child or supreme automatically. The complete default is `[2,2,2,2,2,2,0,0,0,0,0,0,0,0,0]` |
| BR-AR14 | Record both applied presets, their schema, and the RulesetVersion with every result; later tuning never rewrites historical outcomes, stored six-rank rows or Rating |

**Node order and prerequisites are a serialization contract**, not
alphabetical order. Names below are English UI names. `DPS` is
`Archetype::Dps` in code.

| Index | Family / node | Cost | Requires |
|-------|---------------|------|----------|
| 0 | War / Guard | 1 | none |
| 1 | War / Assault | 1 | none |
| 2 | Precision / Piercing | 1 | none |
| 3 | Precision / Endurance | 1 | none |
| 4 | Vitality / Resilience | 1 | none |
| 5 | Vitality / Renewal | 1 | none |
| 6 | War / Shieldwall | 1 | Guard 1 |
| 7 | War / Battle Rush | 1 | Assault 1 |
| 8 | War / Combined Arms | 2 | Guard 2, Assault 2, Shieldwall 1, Battle Rush 1 |
| 9 | Precision / Hunter's Focus | 1 | Piercing 1 |
| 10 | Precision / Quickdraw | 1 | Endurance 1 |
| 11 | Precision / Deadeye | 2 | Piercing 2, Endurance 2, Hunter's Focus 1, Quickdraw 1 |
| 12 | Vitality / Unshaken | 1 | Resilience 1 |
| 13 | Vitality / Emergency Care | 1 | Renewal 1 |
| 14 | Vitality / Purification | 2 | Resilience 2, Renewal 2, Unshaken 1, Emergency Care 1 |

The six original nodes keep their values:

| Index | Node | Base Archetypes | Per rank | Rank-2 maximum |
|-------|------|-----------------|----------|----------------|
| 0 | Guard | Tank, DPS | +4% base max HP | +8% |
| 1 | Assault | Tank, DPS | +3% base damage | +6% |
| 2 | Piercing | Ranged | +3% base damage | +6% |
| 3 | Endurance | Ranged | +4% base max HP | +8% |
| 4 | Resilience | Support, Control | +4% base max HP | +8% |
| 5 | Renewal | Support with an existing heal | +5% base direct/splash heal | +10% |

**War — front line and mixed pressure.** Shieldwall gives Tanks 6% damage
reduction while at least one allied conventional DPS is alive; more DPS
never stack it. Battle Rush gives DPS 12% movement speed while at least
one allied conventional Tank is alive. Combined Arms, the War supreme,
adds 5% Tank damage reduction and 8% DPS damage while both roles are
alive. These reductions combine with Augment reduction and obey its 20%
global cap.

**Precision — range and target control.** Hunter's Focus gives Ranged 12%
damage against a target already slowed or rooted when the hit resolves;
the hit that applies the status does not count. Quickdraw gives Ranged 15%
attack speed when the current target is within two tiles (2 000
milli-tiles), respecting `MIN_INTERVAL`. Deadeye, the Precision supreme,
adds 0.25 tile (250 milli-tiles) of attack range to conventional Ranged units; the owner's +0.5 tile proposal measured 57.0% against the default (above the 55% investigation line) and +0.1 tile only 51.6% (P40 audit, BR-AR10).

**Vitality — support, survival and recovery.** Unshaken shortens incoming
slow and root durations on Support and Control by 25%. Emergency Care
raises direct heal efficiency by 15% when the ally is below 40% HP before
the heal. Purification, the Vitality supreme, removes slow and halves the
remaining root whenever a real heal lands on a conventional ally.

**Builds.** A complete family costs 8 points; with 12 available, the
player can finish one supreme and spend the remaining 4 in another family,
or distribute 4/4/4 across all three without a supreme. The hybrid build
must remain viable; supremes are deliberately conditional. The default
preset is the 4/4/4 original-node build above; it selects no conditional
node automatically.

For each affected attribute, add percentages by source before scaling:

```text
damage = floor((base_damage + Evolution_flat_damage)
               * (10000 + Evolution_bp + Synergy_bp + Arcana_bp + Augment_bp) / 10000)
max_hp = floor(base_hp * (10000 + Evolution_bp + Synergy_bp + Arcana_bp + Augment_bp) / 10000)
heal = floor(base_heal * (10000 + Evolution_heal_bp + Arcana_heal_bp + Augment_heal_bp) / 10000)
```

**Acceptance hypotheses, not claims of balance.** Against the default,
investigate scores above 55% or individual matchups above 65%; require at
least three competitive, non-dominated builds and a default within 3
percentage points of the best generalist reference. Use paired seeds,
seat swaps and 95% confidence intervals. A conditional node that never
activates in representative matches is not offering a decision and must
be retuned or rejected. Human validation still needs at least 8
participants and 24 completed matches, with 80% able to explain their
build and one relevant reason for defeat.

## 10. Strategic expansion (P16-F1–F8)

**Owner revision 2026-10-29 (D-41).** The owner switched on, without the
lab gate, an open-ended match, uncapped `Energy` with compounding
Interest (§10.1), and infinite Augment rounds that pause the battle
(§10.6, all 40 of §10.8). Siege, Collapse and every time victory (§10.2)
are removed, not deferred. Shop Reserve (§10.3) stays **Proposed, not active**; Stronghold kinds and
Fortification (§10.4–§10.5) are active by D-46. Balance of
the active parts is **not measured** (§10.9). Rule IDs are stable; never
reuse one.

| Property | Meaning | This design |
|----------|---------|-------------|
| No time-up decision | No result is ever decided by comparing HP at a clock time | yes (BR-M07) |
| Ends only by unit damage | Only units and Strongholds can reduce Stronghold HP | yes (Collapse removed) |
| Progressive structural attrition | Both Strongholds lose HP over time, symmetrically | **no** (removed with Siege) |
| Mathematical termination | Every match ends before a bound computable from the rules | **no**: open-ended by owner decision; the offline harness keeps a 6-hour safety bound, the server none |

### 10.1 Economy (P16-F1) — active

| ID | Rule (active, owner revision 2026-10-29) |
|----|---------------|
| BR-E04 | `Energy` has no gameplay cap; every *generating* source has a documented bound per window instead (regen per tick with the Interest boost, Bounty per window, Forgo per round) |
| BR-E05 | Interest is a regen boost, not a payment: every full 5 `Energy` the player holds adds 25 % of the base regen (`INTEREST_BAND` 3 000 sub-energy, `INTEREST_PER_BAND` 2 500 bp), counting at most 5 steps (25 `Energy`); a player holding 5 regenerates ×1.25, 10 ×1.50, 15 ×1.75, 20 ×2.00, 25 or more ×2.25 (owner revision 2026-10-29, replaces the 10-s payment) |
| BR-E06 | The boost reads the player's balance at the start of each combat tick, before that tick's regen, so a spend lowers it immediately and nothing can be gamed around a payment instant; `Energy` itself is never capped, only the steps counted |
| BR-E07 | Regen is `regen_running × (10 000 + steps × 2 500) / 10 000` sub-energy per tick with the remainder carried in `PlayerState::regen_frac`, so the rate is exact over time; Bounty and Forgo Energy and the regen add with saturating `u32` arithmetic; no regen runs in the countdown, an Augment pause or after the end |
| BR-E08 | An eligible kill is the death by damage of an enemy conventional unit (BR-U08: summons and expiries are never eligible); every Bounty source of a player sums per kill and is capped by the sum of their caps and by a per-player ceiling of 1 Energy per 10-s window (windows reset every 10 s of combat) |
| BR-E09 | No `Augment`, `StrongholdKind` or `Fortification` changes base regen, Interest bands, or Shop probabilities |
| BR-E10 | Purchases, Evolutions, Fortifications and Augments are still rejected when `Energy` is short (BR-E03) |

| Constant | Value | Sub-energy (600 = 1) |
|----------|-------|----------------------|
| Start | 5 (unchanged unless P16-D changes it) | 3 000 |
| Base regen | 17 per tick, unchanged; no `Siege` doubling | — |
| Interest band / boost per band / max bands | 5 Energy / +25 % of base regen / 5 (×2.25 at 25 held) | 3 000 / 2 500 bp / 5 |
| Bounty ceiling | 1.0 per player per window | 600 |

Tick order: intentions (they lower the balance), then regen with the
Interest boost read from that balance, then the Bounty window reset every
10 s. An Augment pause (BR-AG12) runs no regen.

### 10.2 Victory without a clock (P16-F1, revised)

| ID | Rule |
|----|------|
| BR-M07 | A match is never decided by time: it ends only when a Stronghold reaches 0 HP (both on the same tick is a draw) or by forfeit (BR-M03, BR-M06). There is no match length (active, owner revision 2026-10-29) |
| BR-M08 | ~~`Siege` at 4:00 with doubled regen and percentage `Collapse`~~ **Removed** (owner revision 2026-10-29, D-41) |
| BR-M09 | ~~`Collapse` damage rules~~ **Removed** |
| BR-M10 | ~~Termination bound of 6:40~~ **Removed**: matches are open-ended |
| BR-M11 | ~~`ServerLimit` draw at 10:00~~ **Removed**: no gameplay or server time limit ends a match |

`Phase` is `Countdown`, `Running` or `Ended`; `EndReason` is
`StrongholdDestroyed` or `Forfeit`. Forfeit, surrender, reconnect
windows, ranked Elo (draw ½) and result persistence are unchanged.

### 10.3 Shop Reserve and paid Reroll (preview, Proposed)

CANON.shop remains None until the owner's explicit checkpoint. Preview uses
2 reserve slots, 3 new-shop charges, reroll_base 600 sub-energy (1 Energy),
reroll_step 600, no maximum. FULL_POOL and five distinct offers remain.

| ID | Proposed rule |
|----|---------------|
| BR-S06 | Reserve a visible matching kind for free; at most reserve_slots active; stale kind is rejected |
| BR-S07 | Reserved cards stay in the same slot at every new Shop; other slots draw distinctly excluding held kinds |
| BR-S08 | Each purchase or Reroll consumes one charge of every kept reservation; at zero the card stays visible until the next new Shop |
| BR-S09 | Buying a reserved card ends its reservation and redraws its slot immediately; Release leaves the card visible until the next new Shop |
| BR-S10 | An expired or released reservation cannot be renewed before its slot is redrawn |
| BR-S11 | Reroll pays base + step × consecutive rerolls, optionally capped; buying a unit resets the streak; stale displayed price or insufficient Energy spends nothing |
| BR-S12 | Reserve, Release and Reroll require Running and are Paused during Augment selection |
| BR-S13 | Reservations, charges and Reroll price/streak are private to their player, excluded from opponent and spectator views |
| BR-S14 | Intentions apply in player sequence order before step; each sees the preceding intention's Shop |

### 10.4 Stronghold kinds — active (owner decision 2026-10-29, D-46)

The owner's product spec of 2026-10-29 replaces the earlier proposal
(BR-K01–K08: Treasury, Redoubt, Ballista, Banner on one shared Walls/Arms
tree). Those IDs are retired, never reused. Values below are **Proposed**:
the lab matrix in `docs/media/p17/lab/` is the only evidence so far.

| ID | Rule |
|----|------|
| BR-SK01 | Every player, guests included, picks one of four `StrongholdKind`s for free (Treasury, Bastion, Artillery, Command); nothing unlocks or changes them. The choice changes only while idle; queueing, creating or joining a room freezes it until cancel or match end; pairing copies it into the match; reconnect keeps it. No choice = Bastion. Accounts keep their kind. `Classic` is the reference tower for tests and the lab, never selectable online |
| BR-SK02 | Treasury **Spoils**: each eligible kill (an enemy conventional unit's death, BR-U08: never a summon, an ally or an expiry) credits its owner 0.15 Energy, at most 0.6 Energy per 10 s Bounty window; paid from the authoritative death in the step it happens, independent of the Augment Bounty |
| BR-SK03 | A kind sets the tower's base max HP and damage as basis points of the reference tower (table below); range 6 tiles and one shot per 20 ticks unless a Fortification changes them. Every value acts on the real shot, not only on the HUD |
| BR-SK04 | Command **War Banner**: own conventional units (never summons, never Strongholds) within the radius of their own tower deal +5 % damage, read from their position at hit time and added once to the other damage bp (BR-X01); nothing is stored on the unit, so it ends the moment it leaves |
| BR-SK05 | `Fortify{branch}` buys the next tier of the player's own tower: tier 1 (3 Energy) picks branch A (0) or B (1) for the rest of the match, tier 2 (4 Energy) repeats it. The server rejects a third tier or a second purchase of a tier (`MaxTier`), a branch change (`WrongBranch`), too little Energy (`NotEnoughEnergy`), an open Augment selection (`Paused`) and anything after the end (`NotRunning`). Its own price, never the unit `EVOLVE_COST` |
| BR-SK06 | Fortification percentages are of the kind's own base and add up across tiers; `interval` steps replace the base interval |
| BR-SK07 | A max HP change keeps the HP fraction, rounded down, never a free heal (`hp' = floor(hp × max' / max)`, alive stays ≥ 1). Effects measured against "max HP" (Last Bastion, Stone Mend, Comeback's HP comparison by fraction) use the seat's own max |
| BR-SK08 | Both towers' kind, tier, branch and max HP are public in every view and on the model |
| BR-SK09 | The practice bot's kind follows the match seed (`SELECTABLE[seed % 4]`), so a practice match replays exactly; the bot fortifies only a player kind, never `Classic` |

| Kind | Owner name | HP bp | Damage bp | Today (4 500 / 53 ref.) | Passive |
|------|------------|-------|-----------|------------------------|---------|
| `Treasury` | Torre do Tesouro | −700 | −800 | 4 185 HP / 48 | Spoils (BR-SK02) |
| `Bastion` | Torre Bastião | +1 500 | −1 000 | 5 175 HP / 47 | none: the most HP (Fortified Structure) |
| `Artillery` | Torre de Artilharia | −1 500 | +1 000 | 3 825 HP / 58 | none: the most damage |
| `Command` | Torre do Comando | −400 | −800 | 4 320 HP / 48 | War Banner +5 % within 4 tiles (BR-SK04) |

Treasury and Command keep the product hypotheses' ratios (4 200/60 and
4 300/60 against a 4 500/65 reference) on today's tower. Bastion
(hypothesis 5 200/55) and Artillery (4 000/80) started there too (−1 500
and −1 100 / +2 300 bp); the first lab matrix had Artillery win 61–67 %
against every kind for a near-tower army and Bastion lose 62 % to it, so
Artillery moved to −1 500 / +1 000 and Bastion's damage to −1 000
(`docs/media/p17/lab/`, P17 report). Artillery keeps the most damage,
Bastion the most HP. `Bastion`, `Artillery` and `Fortress` are
also unit Evolution form names; the owner chose these names, and the
picker and HUD always show them as Stronghold names.

### 10.5 Fortification — active (D-46)

| Kind | Branch A tier 1 → tier 2 | Branch B tier 1 → tier 2 |
|------|--------------------------|--------------------------|
| Treasury | **Armored Vault**: Reinforced Vault +12 % max HP → Grand Vault +12 % more | **Mint**: Coinage 0.20 Energy per kill → Royal Mint 0.25 per kill, window cap 0.9 |
| Bastion | **Fortress**: Reinforced Walls +15 % max HP → Imperial Fortress +15 % more | **Sentinel**: Watchtower +1 tile range → Royal Sentinel +15 % damage |
| Artillery | **Heavy Cannon**: Heavy Shot +20 % damage → Great Cannon +20 % more | **Rapid Fire**: Quick Reload 20 → 17 ticks → Advanced Mechanism 14 ticks |
| Command | **War Standard**: War Cry +8 % → Supreme Command +11 % | **Forward Banner**: Outpost radius 5 tiles → High Command 6 tiles (+5 % stays) |

Costs: tier 1 3 Energy, tier 2 4 Energy (`FORTIFY_COST`). The Royal Mint
ceiling (0.9 Energy per window) was not in the product spec, which asked
for a moderate raise: it is +50 % over 0.6, so at most 0.09 Energy a
second from Spoils, still below the base regen; lab evidence pending. Artillery has no splash:
the product spec asks for none unless evidence needs it.

### 10.6 Augments (P16-F5–F7) — active, owner revision 2026-10-29

| ID | Rule (active) |
|----|---------------|
| BR-AG01 | An `AugmentRound` opens every 60 s of effective combat (1 200 combat ticks), the first at 1:00 of running time, for as long as the match lasts; selection pauses never count toward the next draw |
| BR-AG02 | Each round offers four distinct Augments, read in order from one shuffled deck of all 40; after ten rounds every Augment has been offered once and the deck is reshuffled (rarity profiles are retired) |
| BR-AG03 | Both players receive the same four options, drawn by the server from a match stream (seed stream 3) that clients never see |
| BR-AG04 | A player who has not confirmed when the selection window ends forgoes the round; nothing is ever assigned automatically |
| BR-AG05 | A player confirms at most once per round: buy one option (`ChooseAugment{round, option}`) or decline (`Forgo{round}`). A confirmation is final; any later choice for that round is rejected (`RoundClosed`) |
| BR-AG06 | Declining or expiring grants `forgo_energy` once per round: canon **0** (a +2 value exists only in lab tests; owner question). Disconnecting and reconnecting changes nothing |
| BR-AG07 | A player short of `Energy` cannot buy in that round (`Energy` is frozen during the pause) and may buy in a later one; a match that ends during a pause grants nothing |
| BR-AG08 | Copies are unlimited: the same Augment may be bought again in any later round and copies stack linearly (counts saturate at 65 535). There is no sell, swap or reroll |
| BR-AG09 | Each player's applied Augments and copy counts are public to both players |
| BR-AG10 | An Augment applies to summons only if its text names Beetlings; it never makes a summon count for Synergy or Bounty |
| BR-AG11 | Augment effects are computed only by the simulation; clients send a round and an option index, never an effect |
| BR-AG12 | **Pause.** While a round is open the whole battle is frozen for both players: no movement, attack, damage, healing, cooldown, status, summon, Synergy, `Energy` regen, Interest, purchase or Evolution. `Purchase` and `Evolve` are rejected (`Paused`); only `ChooseAugment` and `Forgo` are accepted. The field stays visible |
| BR-AG13 | **Two clocks.** The combat tick is frozen during a pause; a separate selection clock (one per server step, 20 per second) keeps running and closes the window after **20 s** (400 steps, `AUGMENT_WINDOW_TICKS`; owner revision 2026-10-29, was 60 s). The round resolves on the first step after both players confirmed, or at the deadline, whichever is first; one confirmation never resumes the battle. A disconnected player does not extend the window once their connection-pause budget is spent (BR-M03; while it lasts the selection clock stands still too, **Proposed** P71); a reconnecting player gets the round, the time left and their confirmation back. Replays record intentions by selection clock |
| BR-AG14 | **Hidden choices.** A purchase is validated and paid when confirmed, but the opponent sees neither it nor a Forgo until the round resolves. At resolution both outcomes apply in seat order on the same step and become public; combat resumes on the next step for both |

| Rarity | Cost | Forgo |
|--------|------|-------|
| Bronze | 3 | 0 (canon) for any round |
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
| BR-X09 | Revised by the owner (2026-10-29): Augment copies are unlimited, so max-HP, damage and Stronghold-damage percentages from copies grow linearly without a cap (saturating `i64`/`u16` arithmetic). Attributes with a cap keep it: attack speed (BR-X02), move (BR-X03), damage reduction (BR-X05), lifesteal (BR-X06), status duration (BR-X07), heal bonus and heal cut ≤ 100 %, Bounty ceiling (BR-E08); every timed effect keeps its cooldown or window |
| BR-X10 | Stronghold healing exists only through named effects with a per-window cap and never exceeds max HP; at most one Stronghold shield exists at a time and it absorbs unit and Stronghold damage only |
| BR-X11 | Conditional effects read the state when they apply — position and HP-fraction conditions at hit resolution, Synergy-count conditions after that tick's Synergy update — in the simulation's fixed seat and unit-id order; nothing caches a condition across ticks |
| BR-X12 | **Proposed (D-60).** A Cargo Crab's carry bonus is one move source, `max(base, branch)`, never their sum. Base +2 000 bp; branch none on A1 and A2, +2 000 on B1 (Faster Legs), +2 000 on B2 steady and +4 000 on B2 during its 4 s Overdrive surge. So A1 = A2 = B1 = B2 = +2 000 bp and a surging B2 = +4 000 bp. It joins the other move bonuses in Σbp under BR-X03 (≤ +5 000). A crab that carries moves at `max(crab speed, passenger speed) × (10 000 + Σbp) / 10 000`; each speed is read after its own Slow, so a slowed passenger lowers the pair only down to the crab's own speed |

### 10.8 Augment catalog (40, Proposed)

Index order is a serialization contract (P16-F5). **U** = Universal
(useful to any army); otherwise the tag names who benefits. Percentages
use BR-X01–X07. Per-Augment analysis (interactions, counters, exploits,
tests, hypotheses) lives in [P16-F6](phases/P16F6-augment-catalog.md).
Owner revision 2026-10-29: all 40 are active (★ marked the planned first
batch only), and each copy adds its full effect again (BR-AG08).

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
Universal per rarity: 3 / 4 / 7 / 4 (★ subset: 2 / 3 / 3 / 3).

### 10.9 Balance targets for the expansion (Proposed, P16-F gates)

**Not measured.** The owner revision went live without running these
gates; no `tbg-balance` evidence exists yet for uncapped `Energy`,
Interest or infinite Augments, and the reference bot forgoes every
round. Rows about `Siege`, `Collapse`, running-time bounds and Augment
triples no longer apply.

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

## 11. Roster expansion (active, values Proposed)

Fifteen new units are specified in [ROSTER-EXPANSION.md](ROSTER-EXPANSION.md). Since the owner switch of 2026-10-30 (D-50) all fifteen are in the `Shop` online, ahead of the P50 tuning and P53 plan: their rules are active and every number stays **Proposed** until the balance lab confirms it (D-33).

| Unit | Faction | Class | Cost | HP | Dmg | Every | Range | Speed | Identity in one line |
|------|---------|-------|------|----|-----|-------|-------|-------|----------------------|
| Rotcap | Plants | Ranged | 3 | 200 | 12 | 1.5 | 3.5 | 0.9 | Each hit adds a poison stack (6/s, 10 s, ≤ 6 per victim) that outlives it |
| Farwatch | Humans | Ranged | 3 | 150 | 70 | 4.0 | 12.0 | 0.9 | **Proposed (P71 owner, 2026-11-01):** shoots units or the enemy Stronghold within its existing range; normal nearest-target and Guard rules apply |
| Leechroot | Plants | DPS | 3 | 300 | 10 | 1.6 | 1.2 | 0.9 | Damage dealt to enemy units becomes permanent stacks, no maximum; lost on death |
| Bombard | Machines | Ranged | 5 | 1 000 | 24 | 2.2 | 4.0 | 0.5 | Blast radius 1.0 hits every unit, both sides and itself |
| Old Finn | Humans | Control | 3 | 240 | 8 | 2.0 | 5.25 | 0.9 | Fish: −15 % move each, ≤ 3, 5 s; never slows attacks |
| Drumguard | Humans | Support | 3 | 260 | — | — | 2.0 | 1.0 | Bond to the nearest ally: +25 % move, −15 % damage taken |
| Longaxe | Humans | DPS | 4 | 360 | 100 | 2.4 | 3.0 | 0.7 | Heavy melee that strikes over its own front line |
| Oilpress | Machines | Tank | 7 | 3 600 | 25 | 2.5 | 0.8 | 0.3 | Most HP in the game; while engaged, own units cannot walk past it |
| Antidote Brewer | Plants | Support | 3 | 260 | — | 4.0 | 3.0 | 0.9 | Cleanses the most afflicted ally and wards it 1 s |
| Puppetmaster | Undead | Control | 4 | 260 | — | 8.0 | 4.0 | 0.9 | The hardest-hitting enemy in range strikes its own ally once |
| Smuggler | Humans | Support | 6 | 320 | 10 | 1.5 | 6.0 | 0.6 | Revolver; drops crates 0.4 tile behind; every 20 s a crate opens into a random unit — or nothing |
| Shatterback | Plants | Tank | 4 | 560 + 4 × 100 plates | 30 | 1.5 | 0.8 | 0.55 | Each broken plate: permanently faster moves and attacks |
| Blood Furnace | Undead | DPS | 4 | 440 | 36 | 1.5 | 0.8 | 1.08 | Each hit burns 6 % max HP; damage grows up to ×4 as HP falls |
| Bloodweaver | Undead | Support | 3 | 300 | 30 | 2.0 | 3.0 | 0.9 | Siphon: actual enemy-unit HP removed heals the lowest-percent wounded conventional ally anywhere in the lane |
| Cargo Crab | Machines | Tank | 5 | 1 400 | 20 | 1.5 | 0.8 | 0.5 | Carries one non-Tank ally: pair moves at max(crab, passenger) × (1 + carry bonus), the bonus being max(+20 % base, branch), BR-X12; single-target attacks hit crab first; dismounts on death with intact HP and 3 s lock (§C.15 of ROSTER-EXPANSION; D-56, D-60, Proposed) |

## 12. Arenas (active, values Proposed, ARENAS-G, D-55)

Every match is played in one of eight `Arena`s. Each applies small modifiers,
**the same for both sides**; the Forest is the neutral reference.
Values are **Proposed** until the balance lab confirms them (D-33).

| Arena | Effect on both sides |
|-------|----------------------|
| Verdant Forest | None |
| Frozen Tundra | Troop movement speed −10 % |
| Volcanic Forge | Troop attack speed +8 % |
| Ancient Ruins | Each player's first successful `Evolution` costs 1 `Energy` less |
| Storm Valley | Base `Energy` regeneration +8 % |
| Celestial Citadel | **Proposed:** troop movement speed +8 % |
| Clockwork Bastion | **Proposed:** troop attack speed +8 %, movement speed −5 % |
| Desert Relics | **Proposed:** base `Energy` regeneration −8 % |

- **BR-AR01** Online, the server draws the `Arena` uniformly at pairing
  (every mode: casual, ranked, rooms, challenges, rematches), before the
  match starts, independently of the match seed. Both players and every
  reconnect see the same `Arena`; each new match draws again. Clients
  never choose an online `Arena`.
- **BR-AR02** Practice uses the `Arena` the player chose on the Arenas
  page (Forest by default); "Play again" keeps it.
- **BR-AR03** Troops are every unit on the `BattleLine`, summons and
  carried cargo included; `Stronghold`s, `Augment`s and `Arcana`s are
  unchanged. Movement and attack-speed effects join the other basis-point
  bonuses with one truncation (BR-X01): interval = base × 10 000 /
  (10 000 + attack bp), then the existing caps.
- **BR-AR04** The Ruins discount is spent only by a successful `Evolution`
  (a rejected one keeps it), once per player, stacks with `Augment`
  discounts, and never takes the price below 1 `Energy`. The view shows
  each player's current `Evolution` prices.
- **BR-AR05** The Storm bonus adds 800 bp to the base regeneration beside
  Interest (BR-E05); the existing remainder carries the rounding. Desert
  subtracts 800 bp from that same base; it does not multiply Interest.
- **BR-AR06** **Proposed, owner-requested expansion (2026-11-01):** append
  Celestial, Clockwork and Desert after the original five identities. Online
  draws stay uniform over the entire catalog. Celestial adds 800 movement bp;
  Clockwork combines +800 attack-speed bp and −500 movement bp; Desert adds
  −800 base-regeneration bp. Apply all fields once through BR-AR03/05, with
  existing caps and rounding. No Faction, Stronghold, damage, HP, geometry,
  price or resource-cap exception. These initial playtest values are **Proposed**
  pending D-33 evidence; the five existing arena values stay unchanged.
  Protocol 30 protects older peers from the appended Arena variants.

## 13. P58 roster expansion — Proposed, inactive

**Every new value and rule in this section is Proposed: an initial balance
hypothesis, not an approved rule, measured result or active roster entry.**
No existing section, production value, Shop pool or gate threshold is changed.
The owner requested planning only on 2026-10-30 and specified that text,
never image labels or statistics, controls character names and gameplay.
Execution requires subsequent authorization and the dependencies in
[P58](phases/P58-roster-expansion-planning.md). The [catalog](roster/P58-CATALOG.md)
defines identity/art and the [integration contracts](roster/P58-AUDIT-AND-INTEGRATION.md#3-shared-competitive-contracts-all-proposed-inactive)
define resolution semantics. This section is the single numerical registry.

A value marked **completion** is not stated by the owner's text. It is
Proposed for review and listed, with its owner decision, in 13.6.

**Proposed implementation state (D-71, 2026-10-31):** the shared independent
six-tick special windup and the separate non-stacking venom family are bounded,
serialized UnitFx slots. Both start empty on legacy units. This extends replay
state bytes without changing their rules, numbers, Shop pools or expected match
outcomes. The targeted P59 pilot consolidation measured all three legacy seeds
with identical winner/reason/tick before re-freezing their hashes (D-71).
No expected outcome/tick changed to accommodate the layout.

**Proposed counter policy (D-74):** Ward blocks a new hostile mark/Slow/ability-cut
payload, while raw basic damage still resolves; Oilpress's control immunity blocks
Slow, not the independent mark/cut. Brew clears new weakening and ability-cut slots
and every Crowcaller mark on its target without clearing Ravager prey sequences or
legacy Augment/form cuts. Each active new harmful family (mark, weakening, ability
cut, bleed, venom) counts once for Brew eligibility, using the existing one-stack
score unit. New one-slot DoTs still consume strip quota before ordinary poison.
This extends only the new families; the existing Brew target-ranking policy stays.

### 13.1 Proposed base forms

Cost in Energy (×600 in state); interval in ticks (20/s); range in
milli-tiles (1,000/tile); movement in milli-tiles/tick (÷50 = tiles/s).
Draft speed conversion rounds halves upward. For example 0.55 tiles/s
becomes 28, actually 0.56 tiles/s; 0.45 becomes 23, actually 0.46.
HP, damage and healing are integer HP. DPS is damage ×20/interval,
before windup interruption, Synergy, Augments, Arcanas or ability value.
All base healing is zero; none of these texts specifies an ordinary healer.

| ID | Text name | Proposed Faction / Archetype | Cost | HP | Damage | Interval | Range | Move | Base DPS |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| R01 | Bastion | Machines / Tank | 6 | 1400 | 30 | 40 | 800 | 25 | 15.00 |
| R02 | Demolidor | Machines / Tank | 6 | 1250 | 80 | 32 | 1000 | 28 | 50.00 |
| R03 | Crowcaller | Undead / Ranged | 4 | 520 | 58 | 34 | 4800 | 31 | 34.12 |
| R04 | Gatilho | Humans / Ranged | 4 | 230 | 25 | 17 | 3200 | 53 | 29.41 |
| R05 | Matraca & Tico | Humans / Tank | 5 | 900 | 60 | 38 | 900 | 28 | 31.58 |
| R06 | Capivardo | Plants / Tank | 5 | 1300 | 38 | 40 | 800 | 23 | 19.00 |
| R07 | Tridente | Plants / Tank | 6 | 1250 | 55 | 40 | 1100 | 28 | 27.50 |
| R08 | Grumak | Monsters / Tank | 6 | 1600 | 40 | 40 | 800 | 20 | 20.00 |
| R09 | Rasgarr | Monsters / Dps | 4 | 310 | 48 | 24 | 800 | 58 | 40.00 |
| R10 | Esporax | Monsters / Ranged | 4 | 260 | 42 | 36 | 4200 | 38 | 23.33 |
| R11 | Murmúrio | Monsters / Control | 5 | 380 | 15 | 40 | 3000 | 33 | 7.50 |
| R12 | Sentinela Solar | Celestials / Tank | 5 | 1050 | 35 | 40 | 800 | 28 | 17.50 |
| R13 | Oráculo Luminar | Celestials / Control | 4 | 300 | 15 | 50 | 3000 | 38 | 6.00 |
| R14 | Arqueira Celeste | Celestials / Ranged | 4 | 210 | 55 | 40 | 4000 | 43 | 27.50 |
| R15 | Justiçar | Celestials / Control | 6 | 700 | 35 | 46 | 2500 | 28 | 15.22 |
| R16 | Mandíbulo | Insects / Tank | 6 | 1400 | 45 | 40 | 800 | 23 | 22.50 |
| R17 | Louva | Insects / Dps | 4 | 280 | 50 | 24 | 800 | 60 | 41.67 |
| R18 | Vespira | Insects / Ranged | 5 | 260 | 35 | 30 | 3800 | 45 | 23.33 |
| R19 | Fiandeira | Insects / Control | 4 | 350 | 15 | 50 | 3000 | 33 | 6.00 |
| R20 | Crematumba | Undead / Tank | 6 | 800 | 12 | 10 | 2800 | 23 | 24.00 per eligible victim |
| R21 | Brocatanque | Machines / Ranged | 5 | 900 | 0 | 44 | 4500 | 23 | 0; drills are its output |

R01's entire base row is a completion hypothesis: T01 is incomplete.
R20's cost/HP/movement and Tank assignment are completion hypotheses;
damage, interval and range come from T11's flame. Other base rows preserve
the text apart from integer speed conversion. No printed PNG number is
used. R21's interval is its production cadence, not a zero-damage attack.

The table preserves the P59 reference. Its R07 range is superseded in the current
P70 candidate by the explicitly Proposed 2,000-unit reach in §14 below; other
R07 base values remain unchanged. D-101 applies that documented override to
current candidate assertions and retains the original table and revision-2
deterministic references. Ruleset 6 is not a D-33-approved balance release.

### 13.2 Proposed primary abilities

Times are ticks; percentages use basis points (1%=100 bp). Parameters
absent from a text are explicitly completed here; they are not attributed
to the owner. Common windup is the existing six ticks. Unless specified,
special range equals base attack range and ability target is an enemy unit.

| ID | Ability and complete numerical parameters |
|---|---|
| R01 | Muralha Viva: period 160; anchored self-reduction 1500 bp for 60; no Guard/taunt. Entire base ability is a completion hypothesis |
| R02 | Esmagamento em Área: period 200; caster-centered radius 2500; damage 120; no displacement or true interruption |
| R03 | Marca do Corvo: readiness every 140; next valid basic hit marks for 80; Slow 2000 bp, heal-cut 3000 bp, own basic damage vs own marked target +1500 bp |
| R04 | Tiro Cruzado: every fourth completed enemy-unit primary hit; one additional target within 1500 of primary; secondary damage 6000 bp of current static basic damage |
| R05 | **Proposed P71 owner change (2026-11-01): Tick pistol**, period 30 ticks (1.5 s), range 3000, damage 12; nearest targetable enemy troop with Guard priority; single target, no Slow/splash; independent of Rattle club, movement and counters. Existing six-tick locked special windup, root/death cancellation and seat damage bonuses apply; Rattle Evolution damage bonus is excluded |
| R06 | Barrigada do Brejo: period 200; caster-centered radius 1500; damage 65; Slow 1500 bp for 40 |
| R07 | Rasgo Sangrento: period 180; primary-centered radius 1500; damage 75; one bleed slot, 5 HP per 20 ticks, four pulses |
| R08 | Couraça Primordial: each positive HP-damage receipt adds 300 bp reduction for subsequent hits, five stacks maximum / 1500 bp; all expire after 60 ticks without HP damage. Step size is a completion |
| R09 | Frenesi Predatório: next consecutive hit against same unit gains 800 bp per previous hit, maximum three increments / 2400 bp; reset after 60 ticks without a valid hit |
| R10 | Cuspe Corrosivo: every third completed enemy-unit primary hit; flat damage 18 to at most two other enemies within 1500 of primary. Radius is a completion |
| R11 | Pulsação do Medo: period 160; Slow 2500 bp for 40 on primary; no separate ability damage |
| R12 | Barreira Solar: period 160; lowest HP-fraction other living ally within 3000, ties distance/id; shield 100 for 80; no self-shield. Range/exclusion are completions |
| R13 | Pulso Harmônico: period 200; weaken enemy basic damage by 1500 bp for 60, maximum weakening 2000 bp across this family; no healing pulse |
| R14 | Flecha da Purificação: every fourth completed enemy-unit hit replaces that basic damage with 80 and removes its temporary Unit shield immediately before damage; no piercing chain |
| R15 | Chuva Celeste: period 200; one damage pulse 65, primary-centered radius 1300; no persistent ground zone |
| R16 | Carapaça Erguida: period 200; self-reduction 1500 bp for 60; no allied aura |
| R17 | Golpe Preciso: fourth consecutive completed hit on same enemy unit adds 60 damage, then sequence resets; target change resets; no dash |
| R18 | Ferrão Venenoso: every third completed enemy-unit hit applies one non-stacking poison slot, 5 HP per 20 ticks, four pulses |
| R19 | Teia Adesiva: period 160; damage 25 plus Root 30 on primary |
| R20 | Fogo Profano: each 10-tick basic cycle applies damage 12 to every other living unit in the forward segment of length 2800, allies and enemies equally; heal-cut 3500 bp for 40, no target cap. Self excluded. Primary enemy Stronghold may take one ordinary 12-damage hit, never collateral flame/cut; tower treatment is a completion |
| R21 | Ninho de Brocas: every 44 ticks with a target within 4500, spawn one Microbroca at producer x; no direct damage. Eight living drills total per seat across producers/ranks; same-rank allied drills within 200 may fuse; maximum rank three. Trigger/radius are completions |

### 13.3 Proposed Evolution parameters

Columns are **Tier 2 / Tier 3** (stored tiers 1/2). Stat deltas accumulate
on base; absolute ability values **replace**, not add. A Tier 3 form keeps
its own Tier 2 benefits. Existing Energy price is 2 then 4; no new price
curve is proposed. Names identify the original branch, not extra systems.
Where the source gives a single unsplit benefit, intermediate values or
an additional modest benefit below are completion hypotheses for testing.

| ID | Branch A: Tier 2 → Tier 3 | Branch B: Tier 2 → Tier 3 |
|---|---|---|
| R01 | Fortaleza Móvel: +2000 bp HP → anchor duration 80 | Peso da Terra: +1500 bp basic damage → basic hits Slow 1500 bp for 20 |
| R02 | Punhos Reforçados: +2000 bp special damage → radius 3000 | Sistema de Choque: period 160 → basic hits Slow 1500 bp for 40 |
| R03 | Corvo Pestilento: cut 4000 bp → mark duration 110 | Besta da Ruína: +1500 bp basic damage → own mark bonus 2500 bp |
| R04 | Pistoleiro Veloz: +1500 bp attack speed → +1000 bp movement | Bala Ricocheteante: secondary ratio 7500 bp → search radius 2000 |
| R05 | Fúria do Grandão: +2000 bp HP → +2000 bp basic damage | **Proposed P71:** Tick pistol: period 24 → damage 18. Rattle branch remains HP +2000 bp → club damage +2000 bp; each branch changes only its brother’s attack channel |
| R06 | Barriga de Pedra: +2000 bp HP → 1500 bp self-reduction for 40 after impact | Onda do Brejo: period 160 → radius 1900, Slow 2000 bp |
| R07 | Fúria Oceânica: +2000 bp HP → bleed 7 per pulse | Maré Sangrenta: period 150 → radius 1900 |
| R08 | Couraça Ancestral: +2000 bp HP → stack expiry 80 | Presas de Pedra: basic damage +1500 bp vs units with base range ≤1600 → replace bonus with 2500 bp |
| R09 | Caçada Voraz: +1500 bp attack speed → +1000 bp movement | Frenesi Profundo: maximum four increments / 3200 bp → reset gap 80 |
| R10 | Mandíbula Abissal: +1500 bp basic damage → +400 range | Salva Corrosiva: secondary damage 24 → maximum three additional targets |
| R11 | Eco do Medo: +500 range → Slow duration 50 | Pulso Mental: period 130 → +2000 bp basic damage |
| R12 | Fortaleza da Luz: +2000 bp HP → shield 130 | Proteção em Grupo: second distinct ally shield 50, same expiry → +500 special range |
| R13 | Luz Restauradora: weakening duration 70 → duration 80 | Aura Sagrada: period 180 → period 160; branch names do not grant healing/aura |
| R14 | Precisão Divina: +1000 bp basic damage → another +1000 bp basic damage | Marca Celeste: proc every third hit → empowered damage 90; no team damage mark |
| R15 | Fúria Celeste: +2000 bp special damage → +1000 bp HP | Julgamento: radius 1600 → Slow 1500 bp for 20 on affected enemies |
| R16 | Carapaça Reforçada: +2000 bp HP → protection duration 80 | Guarda do Enxame: next enemy-unit primary after protection ends adds 40 → +1500 bp basic damage; no area aura |
| R17 | Caça Implacável: +1500 bp attack speed → +1000 bp movement | Lâmina Precisa: bonus 70 → bonus 85 |
| R18 | Veneno Profundo: poison 6 per pulse → poison 7 per pulse | Revoada: +1500 bp attack speed → +400 range |
| R19 | Rede Ampliada: period 145 → period 130 | Teia Protetora: +500 special range → Slow 1500 bp for 40 after Root ends; no shield |
| R20 | Fornalha Fúnebre: +2500 bp HP → flame range 3360 | Chama da Pestilência: +2000 bp flame damage → cut 5000 bp; friendly damage remains identical |
| R21 | Fábrica de Enxame: production period 38 → merged drill shield 25 for 40; merge radius remains 200 | Broca de Cerco: Duobroca damage +1500 bp → Megabroca damage +2500 bp vs units whose printed base HP ≥1000; no Stronghold bonus |

Names/secondary completions for R08–R11 and unsplit R12–R19 branches are
proposals because the texts do not define every name/step. They preserve
the text's role rather than importing image abilities. R16's empowered
flag stores one charge, never accumulates; consumed by next valid unit
hit, overwritten by the next completed protection if unspent. R19 keeps
one pending post-Root Slow; cleansing clears it and any extended Root
delays it. R21 Evolution benefits snapshot at summon birth; merge uses the
lowest-id parent producer's recorded branch/tier, not a stronger ally's
current upgrades. All resulting rank stats derive from that profile.

### 13.4 Proposed drill summon forms and fusion

| Summon | Rank | HP | Damage | Attack ticks | Move/tick | Range | Lifetime ticks |
|---|---:|---:|---:|---:|---:|---:|---:|
| Microbroca | 1 | 45 | 18 | 14 | 95 | 600 | 240 |
| Duobroca | 2 | 110 | 40 | 17 | 73 | 600 | 280 |
| Megabroca | 3 | 260 | 90 | 22 | 50 | 600 | 320 |

Rank is a summon form, not purchasable Evolution. Range and Duobroca's
integer-rounded speed are completion hypotheses. The eight-entity cap,
finite lives, two-equal-rank fusion and maximum rank three come from T12.
At cap, production waits ready; no queued burst or refunded Energy.
Fused lifetime restarts at its rank lifetime; with only two promotions
a Microbroca lineage lives at most 240+280+320 ticks. See the catalog for
damaged parents, control/status inheritance, ownership and merge order.

### 13.5 Proposed Factions and release hypotheses

Append Monsters / Celestials / Insects after the existing four Factions;
all use the existing thresholds 2/4 living conventional units. Monsters:
damage +500/+1000 bp (completion). Celestials: max HP +500/+1000 bp (T09).
Insects: movement +500/+1000 bp (T10), sharing the existing +5000 bp
movement ceiling. Bonuses replace the lower threshold, never add twice.
No new healing, flight, armor, stealth, knockback, Augment or Arcana system.

The online Shop pool is 25 kinds today (D-50, `CANON.pool`). With all 21
purchases it would be 46 (25 + 21). These are **models**, not a Shop change.
Five distinct uniform offers imply per-kind probability 5/N, pair
co-occurrence 20/[N(N−1)], and expected fresh offer sets N/5. Never call
these expected *free rerolls*: another purchase is required to redraw.
No global economy/Shop rule is changed to compensate.

The current purchase-share gate is 2.5 % per kind (`tbg-balance/src/gate.rs`,
O-R2), chosen for 25 kinds. The even share at 46 kinds is 2.17 %, so 2.5 %
cannot hold there; 0.6/N gives 1.30 % at 46 kinds (2.4 % at 25).
**Owner decision pending:** consider a floor of 0.6/N per kind (60% of
uniform share), alongside conditional purchase-when-offered and enough
observations. Do not change an existing threshold or contract to pass.

Future measurement protocol (Proposed methodology, not balance proof):
256 paired seeds for screening; 2048 disjoint confirmation pairs; reaction
intervals 1/3/5 views; all existing Energy-equal duel budgets 20/40/60;
candidate combos investigated above 60% score against the strategic pool;
ability-value and Evolution buy-versus-base ablations; incomplete games
reported as censored at the lab's own limit, never converted into gameplay
draws. Keep pre-existing gate failures visible separately. No release
until owner approves any new thresholds, evidence and fun/look.

### 13.6 Completion register (Proposed, for owner review)

| ID | Completed value | Where | Owner decision |
|---|---|---|---|
| C1 | R01 base row: Machines / Tank, cost 6, HP 1 400, damage 30, interval 40, range 800, move 25 | 13.1 | Bastion base completion (T01 is a fragment) |
| C2 | R01 Muralha Viva, whole ability: period 160, 1 500 bp for 60, no Guard or taunt | 13.2 | Bastion base completion |
| C3 | R20 cost 6, HP 800, movement 23, and the Tank assignment | 13.1 | Crematumba cost, HP and movement |
| C4 | R20 basic-attack treatment (12 damage every 10 ticks over a 2 800 segment) and the Stronghold rule (one ordinary 12-damage hit, no collateral) | 13.2 | Crematumba |
| C5 | R08 step size: 300 bp per stack | 13.2 | Review only |
| C6 | R10 secondary radius: 1 500 | 13.2 | Review only |
| C7 | R12 range 3 000, and the no-self-shield exclusion | 13.2 | Shield semantics (R12/R14) |
| C8 | R21 trigger (target within 4 500) and radii (4 500 trigger, 200 merge) | 13.2 | Brocatanque production and radius |
| C9 | Drill summon Range 600 (all three forms); Duobroca speed 73 (integer-rounded) | 13.4 | Brocatanque |
| C10 | R21 merge resolution (lowest-id parent's branch and tier, snapshot at birth) and the two-step Evolution (gaps in T12) | 13.3, 13.4 | Brocatanque; no-fusion fallback |
| C11 | Monsters Synergy damage +500 / +1 000 bp | 13.5 | Monsters damage Synergy |
| C12 | Evolution names and secondary values for R08–R11 and the unsplit R12–R19 branches; names the texts do not define are Portuguese placeholders | 13.3 | Owner naming; English-only rule |

C1–C4 come from the audit's missing-information note (audit §1) and the
prose above the tables. C5–C12 are marked in the prose of 13.1–13.5. The
source texts T01–T12 were not re-read for this register, so a reviewer should
confirm that every unmarked value is in the owner's text.

### 13.7 Decisions taken by the agent (Proposed)

- **Unit shield (R12, R14):** one amount and one expiry per unit; a stronger
  grant replaces, an equal one refreshes, a weaker one is ignored. The Cargo
  Crab `shield_bp` stays dormant (D-63).
- **Factions (§13.5):** Monsters, Celestials and Insects are appended. Their
  Synergy is 500 / 1 000 bp, and Insects movement shares the 5 000 bp move cap
  (D-64).
- **Names and owner items 1–8:** English names and the values in D-65 and
  `docs/roster/P59-NAMES.md`.


## 14. Live battles and spectating (P47, Proposed)

Anyone may watch a match that is being played from the public queues, in
real time, from the lobby. Watching is listen-only and shows only public
information. Values are **Proposed** (owner defaults O-1–O-6 in
`docs/phases/P47-live-spectate-and-presence.md`).

- **BR-LV01** Only live matches paired from the **public queues** (Casual
  quick match, Ranked) are listed and watchable. Private rooms, challenges,
  rematches and practice never are.
- **BR-LV02** Any session (guest or account) may watch while **idle**.
  Watching is listen-only: a spectator has no seat, and any `Intent` or
  `Surrender` from it is refused like `NotInMatch`.
- **BR-LV03** A spectator receives only the public view: no Energy, regen,
  Shop offers, `last_seq`, Evolution prices, RNG, or a pending Augment
  pick (BR-S05, BR-AG14). Augment options of an open round are public; each
  seat's *resolved* flag is public, its choice only after resolution.
- **BR-LV04** The spectator view has no delay (0 s).
- **BR-LV05** Both players see the number of spectators, never who.
- **BR-LV06** Presence: `online` counts sessions with an attached
  connection, accounts once; `in_battle` counts the seats of running
  listed matches. Online accounts are listed with name, rating and status
  (`idle`, `queued`, `playing`, `watching`). Guests are counted, never
  listed. No ids, IPs, tokens or `LiveId` of a private match leave the
  server.
- **BR-LV07** The featured battle is the listed match with the highest
  mean rating (a guest counts as 1000), ties broken by the longest running
  and then the lowest `LiveId`. Once featured it stays featured for at least
  30 s unless it ends.
- **BR-LV08** A listed match has a random, non-sequential 64-bit `LiveId`
  drawn from the OS RNG. It is valid only while the match is live and is
  shareable as `?watch=<16 hex>`.
- **BR-LV09** Caps: 200 spectators per match and 2 000 in total. A full or
  unknown match answers `SpectateUnavailable`, never a disconnect. A slow
  spectator is dropped without ever affecting a seat.

Champion adjustments (owner 2026-10-31, **Proposed**): Longaxe range 3.0,
Old Finn range 5.25, Leechroot HP 300, Blood Furnace interval 30 ticks and
speed 54, Nester range 3.0, Smuggler damage 10 / interval 30 / range 6.0.
Cargo remains every 400 ticks (branch B 360); Faster Deliveries no longer
modifies attack speed. CRATE_DROP_BACK = 400 milli-tiles, clamped to the lane.

Bloodweaver Siphon (**Proposed**, owner 2026-10-31): attacks normally;
primary damage to enemy units heals the lowest exact HP fraction among
living wounded allied conventional units (self included), ties nearest then
lowest id. No Stronghold healing trigger; no overkill or shield/plate damage
counts. Healing uses Support bonuses, heal cut and Purification, separately
from the BR-X06 lifesteal cap. Base heal_bp 10,000; A1 adds 3,000 damage bp;
A2 doubles healing below 20% HP. B1 adds 2,500 attack bp; B2 replaces Siphon
with heal_bp 12,000 split equally across up to two recipients chosen before
healing. Event::Transfer now means enemy-to-recipient, actual HP restored.

PR #21 consolidation correction (owner 2026-10-31, **Proposed**, D-68):
Smuggler attack range is 6.0 tiles, obeying the existing Stronghold range
cap. Protocol-20 roster slots and preview-only Shop state coexist in the
deterministic state; hash goldens are recorded for that integrated schema.
Reserve/Reroll stays disabled online pending its separate owner checkpoint.

**R14 dispel policy — Proposed (D-81):** Arrow of Purification removes the
victim's temporary Unit shield as immediate damage preprocessing. Ward's hostile
status immunity does not block this dispel or get consumed by it. It removes no
plate, Stronghold shield, status, HP or max-HP modifier.

**R16 primary-bonus reading — Proposed (D-83):** the expiry-earned retaliation
is added once to the next valid primary's raw basic damage, with the registry
bonus scaled by static caster percentages. The combined primary then uses normal
basic weakening/conditional modifiers and victim protection as one hit. A fully
shield-absorbed valid primary still consumes the charge; no second on-hit is run.

**R17 sequence reading — Proposed (D-84):** the fourth-hit bonus joins one
primary under the R16 primary-bonus reading. Changing target, including changing
to a Stronghold, clears the sequence; a canceled same-target windup never advances
or clears it. Idle time alone never expires it. Target death clears it.

**R18 cleanse order — Proposed (D-85):** a Brew's limited DoT strip budget takes
bleed first, Hornet's separate venom second, then existing poison stacks in their
previous strongest-first order. Each removed family spends one strip. Ward blocks
a new venom payload but never cancels an already applied DoT. Venom uses the same
static-on-application pulse scaling and refresh phase contract as R07 bleed.

**Proposed R19 implementation reading (D-86):** post-Root Slow duration snapshots
outgoing duration Augments and victim Arcana once when the web applies. Its queued
transition checks current Ward/control immunity and ordinary Slow priority; it
cannot replace a refreshed Root. Brew discards the queue with the cleansed Root.
The B range bonus affects only the independent special, never basic attack range.

**Proposed R20 implementation reading (D-87):** Profane Fire replaces the ordinary
basic output after common betrayal handling. Static basic damage (including B)
and current weakening apply once and identically to every eligible unit; it never
uses target-conditional offensive procs, lifesteal or Essence. Surviving victims
receive cut even if shields absorbed damage; current Ward prevents a new cut.
The primary Stronghold takes that one basic amount, with no collateral tower/cut.
An appended public event carries both exact authoritative segment endpoints.

**Proposed R21 implementation reading (D-88):** three appended summon kinds encode
rank; existing tier/branch and summoner record the producer's birth profile. They
never enter Shop pools or accept purchased Evolution. Global arena modifiers still
apply as to existing summons, with no Arcana, Synergy or general Augment inheritance.
Fusion preserves active Ward/confusion-immunity deadlines and self-reduction
strength/duration as well as the catalog's negative families. No haste, bond or
carrier link survives. Parent replacement emits fusion rather than Death.

**Proposed R21 external Mark reading (D-88):** hostile Crowcaller Marks follow
the fused replacement with their existing deadlines. Offensive focus sequences
do not transfer to the new target id. Fusion therefore never strips hostile Mark.

**Proposed R21 control inheritance clarification (D-88):** choose Root over Slow
and preserve the maximum remaining control duration across both parents. A longer
Slow on one parent therefore extends the inherited Root; fusion cannot erase that
parent's remaining control window.
## 16. Troops and local training (P62, Proposed)

**Proposed — planning only, 2026-10-31.** The owner requested a Troops
catalog, champion animation/lore/stats, exact-team training against chosen
champions or dummies, and Easy/Medium/Hard/Impossible bots. Publication of
this plan does not activate these rules. See
[P62](phases/P62-troops-training-lab.md) and its
[acceptance cases](planning/TROOPS-TRAINING.md). Sections 14–15 are reserved
by the separate P60/P61 planning branch; this gap is intentional.

- **BR-TR01 — Access and roster.** The top Troops icon and lower card open
  one catalog for guests and accounts while idle. Playable entries come
  from the reviewed active roster, not every declared kind. Staged kinds
  are labeled In development; summons are subordinate entries. Reading
  does not unlock content or alter a ranked roster.
- **BR-TR02 — Complete champion information.** Every active champion has
  reviewed lore, Faction, Archetype, abilities, conditional effects and all
  available Evolution forms. Numeric stats derive from canonical content
  and effective-stat functions with explicit neutral/scenario/live context;
  no second balance table. Show costs, HP, damage, interval, range, speed,
  splash, healing, statuses and ability-specific parameters with units.
- **BR-TR03 — Preview.** Idle, Walk, Attack/Action and Dance use the actual
  champion/form and appropriate rig. Passive supports show a real action.
  Preview motion has no combat or economy effect; Dance requires reviewed
  animation and is not a hidden gameplay action.
- **BR-TR04 — Exact teams.** The combat laboratory accepts 1–12 initial
  units per side, including duplicates and valid form/position choices,
  or 1–5 dummies on the target side. Three selected units means exactly
  those three at Start. Ability summons obey normal rules and do not count
  as selected slots. The limits are local safety limits, not online caps.
- **BR-TR05 — Controlled combat.** Laboratory teams use the canonical combat
  order and abilities, with Shop, Energy spending, councils, tower fire and
  Stronghold auras disabled. Defaults are neutral Arcanas/Augments, Forest
  and automatic Synergy from real champions; advanced supported modifiers
  are explicit in setup/results. A chosen three-kind team never replaces
  the five-distinct-offer Shop pool. Full practice retains canonical match
  rules and the normal Shop; exact enemy selection is a laboratory option.
- **BR-TR06 — Dummy semantics.** Wooden dummies are inert summon-like
  targets: no attacks, movement, cargo, Bounty, Synergy contribution,
  Evolution or expiry; zero innate damage reduction. HP presets: 1,000 or
  10,000. Finite targets die normally. Renewable targets die, then respawn
  cleanly next tick with a new ID; counters retain the trial history.
  Effects requiring conventional enemies do not trigger; explain this and
  offer chosen champion opponents. Do not imply universal target eligibility.
  An empty renewable target side awaiting respawn does not end the trial;
  elimination of all attacking units still does.
- **BR-TR07 — Control and limits.** Start, Pause, Resume, one-tick Step,
  Reset, edit setup and Exit are local controls. Config edits apply to a
  new trial; same seed/config/commands/content reproduce it. Speed is 1x.
  Focus loss pauses the laboratory. Side elimination ends finite combat;
  mutual elimination draws. At 300 active seconds, 256 living entities,
  or 4,096 retained instance-result rows, pause and preserve the trial
  rather than fabricate a winner, lose counters or delete units silently.
- **BR-TR08 — Measurements.** Count actual HP removed/restored, excluding
  overkill/overheal; report shields/plates separately. Attribute direct,
  periodic, splash, friendly and summon effects by actual source. Team
  totals count each mutation once. DPS is HP damage times TICK_HZ divided
  by active ticks since Start/reset; paused time is excluded and zero time
  shows no rate. Finite kill time and per-life renewable kill times are
  distinct. Nominal basic-attack DPS is labeled as an estimate.
- **BR-TR09 — Fair profiles.** Easy/Medium/Hard bots consume their own legal
  view and send ordinary PlayerIntentions, with no hidden opponent data or
  resource/stat advantages. At a 10 Hz view cadence, proposed decision
  spacing is Easy 15–25 steps, Medium the existing roughly 4–9-step Bot,
  Hard 2–4 steps. Hard uses a bounded deterministic legal-action scorer;
  the existing generic Bot remains the baseline for other callers.
- **BR-TR10 — Impossible proposal.** Recommended: Hard policy at 1–2 view
  steps plus a visible 2x ordinary Energy regeneration advantage for the
  bot, practice-only. No starting-Energy, Bounty, combat-stat or hidden-data
  advantage; no extra regen during countdown, council or pause. The owner
  must select this assisted proposal or fair-only alternative before its
  activation (P62 O03). The label does not guarantee an unbeatable bot.
- **BR-TR11 — Isolation.** All experiments and practice results stay local.
  They cannot affect rating, ranked stats, account records, rewards,
  integrity decisions or public spectating. Training controls are not wire
  messages. Starting training cannot silently abandon an online session.
- **BR-TR12 — Desktop completeness.** Validate both active desktop entry
  points, English/pt-BR, keyboard/mouse, state cleanup and supported forms.
  The initial visual matrix is 1366×768 and 1920×1080. Native coverage is
  recorded per actual OS; phone layouts/touch are outside P62.

**Proposed validation budgets, not gameplay guarantees:** one active preview
figure; warm interaction p95 below 100 ms and frame-time p95 within +10% of
the comparable baseline on the agreed desktop machine; 20 open/reset/exit
cycles for lifecycle inspection. Named bot calibration, if explicitly
authorized: 24 seeds with swapped seats for each of three adjacent pairs
(144 matches), at most 6,000 ticks per experimental game; report truncation
separately. A 60% stronger-profile score is a screening goal with paired
uncertainty, not a human win-rate claim. All validation remains deferred
until implementation; owner look/fun/lore decisions remain open.
## 14. Ranked integrity — Proposed, inactive (P60, D-87/D-89)

Planning only. These candidate rules do not amend active BR-R03–05 until
the owner adopts enforcement. Shadow collection preserves existing Elo.
See [P60](phases/P60-ranked-integrity.md) and the
[joint execution program](planning/RANKED-PROGRAM.md).

- **BR-RI01 — Proposed.** Ranked remains one open account-only FIFO queue,
  with no rating bands, bots, selectable opponents or integrity delays.
- **BR-RI02 — Proposed.** A repeat/source budget either accepts both
  existing applied Elo deltas intact or applies zero to both; it never
  clips just a winner, changes K or asserts baseline Elo is zero-sum.
- **BR-RI03 — Proposed.** Only authoritative public ranked outcomes with
  two distinct connection-bound accounts enter settlement. A unique match
  key settles once. Private, Challenge, rematch and Practice never rate.
- **BR-RI04 — Proposed.** Optional pair budgets count the sum of positive
  points per beneficiary/opponent in a rolling window. Losses do not refund
  allowance. Test a prospective full award; equality is allowed, exceeding
  a limit makes both deltas zero. Matches remain playable and recorded.
- **BR-RI05 — Proposed.** Optional global source budgets count positive
  inflow to a beneficiary from all immature opponents. Maturity uses age
  and previously Applied games. On adoption of enforcement, NoChange games
  do not advance competitive W/L/D/game counts, placement or maturity;
  they stay in result history. Shadow preserves current BR-R05 behavior.
- **BR-RI06 — Proposed.** Normal/Review/Invalidated describes evidence,
  separately from Applied/NoChange rating. Isolated IP, short duration,
  Surrender, new account, reconnect or repeated friends never proves fraud.
  Only the specified relationship-plus-behavior combination flags Review.
- **BR-RI07 — Proposed.** Review settles normally and is excluded from
  balance statistics until cleared. Automatic holds are not enabled by
  this proposal. Ranking contains committed applied rating only.
- **BR-RI08 — Proposed.** A proposed pre-Running forfeit rule awards zero
  to both without implying misconduct; ordinary Surrender/disconnect after
  Running remain outcomes under BR-M03/M06. There is no gameplay timeout.
- **BR-RI09 — Proposed.** Adjudication is append-only, attributable and
  reversible. Human invalidation/restore replays frozen original deltas
  from a cutover baseline for affected accounts; later opponents are not
  counterfactually rerated. Original results and decisions remain intact.
- **BR-RI10 — Proposed.** Pre-cutover matches are not retrospectively
  investigated or invalidated by this feature. New policy versions do not
  retroactively change old eligibility decisions.
- **BR-RI11 — Proposed.** No automatic account bans, suspensions,
  fingerprinting or ML. Do not persist IP derivatives in the MVP. A neutral
  private receipt and an owner-provided appeal contact accompany enforcement.
- **BR-RI12 — Proposed.** Rating updates and authoritative result records
  commit atomically. Durable retry is idempotent; unavailable storage is
  visible and cannot fabricate an outcome. Optional analytics never rolls
  back a committed rating. No client-supplied statistics are accepted.

All following new policy/acceptance values are **Proposed**, not measured
production facts or automatic activation authority:

| Parameter | Proposed value / decision |
|---|---|
| Default integrity mode | Shadow; no reward restriction |
| Pair positive-inflow budget | 120 points per beneficiary/opponent per rolling 7 days (604800 seconds); 40/7 rejected as too restrictive |
| Immature-opponent global inflow budget | 120 points per beneficiary per the same rolling 7 days |
| Mature source | Age at least 7 days and at least 20 previously Applied, non-Invalidated results |
| Review relationship | At least 5 pair games/7 days; at least 80% of donor games against recipient; at least 80% losses and net recipient gain at least 40 |
| Independent behavior evidence | At least 3 such losses with no purchases/Evolutions/Fortifications/applied Augments, or voluntary Surrender before 15 combat seconds |
| Supporting newcomer concentration | At least 80% of positive inflow from immature sources and at least 5 source accounts; never sufficient for Review |
| Context windows | 1 hour and 7 days; active distinct accounts cached every 10 minutes |
| Shadow evidence before gate consideration | At least 7 days, 50 games, 5 accounts; review all flags and legitimate would-limit matches; owner decision still required |
| Automated hold | Disabled; 24-hour auto-release is an unelected future candidate requiring a separate design/decision |
| Optional future IP pseudonym retention | At most 7 days, daily key rotation; no collection in this MVP |
| Integrity collector / facts / history budget | At most 4 KiB extra per match / 8 KiB facts / 10000 streamed history rows; one additional indexed history query per completion |
| Integrity end-of-match latency targets | Evaluator CPU p95 <=5 ms; storage/journal p95 <=20 ms; queue-wait p95 delta <=5 ms at equal local load; no new pairing await |
| Durable final journal | At most 32 KiB per file, 32 MiB total; one retry worker at 1/5/30-second delays, then operator-visible pending state |

## 15. Ranked statistics — Proposed, inactive (P61, D-88/D-89)

- **BR-ST01 — Proposed.** Main statistics include only complete,
  post-collection public Ranked matches that reached Running, with two
  distinct authoritative accounts, Normal risk and Applied settlement.
  A valid zero-delta draw is included; Review, Invalidated, NoChange and
  missing/overflowed summaries are excluded with recorded coverage gaps.
- **BR-ST02 — Proposed.** Every record has immutable ruleset revision,
  source hash and summary schema. Gameplay/content changes, including
  mid-patch fixes, receive a new revision. Different revisions/hashes never
  mix; old missing telemetry is not reconstructed by guesswork.
- **BR-ST03 — Proposed.** A match contributes two sides to feature metrics
  and one match to duration/end/Arena metrics. A purchased feature is
  present once per side regardless of copies. Generated units are not
  purchases. Faction/Archetype/Arcana exposures may overlap.
- **BR-ST04 — Proposed.** Binary win rate is wins/exposed sides; draws
  remain in the denominator. Score rate is (wins+half draws)/exposed sides
  and is named separately. Buyer/nonbuyer deltas are associations, not
  causal effects. Every displayed rate identifies its denominator.
- **BR-ST05 — Proposed.** Evolution records historical side-level reach by
  kind/form and successful transitions per tier, including dead units.
  Side-level reach and purchased-copy conversion have distinct denominators.
  Synergy records real activation/time. Augment conditional pick uses
  completed offered seat-rounds, distinguishing unresolved cancellations.
- **BR-ST06 — Proposed.** Arena frequency is assigned frequency, not a
  player choice; compare seat-A score and match metrics, not pooled
  two-sided win rate. Matchup matrices count each match once.
- **BR-ST07 — Proposed.** Public small/concentrated cohorts are suppressed,
  including complements and comparisons that could reveal a small group.
  No account IDs/names/keys/IPs, raw match IDs, seeds, risk facts or fine
  timestamps enter the public aggregate API. Safeguards reduce disclosure
  risk but are not a guarantee of anonymity.
- **BR-ST08 — Proposed.** Raw summaries are authoritative and aggregates
  are disposable. Review/clear/invalidate/restore changes eligibility and
  forces a coherent rebuild; incomplete or stale generations are not
  silently presented as complete current data.
- **BR-ST09 — Proposed.** All ratings view is primary; public bands,
  per-player analysis and arbitrary query cuts are deferred. Desktop
  Statistics uses the existing lobby and both supported languages.
- **BR-ST10 — Proposed.** Online data produces investigation candidates;
  D-33 lab confirmation and owner approval precede balance changes. Patch
  notes cite online sample/uncertainty and lab evidence, with before/after
  values and a new revision. No automatic buff or nerf from popularity.
- **BR-ST11 — Proposed.** Collection uses bounded command/event counters
  outside the simulation; no per-tick scan/log/SQL or extra simulation-state
  fields. This has small nonzero bookkeeping cost. Exact damage attribution
  is deferred, not replaced with false zeros.
- **BR-ST12 — Proposed.** Optional summary persistence follows authoritative
  settlement and never changes its outcome/rating. Recovery uses match-key
  idempotency; irrecoverable missing data is an explicit coverage gap.

All new thresholds/technical budgets below are **Proposed**:

| Parameter | Proposed value / decision |
|---|---|
| Ruleset / summary / public API initial revision | 1 each, independent of postcard version |
| Main internal landmark | 60 combat seconds (1200 ticks); keep both sides of matches reaching it; early finishes reported separately |
| Public feature row | >=50 exposed sides, >=5 accounts, >=3 pairs; largest account <=40% of exposures; complement qualifies separately |
| Public match-level row | >=25 matches, same account/pair/concentration protections |
| Public Augment conditional pick | >=50 offered seat-round opportunities from >=25 matches; same account/pair protections |
| Public matchup matrix cell | >=100 matches, >=10 accounts, >=5 pairs; largest account <=40% |
| Interval | Wilson 95%, z=1.96 for binary proportions; descriptive under repeated-player/match clustering |
| Public rounding / release batch | Approximate n rounded down to tens; percentage/CI to whole percentage points; at least 10 new eligible matches between numeric publications, except immediate withdrawal |
| Public time and cohort cuts | Fixed ruleset only, All rating; daily-precision update labels |
| Candidate investigation | >=200 exposed sides, >=10 accounts, >=5 pairs, largest account <=25%; Wilson wholly below 45% or above 55%, or unit presence below 2% / above 60% with >=1000 denominator sides; descriptive, not automatic tuning |
| Future dynamic bands | Consider at >=300 accounts and >=1000 matches/patch; tertiles of per-account first-match ratings, avoid splitting ties, collapse when data is insufficient |
| Collector / summary / API/cache sizes | <=8 KiB extra collector, <=16 KiB JSON summary, <=64 KiB immutable ruleset catalog, <=64 KiB uncompressed API response, <=128 rows, <=1 MiB aggregate cache |
| Summary CPU / optional DB target | Encode p95 <=1 ms; optional insert p95 <=5 ms; one optional row insert per match |
| Rebuild budget | At most once per 300 s when dirty; one worker; <=100000 matches / <=2 s per snapshot, otherwise unavailable and an operator diagnostic |
| Public cache | 60-second browser/shared cache; no stale-while-revalidate; current and immediately previous ruleset, All band, seven dimensions |
| Query/error bounds | <=256 query bytes; Retry-After 5 s for unavailable; existing API 120/IP/min applies; every error no-store |
| E1->E2 quality gate | >=99% complete summaries over at least 50 eligible completions, privacy and desktop acceptance; sparse public rows remain suppressed |

Full definitions, recovery details, bias qualifications and implementation
contracts are in [P61](phases/P61-ranked-stats.md). These candidate values
do not change existing D-33 gate thresholds or active gameplay values.

## 17. Arena victory celebration (P63, Proposed)

**Proposed — planning only, 2026-10-31.** See
[P63](phases/P63-victory-celebration.md). The owner's requested outcome is
an arena celebration with the winner's characters, alive or dead, facing
the player and performing individual animations. No rule below is activated
by publication of the plan.

- **BR-VC01 — Result first.** Only the authoritative confirmed winner
  triggers victory. Presentation cannot change combat, Outcome, rating,
  rewards or duration. Draw/unknown/canceled results do not celebrate.
- **BR-VC02 — Complete cast.** Include the winner's participating instances,
  dead and surviving, with duplicates and each instance's last attained
  Evolution form counted once. Recommended eligibility includes animated
  combat summons, excluding inert Crates, Strongholds, props and training
  dummies; confirm P63 O01. Never derive the complete cast only from survivors
  or views observed since reconnect. A genuinely empty cast stays empty.
- **BR-VC03 — Arena and facing.** Show intact cosmetic figures in the match
  arena, in a spaced formation facing the camera on either seat. They are
  presentation copies, not resurrected simulation units or targets.
- **BR-VC04 — Current and future motion.** Reuse actual per-kit Victory
  motions now; add a shared override seam and one owner-approved signature
  Dance pilot. Future unique dances are authored per character. Shared or
  arm-raising fallbacks are labeled accurately in the content inventory.
  Loop within each figure's slot without combat events or repeated audio.
- **BR-VC05 — Result interaction.** The winner can use Menu, retry/rematch
  and Hide celebration immediately. Continue processing rating/rematch
  messages. Duplicate end messages cannot replay audio or recreate the
  scene. Proposed initial losers/spectators retain their ordinary result.
- **BR-VC06 — Scale proposal.** Show all figures together through 64;
  larger casts use explicit pages of at most 64, with total count and every
  participant reachable. Confirm P63 O02 or adopt a measured all-at-once
  alternative; no silent truncation or online army cap. Decode compact
  counts without allocating every logical figure.
- **BR-VC07 — Safety and preferences.** Scene ownership and cleanup isolate
  celebration from match state. Reduced motion uses a static pose/camera;
  existing audio preferences apply. An unusable cast summary falls back to
  the ordinary result without invalidating the outcome.

Proposed validation: desktop 1366×768 and 1920×1080; 20 lifecycle cycles;
under the paging proposal, at most 64 visible figures and one camera
override; p95 frame-time within +10% of a comparable same-arena/cast scene
on the agreed desktop machine. All measurements and owner visual review
are deferred until implementation. No phone/touch work is included.


### P63 execution amendment — owner decisions, 2026-10-31

O01 and O02 are approved for implementation: BR-VC02 includes animated
summons, excludes inert Crates, and BR-VC06 uses complete pages of at most
64 figures. For BR-VC04, the owner requests a shared standard using existing
Victory motions now, with per-character overrides ready for future content;
the exclusive pilot requirement is deferred by the owner. No unique-dance
roster is claimed. Result identity and the exact aggregate use protocol 28
on main's real v17 schema; unmerged v20/v27 fields are not assumed present.
All behavioral/presentation additions remain Proposed pending the recorded
acceptance and visual review. No combat or rating values change.


## Consolidated roster activation — Proposed, owner 2026-10-31

The owner requested all open PR work, all 21 P59 champions and their native
compatibility in one installable/deployed candidate, explicitly rejecting a
partial staged delivery. FULL_POOL/CANON now contain 46 conventional champions;
ALL_KINDS contains those 46 plus Beetling, Crate and three drill ranks (51).
All existing §13 starting stats, forms and abilities remain Proposed and
unchanged. No summon enters the Shop and the ten-kind reference pool remains
for existing C7 planted-imbalance contracts. This activation changes seeded
Shop draws, so deterministic fixtures are intentionally consolidated in the
same roster-activation commit; it never justifies weakening a balance gate.
D-33 evidence and owner look/fun review are recorded in the consolidation report.
Shop reservation/Reroll remains the existing Practice preview rule, not a new
online economy change.

### Proposed P70 Trident size, thrust and basic reach (owner 2026-10-31)

**Proposed P70 candidate: 2,000 simulation units (2.0 tiles), pending D-33
and owner look/fun acceptance.** The isolated execution branch implements
this authoritative base range, inherited unchanged by Base, A1, A2, B1
and B2. The frozen reference remains 1,100 (1.1 tiles). Ruleset revision 3
identifies the candidate; it is not an approved release balance value.

The owner requests a taller shark, a longer trident and a two-handed frontal
spear thrust. Generated-mesh measurements give Base height 2.235 and weapon
length 2.62656 world units. Contact-pose calibration compares 1.1, 1.6, 1.8,
2.0 and 2.2 tiles. At 2.0 the point projects about 0.23–0.37 tiles beyond
the target center in evolved forms; at 1.8 it projects 0.40–0.55 beyond.
The 2.0 candidate is the smallest sampled increase keeping all five forms
within roughly a normal target body's depth. This is a visual proposal,
not balance evidence or a new hitbox. Actual opponents can close during
windup; live before/after contact remains a separate acceptance check.

Nearest-enemy/Guard/Stronghold selection and the existing special's primary
selection algorithm are unchanged. Its primary search already uses the
unit's effective range, so it also acquires primaries within the candidate
reach; Bloody Rend remains centered on that locked primary with its original
radius, damage, bleed and period. No new targeting rule or effect radius is
introduced. The wider acquisition interval must be included in D-33 review.

This narrow exception does not authorize changes to Trident's damage,
cadence, HP, Energy cost, movement, targeting semantics, Bloody Rend area
or bleed, or to any other champion. Preserve strike timing and server
authority. See [P70](phases/P70-champion-visual-identity-polish.md).

P60/P61 journal/summary Faction arrays follow FACTION_COUNT=7, preserving
legacy four-Faction records by zero extension. P63 counts every actual drill
spawn, including fusion outputs, without deleting consumed parents from the
historical participant aggregate. Cosmetic figures never affect simulation.

The lab off-pool instrumentation regression uses the unchanged ten-kind
reference ruleset, so it continues verifying that spawning an off-pool kind
never activates that kind. It no longer assumes the newly activated CANON
pool excludes Bonehorn. Its exact damage/HP assertions are retained.

### Canonical trace identity after the 46-champion activation

The owner-authorized consolidated CANON pool contains 46 conventional kinds.
The C2 per-tick trace fixtures now identify that already active revision,
matching the final hashes in contract_determinism. The five documented seeds
are collected by examples/trace_canon.rs with the authoritative headless
cadence; no additional balance value, gameplay rule or time limit changes
in this trace-identity correction. Previous traces describe the earlier pool.


### Proposed P71 cosmetic readability and Stronghold wear (owner 2026-11-01)

Presentation follow-up only: unit team identification moves from body bands
and patches to persistent foot rings; status effects keep allegiance visible.
Evolution displays no star at Base, one at tier 1 and two at tier 2 on both
branches, in bright yellow with a dark outline. Intrinsic clothing colors
and equipment carry character/branch identity. This does not alter Evolution.

Stronghold appearance reflects each completed **5% loss of current max HP**:
`floor(20 * (max_hp - clamp(hp, 0, max_hp)) / max_hp)`, with safe integer
intermediates and a defensive denominator of at least one. There are 21
cosmetic states, 0 intact through 20 destroyed. Healing and BR-SK07
Fortification reconcile against the new authoritative HP/max HP view;
reconnect initializes directly without replaying past damage effects.
Cracks, missing pieces, debris, smoke and fire are cosmetic. A living tower
retains a plausible visible emitter and its exact existing firing capability.
Only authoritative zero HP permits the final destroyed presentation.

Pyre Warden's white/red/black model and red/orange flame replace its spectral
colors, preserving ProfaneFlame's trigger, timing, exact segment, friendly
fire and damage. Stronghold animation refinement likewise preserves
BR-SK01–09, targets, damage, cadence, impact rules, costs and outcomes.
The fire/smoke onset bands and palette/animation tuning in
[P71](phases/P71-combat-readability-and-stronghold-polish.md) are Proposed art,
not balance rules. No new simulation state, protocol or mechanical range
change is authorized by P71. P70's Trident balance acceptance stays separate.


#### P71 Bonehorn head occupancy clarification (Proposed presentation)

The owner's 2026-11-01 follow-up includes Bonehorn's head/muzzle/horn in its
visible body blocking area. Under the existing D-57 display layout, opposing
and allied figures must account for that forward extent instead of letting
the head enter another body. Stable cached form bounds and bounded cosmetic
separation preserve credible contact; HUD, selection and attack anchors follow
the final display. This does not add a targetable head zone, physical server
collider, damage multiplier, new range or authoritative movement rule. If
presentation cannot fit existing combat spacing, report the concrete conflict
before specifying a mechanical exception. See P71 S1d/S1e.


#### P71 Shatterback head occupancy follow-up (Proposed presentation)

The owner extends the existing D-57/P71 visible-body spacing to Shatterback on
2026-11-01. Its cached form envelope includes the head, beak, torso and limbs
through Idle, Walk and its body-driven melee attack. The displayed crowd
footprint projects that envelope at the current figure facing and reserves its
existing cosmetic lunge. Apply it to allied and opposing figures, including
Evolution growth and after detachable plates are lost. Do not use a plate's
remaining HP to shrink the flesh body. This changes client display offsets only:
no authoritative position, collision, attack range, stat, protocol or outcome
changes. The same request removes the orange/gold stripe from B1 shell plates;
Base, A1 and both tier-2 plate identities retain their existing art.

## 19. Native updates and match continuity (P66, Proposed)

**Proposed — planning only, 2026-10-31.** See
[P66](phases/P66-native-updates-and-release-safety.md) and
[release contracts](planning/P66-RELEASE-CONTRACTS.md). This does not authorize
installation, real signing, publication of executables or deployment.

- **BR-UP01:** Linux managed installs check for updates asynchronously at
  startup and show localized availability at a safe idle point. Windows
  follows the same contract after native platform verification. Updates
  preserve user data and require no uninstall/reinstall after trusted adoption.
- **BR-UP02:** routine update prompts/application do not interrupt playing,
  reconnecting or time-sensitive session state. No forced client restart,
  expensive extraction or default update download during online matches.
  Incompatible minimum builds restrict new admissions, not current combat.
- **BR-UP03:** signed verified complete releases activate transactionally.
  Candidate health and permitted fallback are explicit; no partial/mixed
  assets. Reopen recovers journal state and retains original-generation
  resume capability. Existing unsigned installs need a trusted bootstrap.
- **BR-UP04:** after healthy activation show exact installed-version notes
  until durable acknowledgement, even after closing/reopening. A rollback
  reports recovery, not successful installation. EN/pt-BR and offline notes
  are part of the initial contract.
- **BR-UP05:** each online match retains its admitted generation/rules/
  content/features/wire. Compatible client/backend-only patches can release
  independently; incompatible generations coexist or wait for safe drain.
  No new match-duration cap, live balance swap or forced migration is added.
- **BR-UP06:** old workers retire only after matches, durable results and
  protected reconnect/delivery state settle. Long drain defers promotion;
  no routine 300/330-second cutoff. Rollback redirects new admission without
  terminating existing matches on either generation.
- **BR-UP07:** emergency security containment and arbitrary machine failure
  have explicit separate policies. Planned deployment continuity does not
  guarantee crash failover. Capacity for thousands requires measurements,
  overlap headroom and later multi-host coordination as demand grows.
- **BR-UP08:** the startup update modal appears at most once per launch per
  build; Later keeps a non-modal badge. A required update blocks only new
  online admissions: Practice vs Bot, Settings and installed notes stay
  usable offline on the installed build.
- **BR-UP09:** public matchmaking admits one active match contract at a time.
  A new contract is trialled on the opt-in preview channel, its client is
  published ahead of a scheduled switch, and behind clients then receive
  UpdateRequired at queue/room time while already-admitted matches finish on
  their generation. Percentage canaries apply to same-contract releases only.
  Rooms, challenges, rematches and spectating never cross contracts.
- **BR-UP10:** every release ships EN and pt-BR notes as plain text in the
  D5 charset. A release whose computed ruleset/content identity changed must
  include a gameplay-changes section.

Proposed initial operational values: metadata file 1 MiB, descriptor 64 KiB,
notes 512 KiB; archive 2 GiB compressed/8 GiB extracted/100,000 entries;
initial attempt plus at most 2 automatic retries; download inactivity 30 s
(Proposed, not a total-transfer deadline); discovery deadline 5 s
and idle periodic check interval 6 hours; candidate readiness deadline 60 s;
at least current/previous builds plus pinned builds; automatic highlights
through 10 releases. Canary steps internal/5%/25%/100% concern new eligible
admissions of same-contract releases only. New-contract client published at
least 24 h before its scheduled switch; client rollout halts at 2% failed or
recovered outcomes over at least 100 reports; notes at most 40 items of 280
characters per locale. A 5% p99 tick/view regression is an initial stop
criterion
on an agreed paired fixture, never an existing measured SLO. Security/result
loss stops rollout independently. Final limits/OS/channel/signing/emergency/
retention/player-default decisions O01–O05 remain owner gates before activation.

## 21. Persistent authentication — Proposed, P68-A (owner extension)

Sections 18–20 remain reserved by independent P65–P67 plans, absent from this
consolidated base. These rules specify the owner's P68 authentication scope;
no runtime feature is enabled by this documentation. D-97 and
[P68-A](planning/P68-PERSISTENT-AUTH.md) own engineering details.

| Rule | Contract |
|---|---|
| BR-AS01 | Access Tokens are opaque, independently random 256-bit credentials, valid for at most 15 minutes (900 seconds), bounded by family expiry and revocable earlier |
| BR-AS02 | Refresh Tokens are independent random 256-bit credentials; each device-family expires 30 days (2,592,000 seconds) after initial login. Rotation never extends this absolute lifetime |
| BR-AS03 | Rotate refresh on successful renewal in one transaction, retain consumed hashes through family expiry, and revoke the affected family on replay outside the defined bounded retry policy |
| BR-AS04 | Access exists only in client/server memory; persist native refresh only in Linux Secret Service or current-user Windows Credential Manager with Local persistence; web uses an HttpOnly/Secure/SameSite=Strict host cookie, Path=/ and no Domain |
| BR-AS05 | No new automatic AccountKey/token persistence in ordinary preferences, localStorage/sessionStorage, URLs, logs or caches. Explicit manual key backup remains possible; incomplete legacy conversion preserves the pre-existing recovery key until safe migration completes |
| BR-AS06 | A match reconnect handle is separate from account authentication. Account resume requires matching valid Access and family/account binding; guest handles never grant an account or seat |
| BR-AS07 | Natural Access expiry requires renewal for new account operations/matches but does not forfeit an already admitted match: its existing seat-scoped lease may finish that match. Logout-all and security revocation invalidate that lease; an ordinary logout is refused while the family holds a seat (`in_match`). Reconnect inside the seat's hold (BR-M03) may present the expired Access of that binding for up to 180 s after expiry to take back only that seat, then renews on the socket (**Proposed**, P71) |
| BR-AS08 | Persistent families may exist on several devices, but only one account controller is active. Explicit login/Continue here performs takeover; background refresh, passive restoration and extra tabs cannot steal the controller |
| BR-AS09 | Revocation invalidates all Access/connection bindings of its scope before a successful acknowledgment; logout-all requires manual AccountKey verification. Security revocation is not delayed by an active match |
| BR-AS10 | Legacy key removal requires successful refresh persistence and confirmation plus manual-backup acknowledgment. Failures never silently create a new account, erase history or fall back to new plaintext secret storage |
| BR-AS11 | Vault denial/unavailability yields an explicit session-only mode. Browser uses protected persistent or session cookies according to player choice; blocked cookies permit Access-only temporary login. Session restore may retain a session cookie; logout is the reliable revocation action |
| BR-AS12 | Auth checks, database operations and vault I/O stay outside deterministic simulation/combat ticks; all normal seat/intention/outcome authority remains on the server |

Proposed auth operational defaults, not balance values or capacity evidence:

| Setting | Value / meaning |
|---|---|
| Access renewal lead | 60 seconds before server-advertised expiry; suspension/offline recovery does not extend validity |
| Rotation/registration retry cache | 300 seconds (P71, Proposed; was 30): a lost reply mid-match is recovered by the same operation, never read as reuse; same-grant idempotency only, never another successor; rotation conflicts disclose no tokens |
| Token width | Access, Refresh and registration retry proof: 32 random bytes, 64 hex characters over HTTP |
| Nonsecret ID width | Family, device and operation IDs: 16 random bytes; no authentication power |
| Auth JSON request limit | 4 KiB, additionally strict field/shape validation |
| Auth request deadline | 5 seconds; timeout is not proof that a rotation did not commit |
| Persistent family cap | 10 nonexpired/nonrevoked families per account; refuse additional persistence explicitly, never silently evict an active match |
| Refresh family budget | 10 requests/minute per family; plus existing separate IP login/API budgets, with Retry-After |
| Live Access index cap | 4 × configured maximum connections (8,000 at current default); at most 16 unexpired Access entries per family; prune expired entries, then refuse excess issuance without terminating admitted matches |
| Secret response-cache cap | At most configured maximum connections (2,000 currently), with one current retry grant per family/operation; reserve capacity before committing issuance |
| Cleanup batch | At most 256 token/family rows total per maintenance turn; indexed expired-history deletion before empty families, no unbounded cascade on the busy writer |

Use only minimal private device-family IDs, channel, hash history and
creation/expiry/revocation timestamps; no hardware fingerprint, precise
location or new persistent IP/UA history. Family limits and rate refusals
must have localized actionable states. Offline logout is marked pending
until server revocation is acknowledged; no restored login while pending.
No combat, Energy, Shop, Rating or balance number changes in this extension.

### P68-A implementation contract activation (D-97)

The Linux-first candidate activates protocol **30**, appending client tags
20–22 (`AuthenticatedHello`, `Reauthenticate`, `ClaimAccount`) and server
tags 31–32 (`AuthState`, `AuthRequired`). Tags 0–19 / 0–30 retain their wire
shapes. Legacy WS account variants remain decodable but are refused; account
issuance moves to isolated auth HTTP routes. Existing manual AccountKeys
remain valid. The server schema advances from 8 to additive schema 9.

This replaces transport assumptions in BR-P01–BR-P04 and registration test
fixtures, preserving name validation, failed-attempt refunds, ranked data,
explicit takeover and refusal of account changes during an admitted match.
The revision adds no simulation, balance or outcome change. Browser JSON
includes the server issuance time so clients can schedule renewal using a
monotonic duration instead of trusting the device's wall clock.

### Consolidation with P66 (D-99, Proposed)

The consolidated P66+P68 line keeps wire protocol **30** (P66 adds no wire
type; its admission contract travels over HTTP and the socket URL) and moves
the server schema to **10**, which holds both the P66 release directory and
these auth families whichever v9 a database already had. A native client
restores its P66 resume record (handle, slot endpoint, family binding) only
through this auth coordinator; BR-AS06 binding is unchanged and a guest
record never yields an account. Auth requests follow the release slot that
holds the socket. No combat, Energy, Shop, Rating or balance value changes.

## 20. Commerce, recovery and opponent preferences (P67, Proposed)

**Proposed — planning/documentary base only, 2026-10-31.** See
[P67](phases/P67-commerce-recovery-foundation.md) and the
[new module folders](modules/commerce/README.md). No real payment, email,
invoice, item catalog or authentication change is activated in this branch.

- **BR-CO01:** preserve BR-C01–C03: paid items are cosmetic/account-bound,
  with validated server evidence. Cash checkout is separate from core Shop/
  Energy. Existing free cosmetics are not made paid implicitly.
- **BR-CO02:** verified recovery email is a prerequisite for paid checkout;
  name/key play and guests remain available. Enrollment/change needs trusted
  proof. Recovery rotates the key on the same AccountId, preserves purchases
  and ranking, consumes a temporary challenge and revokes stale sessions.
  Email receives a temporary link, not the permanent account key.
- **BR-CO03:** initial payment scope is Asaas Pix and fixed nontransferable
  cosmetic items. No subscriptions, wallet, gift transfers, paid randomized
  rewards or competitive advantage. Orders freeze server prices/legal terms;
  integer BRL minor units are authoritative.
- **BR-CO04:** normal grant follows verified matching PIX receipt and unique
  source entitlement application. Client return/screenshot/QR creation is
  insufficient. Duplicate/late/out-of-order facts and unknown outcomes are
  reconciled; refunds/holds compensate specific sources without gameplay bans.
- **BR-CO05:** Zaflas is intended fiscal issuer, pending real legal/company/
  accounting setup. Invoice authorization, payment and item delivery remain
  distinct linked records. Fiscal document type/rates and consumer/privacy/
  younger-player policy require appropriate review before sales.
- **BR-CO06:** opponent emotes may be locally muted, and optional opponent
  cosmetics replaced by approved baselines. Preserve actual kind/form,
  StrongholdKind, timing, team/status/projectile readability and equipment
  ownership. Paid ownership cannot override the viewer's preference.
- **BR-CO07:** itch.io package transactions remain a separate platform
  channel; they do not automatically become Asaas charges or cosmetic grants.
  Provider/email/fiscal integrations remain off until explicitly authorized.

Proposed initial recovery limits: temporary recovery link 15 minutes,
verification 24 hours, three dispatches/hour/account/destination with bounded
IP controls. Initial documentary order schema permits up to 20 distinct
cosmetic lines, quantity 1 each; fixture SKU/2,000-cent value is illustrative,
not an approved catalog/price. Opponent mute/hide defaults false pending
younger-player policy. Schema version 1 is local to documents, not a game
protocol or DB version. O01–O05 confirm real policies before activation.

Offline execution fixtures additionally propose 128 outstanding challenges,
64 captured mail messages, 256 sliding-window admission records and 12
requests/hour per synthetic source IP. These are test resource bounds only,
not approved live operational limits. The owner deferred fiscal work and
prioritized recovery and opponent mute; no commercial launch is approved.


## Main/auth and eight-Arena integration — 2026-11-01

The owner requests reconciliation of `main` authentication, refresh and reconnect
with the P71 character and three-Arena branch, followed by deployment to the
owner-reset VPS after its connection settings are supplied. This explicitly
authorizes this integration and that deployment; it does not promote a signed
client update or alter the existing pending balance/visual acceptance gates.

Both independent branches allocated protocol 30 to different contracts: D-97
authentication on `main`, and the appended Arena variants on P71. Their combined
contract uses protocol **31** so neither older peer can silently enter an
incompatible match. Preserve all message tags, Arena IDs 0–7, D-97 lifetimes,
account/seat authority, schema 10, and P71 ruleset revision 6/source identity.
No additional gameplay number changes. See
[the integration report](phases/BACKEND-main-integration-20261101.md).

### Proposed browser release resource bounds (2026-11-01)

Operational only; no combat, matchmaking, reward or match-duration value changes.
The independent browser publisher accepts at most 256 MiB compressed, 512 MiB
expanded, 4,096 payload files, a 1 MiB manifest and a 2 MiB HTML entry. These
are **Proposed** initial updater limits under §19/O04. The static publication
worker has a five-minute timeout; that limit never applies to game processes
or match drain. Browser releases retain immutable source-addressed assets and
legacy files; no automatic retention deletion is approved.

## Proposed P73 — Active players, challenges, rematches and match history

Owner request, 2026-11-01. **Implemented behind explicit rollout flags;
production activation and acceptance remain pending.** See [P73](phases/P73-active-players-and-direct-challenges.md).

- **BR-AP01 — Directory.** Active players lists every connected account
  across the currently known service slots, with whole-directory name
  search and paging. Preserve BR-LV06: guests count toward connected
  sessions but are not publicly enumerated; no private room/match or
  credential data appears. Display partial/stale coverage honestly.
- **BR-AP02 — Availability.** Public activity does not grant permission to
  pair. Only a genuinely Idle, connected, authorized controller in a
  compatible admitting ownership domain can send/accept a direct invite.
  Queues, rooms, pending link challenges, watching and matches are not
  forcibly interrupted. Private activity may be shown only as coarse busy.
- **BR-AP03 — Explicit consent.** Send is the inviter's explicit consent;
  only the addressed recipient can accept. Recheck both participants at
  acceptance; consume once and use the existing private Challenge path.
  Crossed/replayed clicks never implicitly accept or start duplicate games.
  Direct games do not affect rating or enter a public queue.
- **BR-AP04 — Participant-bound notifications.** Random short-lived direct
  invitation IDs are disclosed only to participants and authorize no bearer
  join. Public directory rows disclose no such IDs or session/account keys.
  Reuse existing shareable Quick Challenge links separately; they are not
  equivalent to a recipient-bound notification.
- **BR-AP05 — Last-opponent rematch.** Extend BR-R09 ticket eligibility to
  completed online PvP Challenge, Casual, Ranked and human private-room
  games. Maintain one latest valid opponent ticket per participant, the
  existing configured rematch TTL (currently 120 s), mutual consent, fresh
  seed and private Challenge mode with no rating effect. A private guest
  recent-opponent card does not make guest sessions public. Practice and
  spectators create no ticket. Opening Active players is navigation, not
  an implicit decline; explicit leave/decline and newer activity still
  clean up the offer.
- **BR-AP06 — Bounded invites.** Proposed initial direct-invite TTL is 60 s,
  with 1 outgoing invitation per controller, 5 incoming per recipient,
  15 s per sender/recipient-pair cooldown and 512 live directed records
  per process. Retain current general WS and invite-IP limits (currently
  30/min), with account or guest-session keys as appropriate. These are
  tunable initial product defaults, not proven capacity figures.
- **BR-AP07 — Opt-out and continuity.** A retained-session setting can
  disable incoming direct challenges and close pending incoming notices;
  it does not disable explicit Quick Challenge links. Reconnect restores
  only the unexpired same-session state; it does not extend TTL or revive
  terminal records. Logout/revocation/takeover invalidate directed invites.
  No notification or acceptance can pause, move or forfeit a live match.
- **BR-AP08 — Global snapshot and rollout.** HTTP reads bounded public
  snapshots; no SQL or actor mutation per directory read. Fixed release
  slots aggregate local-only snapshots in a background task, with at most
  2 configured local peers concurrently, 750 ms timeout, 2 MiB response
  limit and 15 s freshness cutoff as Proposed initial limits. Internal
  snapshots remain loopback-only and publicly denied by the proxy.
  Incompatible/draining peers stay visible but unavailable; no cross-slot
  match migration is introduced. Missing peers produce partial coverage,
  never fabricated global zero. Preserve the existing 5 s client refresh
  baseline with backoff/hidden-page suspension; private invites use WS push.

- **BR-AP09 — Recorded history.** Account-bound completed online Ranked,
  Casual, Challenge/rematch and human private-room games enter persistent
  result/statistics history. Freeze account-to-seat ownership at admission.
  Practice/demo/training, spectators and unbound guest sessions do not gain
  a persistent online-account history through this feature.
- **BR-AP10 — Authoritative statistics.** The result/reconnect and stored
  detail use the same bounded server-terminal personal summary. Count
  successful purchases/Evolutions/Fortifications and finalized choices;
  freeze Stronghold HP, mode, arena, outcome and both combat/wall duration.
  Current client Stronghold-hit amounts are not total effective damage.
  Unsupported or missing old statistics are null/unavailable, not zero.
- **BR-AP11 — History privacy.** The account chooses Public or Only me,
  default private for existing/new accounts. The persisted server choice
  governs all past/future history and legacy public `recent` routes.
  Owner reads require authentication/ownership. Public projections contain
  only that player's permitted summary; private-game/guest opponent data,
  other-seat private stats and internal match/session keys are not exposed.
  Aggregate public rating/record remains governed by existing profile rules.
- **BR-AP12 — Idempotent recording.** One canonical terminal source key,
  independent per-participant entry IDs and a transaction/journal boundary
  prevent duplicates on result replay, reconnect, retry or recovery.
  Pending and Stored are distinct; an acknowledgment of Stored requires
  durable DB success. History failure cannot alter the outcome or Elo.
  Preserve bounded release-slot pending work and explicit recovery gaps.
- **BR-AP13 — Legacy integrity.** Backfill only verified account-bound
  Ranked rows, with partial/result-only quality where needed. Reuse source
  keys so new recording and backfill cannot duplicate a game. Never infer
  old non-ranked ownership from names, IPs or client logs. New non-ranked
  history begins at feature activation unless later verified data exists.
- **BR-AP14 — Viewing-only history.** This MVP shows results and personal
  statistics. Historical entries do not authorize replay/export, editing,
  deletion, challenge or a retrospective rematch. Current P73 rematch
  consent and eligibility still come from the live server-issued ticket.
- **BR-AP15 — Bounded history reads.** Proposed list defaults are 20 rows,
  maximum 50, with indexed keyset paging; history reads have a separate
  initial limit of 4 concurrent requests, 2 s timeout and a 32 KiB detail
  response bound. Reuse the existing journal pattern's 32 KiB entry and
  32 MiB total limits for the separate history-pending directory. These
  are initial tunable bounds, not performance guarantees. No automatic
  history purge is authorized; retention/deletion needs an owner decision.
- **BR-AP16 — Visibility change and rollout.** Privacy updates commit
  atomically with a revision and require the expected revision; stale
  concurrent saves conflict rather than silently restoring public access.
  New public requests after commit obey the current setting. History and
  legacy profile routes cannot bypass it through cache/old-slot fallback.
  Enable only after compatible privacy guards reach all public-serving
  slots; rollback preserves guards, settings, additive data and journals.

The required [P73 history sub-spec](planning/P73-MATCH-HISTORY.md) defines
recording, schema, API, security, failure/recovery, UI and H01–H18 acceptance.
It authorizes only the described account-history persistence/privacy
extension. No combat, economy, rating formula, authentication lifetime or
paid infrastructure change is authorized by P73.

## Champion completion — Proposed local preview (CC01–CC17)

**Proposed, owner-authorized 2026-11-01.** Seventeen new conventional kinds
complete nine champions per Faction. This initial implementation is a local
Practice/Training preview. The online FULL_POOL keeps its existing 46 kinds;
COMPLETION_POOL contains 63. Existing units, economy, Shop rules, Evolution
prices, Synergy and Strongholds retain their values. Initial new numbers are
hypotheses, not balance-lab results or production acceptance.

Every new champion has one Archetype and four Evolution forms. Base and
form display names, identity and full visual contracts are in
[the catalog](champion-completion/catalog.json) and
[execution phase](phases/CHAMPION-COMPLETION.md).

### Initial base registry

Cost is Energy. Interval is ticks; range and movement are milli-tiles and
milli-tiles/tick. The four Supports have no damaging basic. Their base heals
are Lastlight Cantor 36, Mire Tender 34, Dawnmender 34 and Nectar Nurse 12
per channel pulse. All new values in this section are **Proposed**.

| ID | Champion | Faction / Archetype | Cost | HP | Damage | Interval | Range | Move |
|---|---|---|---:|---:|---:|---:|---:|---:|
| CC01 | Gravecutter | Undead / DPS | 4 | 300 | 48 | 24 | 800 | 54 |
| CC02 | Lastlight Cantor | Undead / Support | 3 | 280 | 0 | 32 | 2800 | 40 |
| CC03 | Cragclaw | Monsters / DPS | 4 | 340 | 44 | 26 | 850 | 50 |
| CC04 | Hushfang | Monsters / DPS | 4 | 300 | 46 | 24 | 800 | 60 |
| CC05 | Spinehurler | Monsters / Ranged | 4 | 260 | 42 | 34 | 4200 | 40 |
| CC06 | Mire Tender | Monsters / Support | 3 | 310 | 0 | 34 | 2700 | 38 |
| CC07 | Gorgehowl | Monsters / Control | 4 | 330 | 18 | 36 | 1800 | 42 |
| CC08 | Sunlancer | Celestials / DPS | 4 | 320 | 44 | 24 | 1000 | 52 |
| CC09 | Eclipse Dancer | Celestials / DPS | 4 | 290 | 42 | 22 | 800 | 56 |
| CC10 | Starbreaker | Celestials / DPS | 5 | 330 | 58 | 34 | 900 | 44 |
| CC11 | Dawnmender | Celestials / Support | 3 | 290 | 0 | 32 | 2800 | 40 |
| CC12 | Prismancer | Celestials / Ranged | 4 | 250 | 32 | 30 | 4000 | 42 |
| CC13 | Leafcutter | Insects / DPS | 3 | 260 | 34 | 22 | 750 | 56 |
| CC14 | Blastbeetle | Insects / DPS | 3 | 240 | 32 | 24 | 700 | 58 |
| CC15 | Needlewing | Insects / Ranged | 4 | 240 | 38 | 30 | 3900 | 48 |
| CC16 | Nectar Nurse | Insects / Support | 3 | 270 | 0 | 36 | 2600 | 44 |
| CC17 | Stridulator | Insects / Control | 4 | 310 | 18 | 36 | 1900 | 42 |

### Shared competitive reading

- **CC-C01:** old kind tags remain unchanged; new tags append after Megadrill.
  Purchases, deployment, Evolution, damage/healing and outcomes stay authoritative.
- **CC-C02:** no new ability heals a Stronghold or receives special Stronghold
  damage. Specials, secondary hits, healing triggers and on-death bursts affect
  units only. Ordinary damaging basics retain ordinary Stronghold targeting.
- **CC-C03:** all added values are integers. Timed victim families are bounded:
  one echo, one regrowth and one solar spark. They never stack between copies.
  Stronger replaces, equal refreshes, weaker is ignored until expiry.
- **CC-C04:** base attack windup and existing timing remain unchanged. Gorgehowl
  has an explicit two-tick interrupt windup, not a zero-time cancellation;
  all other independent new specials use the shared six-tick windup.
- **CC-C05:** A1 and A2 each add 1500 bp to basic damage (or healing for a
  Support). B1 adds 300 range; B2 adds 1500 bp attack speed. Ability-specific
  branch values replace their earlier step; they never sum implicitly.
- **CC-C06:** one actual basic hit, not animation contact count, advances a
  striker proc. Stronghold hits, secondary damage, DoTs and betrayal never
  advance or activate new unit-only procs.
- **CC-C07:** source death does not erase an applied echo, spark, regrowth or
  bleed. Normal healing reductions, protections and cleanse policies apply.
  New public durations, readiness and channel facts reconstruct after reconnect.
- **CC-C08:** a new local protocol/schema identity accompanies any changed view
  shape. The current production server is not silently treated as compatible.

Ability parameter registry and focused regression scenarios are recorded with
the implementation in content/completion.rs and the following execution notes.

### Proposed ability parameters and identity amendment

A/B are the selected branch step (0, 1 or 2). The following values are
**Proposed** for local testing only. Percentages are integer basis points.

- **CC01:** below 3500 bp victim HP, +3000 + A×1000 bp basic damage.
- **CC02:** healing leaves one echo of 18 + B×6, lasting 80 ticks.
  Next nonlethal HP damage consumes it once; it cannot resurrect.
- **CC03:** every third unit hit sweeps 22 + A×8 damage within
  900 + B×350 of the primary; at most two other enemies (three at B2).
- **CC04:** 20 moving ticks charge +5000 + A×1500 bp on the next
  unit basic. Any Slow/Root clears movement progress and charge.
- **CC05:** every third unit hit pierces one enemy up to 1600 behind
  the primary for 6000 + B×1500 bp of static basic damage.
- **CC06:** each heal leaves 5 + B×3 healing for four pulses, 20 ticks
  apart. Actual HP damage cancels regrowth; absorbed shield damage does not.
- **CC07:** ready every 160 − A×20 ticks. Two-tick windup; cancel one
  still-pending special of the closest legal enemy. Victim immunity lasts
  80 ticks; Ward and existing control immunity protect it. No basic cancel.
- **CC08:** every third unit hit leaves a 60-tick solar spark of
  16 + B×6 damage. Only another conventional ally with printed base
  range ≤1600 consumes it on a unit basic. Cleanse removes the spark.
- **CC09:** alternate solar and lunar unit hits. A solar hit against a victim
  above/equal 5000 bp HP adds 1500 + A×500 bp basic damage; a lunar
  hit against a victim below that grants 600 + B×300 bp lifesteal from
  actual enemy HP removed,
  within the existing combined 1200 bp cap. No shield or Stronghold healing.
- **CC10:** basic bonus is 400 + A×100 bp of the victim's printed
  base HP, capped at 50 + A×10. Ordinary mitigation and shields still apply.
- **CC11:** at heal preparation snapshot up to two enemies within 1200
  of the patient; each adds 8 + B×2 healing, scaled by healing modifiers.
- **CC12:** ready every 160 ticks; six-tick windup locks the announced
  primary coordinate. Deal 45 + A×15 to up to three enemies within
  800 + B×300 of that fixed coordinate even if the primary dies.
- **CC13:** basic applies the existing single bleed family, 4 + A damage
  for three pulses. Producer kind is retained for offline attribution.
- **CC14:** combat death bursts once for 60 + A×15 damage within 900,
  at most three enemies. No ally/Stronghold hit or recursive beetle burst.
- **CC15:** every third unit hit adds 3000 + A×1000 bp if the victim
  is winding a basic; it never cancels that attack.
- **CC16:** heal begins a locked patient channel: 4 + B pulses separated
  by five ticks, first pulse at 10000 + A×3000 bp healing. Remaining
  pulses retain snapshotted base healing. Root, death, or lost range cancels.
- **CC17:** ready every 160 − B×20 ticks; six-tick windup applies
  standard 2000 bp Slow for 30 + A×10 ticks to at most two enemies
  within 700 of the locked primary. Existing Root priority remains.

Fusion preserves victim echo/regrowth/spark and interruption immunity,
without inheriting a producer's attack sequence or channel. Strongest
compatible family survives; remaining duration/pulses are conservatively
retained. Existing summon limits and targeting rules remain unchanged.

**CC-ID01:** register 68 kinds, preserve the first 51 tags, keep the
46-kind CANON Shop, and expose 63 conventional kinds only in the local
preview. Protocol 32 identifies the new public windup/channel/effect shape;
31 was already used by another local preview. Ruleset revision 3 includes
the six new gameplay modules in its source identity. Empty completion
state and bleed producer slots change legacy state checksums, so the
existing three golden outcome/winner/tick guards must remain intact
while schema-only hash values are refreshed with observed evidence.

**CC-ID02:** private TrainingScenario validation accepts COMPLETION_POOL,
not the online Shop pool. All 85 forms can be placed explicitly; summons
remain rejected. Training still bypasses economy and never sets a match
outcome. Online intention/admission rules are unchanged.

CC16 Root cancellation clears the authoritative channel immediately, including
its public reconnect facts; no delayed visual thread survives that Root.

CC-ID01 appends the 68 Proposed Evolution records to the frozen registry
contract, preserving all 204 pre-existing rows verbatim. Names come from
the bilingual catalog; numeric deltas follow CC-C05.


### CC-ID03 local preview integration — Proposed, owner-selected PR44 baseline

The local CC01–CC17 preview now includes PR44 head `0b56bb4` and its existing
D-103/P71 rules and laboratory presentation. Preserve the PR's established
values and the seventeen Proposed additions; this integration does not tune
balance. Its combined schema/source identity uses wire 33 and ruleset 7,
with the collision/migration documented in DECISIONS CC-ID03. The online
FULL_POOL remains 46; local Practice/Training uses 63 conventional kinds.
No production activation or empirical balance acceptance is implied.


CC-ID03 measured identity migration: all three PR44 legacy reference matches
retained their winner, Stronghold-destruction reason and tick (4044, 9982,
10382). Empty completion fields change serialized checksums only; revision-7
references append beside the unchanged revision-2 and revision-6 contracts.
The additive P73 protocol/history module is explicitly excluded from combat
source hashing because it contains transport/API schemas, guarded by wire 33.

## Ogre expansion — Proposed OG-01 (owner, 2026-11-01)

The owner requests an eighth Faction, **Ogres**, using the existing SiegeWalker
identifier/displayed **Ogre Archer / Arqueiro Ogro** and eight new conventional
champions. The owner also approves a replacement Machine to retain nine per
Faction in the authored preview roster: **72 conventional champions**, eight
Factions, nine each. Existing kind and Faction indices are preserved; append
Ogres after Insects and the nine kinds after Stridulator. Summons never count.
The private preview pool grows to 72; the established online pool remains 46
until separate competitive balance acceptance. The public landing catalog
lists all authored champions and visibly identifies preview-only availability.

**Proposed, explicitly requested range change:** Ogre Archer moves from Machines
to Ogres and has the Archer's exact range in every form: base 4500, A1/A2 6000,
B1/B2 4500. Its other stats, splash, Evolution effects, large-unit rules and
server authority are unchanged. Cosmetic equipment clearance never changes
collision, targeting, damage or the permitted authoritative lane offsets.

**Proposed Ogre Synergy:** the existing 2/4 living-unit thresholds grant +500/
1000 bp attack speed, through the existing AttackSpeed effect and caps.

The nine new definitions and all integer ability parameters are in
`content.rs` and `content/completion.rs`; their bilingual forms and unique art
briefs are in [the expansion catalog](ogre-faction/catalog.json). These are
initial proposals, not a competitive balance or deployment claim. Reused typed
ability primitives retain their unit-only targeting, deterministic ordering,
caps, server windups and no-resurrection rules.

- Basalt Bulwark: Tank, 740 HP, 34 damage, 32-tick interval, 900 range, 35 speed;
  unit-only max-HP chip damage capped at 24 (B1/B2 cap +8 per tier).
- Cinder Maul: Dps, 400/45/30/900/43; every third hit sweeps 20 damage into up
  to two nearby enemies (A damage +7/tier; B radius +250/tier, B2 third target).
- Hook Brawler: Dps, 340/40/25/1000/50; +2500 bp unit damage below 30% HP,
  A branch +1000 bp per tier. No hook displacement is implied by the visuals.
- Flint Runner: Dps, 300/35/22/800/62; 24 walking ticks charge a +4000 bp next
  hit, A +1500 bp/tier. No teleportation or client-authored movement.
- Clay Binder: Control, 360/22/34/3000/36; 20% Slow for 20 ticks on hits;
  periodic 24-damage earth pulse, 180-tick period, 750 radius, three targets.
  A period -15/tier; B damage +8 and radius +150/tier.
- Gust Drummer: Control, 420/20/36/1900/38; 12-tick interrupt windup, 180-tick
  period, eight-tick lock; A period -15/tier, B lock +2/tier.
- Hide Mender: Support, 360 HP, zero damage, 32-tick interval, 2800 range,
  38 speed, 30 heal; four regrowth pulses of 4 at 20-tick gaps, B +2/tier.
- Pebble Slinger: Ranged, 280/36/28/4100/48; every third-hit opportunity
  window adds 3000 bp against a foe's hostile windup, A +1000 bp/tier.
- Rivet Ballista: Machines/Ranged, 290/37/30/4300/40; every third rivet
  pierces one target within 1200 behind for 4500 bp, B +1000 bp/tier.

All new units cost 2400 Energy subunits except Basalt Bulwark 3000 and Hide
Mender 1800. All have zero base splash and base heal except Hide Mender.
A1/A2 add 1500 bp per tier to damage (Tank HP, Support heal); B1 adds 250 range,
B2 adds 1500 bp attack speed. The standard Evolution rules remain unchanged.
Basalt chip ratio is 300 bp, A +100 bp/tier. Source values are initial **Proposed**
local-preview values; D-33 targeted screening and full competitive acceptance
are recorded separately in the expansion report.

### OG-01 tuning candidate 9 — Proposed, D-33 initial-screen response

The initial non-healer values above are preserved as the ruleset-8 proposal.
The first bounded D-33 screen found all eight new solo kits losing the three
reference matchups. Candidate 9 raises their base combat value without changing
the requested Ogre Archer range, Energy costs, abilities, forms or roster:
Basalt Bulwark 1080/42/26/900/40; Cinder Maul 420/64/28/900/43;
Hook Brawler 380/56/24/1000/50; Flint Runner 340/46/20/800/62;
Clay Binder 380/44/30/3200/36; Gust Drummer 460/32/28/1900/38;
Pebble Slinger 360/60/22/4100/48; Rivet Ballista 340/65/22/4300/40.
Order is HP/damage/interval/range/speed. Hide Mender remains unchanged.
The earth pulse's already-authoritative center is visible in the public facts,
so its telegraph survives snapshots/reconnects. Drum interruption remains
special-only and never cancels a basic windup. These values remain Proposed;
the second targeted screen and limitations are in the expansion report.

## Rolling remedy — Proposed OG-02 (owner, 2026-11-01)

The owner replaces Gust Drummer with **Barrel Keeper / Mestre do Barril**.
The stable `GustDrummer` id is retained. Its Archetype becomes Support; the
former interrupt is removed. Ogres deliberately now have two Supports and
one Control, preserving nine champions. No other Ogre combat kit changes.

Proposed base: cost 2400, HP 460, damage 0, interval **200 ticks / 10 s**,
range 3000, speed 38, healing 90. The ordinary six-tick heal windup launches
a finite rolling cask toward the most damaged **other** ally in range. The
usual attack-speed/Slow interval rules apply; there is no second short heal
or invisible instant healing at release. A1/A2 each add 1500 bp healing;
B1 adds 250 range and B2 adds 1000 bp healing, plus normal Evolution growth.

Launch snapshots heal amount, damage, radius, tier/branch and owner. The
barrel travels at 220 milli-tiles/tick, at most the launch range, in its locked
one-dimensional direction. Swept contact radius is 180. It collides with the
first living targetable conventional unit or summon of either seat, excluding
the caster; distance then unit id resolve ties. It never homes, targets a
Stronghold or passes through an intervening full-health ally. A missed cask
expires without healing. A launched barrel survives caster death and is part
of deterministic replay, seat views and PublicView. Paused combat freezes it.

Any contact bursts healing 90 (snapshot-scaled by form/bonuses) to all living
allies within 1250 milli-tiles of the collision point, including the caster
and carried allies in the area. Healing reduction and max-HP clamps apply.
An intercepting enemy alone takes 8 auxiliary damage; no enemy area damage,
control, on-hit proc or resurrection. B branch adds 250 radius and 2 damage
per tier. At most one active barrel per caster. Clients interpolate these
facts on the same snapshot timeline as units; they never decide collisions.
The cask carried by the model is hidden during authoritative reload.

## Abyssals and Plant replacement — Proposed AB-01 (owner, 2026-11-01)

Append **Abyssals / Abissais** after Ogres. Move existing **Trident / Tridente**
from Plants to Abyssals, preserving its id, stats, abilities and Evolutions;
Faction-dependent bonuses now follow Abyssals. Add **Pod Snapper / Estala-Vagens**
as the Plants replacement, an authored Ranged champion. The Abyssal roster is
Trident plus eight new champions: one Tank, three Dps, two Ranged, two Control,
one Support. Nine new kinds append after RivetBallista; no old tag is reordered.
The authored private preview has **81 conventional champions, nine Factions,
nine each**. FULL_POOL stays 46; new kinds are not activated online.

Proposed Abyssal Synergy: the existing Movement effect grants +500/+1000 bp
at the established 2/4 living-unit thresholds, under the shared movement cap.

Initial base columns below are cost, HP, damage, interval ticks, range,
movement per tick and healing. Every value is **Proposed**, requiring D-33
screening and separate competitive acceptance. Form A1/A2 each add 1500 bp
damage (healing for Foam Weaver), B1 adds 250 range, B2 adds 1500 bp attack
speed, besides normal Evolution growth. Bilingual forms and art briefs:
[the Abyssal catalog](abyssal-faction/catalog.json).

- Nacre Duelist: 2400/370/57/22/950/54/0; Opportunity every 3 hits, +3500 bp
  against a winding basic attack; A +1000 bp/tier.
- Reef Lancer: 2400/410/58/26/1300/48/0; every 3 hits pierces one enemy
  within 1000 behind the primary, 4500 bp damage, B +1000 bp/tier.
- Trench Fist: 2400/480/62/30/850/42/0; unit-only printed-base-HP chip of
  250 bp, A +100 bp/tier; cap 22, B +8/tier.
- Pearl Slinger: 2400/350/58/24/4100/48/0; every 3 hits splashes 18 damage
  to at most two other enemies, radius 650; A +6 damage/tier, B +200 radius/tier.
- Abyss Harpooner: 2400/360/64/26/4300/42/0; one non-stacking bleed,
  4 damage ×3 pulses, A +1 damage/tier.
- Undertow Herald: 2400/420/34/30/2500/40/0; every 170 ticks, Slow 2000 bp
  for 30 ticks on at most three enemies within 850 of the primary;
  B -15 period/tier, A +10 duration/tier.
- Trench Lamplighter: 2400/370/42/30/3300/38/0; announced fixed-center pulse
  every 180 ticks, 30 damage to at most three enemies within 700;
  A +10 damage/tier, B +200 radius/tier.
- Foam Weaver: 1800/340/0/34/2800/42/30; healing leaves a 16-HP echo for
  70 ticks, consumed by the next nonlethal HP hit; B +6 echo/tier.
- Pod Snapper (Plants): 2400/390/52/26/3900/40/0; every third hit applies a
  60-tick non-stacking seed mark: another ally's next melee hit adds 18
  auxiliary damage, B +6/tier.

All reused typed primitives retain their existing targeting, attribution,
shared caps and deterministic replacement rules. No new flight, ocean Arena,
client targeting, stealth, teleport, economy or Shop rule is introduced.
Wire 35 and ruleset 10 identify the ninth Synergy slot and rolling-cask state.

### AB-01 bounded-screen follow-up — Proposed, supersedes the two initial rows

The initial 108 seat-swapped Energy-20 duels are recorded in
[balance-initial.csv](abyssal-faction/balance-initial.csv). They are a bounded
screen, not the competitive gate. Nacre Duelist scored zero against each of
Shieldbearer, Archer and Longaxe; Pearl Slinger scored one against all three.
Before native delivery, propose **Nacre Duelist 430 HP, 64 damage, 22 interval,
1050 range, 54 speed**, and **Pearl Slinger 330 HP, 46 damage, 26 interval,
4000 range, 48 speed; third-hit splash 14 damage**. Cost remains 2400 each.
Other AB-01 parameters and the staged-only boundary stay unchanged. Recheck
only those two kinds against the same three references (12 seat-swapped duels).
Team compositions, every form/budget, specialist counters and full competitive
acceptance remain pending; no gate threshold is changed.

The 12-duel follow-up ([raw data](abyssal-faction/balance-follow-up.csv)) gives
Nacre Duelist score 1 against Shieldbearer and 0 against Archer/Longaxe.
Pearl Slinger's reduced candidate scores 0 against all three; use the
intermediate **Proposed final base 340 HP / 52 damage / 24 ticks / 4100 range /
48 speed**, still cost 2400 and third-hit splash 14. Recheck only Pearl Slinger
in six seat-swapped duels. This final row supersedes its two starting rows;
no other new or established champion is retuned by this follow-up.

The final six-duel Pearl Slinger screen
([raw data](abyssal-faction/balance-final.csv)) scores 1 against Shieldbearer
and Longaxe and 0 against Archer. Other initial candidates and Trident's
before/after transfer retain their original screen records. These bounded
results justify the two initial proposals only; they do not establish full
team/form/budget balance. The final proposal is revision 10, source hash
`523044fc973bac39`; canonical trace validation pins that exact identity.


## TW-01 / PK-01 / GD-01 — Proposed owner revision (2026-11-02)

Owner authorizes stronger Strongholds and Fortifications, width-limited
BattleLine occupancy, and Gods with one gigantic Colossus. These proposals
supersede conflicting earlier numeric rows. Competitive acceptance remains
subject to D-33; the bounded screen is not the full gate.

**TW-01.** Reference Stronghold HP 6750 and damage 159 (formerly 4500/53).
Printed base HP/damage by kind: Classic 6750/159, Treasury 6277/144,
Bastion 7762/141, Artillery 5737/174, Command 6480/144. HP is +50%, truncated
to integer, and damage is exactly triple each former printed value.
Fortification costs become 3600 then 6000 sub-Energy (6 then 10 Energy).
Existing branch lock, proportional HP rescaling and maximum tier 2 remain.

- Treasury: 270 sub-Energy per eligible kill, cap 1080 per existing 200-tick
  window; A +2500 HP bp per step; B +90 spoils per step and +540 cap at B2.
- Bastion: 1500 bp incoming damage reduction, before shields; A +3000 HP bp
  and +500 armor bp per step; B1 +1500 range, B2 +3000 damage bp.
- Artillery: primary shot also damages at most three other living enemies
  within 1000 milli-tiles, sorted by distance then ID, for 4000 bp of shot
  damage. A +4000 damage bp and +2000 splash bp per step. B intervals 14/10
  ticks. No friendly splash; collateral conventional kills count toward the
  existing Tower Fury cap/duration. Zero configured tower damage means no fire,
  including the D-33 duel Arena. It never becomes a one-damage shot.
- Command: aura 1500 bp in 5000 milli-tiles; A +750 aura bp per step;
  B +1500 aura radius per step. Existing eligibility and additive caps remain.

**PK-01 activation.** Physical lanes are a private Practice/Training and
explicitly opted-in lab candidate. Long-running preview bot probes exposed
persistent stalemates, so competitive canonical matches keep legacy movement
and presentation until D-33 acceptance. This flag is server-owned and public
in both views; no PlayerIntention can enable it. The entire roster is
covered in the local preview. Do not claim ranked activation or completed
competitive acceptance.

**PK-01.** The BattleLine has 5000 milli-tiles of usable lateral width,
with a matching 5.08-tile path. Fifty integer columns of 100 reserve
champion body, arms and held equipment. The normative per-kind base profile
is [content/occupancy.rs](../crates/tbg-core/src/content/occupancy.rs), derived
from the [native mesh measurements](colossus-fortifications/native-footprints.json)
over all five forms, Idle/Walk/attack, 16 samples and six front/rear/lateral
aim cases. Each measured bound is rounded up to 50 milli-tiles with 100 extra
clearance; tier growth is 8% per tier. Profiles wider than the corridor occupy
all columns and cannot be passed; their visuals may extend beyond the path.
This conservative weapon envelope bounds dense ranged rows (two base Archers;
capacity also depends on tier and mixed widths). A three-tile initial candidate
blocked ranged units behind idle Supports and did not finish the bounded bot
probe; five tiles permits those bypasses while retaining physical capacity.

Capacity at a longitudinal position is the set of non-overlapping column
reservations, not an Archetype quota or a global army cap. Same-seat following
spacing is follower front + leader back + 100; opposing body separation is
both body half-lengths +100. Ordinary unit reach stays root-based, with a floor at the two body
half-lengths plus the 100-milli-tile contact gap. This lets a short melee
attack hit touching bodies without adding free reach to ranged attacks or
heals. Stronghold reach stays root-based. Existing
combat targets and area radii remain longitudinal; lateral columns govern
occupancy and passage, not a new two-dimensional targeting game.

Prefer existing placed units, then front-to-back progress, seat, and stable
ID. Preserve slots while stationary. A bypass needs a clear swept lateral
strip, moves at most one column per tick and locks for 20 ticks on completion
or a blocked transition. The server supplies lateral position in UnitView;
clients interpolate it on the same snapshot timeline and apply no second
live repulsion solver. Stress tools retain their separate cosmetic solver.

Paid purchases and summons with no room wait in a deployment queue; do not
discard them, attack, heal, block, generate Synergy or tick their action
cooldowns while waiting. Summon expiry still advances on the combat clock.
New births first try the base anchor. If a stationary backline blocks it,
choose the nearest free footprint in the owner's first third of the
BattleLine (endpoint 5333 for Seat A, mirrored for Seat B), without appearing
beyond a living hostile front. This changes only a new unit's initial
materialization, never teleports a deployed unit. Wait if that zone is full.
Release them deterministically when room opens. Only the owner's queued
count is exposed; queued identities and the enemy count are private. Training
and the balance duel's elimination/HP accounting include waiting units.
The Practice bot prefers Fortification/Evolution while its own entrance has
a queue; after those upgrades it seeks narrow attackers that can use free
strips and can attack a Stronghold. This is opponent policy, not a restriction
on player purchases.
An Evolution that cannot fit while preserving every deployed ally is rejected
with LaneFull before spending Energy. A forced displacement that leaves no
legal projection currently retains the active unit targetable; resolving all
teleport/displacement congestion is a documented remaining limitation.

Rolling-cask damage, collision and healing retain OG-02 longitudinal rules.
Their immutable launch origin/target and exact lateral burst coordinate are
public presentation facts so the rendered cask starts at its carrier and
meets the actual contact lane; clients decide no hit or heal.

**GD-01.** Append Gods (Deuses), currently exactly one conventional champion,
Colossus (Colosso), Tank: cost 10800 sub-Energy = **18 Energy**, HP **2400**,
damage **170**, interval **110 ticks**, range **4500**, movement **22**
milli-tiles/tick, full-damage frontal splash **1800**. Base damage is exactly
2x SiegeWalker and range equals its corresponding form. The obelisk is one
real prop, carried vertically on the back, drawn and slammed in the attack
direction. Splash excludes enemies behind the attacker relative to that hit.

- A1 Sun Obelisk / Obelisco Solar: +2500 damage bp, +1500 range.
- A2 Pillar of Dawn / Pilar da Aurora: +600 splash radius.
- B1 Ancient Monolith / Monólito Ancestral: +3000 HP bp.
- B2 Eternal Monument / Monumento Eterno: another +3000 HP bp.

Existing Evolution common bonuses/costs apply. Gods has no Synergy bonus
while its roster has only one kind. It cannot ride a Cargo Crab (Tank).
The private authored roster becomes **82 champions / 10 Factions**: nine
Factions of nine and Gods of one; FULL_POOL remains 46. New Gods content is
Practice/Training preview only. Team identity stays on foot rings, with no
body plaques or seat recoloring. Wire **36**, gameplay revision **11** identify
the new layout/rules; older enum indices and historical fixtures are retained.


TW-01 canonical trace identity: revision 11, source `a0a1d508c59e5841`.
The exact tower boost changes bot outcomes/ticks; append the observed five
canonical trace fixtures without changing any historical entry. Physical
lanes remain false in this baseline and true in the private PK-01 scenarios.


GD-01 bounded tuning: the first 3600-HP/70-tick candidate won all eight base
reference duels at Energy 18 and all eight evolved counter duels at Energy 24.
The 2400-HP/70-tick follow-up still won the eight evolved cases. The final
Proposed candidate is **2400 HP, 110-tick interval (5.5 s)**; the owner-fixed
18-Energy cost, 170 damage and Ogre Archer range stay intact. In the bounded
Energy-24 base panel, Sawbot, Farwatch and Mantis beat it on both seats;
Ravager loses. All four evolved reference forms still lose at that budget.
This identifies initial counterplay, not full team/form/budget acceptance.
Raw measurements and scope are in [the report](phases/COLOSSUS-FORTIFICATIONS-PACKING.md).

TW-01 / GD-01 Proposed contract migration: revision 11 pins the 6750/159
reference tower, 6/10-Energy Fortification, stronger branch/passive values and
Gods' zero-bonus Synergy row. Historical D-46 assertions stay in the contract
files behind their legacy revision path; unchanged ownership, spending,
window, aura timing and proportional-rescaling assertions still apply.

TW-01 opponent policy: once the existing six-unit preference threshold is
reached, a Practice bot with an unfinished selectable Stronghold saves the
actual next Fortification price before buying more troops. Classic matches
are unchanged by this policy. This is a bot spending preference, never a
player purchase restriction or a changed Fortification price.

PK-01 lab boundary: ordinary lab placement hooks do not change the physical-lanes
flag. Canonical duel matrices and gates retain canonical movement; the
Colossus candidate screen explicitly calls the preview duel path. This keeps
offline canon equivalent to online canon while the private proposal is tested.


## RESTORE-01 — Proposed owner correction (2026-11-02)

Supersedes PK-01 activation. The owner rejected physical passage blocking and
waiting purchases after observing stationary armies and delayed pieces.
Restore legacy movement in all normal constructors, including private Practice
and Training: purchases spawn immediately, allies do not block authoritative
progress, and Evolution never waits for footprint capacity. Keep the five-tile
BattleLine floor, every TW-01 tower value/ability and all Gods content.
The rejected PK-01 experiment remains explicit lab-only for historical tests;
it must not be enabled by Practice, Training or placement hooks.

Crowd separation is presentation only. Keep old pushing/overtaking and existing
longitudinal offset bounds. Use 2.1 tiles of lateral centre clearance on the
wide floor, a 60 Hz cosmetic solver (at most two catch-up steps), stable
anchor/ID pair ordering, 12/s offset easing, 4 tiles/s maximum cosmetic offset
travel and a 0.002-tile stationary deadband. New figures appear immediately;
only cosmetic displacement is eased, never authoritative movement. Aim envelopes
expand clearance without shifting the sort anchor. These are presentation
parameters and cannot change attacks, outcomes, Energy or unit entry.
Ruleset 12 identifies the changed preview default; wire 36 layout is preserved.
Canonical replay checksums/outcomes remain those of revision 11.


## RESTORE-02 — Proposed exact movement restoration (2026-11-02)

The owner rejects RESTORE-01 cosmetic easing as well. Supersede its fixed-step,
stable-anchor ordering, expanded aim-anchor envelope and extra display smoothing.
Restore the pre-PK-01 client pushing/overtaking solver and direct display offset
application exactly, changing only lateral centre clearance from 1.1 to 2.1 tiles
to use the retained five-tile BattleLine. Preserve immediate deployment and
authoritative legacy advancement in every normal mode. No passage capacity or
waiting queue is reintroduced. This is presentation-only; wire/ruleset identity,
towers and champion balance remain unchanged.

## ARENA-CLEAR-01 — Proposed decorative clearance (2026-11-02)

Keep the existing five-tile BattleLine and its paved border. Move arena dressing
outside that complete road rectangle in every Arena; do not narrow it to fit
scenery. Fire and animated cloth must share their actual pedestal/fence origins.
Move compound props from their authored origins so geometry, glow and attachments
stay together. Stronghold-attached banners remain part of the Stronghold.


## GD-02 — Proposed Colossus arm clearance (2026-11-02)

Use the Colossus form's measured body/arm/hand/equipment envelope in the existing
cosmetic crowd solver, including current facing and Evolution scale. The same
equipment selector must govern cached measurement and live crowd participation.
Keep RESTORE-02 direct pushing, the five-tile floor and immediate purchases; this
visual clearance must never enable physical queues or block authoritative advance.
No Colossus stats, damage, range or cost change.


## SIDE-01 — Proposed fixed blue/red sides (2026-11-02)

Owner-approved: the server independently assigns the paired participants to
Seat A (blue, left Stronghold, advancing right) and Seat B (red, right Stronghold,
advancing left), with equal probability, regardless of queue arrival order.
Apply to Ranked, Casual, private rooms/invitations and each fresh rematch.
The existing connection-bound seat owns commands and outcomes. Freeze it for
the match, including reconnection, summaries, spectators, replay and Elo.
Pair selection and rating calculations are unchanged.

All desktop players and spectators share the same blue-left/red-right camera
and world coordinates. The red player controls the right Stronghold; ownership
is separate from world side/color. Body art stays intrinsic, affiliation only
in foot rings and interface. Show the player's localized side alongside Arena
and in its introductory banner; spectators see a neutral blue/red legend.

Local Practice draws the human side from an independent seeded stream and puts
the bot on the opposite seat. Human Arcanas/Stronghold, private views, commands
and result interpretation follow the assigned seat. Existing disclosed Impossible
bot assistance uses the bot's own balance/seat. Editor Training layouts retain
their authored ally/enemy arrangement. No unit/Stronghold balance, Shop, economy,
world width, pushing or selection order changes.

## RESTORE-03 — Proposed departure formation restoration (2026-11-02)

The owner's correction concerns lateral departure before contact, not passage
blocking. Preserve the original preferred lateral formation scale of 1.1 while
retaining the widened road's 2.1 centre boundary. Extra road room is available
for actual contact/overtaking; it must not itself double birth/preferred offsets.
Keep the original separation, return motion, body/weapon clearance and immediate
authoritative advancement. No physical queue, capacity gate or extra smoothing.

## R13 — Proposed source-identity maintenance (2026-11-02)

Register ruleset 13 / `b66993d9e87a9cfc` for equivalent integer divisibility syntax
required by the pinned compiler lints. Keep every value, outcome branch,
wire tag and summary version from revision 12. This is source maintenance,
not an additional balance adjustment or full D-33 acceptance.

## INT-01 — Proposed consolidation of existing TW-01 / GD-01 contracts (2026-11-02)

R13 keeps the exact R12 tower values and traces. Extend the explicit revision
selectors and append its canonical C2 traces; retain all historical rows. The
registered roster is 87 kinds (82 conventional, five summons), including Gods/
Colossus and its four Evolution forms. FULL_POOL remains 46. These are existing
TW-01/GD-01 values, not another activation or balance change.

CC-ID03's old PR44 tick/winner fixtures describe the period before TW-01. Append
R13's already registered TW-01 outcomes/ticks instead of claiming stronger towers
preserve old combat. Unit-ability fixtures isolate towers where tower damage is
not their subject; carrier-protection fixtures keep the carrier alive throughout
their sampling window. The Furnace multiplier fixture retains its explicit old
53-damage incoming shot only to measure the unchanged 36-damage hit and 26-HP burn.
The Shop-enabled-but-unused comparison keeps the same canonical pool on both
sides; the private preview's deliberate 82-champion pool is a separate feature.
No new competitive number, rule, source identity or online-roster activation is
introduced by this reconciliation. D-33 acceptance remains separate.

## GD-04 — Proposed Gods expansion: Volthar and Astreon (2026-11-02)

Owner request, values **Proposed** (D-33 acceptance pending). Append two Gods
champions after Colossus to the local Practice/Training preview pool (84 kinds);
FULL_POOL stays 46 and online canon is unchanged. Every form row changes only
the champion's ability; Evolution costs and stat contracts are the common ones.
Gods keeps its zero-bonus Synergy row (BR-Y02): a Gods Synergy is an owner
decision, not part of this phase. Ruleset **14** / `f354f651c056d615`, wire **37**.

**Volthar — Deus da Tempestade (Storm God)**, Ranged, 12 600 sub-Energy
(**21 Energy**), HP **2200**, damage **130**, interval **30 ticks** (1.5 s),
range **5500**, movement **35** milli-tiles/tick (0.7 tile/s), no splash.

*Storm Judgment / Julgamento Elétrico.* Each Volthar counts its own completed
basic hits on enemy units (Stronghold hits never count, never reset). The fourth
resets the count and starts one discharge from the primary: up to `jumps` links,
each to the nearest living targetable enemy unit within `radius` of the previous
link (longitudinal distance, ties by lower id), never one already struck, never
an ally, never a Stronghold. Each link deals its share of static basic damage as
auxiliary damage (damage reduction and shields apply; no basic-hit hooks, no
lifesteal, no further discharge). The chain stops when no link qualifies. A link
killed by its jump still anchors the next one.

| Form | Jumps | Radius | Shares (bp of basic) | Trigger hit | Extra |
|------|-------|--------|----------------------|-------------|-------|
| Base | 3 | 2200 | 7000 / 5000 / 3500 | — | — |
| A1 Storm Herald / Arauto da Tempestade | 4 | 2200 | 7000 / 5000 / 3500 / **2500** | — | — |
| A2 Celestial Sovereign / Soberano Celestial | 4 | 2800 | 8000 / 6000 / 4500 / 3500 | — | — |
| B1 Hammer of Wrath / Martelo da Ira | 3 | 2200 | 7000 / 5000 / 3500 | +3500 bp | — |
| B2 Divine Tempest / Tempestade Divina | 3 | 2200 | 7000 / 5000 / 3500 | +5500 bp, Root 10 ticks | 3000 bp to ≤ 2 |

The A1 fourth share (2500) is an agent proposal: the request named four links
but only three shares. B1/B2 add their trigger share once to the primary, before
situational percentages (BR-X01); B2 replaces B1. B2's Root uses the ordinary
on-hit path (ward and control immunity block it). B2's extra discharge picks
the nearest living enemies to the primary within 2200 that the chain missed.

**Astreon — Deus da Ruptura (Rupture God)**, DPS, 13 800 sub-Energy
(**23 Energy**), HP **1900**, damage **190**, interval **40 ticks** (2 s), range
**2200** (spear reach between melee and Ranged), movement **37** milli-tiles/tick
(0.74 tile/s; 0.75 is not an integer per tick), no splash.

*Verdict Lance / Lança do Veredito.* Each Astreon tracks completed basic hits
on one marked enemy unit. A hit on another unit restarts the sequence at one; a
windup on anything but the marked unit, a Stronghold hit or the marked unit's
death clears it. On the `every`-th consecutive hit the basic hit adds
`min(max_hp × share, cap)` of the target's **max HP** after offensive percentages
and before damage reduction and shields (not true damage), then the sequence
restarts. B forms add an execution share of static basic damage on that same hit
when the target's HP is below 40 % of its max HP at hit resolution (BR-X11).
No splash, lifesteal or control immunity is added.

| Form | Every | Max-HP share | Cap | Below 40 % |
|------|-------|--------------|-----|------------|
| Base | 4 | 500 bp | 240 | — |
| A1 Giant Piercer / Perfurador de Gigantes | 3 | 500 bp | 240 | — |
| A2 Primordial Lance / Lança Primordial | 3 | 700 bp | 300 | — |
| B1 Final Sentence / Sentença Final | 4 | 500 bp | 240 | +3500 bp |
| B2 Absolute Verdict / Veredito Absoluto | 4 | 500 bp | 240 | +6000 bp (replaces B1) |

Presentation: the server emits a public `StormArc {unit, from, to, jump}` per
link (jump 0 = B2 extra) and the existing `AbilityResolved`; clients only draw
them. Both counters share the existing public per-kind progress bits; every
other system (Synergy rows, Arcanas, Augments, Interest, Shop, Evolution costs,
Strongholds, carry rules) applies unchanged. Smuggler cargo keeps its pre-GD-04
draw table: Volthar and Astreon never come out of a crate, so canonical online
matches and their traces are identical to revision 13. PK-01 lab footprints are authored
provisionally until the native mesh export measures the new forms.


## P74 — Proposed War Auxiliaries (`Mineiro` & `Mercador`, 2026-11-02)

War Auxiliaries are off-lane economic and shop specialists hired with `Energy`
during `Phase::Running`. They occupy two dedicated HUD slots in the bottom-right
panel, separate from the five conventional `Shop` offers, and appear as animated
low-poly 3D precincts and figures behind each seat's `Stronghold` outside the
`BattleLine`.

- **BR-AX01 (Slots & Availability, Proposed):** Both seats have two fixed
  auxiliary slots (`0 = Miner / Mineiro`, `1 = Merchant / Mercador`), starting
  in `AuxiliaryPhase::Available`.
- **BR-AX02 (Hiring Authority, Proposed):** `PlayerIntention::HireAuxiliary { slot }`
  is valid only during `Phase::Running` when no Augment round is open, `slot < 2`,
  the slot is `Available`, and the seat has enough `Energy`.
- **BR-AX03 (Single Hire per Match, Proposed):** Each auxiliary can be hired at
  most once per seat per match (`Available -> Preparing -> Active -> Completed`);
  repeat attempts return `Rejection::AuxiliaryAlreadyHired`.
- **BR-AX04 (Miner Cost & Preparation, Proposed):** Hiring `Mineiro` (slot `0`)
  costs `6` `Energy` (`3_600` sub-energy) and enters `Preparing { ticks_left: 240 }`
  (`12 s` at `20 Hz`).
- **BR-AX05 (Miner Active Production, Proposed):** When preparation finishes,
  `Mineiro` enters `Active { ticks_left: 3_600, .. }` (`180 s`) and generates
  `+0.06` `Energy`/s (`36` sub-energy/s) via exact integer Bresenham steps
  (`(elapsed * 36) / 20 - ((elapsed - 1) * 36) / 20`), yielding `6_480`
  sub-energy (`10.8` `Energy` gross, `+4.8` `Energy` net) before entering
  `Completed`.
- **BR-AX06 (No Interest Multiplier on Miner Output, Proposed):** `Mineiro`
  production is added after `regen_energy()` and is never multiplied by
  `InterestRule::regen_bp`.
- **BR-AX07 (Merchant Cost & Activation, Proposed):** Hiring `Mercador` (slot `1`)
  costs `4` `Energy` (`2_400` sub-energy) and enters
  `Active { ticks_left: 0, progress: 0, activations_used: 0 }` immediately.
- **BR-AX08 (Post-Hire Purchase Counter & Free Rerolls, Proposed):** Only
  conventional champion purchases (`PlayerIntention::Purchase { offer }`) made
  after hiring `Mercador` increment `progress`. Every `3` purchases reset
  `progress` to `0`, increment `activations_used`, grant `+1` `free_rerolls`
   to the seat, and emit `Event::AuxiliaryTriggered { seat, kind: Merchant }`.
- **BR-AX09 (Free Reroll Consumption & Streak Preservation, Proposed):** While
  `free_rerolls > 0`, `shop::reroll_cost` returns `Some(0)` and `Reroll { cost: 0 }`
  consumes one free reroll without spending `Energy` or incrementing
  `reroll_streak`.
- **BR-AX10 (Merchant Cap, Proposed):** After `3` activations (`9` post-hire
  champion purchases), `Mercador` enters `AuxiliaryPhase::Completed`; any unspent
  `free_rerolls` remain usable.
- **BR-AX11 (Public Precinct Visibility, Proposed):** Both seats' `AuxiliarySlotView`
  states and `Event::{AuxiliaryHired, AuxiliaryTriggered}` are public in
  `MatchView` and `PublicView` so both players and spectators see the 3D
  precincts, construction animations, and Merchant Caravel ship; `free_rerolls`
  remains private to the owning seat's `MatchView`.

## INT-06 — Proposed P74/GD-04 consolidation contract (2026-11-02)

The owner requested one integrated delivery of P74 and GD-04. War Auxiliary
prices, preparation, production, purchase counts and activation caps remain
exactly BR-AX01–BR-AX12; Volthar/Astreon remain in the 84-unit local preview
and FULL_POOL stays 46. Wire 38 carries the combined public auxiliary phases,
private free-reroll balance/quote and StormArc events. R15 includes auxiliary
state in deterministic hashes and replay serialization; no phase or currency
may disappear after a roundtrip. These repairs introduce no new balance value.

## 22. Administration, moderation, key recovery and match social (P75, Proposed)

**Proposed — planning only, 2026-11-02, inactive until P75 is implemented,
reviewed and activated by the owner.** Owner request: required player
email, in-game key recovery by email without losing anything, a complete
administration suite with bans and moderation, account reports, real-time
match chat, emojis with three slots and moderated custom emojis/GIFs, and a
single Owner administrator claimed by email. See
[P75](phases/P75-admin-moderation-and-match-social.md) and its
[requirements](planning/P75-REQUIREMENTS.md) and
[contracts](planning/P75-CONTRACTS.md). No combat, Energy, Shop, Evolution,
Synergy, Rating formula or balance value changes. BR-P01/BR-P02 are amended
by the P75 implementation in the same commits as their contract tests
(registration requires an email; the key becomes recoverable); until then
they stay as written above.

| Rule | Contract |
|---|---|
| BR-ID01 | A new `Account` needs an email address at creation; the account, its `AccountKey` and the unverified address are created together. Guests need nothing |
| BR-ID02 | An address becomes the `RecoveryEmail` only after an emailed code is entered in game. Addresses are private: never shown to other players, in public pages, logs or URLs |
| BR-ID03 | Pre-P75 accounts keep playing; adding an address needs the current `AccountKey`. Changing a verified address needs the key plus codes from the old and new addresses, or the key plus a 7-day hold the old address can cancel |
| BR-ID04 | At most 5 accounts per address; blocked addresses cannot register |
| BR-RC01 | "Lost your key?" asks for the `PlayerName` and the address and always answers the same way. A code goes only to a verified address that matches the name |
| BR-RC02 | Entering the code replaces the key **on the same Account**: name, `Rating`, statistics, ranked records, match history, Arcanas, Stronghold kind, emojis and every other account record stay exactly as they were. The new key is shown once in game; the old key and every session stop working at once |
| BR-RC03 | Recovery codes are single-use and expire after 15 minutes or 5 wrong attempts; keys are never emailed. Losing both the key and the verified mailbox cannot be recovered |
| BR-AD01 | `Operator`s are separate from `Account`s. There is exactly one Owner, created from the host's configured address with no key; the Owner claims an `AdminKey` through an emailed code and recovers a lost one the same way. A new key revokes the previous key and sessions |
| BR-AD02 | Console sign-in uses the `AdminKey` plus an emailed login code (Proposed default). The Owner may invite Moderators with fewer powers. Every Operator action needs a reason and is audited permanently |
| BR-MD01 | An Operator may suspend an `Account` for 1 hour to 365 days or permanently, restrict its chat or emojis (1 hour to 90 days), or sign it out everywhere. Moderators may suspend for at most 30 days |
| BR-MD02 | A suspension takes effect at once on every server: sessions end, queues and invitations are cancelled and a seat in a running match loses by forfeit, rated exactly like a surrender (BR-M06, BR-R04). The player sees the end time and reason category |
| BR-MD03 | A suspended account is hidden from the Leaderboard, directory, live list and public profile while suspended. Suspensions never edit ratings, results or anyone's history; lifting one restores everything |
| BR-MD04 | Reports never punish automatically; no Operator action can change a match result, rating or history (ranked corrections stay the D-89 command-line flow) |
| BR-RP01 | A signed-in player may report another account after an online match they played together (within 7 days), from a public profile/directory, or for a custom emoji shown in that match. Reasons: cheating, abusive chat, offensive name, offensive emoji, harassment, griefing, impersonation, other; optional comment ≤ 500 characters |
| BR-RP02 | One open report per reporter, target and match; at most 10 reports per day. The reporter is never revealed to the target |
| BR-CH01 | During an online PvP match (any mode) the two seats may exchange `MatchChat`. Spectators, practice and training have none. Text never enters the simulation |
| BR-CH02 | Sending text requires an account with a verified address and no chat restriction; guests read only. Messages are 1–120 Latin-1 characters, at most 3 per 5 seconds and 20 per minute; blocked terms are masked |
| BR-CH03 | Each player can mute the opponent's text and emojis and choose Chat All / Emojis only / Off. Chat starts disabled until the owner switches it on |
| BR-EM01 | Every account has three `EmojiSlot`s (guests use the defaults), frozen at pairing. Sixteen standard emojis ship with the game; their ids never change or get reused |
| BR-EM02 | A seat may send an emoji from its slots at most once per 2 seconds and 8 times per 30 seconds; it shows next to the sender's name for 2.5 seconds |
| BR-CE01 | A player with a verified address may upload PNG, GIF or static WebP (≤ 1 MiB). The server converts it to at most 24 frames of 128 px within 4 seconds; nothing is usable until an Operator who saw every frame approves it |
| BR-CE02 | Approved custom emojis are usable only by their uploader; an Operator may revoke one at any time, which removes it from every slot at once. At most 3 uploads per day and 10 kept per account |

**Proposed operational values (P75):** codes 10 characters
(`XXXXX-XXXXX`); verification/claim codes 24 h; recovery 15 min; console
login code 10 min; console sessions idle 30 min / absolute 8 h, ≤ 3,
sensitive re-entry 5 min; mail retries up to 8 with backoff to 6 h; outbox
≤ 2,000 pending; transcripts ≤ 200 lines/32 KiB kept 7 days; closed reports
180 days; rejected emoji sheets 30 days; global pending emojis ≤ 500;
initial switches: chat off, emojis on, custom display off, uploads off,
reports on, registration on. All are owner decisions O-P75-01…13.
