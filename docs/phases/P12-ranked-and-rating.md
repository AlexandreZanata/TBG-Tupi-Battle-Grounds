# P12 — Ranked mode, separate queues, and rating

**Depends on:** P11 reviewed and merged.
**Branch:** `phase/p12-ranked-and-rating`.
**Difficulty:** medium. The rating math is small; the care is in doing it
once, atomically, on the server.

## Goal

Competitive play gets its own section: a **Ranked** queue for accounts,
next to the existing **Casual** play (quick match, private rooms,
practice). Every ranked win, draw, or loss changes both players' rating by
an Elo update that depends on **both** ratings. Pairing stays FIFO and
ignores rating (few players at launch).

## Deliverable

- `tbg-core/src/rating.rs` (BR-R03, BR-R05) with tests and a convergence
  simulation.
- Ranked queue, ranked match records, atomic rating updates in the store.
- Protocol v3: `RankedMatch`, extended `MatchStart`, `RatingChanged`.
- Client menu with **Casual** and **Ranked** sections; ranked badge in the
  HUD; rating change on the result screen.

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §3 rules BR-R01–BR-R07 and §5 "Ranked rating"
- `docs/DECISIONS.md` D-18
- Core: `src/protocol.rs`, `src/account.rs`, `src/state.rs` (`Outcome`, `EndReason`), `src/content.rs` (unit → faction)
- Server: `src/lobby.rs` + `src/lobby/*`, `src/match_task.rs`, `src/match_task/report.rs`, `src/store/*`
- Client: `src/ui/menu.rs`, `src/ui/result.rs`, `src/ui/hud.rs`, `src/sync/messages.rs`

## Do NOT

- Pair by rating, add rating windows, seasons, decay, or placement
  matches beyond the K rule below.
- Let a guest enter the ranked queue, or change rating anywhere but the
  store transaction.
- Change match rules: ranked and casual matches are the same simulation.

## Rating (implement exactly; values are GAME-DESIGN §5 "Ranked rating")

```rust
// tbg-core/src/rating.rs — not simulation state; f64 is fine here.
pub const START_RATING: u16 = 1000;
pub const RATING_FLOOR: u16 = 100;
pub const PLACEMENT_GAMES: u32 = 20;
pub const K_PLACEMENT: f64 = 40.0;
pub const K_STANDARD: f64 = 24.0;
pub enum Score { Win, Draw, Loss }            // 1.0 / 0.5 / 0.0
pub fn expected(own: u16, opp: u16) -> f64 {  // 1 / (1 + 10^((opp - own) / 400))
pub fn delta(own: u16, opp: u16, own_games_before: u32, score: Score) -> i32
//   k = K_PLACEMENT if own_games_before < PLACEMENT_GAMES else K_STANDARD
//   d = (k * (s - expected(own, opp))).round() as i32
//   a win is at least +1, a loss at most -1 (a decisive result always moves)
pub fn apply(rating: u16, delta: i32) -> u16  // max(RATING_FLOOR), min(u16::MAX)
```
Each player's delta uses **their own** K, so a placement player moves faster
without inflating a settled player.

## Steps

### 1. Core rating + tests
`rating.rs` as above, exported from `lib.rs`. Tests (cite BR-R03/BR-R05):
equal 1000 vs 1000 win → +20 (placement) and +12 (standard); 1400 beats
1000 with standard K → +2 (expected ≈ 0.909, 24 × 0.091 = 2.2); 2000 beats
1000 with standard K → +1 (0.08 rounds to 0, raised by the ≥ +1 rule);
the 1000 player losing to 1400 loses less than it would to an equal; draw
between 1200 and 1000 moves the 1200 player down and the 1000 player up;
floor holds at 100.

### 2. Convergence simulation
`tbg-core/examples/rating_sim.rs`: 60 players with hidden skill 0..59; win
probability `1 / (1 + 10^((skill_b - skill_a) * 12 / 400))`, 3 % draws;
FIFO pairing of random arrivals; 6 000 matches; seeded PCG32. Print mean
rating, min/max, games per player, and the Spearman rank correlation
between skill and final rating.
- **Done when** the correlation is ≥ 0.85 and the mean stays within
  1000 ± 30. Put the output in the report.

