# Contracts — TBG Arena 5v5 (MOBA mode)

> **Immutable once merged.** Changing anything below needs an explicit
> owner decision recorded in [DECISIONS.md](../DECISIONS.md), a protocol
> version bump where the wire changes, and the matching contract test
> changed in the same commit. Contract tests are append-only.

## 1. Authority and seats (M-04, M-05, M-41)

1. The server alone runs the simulation for online play and decides combat,
   Gold, Essence, purchases, levels and the outcome.
2. A client only sends `MobaIntention`s. The seat is taken from the
   connection; no frame carries a seat.
3. Bots act through the same `MobaIntention` validation as humans.

## 2. Determinism (M-03)

`tbg-moba-core` keeps the same boundary as `tbg-core`: integer state,
id-ordered units, no clocks, no I/O, no async, no graphics, no unseeded
randomness, no HashMap iteration in the simulation. Same seed and same
intentions per tick ⇒ same `state_hash` every tick.

## 3. Wire protocol v1 (M-40 – M-42)

`MOBA_PROTOCOL_VERSION = 1`. Binary WebSocket frames, postcard encoding of:

```text
ClientMsg = Hello { protocol: u16, name: String (≤ 24 chars, no control chars) }
          | Intention(MobaIntention)
          | Ping(u32)
ServerMsg = Welcome { protocol, seat, seed, tick_hz }
          | World(WorldView)       // identical bytes for every socket each tick
          | Private(PrivateView)   // only to its seat
          | Rejected(Reject)
          | Pong(u32)
          | Full
MobaIntention = BuyTroop { slot: u8, to } | Upgrade | MoveCommander { to: Pos(mm) } | CommanderAuto
              | Reroll | ToggleLock { slot: u8 }
PrivateView   = { seat, gold, essence, tier, troops, destination, commander_auto,
                  offers: [UnitKind; 3] (1v1 champions), slot_rules: [u8; 3], locked: u8 }
```

- Variant order is part of the wire: **append new variants only**, never
  reorder or remove; a breaking change bumps the version.
- Limits: client frame ≤ 256 bytes, ≤ 30 frames/s, Hello within 5 s, idle
  timeout 30 s; server frame ≤ 256 KiB. Violations close the socket; the
  server never panics on client input.
- Troops are bought **by Shop slot**, never by naming a champion, so a
  client cannot buy outside its own offers (M-19).
- `UnitView` positions are centi-units (`x / 100` = map units), HP is per
  mille, `attacked` marks an attack or heal on that tick.

## 4. Map coordinates

Map units: `x` east, `y` north, `(0,0)` south-west corner, size 200. Blue
base south-west, Red north-east; Red = Blue reflected by `(x, y) → (200 − y,
200 − x)`. The client renders map `(x, y, h)` at Bevy `(x − 100, h, 100 − y)`.
`core::map` is the single source of layout for both rules and rendering.

## 5. Contract tests

| Test | Guards |
|---|---|
| `m01_ten_seats_two_teams_five_roles` | M-01, M-02 structure |
| `m03_same_seed_and_inputs_replay_identically` | M-03 |
| `m04_bots_never_act_for_human_seats` | M-04 |
| `m05_seat_outside_the_match_is_rejected` | M-05 |
| `m08_inner_tower_is_invulnerable_until_the_outer_falls` | M-08 |
| `m10_buying_spends_the_exact_card_price_of_a_1v1_champion` | M-10, M-18 |
| `m11_troop_cap_limits_alive_troops_per_player` | M-11 |
| `m17_upgrade_spends_essence_and_caps_at_max_tier` | M-17 |
| `m19_shop_offers_follow_the_1v1_rules_with_locks_and_reroll` | M-19 |
| `m20_role_slots_only_offer_the_role_archetypes` | M-19/M-20 |
| `m30_a_ten_bot_match_always_ends` | M-30 |
| `m40_frames_round_trip_and_hostile_frames_are_refused` | M-40 |
| `m41_seats_come_from_connections_and_purchases_are_authoritative` (server) | M-05, M-41 |
| `m42_hostile_frames_close_the_socket` (server) | M-42 |

Run them with `cd moba && cargo test --locked --workspace`.

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
