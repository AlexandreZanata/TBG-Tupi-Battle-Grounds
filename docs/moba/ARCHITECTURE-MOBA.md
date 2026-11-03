# Architecture — TBG Arena 5v5 (MOBA mode)

> How the 5v5 mode is built and why. Contracts that must not change are in
> [CONTRACTS.md](CONTRACTS.md); rules and numbers in
> [GAME-DESIGN-MOBA.md](GAME-DESIGN-MOBA.md). Decision: D-112.

## 1. Isolation from the 1v1 game

`moba/` is a **separate Cargo workspace** (`moba/Cargo.toml`, `moba/Cargo.lock`).

- The 1v1 crates never depend on `moba/*`. The MOBA reads the 1v1
  champions (see §5a) but never edits a 1v1 file.
- The root `Cargo.toml`/`Cargo.lock` are untouched, so the 1v1 desktop
  release policy (`scripts/native_release_policy.py`) never sees a MOBA
  change and the 1v1 packages never rebuild because of it.
- Its own CI workflow (`.github/workflows/moba.yml`, path-filtered to `moba/**`).
- Its own installer and binaries (`tbg-moba`, `tbg-moba-server`).
- Shared by copy: the two OFL UI fonts (`client-moba/assets/fonts`). Shared
  in place: `tbg-core`, the 1v1 champion art sources and the 1v1 battle
  assets (§5a); the installer copies `battle/` and `lobby/native-icons/`
  beside the binary. The lock file started as a copy of the
  root lock so dependency versions match the audited 1v1 set.

Integrating the mode into the 1v1 launcher or server later is a separate,
owner-approved phase (see the P76 backlog).

## 2. Crates

| Crate | Path | Responsibility | Must never |
|---|---|---|---|
| `tbg-moba-core` | `moba/core-moba` | Deterministic rules, map, content, bots, views, wire protocol | I/O, async, clocks, graphics, floats in state, unseeded RNG, HashMap iteration |
| `tbg-moba-server` | `moba/server-moba` | Authoritative room: seats, ticking, fan-out, limits | depend on Bevy; trust client seats or outcomes |
| `tbg-moba-client` | `moba/client-moba` | Rendering, HUD, input, offline host, socket | decide online outcomes |

### Core modules

`types` (glossary), `geom` (mm fixed point, `isqrt` movement), `map`
(single source of the layout for rules *and* rendering), `content`
(Proposed numbers), `state` (match, units, players, events), `intent`
(validation), `sim/*` (one tick: `upkeep` → `spawn` → grid → `combat` →
`movement` → `rewards` → `outcome`), `grid` (uniform spatial hash),
`bot`, `host::tick` (the loop every host runs), `view`, `protocol`.

## 3. Simulation model

- Fixed 20 Hz tick; positions in millimetres (`i32`), map 200 000 mm square.
- Units live in one `Vec` sorted by id (ids only grow; `retain` keeps order),
  so every pass is deterministic and lookups are binary searches.
- A uniform 8-unit grid is rebuilt each tick (allocations reused); target,
  heal and aura queries scan only nearby cells.
- Combat is decided on start-of-tick state and applied simultaneously.
- Measured (opt-level 1 probe, 10 bots, full 25-minute matches): ~28 µs per
  tick at ~170–210 live units — under 0.1 % of the 50 ms budget.

## 4. Network model

- One WebSocket per player at `/moba/ws`; binary postcard frames.
- Handshake: client `Hello{protocol}` within 5 s → server `Welcome{seat}`.
- Per tick the room encodes **one** `World` frame (≈3 KB at peak, all units,
  quantised to centi-units and HP per mille) and shares the same bytes with
  every socket, plus a small `Private` frame (gold, essence, tier) per seat.
- Each socket has a bounded outbox (64 frames). `try_send` drops a frame for
  a slow socket only; the tick loop never awaits a client. `TCP_NODELAY` is on.
- Inbound frames are capped at 256 bytes and 30 per second; malformed frames,
  text frames, oversize frames and 30 s of silence close the socket.
