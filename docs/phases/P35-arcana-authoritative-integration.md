# P35 — Authoritative Arcanas and versioned match records

**Depends on:** P34 reviewed and its balance/owner gate passed.
**Branch:** `phase/p35-arcana-online`.
**Status:** planned; development/test integration, no public deployment.
**Difficulty:** medium–high (state transitions and reconnect security).

## Goal and rules

Select validated presets before matchmaking, freeze them for the match,
and record what the server applied. BR-AR05–BR-AR09 govern authority,
history and identical Casual/Ranked rules. Keep existing identity and Elo.

## Read first (only these)

- `AGENTS.md`; GAME-DESIGN §3 authority/accounts/ranked, §9; DECISIONS D-25.
- P32–P34 reports; ARCHITECTURE §3–§5, §8, §11 and Arcanas addendum.
- Core: arcanas/content/state/sim/protocol/view.
- Server: ws admission/client routing, chan, lobby/session/matchmaking,
  match_task/views/report, store schema/matches/ranked and online tests.
- Client: net/link/local/sync message matches and botclient handlers.

## Affected files and limits

The modules above plus `tests/arcanas.rs` in the server and core protocol
tests. Add small `lobby/arcanas.rs` or match/store helpers where needed.
Update all exhaustive message consumers, including tbg-bots and practice.
No server-side game formulas, client-selected final stats, new dependency,
SQL on the match loop, protocol rates/size changes or public activation.
P37 owns account preset persistence; this phase persists match metadata.

## Wire/state contract

Use these payloads, preserving other current message fields:

```rust
pub type RulesetVersion = u16;
pub struct MatchRules {
    pub ruleset_version: RulesetVersion,
    pub arcana_schema: u8,
    pub presets: [ArcanaPreset; 2], // indexed by Seat, applied selections
}
pub enum ArcanaRejection { BadRanks, BadSchema, BadRuleset, NotIdle }
ClientMsg::Hello { version: u16, ruleset: RulesetVersion, resume: Option<Token> }
ClientMsg::SetArcanas { schema: u8, ruleset: RulesetVersion, ranks: [u8; 6] }
ServerMsg::ArcanasSelected { schema: u8, ruleset: RulesetVersion, ranks: [u8; 6] }
ServerMsg::ArcanasRejected { reason: ArcanaRejection }
ServerMsg::RulesetMismatch { server: RulesetVersion }
ServerMsg::MatchStart { /* retain current fields */, rules: MatchRules }
```

Define compiled ARCANA_SCHEMA_VERSION = 1, and RULESET_VERSION = 1 only
if no competitive version already exists; otherwise advance that counter.
Bump the actual current PROTOCOL_VERSION once (do not assume it is still
3 after P31). Ruleset changes whenever competitive content/rules change;
schema changes for rank meaning/layout; wire version changes for layout.
Never reuse an identifier for different rules. Append enum variants.
Validate raw ranks with core construction and validate_complete; decode
must never panic. Keep the existing 1 KiB limit and connection rate limits.

## Steps (in order; check and commit each)

### 1. Versioned handshake and consumers

Reject incompatible protocol/ruleset before joining a queue or room; client
shows a reload/update message. Old layouts may fail bounded decoding;
they must fail safely, never resume under an incompatible ruleset. Update
admission tests, botclient and all client message patterns in the same step.
Send MatchRules at initial MatchStart and every reattach, not each snapshot.
Presets are public once paired; never reveal seed, offers or enemy Energy.
**Check:** current handshake succeeds; mismatched versions, malformed,
trailing and oversized messages are rejected without panic or state change.

### 2. Idle-only server selection

Initialize each session with the complete default; do not require account
age, playtime or manual setup. A SetArcanas response is authoritative.
Validate schema, ruleset, ranks and complete budget; an invalid request
preserves the previous preset. Map validation failures to BadRanks.
Accept only in Idle. Queued, InRoom and InMatch reply NotIdle; route this
message before existing early-return filters so it is not silently ignored.
UI waits for ArcanasSelected before queueing. Ordered connection messages
must reach the same lobby owner in order. No seat/account ID in this request.
**Check:** invalid ranks/budgets/versions and edits in each non-idle state
are rejected; previous values and opponent selection are unchanged.

