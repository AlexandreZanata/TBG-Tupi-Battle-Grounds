# Game design — TBG Arena 5v5 (MOBA mode)

> Canon of the 5v5 mode. **Every rule and number here is Proposed**
> (owner request, 2026-11-03) and **unbalanced**: balance work starts only
> after the mode is 100 % functional and the owner explicitly asks for it.
> The 1v1 canon ([GAME-DESIGN.md](../GAME-DESIGN.md)) is untouched; this mode
> shares its spirit (buy units, send them, never micro-manage combat) and
> its glossary style, not its rules. Rule IDs `M-xx` are cited by the
> contract tests in `moba/core-moba/tests/contract_moba.rs` and
> `moba/server-moba/tests/room.rs`.

![Overview at 7:03, ten bots](media/p76-overview.webp)

## 1. Pillars

1. **Ten commanders, one valley.** Two teams of five, one role each, on a
   three-lane map with a jungle and a river — readable at a glance.
2. **Strategy over twitch.** A player steers one Commander with a right
   click and wins by *what* champions they buy and *where* they send them.
3. **One roster, two modes.** Troops *are* the 1v1 champions: same names,
   archetypes, base stats, prices (in Gold), Shop rules, portraits, 3D
   figures, animations, shots and impacts. What is new in 5v5 are the lane
   Minions, Towers, the jungle and the Commanders' roles.
4. **No delay for anyone.** The server ticks at 20 Hz, sends one shared
   world frame per tick and never lets a slow player delay the other nine.
5. **Bots everywhere.** Every empty seat is a bot using the same intentions
   as a human, so a match always has ten sides and a human can drop in or out.

## 2. Glossary (use these words exactly)

| Term | Meaning |
|---|---|
| `Seat` | One of ten player slots: Blue `0..5`, Red `5..10`. Assigned by the host from the connection, never from a message. |
| `Team` | `Blue` (south-west base) or `Red` (north-east base). |
| `Role` | `Top`, `Jungle`, `Mid`, `Bot`, `Support` — seat index modulo 5. |
| `Lane` | `Top`, `Mid`, `Bot` road between the two Strongholds. |
| `Destination` | Where a purchased Troop goes: a `Lane` or the `Jungle` ("Destino da Tropa"). |
| `Commander` | The player's own unit on the map, drawn as its role's 1v1 champion. Auto by default; Move/Hold on command. |
| `Troop` | A 1v1 champion (`tbg_core::UnitKind`) bought in the `Shop` and sent to a `Destination`. |
| `Shop` | Five offers per player drawn from the 1v1 online pool, with locks and a paid reroll, as in the 1v1 game. |
| `Minion` | Free lane unit spawned in `Wave`s by each Stronghold. |
| `Wave` | Three melee + two ranged Minions per lane, per team, every 30 s. |
| `Camp` | A jungle pit of neutral monsters (one big, two small). |
| `Tower` | Defensive structure: Outer, Inner, Base per lane, plus two Nexus towers. |
| `Stronghold` | Each team's crystal; destroying the enemy's wins. |
| `Gold` | Currency for Troops (coins on the HUD). |
| `Essence` | Second currency (purple gem) spent on Troop `Upgrade`s. |
| `Upgrade` | Raises the player's Troop tier (+HP and damage for new Troops). |
| `MobaIntention` | The only way a player or bot acts: `BuyTroop`, `Reroll`, `ToggleLock`, `Upgrade`, `MoveCommander`, `CommanderAuto`. |

## 3. Match structure

- **M-01** A match has 10 Seats in 2 Teams of 5 with roles Top, Jungle,
  Mid, Bot, Support. The simulation ticks at 20 Hz.
- **M-02** One map, *Tupi Valley* (`moba/core-moba/src/map.rs`), 200 × 200
  units: three Lanes, a river from corner to corner (presentation only),
  a central plaza, 8 Camps (4 per jungle), 11 Towers and 1 Stronghold per Team.
  Red's layout is Blue's reflected across the anti-diagonal.
- **M-03** Same seed + same intentions ⇒ identical match (integer state,
  id-ordered units, no clocks, no unseeded randomness, no HashMap iteration).
- **M-04** Bots never act for human seats; humans and bots use the same
  `MobaIntention` validation.
- **M-05** The seat of an intention comes from the host (connection or bot
  slot), never from the message.

## 4. Units and combat

- **M-06** Every unit has HP, damage, range, attack interval, speed, armor
  (% damage ignored), sight and a body radius (table in §6).
- **M-07** Decisions use the start-of-tick state and all damage lands at
  once (simultaneous resolution). Targets: keep a valid one, re-scan every
  10 ticks for the nearest attackable enemy in sight.
- **M-08** Tower order: an Inner Tower is invulnerable while its lane's
  Outer stands; a Base while its Inner stands; Nexus Towers while all three
  Base Towers stand; the Stronghold while either Nexus Tower stands.
- **M-09** Units separate softly; structures never move.
- Lane walkers and structures ignore neutral monsters; jungle Troops and
  Commanders hunt them. Monsters are passive until hit and leash 9 units
  from their camp, healing fully at home.

## 5. Economy, shop and progression

- **M-10** Clicking a Shop card (or pressing 1–5) buys that champion for its
  card price and sends it to the chosen Destination; insufficient Gold is
  rejected. Price = the champion's 1v1 Energy cost × 6 Gold.