- Humans take the lowest bot seat on the team with fewer humans; on leave a
  bot resumes the seat mid-match. A full room answers `Full`.
- Bandwidth at peak ≈ 3 KB × 20 Hz ≈ 60 KB/s per player. The upgrade path
  (ponytail) is delta frames plus interest management.

## 5. Client

- Bevy 0.20, same explicit feature set as the 1v1 client minus audio/glTF.
- `link`: one `Feed` (two latest worlds, private view, events, status) fed by
  either `local` (in-process `MobaMatch` + `host::tick`, offline practice) or
  `online` (`ewebsock`). Everything else reads only the `Feed`.
- `scene`: the static valley generated once from `core::map` — terrain
  relief and colour, cobbled lanes, translucent river and bridges, base
  plateaus with walls and banners, the plaza crystal, camps, forest chunks
  merged into a few vertex-coloured meshes (one shared material).
- `units`: one entity per snapshot id, interpolated between the two latest
  worlds, facing movement or target, walk bob, attack lunge; billboard
  health bars; minimap icons on render layer 2; projectiles, sparks, death
  bursts and move markers in `units/fx`.
- `camera`: RTS rig (arrows/edge pan, middle drag, wheel zoom, Space to
  centre) and an orthographic minimap camera drawn into the HUD frame's
  viewport.
- `hud`: 1920×1080 design grid scaled by window height, in the 1v1 battle
  layout (score plates and round clock on top; resource strip and five
  Shop cards at the bottom; Destination and player card beside them).
- `input`: manual UI hit-testing (no picking backend), hotkeys, right-click
  orders with ground ray casts.
- `--capture PATH [--warp S] [--overview] [--seed N] [--role R]` makes the
  Linux reference screenshots used in reports.

## 5a. Shared 1v1 champions

Both modes share one roster (owner request, 2026-11-03):

- **Rules:** `tbg-moba-core` turns each `tbg_core::UnitKind` into a MOBA
  Troop from its 1v1 `UnitDef` (HP, damage, interval, range, speed × 4,
  heal, splash, Energy cost × 6 Gold) and draws Shop offers from
  `tbg_core::types::FULL_POOL` with the 1v1 Shop rules.
- **Figures:** `client-moba/src/main.rs` compiles
  `crates/tbg-client/src/art/mod.rs` as its own `art` module (`#[path]`),
  plus the 1v1 `vfx/impacts.rs` and `vfx/shots.rs` under a small runtime in
  `client-moba/src/vfx/mod.rs` (a verbatim copy of the 1v1 `vfx.rs` asset
  table, motions and spawn helpers). `units/champions.rs` spawns each
  champion with the 1v1 rig and `Animator` (Spawn, Walk, AttackMelee/
  AttackRanged/Cast, Hit, Death clips) and fires the kit's shot on Release
  or lands its impact on Strike. Commanders are their role's champion at a
  larger scale.
- **HUD:** cards, resource strip, score plates, crests, locks and icons use
  the 1v1 assets and a copy of the 1v1 battle chrome helpers
  (`hud/chrome.rs`).
- ponytail: path-shared sources stand in for a shared `tbg-art` crate,
  which would need a 1v1 release (owner decision). Upgrade path: move
  `art/`, `vfx/` and the chrome helpers into a crate both clients depend on.

## 6. Running

```bash
# offline practice against nine bots
tbg-moba
# local room server, then join it (bots fill every other seat)
tbg-moba-server --bind 127.0.0.1:7878
tbg-moba --online ws://127.0.0.1:7878/moba/ws
```

Install or update the Linux build with `moba/scripts/install-linux-moba.sh`
(`--release` for an optimised build).

## 7. Deploy and scaling (future phases)

One room per server process is enough for the skeleton. Scaling path:
several rooms per process (one task each), a lobby/matchmaking endpoint,
reconnect tokens, TLS behind the existing Caddy, and per-room metrics. The
VPS is not touched until the owner asks.