### 3. Protocol v3
`PROTOCOL_VERSION = 3`; append:
```rust
pub enum MatchMode { Casual, Ranked }
ClientMsg::RankedMatch
ServerMsg::RatingChanged { before: u16, after: u16, profile: Profile }
ErrorCode::LoginRequired
```
and change `ServerMsg::MatchStart` to
`MatchStart { you: Seat, opponent: String, mode: MatchMode, opponent_rating: Option<u16> }`
(`opponent_rating` is `Some` only in ranked).

### 4. Ranked queue (lobby)
- Second FIFO queue `ranked`. `RankedMatch` from a guest → `Error{LoginRequired}`;
  from an account that is `Idle` → enqueue, reply `Queued`. `Cancel` leaves
  either queue. Pairing: the two longest-waiting entries of the same queue
  (BR-R02); the queues never mix; private rooms are always `Casual`.
- The match task receives `mode` and, for ranked, both `AccountId`s.
- BR-R04: a forfeit (including a disconnect past the window) is a loss for
  the forfeiting seat and a win for the other.

### 5. Ranked result in one transaction (store)
- Migration v3:
  ```sql
  CREATE TABLE ranked_matches (
    id INTEGER PRIMARY KEY,
    ended_at TEXT NOT NULL,
    a_id INTEGER NOT NULL REFERENCES accounts(id),
    b_id INTEGER NOT NULL REFERENCES accounts(id),
    winner INTEGER,              -- 0 = A, 1 = B, NULL = draw
    reason TEXT NOT NULL,
    duration_s INTEGER NOT NULL,
    a_before INTEGER NOT NULL, a_after INTEGER NOT NULL,
    b_before INTEGER NOT NULL, b_after INTEGER NOT NULL,
    a_factions TEXT NOT NULL,    -- JSON {"Humans":n,"Undead":n,"Machines":n,"Plants":n} purchases
    b_factions TEXT NOT NULL
  );
  CREATE INDEX ranked_by_a ON ranked_matches(a_id, id);
  CREATE INDEX ranked_by_b ON ranked_matches(b_id, id);
  CREATE INDEX accounts_by_rating ON accounts(rating DESC, wins DESC, name_lower);
  ```
- `StoreCmd::RankedResult { a, b, outcome, duration_s, factions: [[u32; 4]; 2] }`
  → in **one** transaction: read both accounts' `rating` and `ranked_games`,
  compute both deltas with `tbg_core::rating`, update `rating`,
  `peak_rating`, `ranked_games`, `wins/losses/draws`, insert the
  `ranked_matches` row, commit. Reply both `(before, after, Profile)`.
- On reply the lobby updates each session's cached profile and sends
  `RatingChanged` to the session's current connection (if any).
- If the store fails, log an error and send nothing; the match itself
  already ended normally.

### 6. Integration tests (`tests/ranked.rs`)
Two logged-in clients ranked → `MatchStart{mode: Ranked, opponent_rating:
Some(1000)}`; play to the end with `TBG_TICK_MS=1`; both get
`RatingChanged` with ±20 (placement, equal ratings) or 0 for a draw;
guest `RankedMatch` → `LoginRequired`; a casual and a ranked waiter are not
paired; a ranked forfeit gives the leaver a loss; `ranked_matches` has one row.

### 7. Client
- Menu: two titled sections. **Casual**: Quick Match, Play with a friend,
  Practice vs Bot. **Ranked**: Ranked Match (shows your rating) — for guests
  the button is replaced by "Create an account to play ranked".
- HUD: a small `RANKED` badge and `Opponent (1034)` when ranked.
- Result screen: `Rating 1000 → 1020 (+20)` once `RatingChanged` arrives
  (count up over 0.6 s); **Play again** re-queues the same mode.
- Bots: `tbg-bots --ranked` (implies `--accounts`).

### 8. Docs
GAME-DESIGN: mark BR-R01–R07 as implemented values (keep **Proposed** until
the playtest); ARCHITECTURE §5 (messages, queues) and §8 (tables).

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Rating unit tests and `rating_sim` (correlation ≥ 0.85) in the report
- [ ] Ranked integration tests pass
- [ ] Browser: two accounts on two devices play ranked; both see the change;
      a guest sees the ranked section locked
- [ ] `tbg-bots --ranked --matches 50` completes with no store errors

## Risks and what to do