- **M-18** A champion keeps its 1v1 HP, damage, attack interval and range
  (both modes tick at 20 Hz in millimetres); it walks 4× its 1v1 speed
  because MOBA lanes are ~12× longer than the BattleLine. Tank archetypes
  ignore 20 % of damage. Champions with a 1v1 `heal` mend the most injured
  ally in range instead of attacking; champions with a 1v1 `splash` also hit
  enemies around their target for 50 %. 1v1 abilities and Evolutions are
  not active in 5v5 yet (P77).
- **M-19** Shop: five distinct offers from the 1v1 online pool, drawn from
  each seat's own seeded stream (1v1 BR-S01/S05); every purchase redraws all
  unlocked offers (BR-S04); locking a slot is free and keeps it through
  redraws (buying it unlocks it); a reroll redraws the unlocked offers for 6
  Gold (one 1v1 Energy).
- **M-11** At most 8 live Troops per player.
- **M-12** Passive income: +1 Gold per second, +1 Essence every 10 s.
- **M-13** Bounties go to the owner of the unit that landed the last hit:
  Minion 4, Troop cost/3, small monster 6, big monster 15 (+2 Essence).
- **M-14** Commander kill: killer +40 Gold +3 Essence; every allied Commander
  within 12 units +15 Gold (assist); the killing Team scores 1.
- **M-15** Tower destroyed: every player of the destroying Team +25 Gold
  +2 Essence.
- **M-16** Team Gold on the scoreboard is the sum of Gold earned.
- **M-17** Upgrade costs `15 + 10 × tier` Essence, up to tier 5; each tier
  gives +10 % HP and damage to Troops bought afterwards.
- **M-20** Commanders level 1–12 from experience shared by every allied
  Commander within 12 units of an enemy death (Minion 20, Troop 25, small
  monster 30, big 60, Commander 120). Level = +80 HP, +4 damage.
- **M-21** A dead Commander respawns at its base after `5 s + 1.5 s × level`.
- **M-22** Commanders within 14 units of their own Stronghold heal ~5 %/s.

## 6. Proposed values

| Unit | HP | Dmg | Range | Attack (s) | Speed (u/s) | Armor | Rule |
|---|---|---|---|---|---|---|---|
| Commander | 900 | 40 | its champion's range (≥ 1.8) | its champion's interval | 4.2 | 10 % | Top Longaxe, Jungle Mantis, Mid Hexer, Bot Celestial Archer, Support Luminous Oracle |
| Minion melee | 300 | 14 | 1.6 | 1.0 | 3.2 | 0 | — |
| Minion ranged | 200 | 18 | 6.0 | 1.2 | 3.2 | 0 | — |
| Any 1v1 champion — 1v1 Energy × 6 Gold | 1v1 HP | 1v1 | 1v1 | 1v1 | 1v1 × 4 | Tank 20 % | M-18 |
| Small monster | 350 | 16 | 1.6 | 1.0 | 3.0 | 0 | passive |
| Big monster | 900 | 35 | 2.0 | 1.2 | 2.8 | 0 | passive |
| Tower Outer / Inner / Base / Nexus | 2500 / 2800 / 3000 / 3500 | 120 | 9.0 | 1.0 | — | 0 | M-08 |
| Stronghold | 6000 | 150 | 10.0 | 1.0 | — | 0 | win condition |

Waves: first at 0:10, then every 30 s. Camps: first at 0:20, respawn 60 s
after cleared. Start Gold 60.

## 7. Commanders

- **Auto** (default, all bots): lane roles follow their lane's most advanced
  allied Minion/Troop, or wait at their outermost Tower; Support joins Bot;
  Jungle clears its own Camps, then helps Mid. Below 25 % HP the Commander
  walks home to heal and returns at 95 %.
- **Move** (right click on ground or minimap): walk there ignoring enemies,
  then **Hold**: fight anything within 7 units of that point.
- **A** returns to Auto. A command given while dead applies after respawn.

## 8. End of match

- **M-30** Destroying the enemy Stronghold wins at once. At 25:00 the
  Team with more Towers standing wins; then higher Stronghold HP; then
  more Commander kills; else a draw. The room starts a new match 10 s later.

## 9. Bots (M-26)

Bots think twice per second. They upgrade when they can; otherwise they buy
the first affordable offer of their role's favourite archetype (Top Tank,
Jungle Dps, Mid Control, Bot Ranged, Support Support), any affordable offer
once Gold piles up (≥ 72), or reroll when nothing fits and Gold ≥ 48. They
send Troops to their role's lane (Jungle: own camps while any is up, else
Mid). Both teams use the same habits; offers differ per seat stream.

## 9a. Minions (new in 5v5)

Every 30 s (first at 0:10) each Stronghold sends a Wave of three melee and
two ranged Minions down each lane. They walk towards the enemy Towers and
Stronghold and attack any enemy Troop, Minion, Commander or structure on the
way (structures in the M-08 order); they never enter the jungle.

## 10. Out of scope until later phases

1v1 champion abilities and Evolutions in 5v5, Commander abilities and items, fog of war, pings/chat, ranked/MMR,
matchmaking across rooms, spectators, reconnect-to-same-seat tokens,
mobile layout, sounds/music, balance.