### 3. Freeze and construct the authoritative match

On QuickMatch/RankedMatch/CreateRoom/JoinRoom, validate the complete
selection, freeze it with the waiting session and reject further edits.
Pairing copies each frozen preset into Player and calls the explicit
new_with_arcanas constructor. Do not read mutable preferences after pairing.
Cancel unlocks editing. Existing account/guest Ranked eligibility is intact.
Token/connection ownership stamps the seat; final attributes come from core.
**Check:** queued owner plus room joiner, both FIFO queues and private rooms
apply the correct independent presets, with unchanged matching rules.

### 4. Reconnect, takeover and local adapters

Reattach sends the original MatchRules; neither login, cached preferences,
new connection nor SetArcanas can replace active presets. Keep existing
reconnect timing/forfeit behavior. Patch LocalMatch's message construction
to the new shape while retaining its neutral behavior until P36.
**Check:** two clients finish a match including reconnect; repeat account
takeover, stale-connection edit and opposite-seat attempts. Assert frozen
metadata and applied HP/damage through to the final result.

### 5. Versioned result metadata and migration

Use the next unused SQLite migration number observed at execution time;
P13–P31 may have added migrations. Add nullable `ruleset_version INTEGER`,
`arcana_schema INTEGER`, `a_arcanas BLOB`, `b_arcanas BLOB` to both matches
and ranked_matches. Each non-null ranks BLOB is exactly six bytes and is
validated on read; encode no preset names or client attributes. Log the
same frozen metadata and source build revision with end-of-match telemetry;
the release report maps every ruleset to its source revision/artifact.
Historical rows remain NULL/legacy, not falsely tagged as the new rules.

Pass metadata by value from the match task to store commands. The ranked
settlement transaction inserts its metadata with both ratings and outcome;
generic match insertion preserves it separately. Preserve P13 queries and
legacy field meanings. No historical outcome/Rating update or destructive
migration. A generic fire-and-forget record failure must not silently turn
ranked settlement into an unversioned result; surface the failure.
**Check:** migrate an old database twice; old rows/ratings are unchanged;
new Casual/Ranked records contain the applied presets and versions;
Ranked rollback leaves no partial ratings/metadata on injected failure.

### 6. Measure wire and memory cost

Encode real MatchStart and steady-state views before/after. Static rules
and ranks should add roughly 14–16 bytes of payload for small versions,
not a new per-unit or per-tick field. Measure rather than assert that estimate.
Do not add derived damage/healing arrays to MatchView. P32-approved combat
status fields, if any, are a separate baseline, not hidden Arcana overhead.
**Check:** view payload bytes are unchanged by Arcana metadata for an
identical state/event trace; every inbound message remains within 1 KiB.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

Also run `trunk build --release` in crates/tbg-client; inspect the current
page for protocol/regression errors, and run server Arcana/online/account/
ranked integration tests. Observe a real two-client match with reconnect.

## Acceptance and reviewer checklist

- [ ] Rank/version/state validation and seat ownership are server-side.
- [ ] Every match path freezes a complete preset; reconnect cannot change it.
- [ ] Practice/bot adapters compile; no UI editor is required yet.
- [ ] Match metadata and ranked ratings are transactionally coherent;
      legacy rows and P13 queries still work.
- [ ] Actual bytes/memory measured; no new tick-time allocation or database I/O.
- [ ] Checks and reviewer pass; do not deploy or launch the extra before P38.

## Phase report

_Not executed. Fill in HEAD, chosen version/migration numbers, exact commands,
security/state-transition results, reconnect/takeover observation, database
before/after and rollback results, packet/memory deltas, deviations/blockers
and reviewer decision. Never include tokens, AccountKeys or host secrets._
