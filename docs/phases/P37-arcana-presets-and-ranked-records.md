# P37 — Account presets, identity and ranked audit integration

**Depends on:** P36 reviewed and editor look approved; P35 metadata tests pass.
**Branch:** `phase/p37-arcana-presets`.
**Status:** planned; no public deployment or new rating system.
**Difficulty:** medium–high (asynchronous account/session races).

## Goal and rules

Recover one active preset across devices for accounts, keep guests equally
powerful, and prove that stored preferences cannot alter a running match.
Use BR-AR01/05–BR-AR09 and existing BR-P03/04, BR-R03–BR-R06.
Elo and FIFO already exist: extend their audit data, never rebuild them.

## Read first (only these)

- `AGENTS.md`; GAME-DESIGN §3 account/ranked rules and §9.
- P35/P36 reports and interfaces; ARCHITECTURE §5/§8 and Arcanas addendum.
- Core protocol/arcanas/rating; server store schema/accounts/ranked,
  lobby accounts/account_done/sessions/arcanas, match report and ownership.
- Client account/selection/pending UI; existing account/ranked/Arcana tests;
  P13 profile/history queries, only where new metadata is read.

## Affected files and exclusions

Add `store/arcanas.rs`, extend store commands/migrations, account completion
routing and client selection UI. Add focused preset persistence/race tests;
split tests/helpers within AGENTS.md limits. Match metadata remains P35's
contract. No new account identifier, email/OAuth, distributed cache,
PostgreSQL, rating formula, matchmaking window, unlock system, preset names
or multiple saved slots. Never log AccountKeys/tokens or accept an account ID.

## Persistence and message contracts

Use the next unused migration number and the existing account primary key:

```sql
CREATE TABLE account_arcanas (
    account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
    schema_version INTEGER NOT NULL,
    ruleset_version INTEGER NOT NULL,
    ranks BLOB NOT NULL CHECK(length(ranks) = 6),
    updated_at TEXT NOT NULL
);
```

SQL length is defense in depth; core validation still checks every rank
and complete budget on write and read. Use parameterized statements.
Add store `LoadArcanas` and `SaveArcanas` commands with bounded channels and
oneshot replies, handled on the existing writer thread. Store account ID
comes from the authenticated session, not any client payload.

```rust
ClientMsg::SaveArcanas { schema: u8, ruleset: RulesetVersion, ranks: [u8; 6] }
ServerMsg::ArcanasSaved { schema: u8, ruleset: RulesetVersion, ranks: [u8; 6] }
ServerMsg::ArcanasLoaded {
    schema: u8, ruleset: RulesetVersion, ranks: [u8; 6], changed: bool,
}
ServerMsg::ArcanaSaveFailed
ArcanaRejection::Busy
```

Bump the actual wire version for these additions. Invalid selection/state
uses P35 ArcanasRejected; unauthenticated SaveArcanas uses LoginRequired;
storage failure uses ArcanaSaveFailed. ArcanasSaved means the write
committed and that idle session adopted it. SetArcanas remains selection
only; it must never falsely report a durable save.
While a preference operation is pending, conflicting selection/save/queue/
room requests receive ArcanasRejected { reason: Busy } without being queued.

## Steps (in order; check and commit each)

### 1. Migrate and round-trip one preset

Missing row means the complete default; it does not limit available nodes.
Compatible schema with changed numeric balance retains complete validated
ranks but returns current ruleset and changed=true so the UI explains the
updated effects. Unknown schema, corrupt rank bytes or invalid budget use
the default with changed=true and a visible review/reset message. Preserve
the old row until an explicit successful save; never silently reinterpret
different node meanings or overwrite historical match presets.
**Check:** old DB migration twice is safe; valid, absent, changed-ruleset,
unknown-schema and corrupt rows load deterministically; save/reopen round-trip.

### 2. Account load without blocking the lobby

Load after register/login and session ownership is established, using the
existing completion pattern. Tag requests with token, connection ID and
the account identity/generation captured at dispatch. Ignore stale replies
after logout, account switch, takeover or disconnection; never deliver one
account's preference to another. While the idle account load is pending,
reject competing selection/save/queue requests with a recoverable busy
response until completion or explicit failure; no deferred command queue
and no await in the lobby. Do not race a stored preset against a new one.

Takeover of a live match must attach immediately using its frozen MatchRules;
load preferences only for the next match and never replace that state.
If a reply arrives after queue/room entry, keep the frozen preset; apply
preferences only once idle again. An unavailable store gives a recoverable
failure; do not fabricate a successful cloud load/save.
**Check:** test every late-reply transition with controlled completions,
including login on a second device during a running match.

### 3. Save acknowledgement and selection ownership

For logged-in users, Apply sends SaveArcanas; guests use SetArcanas plus
validated local storage. At most one pending preference operation per
session. Refuse non-idle saves before dispatch; reject conflicting
queue/room commands until the operation resolves. Revalidate identity and
idle ownership on completion before adopting its selection. A committed
preference for the old account can remain durable, but cannot mutate a
different session/account or a newly frozen match.
Reply ArcanasSaved only after commit; UI distinguishes saved, pending,
rejected and failed. Storage failure retains the prior accepted preset and
offers retry; no silent queue with an unintended draft. Avoid retrying a
completed account operation automatically through an old connection.
**Check:** successful second-device recovery, failed commit, duplicate taps,
logout/switch/takeover during save and queue-after-save preserve ownership.

### 4. Ranked audit and transaction regression

Verify P35's applied metadata reaches ranked_matches in the same transaction
as both Rating changes, including forfeit and draw. The record uses the
match's frozen ranks, not the latest account preference at settlement.
P13 history can display an optional ruleset/preset detail; legacy NULL rows
remain readable. Do not change existing profile response semantics, Elo K,
floors, FIFO queues or public personal data. Static preference/stat history
is not a replay: timed intentions and old rules implementations are absent.
**Check:** change an account's next-match preset after match end but before
delayed settlement; the original match record remains unchanged. Injected
transaction failure leaves neither partial Rating updates nor metadata.

### 5. Parity and end-to-end observation

Compare new guest, new account and established account with identical ranks:
same stats, legal choices and simulation outcomes. No purchase, level,
inventory or Rating participates in modifier calculation.
On two devices: save → reconnect/login → observe preset → finish Ranked
with a reconnect → see matching record and RatingChanged. Repeat Casual
and verify no rating change. Exclude keys from screenshots/artifacts.
**Check:** one complete observed flow plus automated account/seat isolation
and corrupted-data tests, not just database unit tests.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

Also build Trunk release, inspect the browser save/failure states and run
the server account/ranked/Arcana suites. Database calls never run in a tick
or block the lobby. Preserve frame size/rate limits and bounded queues.

## Acceptance and reviewer checklist

- [ ] One preset survives restart/login and is isolated by real account ID.
- [ ] Guest/default availability is identical; partial presets never play online.
- [ ] Save confirmation means durable commit, with recoverable failure states.
- [ ] Stale replies cannot change account, queue, room or live-match selection.
- [ ] Ranked results atomically retain their original versions/presets and Elo.
- [ ] Legacy data/query behavior and real reconnect/takeover remain intact.
- [ ] Checks and reviewer pass; only P38 can approve public activation.

## Phase report

_Not executed. Fill in migration/wire versions, commands, old/corrupt DB
results, asynchronous race matrix, guest parity, two-device observations,
ranked rollback/late-settlement evidence, latency/queue observations,
deviations, blockers and reviewer decision._
