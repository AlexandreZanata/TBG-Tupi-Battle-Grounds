# Contracts — TBG Arena 5v5 (MOBA mode)

> **Immutable once merged.** Changing anything below needs an explicit
> owner decision recorded in [DECISIONS.md](../DECISIONS.md), a protocol
> version bump where the wire changes, and the matching contract test
> changed in the same commit. Contract tests are append-only.

## 1. Authority and seats (M-04, M-05, M-41)

1. The server alone runs the simulation for online play and decides combat,
   Gold, Essence, purchases, Upgrade tiers and the outcome.
2. A client only sends `MobaIntention`s. The seat is taken from the
   connection; no frame carries a seat.
3. Bots act through the same `MobaIntention` validation as humans.

## 2. Determinism (M-03)

`tbg-moba-core` keeps the same boundary as `tbg-core`: integer state,
id-ordered units, no clocks, no I/O, no async, no graphics, no unseeded
randomness, no HashMap iteration in the simulation. Same seed and same
intentions per tick ⇒ same `state_hash` every tick.

## 3. Wire protocol v3 (M-40 – M-42)

`MOBA_PROTOCOL_VERSION = 3`. Binary WebSocket frames, postcard encoding of:

```text
ClientMsg = Hello { protocol: u16, name: String (≤ 24 chars, no control chars) }
          | Intention(MobaIntention)
          | Ping(u32)
ServerMsg = Welcome { protocol, seat, seed, tick_hz }
          | World(WorldView)       // one filtered frame per Team each tick (M-52)
          | Private(PrivateView)   // only to its seat
          | Rejected(Reject)
          | Pong(u32)
          | Full
MobaIntention = BuyTroop { slot: u8, to } | Reroll | ToggleLock { slot: u8 } | Upgrade
              | MoveCommander { to: Pos(mm) } | CommanderAuto  // reserved, always rejected (M-65)
PrivateView   = { seat, gold, essence, tier, troops, destination,
                  offers: [UnitKind; 5] (1v1 champions), slot_rules: [u8; 5], locked: u8 }
```

- Protocol v3 restores five offers/masks, slots 0–2 Role-filtered and
  slots 3–4 all Archetypes. It rejects v1/v2 clients.
- Protocol v2 previously removed avatar fields from SeatView and PrivateView;
  SeatView now contains human/kills/deaths. MoveCommander/CommanderAuto,
  UnitKind::Commander and the old avatar Event tags keep their original
  discriminants and are never applied/spawned/emitted (M-65).
- Variant order is part of the wire: **append new variants only**, never
  reorder or remove; a breaking change bumps the version.
- Limits: client frame ≤ 256 bytes, ≤ 30 frames/s, Hello within 5 s, idle
  timeout 30 s; server frame ≤ 256 KiB. Violations close the socket; the
  server never panics on client input.
- Troops are bought **by Shop slot**, never by naming a champion, so a
  client cannot buy outside its own offers (M-19).
- `UnitView` positions are centi-units (`x / 100` = map units), HP is per
  mille, `attacked` marks an attack or heal on that tick.

Territory revision (owner, 2026-11-03): M-67–M-69 change the visibility
contract, not the frame schema. Every surviving structure is still in
every Team World, so conquered Tower circles can be reconstructed from
public Tower sites without transmitting hidden units or new client claims.
Core territory scenarios are in `tests/contract_vision_territory.rs`.

## 4. Map coordinates

Map units: `x` east, `y` north, `(0,0)` south-west corner, size 200. Blue
base south-west, Red north-east; Red = Blue reflected by `(x, y) → (200 − y,
200 − x)`. The client renders map `(x, y, h)` at Bevy `(x − 100, h, 100 − y)`.
`core::map` is the single source of layout for both rules and rendering.

## 5. Contract tests

| Test | Guards |
|---|---|
| `m01_ten_seats_two_teams_five_roles` | M-01, M-02, M-65: ten strategic Seats without avatars |
| `m03_same_seed_and_inputs_replay_identically` | M-03 |
| `m04_bots_never_act_for_human_seats` | M-04 |
| `m05_seat_outside_the_match_is_rejected` | M-05 |
| `m08_inner_tower_is_invulnerable_until_the_outer_falls` | M-08 |
| `m10_buying_spends_the_exact_card_price_of_a_1v1_champion` | M-10, M-18 |
| `m11_troop_cap_limits_alive_troops_per_player` | M-11 |
| `m17_upgrade_spends_essence_and_caps_at_max_tier` | M-17 |
| `m19_shop_offers_follow_the_1v1_rules_with_locks_and_reroll` | M-19 |
| `m20_role_slots_only_offer_the_role_archetypes` | M-19/M-20 |
| `m20_five_cards_keep_three_role_slots_and_two_full_pool_slots` | M-19/M-20 five-slot revision |
| `m60_each_camp_spawns_its_kind` | M-60 |
| `m61_monsters_stay_passive_until_hit` | M-61 |
| `m62_leashed_monsters_heal_home` | M-62 |
| `m63_cleared_camp_respawns_after_its_timer` | M-63 |
| `m50_enemies_outside_allied_sight_are_absent_from_that_team_frame` | M-50/M-54 |
| `m51_structures_are_always_in_every_frame` | M-51 |
| `m52_each_socket_receives_its_team_frame` (server) | M-52 |
| `m53_vision_is_deterministic` | M-53 |
| `m67_home_half_is_fully_visible_without_scouts_or_standing_home_towers` | M-50/M-67 |
| `m68_only_allied_troops_across_the_river_reveal_enemy_ground` | M-50/M-68 |
| `m69_destroyed_enemy_tower_reveals_only_its_sight_circle_until_match_end` | M-51/M-69 |
| `m65_seats_have_no_avatar_and_manual_control_is_rejected` | M-01/M-65 |
| `m65_each_purchase_chooses_its_lane_then_walks_automatically` | M-10/M-65 |
| `m14_automatic_champion_kills_keep_card_bounty_and_update_score` | M-14/M-65 |
| `m65_socket_cannot_send_manual_unit_orders` (server) | M-41/M-65 |
| `m30_a_ten_bot_match_always_ends` | M-30 |
| `m40_frames_round_trip_and_hostile_frames_are_refused` | M-40 |
| `m41_seats_come_from_connections_and_purchases_are_authoritative` (server) | M-05, M-41 |
| `m42_hostile_frames_close_the_socket` (server) | M-42 |

Queue focused commands from the P77 report at consolidation, under the
shared validation lock. Full-workspace tests require an explicit owner request.

## 6. Isolation and the shared champions (D-112, amended 2026-11-03)

- The 1v1 crates never depend on `moba/`, and MOBA work never modifies the
  root workspace, its lock file or any 1v1 release input.
- The MOBA shares the 1v1 champions **read-only**: `tbg-moba-core` and
  `tbg-moba-client` depend on `crates/tbg-core` (roster, stats, costs,
  pool), and the client compiles the 1v1 figure sources in place
  (`crates/tbg-client/src/art`, `vfx/impacts.rs`, `vfx/shots.rs`) and loads
  the 1v1 assets (`battle/`, `lobby/native-icons/`). A 1v1 change to those
  files therefore changes the MOBA too; the MOBA workflow runs on them.
- `UnitKind::Troop` carries a `tbg_core::UnitKind`; a champion's
  `Debug` name lowercased is its portrait file, as in the 1v1 Shop.