| Problem | Action |
|---------|--------|
| Rating update lost on a crash between match end and commit | Acceptable for the MVP; record it; never retry a result twice |
| Same two friends farming rating | Out of scope now; P13 shows head-to-head history so it is visible; note it |
| `RatingChanged` reaches a reconnecting player late | The menu profile chip refreshes from the cached profile on next `LoggedIn` |

## Reviewer checklist

- Rating changes only inside the store transaction; no client value used.
- Each player's K comes from their own `ranked_games` before the match.
- Casual and private-room matches never touch `accounts` rating columns.

## Phase report

Executed 2026-10-28 on the dev PC (branch `phase/p12-ranked-and-rating`).

### Verification commands (all pass)

```text
$ cargo fmt --all -- --check            # clean
$ cargo clippy --workspace -- -D warnings   # clean (after two fixes, see below)
$ cargo test --workspace               # ok: core 6 account + 5 rating + 15 rules,
                                       # server 10 account + 5 ranked + 12 online
$ cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown  # ok
$ trunk build --release                # success
```

### rating_sim output (step 2)

```text
mean rating 1000.8
min 649 max 1417
games per player min 174 max 231
spearman skill/rating 0.982
```

Correlation 0.982 ≥ 0.85 ✔, mean 1000.8 within 1000 ± 30 ✔.

### Test list (step 6, `crates/tbg-server/tests/ranked.rs`)

Two logged-in clients ranked → `MatchStart{Ranked, opponent_rating:
Some(1000)}`, idle to the end, both `RatingChanged{1000 → 1000}` for the
draw; forfeit (1 s window) → stayer `RatingChanged{1000 → 1020}` with
`wins = 1`, leaver re-login shows 980/`losses = 1`; guest `RankedMatch` →
`LoginRequired`; casual + ranked waiters never pair; casual completion
never sends `RatingChanged` and leaves rating columns at 1000/0.
`ranked_matches` has exactly one row per ranked match.
`tbg-bots --matches 2 --ranked` (register limit raised on the test server
only): 4/4 bots completed; store shows 1020/1/1/0/0 and 980/1/0/1/0 with two
`TimeUp` rows (1000→1020 / 1000→980).

### Browser checks (owner-pending, no devices attached)

Two accounts on two devices play ranked; both see the change; a guest sees
the ranked section locked. Paths are in place (menu sections, HUD badge,
result count-up, Play again re-queues the same mode).

### Deviations

- `tbg_core::protocol::RatingResults` alias for the per-seat
  `(before, after, profile)` triples (clippy `type_complexity` on the
  inline array type).
- Result-screen count-up uses `.clamp(0.0, 1.0)` (clippy `manual_clamp`).
- The ranked-takeover path reuses the P11 `Attach{new_token}` mechanism,
  so a mid-match login still finds the right session for `RatingChanged`.
- GAME-DESIGN needed no edit: BR-R01–R07 and the §5 Elo values already
  state exactly what was implemented (status stays **Proposed** until the
  playtest, as the phase file orders).
- `two_bots_finish_a_match` timed out once when the full workspace suite
  ran online + ranked binaries in parallel (environmental contention;
  green alone and on re-run). Same class as the P10/P11 flakes.

### Blocked items

- Owner checkpoint: two-device ranked match + playtest sign-off (P14 gate).

### Review (reviewer, 2026-10-28)

- Reviewer checklist holds: ratings change only inside the store
  transaction; each side's K uses its own `ranked_games` before the match;
  casual and private matches never touch the rating columns (tested).
- Acceptance item the report did not show, run by the reviewer:
  `tbg-bots --ranked --matches 50 --ramp-ms 20` against a local release
  server (`TBG_TICK_MS=2`): 100/100 bots completed, RTT p95 2 ms, no store
  errors in the log; a second run of 40 matches against the same database
  completed 80/80 after the bot `NameTaken` fix.
- Found while testing P13: the P12 menu overflowed a 1280×800 screen at UI
  scale 1.5 (title and sound toggle cut off); fixed in 664e9ac by pairing
  buttons on shared rows.
- Minor, not fixed: `ranked_done` matches sessions by token only; a player
  who logs out and into another account in the few milliseconds between
  `MatchEnd` and the store reply would see the other account's profile
  until the next `LoggedIn`.
- Still owner-pending: two devices play ranked (P14 gate).
