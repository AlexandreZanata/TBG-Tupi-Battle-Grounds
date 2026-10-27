# Networking — TBG 2.0

> Planned client/server communication. Design only; no implementation in
> Phase 00 beyond versioned placeholder types in `tbg-protocol`.

## Transport

- WebSocket between browser (`tbg-client`) and `tbg-server`, JSON messages
  in `tbg-protocol` (binary encoding deferred until profiling demands it).
- Every message carries `PROTOCOL_VERSION`; mismatched versions are
  rejected with a typed error, never silently interpreted.
- TLS terminates at the reverse proxy (see `INFRASTRUCTURE.md`); the game
  server speaks plain WebSocket behind it.

## Authority model

- The server runs the fixed-tick deterministic simulation (`tbg-core`).
- The client submits `PlayerIntention`s: purchase, evolve, reposition
  (rule BR-A01). Each intention is validated against energy, shop,
  evolution, and match state (BR-A02).
- The client may predict for responsiveness and reconciles to
  authoritative snapshots on rejection (BR-A03).
- Damage, balances, and match results from the client are never trusted
  (BR-A04).

## Planned message shape (illustrative, not implemented)

```rust
// Upstream: client -> server
PlayerIntention { match_id, actor, action: Purchase { shop_slot } | Evolve { unit_id, branch } | Reposition { unit_id, slot } }
// Downstream: server -> client
Snapshot { tick, units, energy, shop, timer } | Rejection { intention_id, reason } | MatchEnd { outcome }
```

Exact schemas are a Phase 04 deliverable in `tbg-protocol`.

## Tick and synchronization (proposed, not finalized)

- Legacy proposal: 10–20 ticks/second server tick (ADR-001 follow-up).
  Final value requires a latency-budget spike in Phase 04.
- Clients interpolate between snapshots; no lockstep requirement on the
  browser.
- Reconnection: a disconnected player gets a reconnect window, then
  timeout/forfeit per documented policy (BR-M03). Snapshot resync on
  rejoin is mandatory; full match replay is not.

## Security requirements (carried from legacy S3)

- Rate-limit public endpoints (login, matchmaking) at the proxy.
- Authenticate every connection before admitting intentions.
- Server-side validation of every action; no secret logic or hidden
  constants trusted from the client.
- No gameplay-affecting data in cosmetic/inventory paths.

## Phase 04 acceptance (for reference)

Local bot matches (Phase 03) must already run the tick loop against
`tbg-core` in-process. Phase 04 wires that loop to WebSocket transport
with two real browsers, reconnect handling, and version-mismatch
rejection — none of which exists yet.
