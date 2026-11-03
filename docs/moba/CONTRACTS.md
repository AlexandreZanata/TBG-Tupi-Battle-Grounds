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
MobaIntention = BuyTroop { troop, to } | Upgrade | MoveCommander { to: Pos(mm) } | CommanderAuto
```

- Variant order is part of the wire: **append new variants only**, never
  reorder or remove; a breaking change bumps the version.
- Limits: client frame ≤ 256 bytes, ≤ 30 frames/s, Hello within 5 s, idle
  timeout 30 s; server frame ≤ 256 KiB. Violations close the socket; the
  server never panics on client input.
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
| `m10_buying_spends_the_exact_card_price` | M-10 |
| `m11_troop_cap_limits_alive_troops_per_player` | M-11 |
| `m17_upgrade_spends_essence_and_caps_at_max_tier` | M-17 |
| `m30_a_ten_bot_match_always_ends` | M-30 |
| `m40_frames_round_trip_and_hostile_frames_are_refused` | M-40 |
| `m41_seats_come_from_connections_and_purchases_are_authoritative` (server) | M-05, M-41 |
| `m42_hostile_frames_close_the_socket` (server) | M-42 |

Run them with `cd moba && cargo test --locked --workspace`.

## 6. Isolation (D-112)

`moba/` never depends on `crates/*` and vice versa; the root workspace,
lock file and 1v1 release inputs are not modified by MOBA work.
