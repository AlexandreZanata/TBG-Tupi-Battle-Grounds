# P16-F1 — Economy and Siege: uncapped Energy, Interest, no time-up decision

> **Owner revision 2026-10-29 (D-41) — executed in reduced form on branch
> `phase/p16f-infinite-economy`.** Goal 1 (uncapped Energy, Interest) is
> active. Goal 2 is replaced: no Siege, no Collapse, no server limit;
> matches are open-ended (BR-M07). Goal 3 (lab gate before the switch) was
> skipped by owner decision and stays open. Goal 4 is done for the economy.
> The combined phase report for the revision is at the end of this file.

**Depends on:** P16-D (balance) reviewed and merged, with the P16-C `saver`
finding fixed or explained in its report. P16-E is not part of this plan.
**Branch:** `phase/p16f1-economy-and-siege`.
**Difficulty:** high. Steps 2, 4 and 7 move goldens; step 5 decides whether
the economy ships at all.
**Owner decisions:** O-4 (Siege/Collapse, D-36), O-5 (uncapped Energy with
Interest, D-35), O-11 (switching canon) — asked at step 6, not before.

## Goal

1. Remove the 10-Energy cap and pay Interest on the lowest balance held
   (GAME-DESIGN §10.1, D-35).
2. Replace the time-up HP comparison with `Siege` and `Collapse`, with a
   proven bound on match length and an unreachable server limit (§10.2,
   D-36).
3. Measure both in the P16-C lab before any player sees them; switch canon
   only on the owner's yes.
4. Show the new economy and the Siege clearly in the desktop HTML HUD and
   the Bevy HUD.

## Starting state (verify first)

Record in the report: HEAD, `PROTOCOL_VERSION` (7 at planning time), the
store migration level, P16-D's canonical values for Energy start/regen,
Stronghold, and its `gate` output on confirmation seeds. Facts at planning
time (re-check each; P16-D may have moved them):

- Energy is `PlayerState.energy: u32` in sub-energy (`ENERGY_SCALE = 600`),
  capped only in `MatchState::regen_energy` (`sim.rs`), from
  `Ruleset.energy_cap`.
- `advance_phase` (`sim.rs`) decides `TimeUp` at `OVERTIME_START` and
  `MATCH_END` by comparing HP; `Phase::{Countdown, Running, Overtime,
  Ended}`; `EndReason::{StrongholdDestroyed, TimeUp, Forfeit}`.
- Code assuming the clock: `headless.rs` (`GRACE`, `MATCH_END + 5`),
  `replay.rs` (`MATCH_END + 5`), `tests/rules.rs`,
  `tests/contract_constants.rs`, the seed-1 golden that ends by `TimeUp`
  (`tests/contract_determinism.rs`), the `TimeUp` row in
  `tbg-balance/src/gate.rs`, the client clock (`ui/hud_update.rs`,
  `assets/battle/hud.js`), result text (`ui/result.rs`), and
  `DRAIN_LIMIT = 300 s` in `tbg-server/src/main.rs`.
- The client hard-codes `ENERGY_CAP` in `ui/battle_hud.rs` and
  `ui/hud_update.rs`; `MatchView` has no cap field.
- Lab policy `Saver` = "bank to the Energy cap, then spend all"
  (`tbg-balance/src/policy.rs`).

If P16-D already changed any of these rules, stop and ask the reviewer to
amend this spec before step 2.

## Rules

GAME-DESIGN §10.1 (BR-E04–E07, BR-E10) and §10.2 (BR-M07–M11). BR-E08
(Bounty) has no source yet and is implemented in P16-F3. Gate rows: §10.9
(economy, duration, Siege rows) and every row P16-D left passing.

## Technical decisions

- **Ruleset-gated** (D-34). New `Ruleset` fields, canonical values keep
  today's game until step 7:
  - `energy_cap: Option<u32>` — canon `Some(6_000)` until step 7, then `None`.
  - `interest: Option<InterestRule>` with `band: u32` (3 000),
    `per_band: u32` (150), `max_bands: u32` (5) — canon `None` until step 7.
    The period is the Shop rotation (`ROTATION_TICKS`, structural): payments
    happen on the same ticks as rotations, never at another cadence.
  - `siege: Option<SiegeRule>` with `start: u32` (running ticks, 4 800 =
    4:00), `base_bp: u32` (10), `ramp_bp: u32` (10), `step: u32` (200 ticks)
    — canon `None` until step 7 (old regulation/overtime path).
  - Structural `const SERVER_LIMIT: Tick` in `content.rs` = `RUNNING_START
    + 12_000` (10:00 running); never a `Ruleset` value.
- **State** (`state.rs`, serialized, so hashes move): `PlayerState.low: u32`
  (Interest low-water mark). No other new state: Siege timing derives from
  `tick`.
- **Order inside `step()`** (keep every existing call in place):
  intentions (before `step`, lower `low` on every spend) → `advance_phase`
  (enters `Siege`; checks `SERVER_LIMIT`) → `regen_energy` (cap only if
  `Some`) → `pay_interest` on rotation ticks (then `low = energy`) →
  `collapse` on Siege second boundaries (end with `Collapse` if any
  Stronghold ≤ 0; both ≤ 0 is a draw) → the rest unchanged.
- Spending goes through one helper `PlayerState::spend(cost)` that lowers
  `low`; `intent.rs` purchase and Evolution use it (later phases too).
- Collapse loss = `floor(max_hp × rate / 10_000)` in `i64`, where `max_hp`
  is the seat's Stronghold max (today `rules.stronghold_hp`; P16-F3 makes
  it per seat) and, for Siege second `s` (1, 2, …),
  `rate = base_bp + ramp_bp × ((s − 1) × 20 / step)` (integer division).
  Applied every 20 ticks of Siege, first at Siege tick 20 (s = 1).
- `Phase::Overtime` is renamed `Siege` in step 7 (same postcard index 2);
  `EndReason` appends `Collapse` and `ServerLimit`; `TimeUp` stays, never
  produced after step 7 (old records keep their strings).
- `MatchView` appends `interest_next: u32` (sub-energy the next payment
  would pay given the current `low`) and `collapse_bp: u16` (current rate,
  0 outside Siege). In Running, `phase_ticks_left` counts to Siege.
- After step 7 the regulation/overtime/TimeUp code path is deleted (no
  orphaned code); `energy_cap` and `interest` stay `Option` so the lab can
  still test caps and no-Interest rulesets.
- Records: `matches` gains `ruleset_fp TEXT` (FNV fingerprint of the
  canonical `Ruleset`, the lab's existing function) through a new additive
  migration, so telemetry before and after each P16-F switch is separable.
- Core `Bot` stays unchanged (the reference); new behaviors are lab
  policies.

## Read first (only these)

- `AGENTS.md`; `docs/GAME-DESIGN.md` §4, §5, §6, §10.1, §10.2, §10.7, §10.9;
  `docs/DECISIONS.md` D-33–D-36.
- `docs/phases/P16C-balance-lab.md` (steps 3, 6–8, 10) and the P16-D report.
- Core: `content.rs`, `ruleset.rs`, `state.rs`, `sim.rs`, `intent.rs`,
  `shop.rs`, `view.rs`, `headless.rs`, `replay.rs`, `protocol.rs`,
  `tests/contract_*.rs`, `tests/rules.rs`.
- Lab: `policy.rs`, `metrics.rs`, `gate.rs`, `knobs.rs`, `tests/contract.rs`.
- Server: `main.rs`, `match_task.rs`, `match_task/report.rs`,
  `store/schema.rs`, `store/matches.rs`, `store/ranked.rs`.
- Client: `ui/battle_hud.rs`, `ui/hud_update.rs`, `ui/hud_top.rs`,
  `ui/result.rs`, `assets/battle/hud.js`, `assets/battle/hud.css`,
  `index.html` (`#tbg-battle`).

## Files

Core `ruleset.rs`, `content.rs`, `state.rs`, `sim.rs`, `intent.rs`,
`view.rs`, `headless.rs`, `replay.rs`, `protocol.rs`; new test file
`tests/economy_siege.rs`; updated contract tests (same commit as the canon
change they guard). Lab `policy.rs`, `metrics.rs`, `gate.rs`, `knobs.rs`,
`tests/contract.rs`. Server `main.rs`, `match_task/report.rs`,
`store/schema.rs` (+ migration), `store/matches.rs`, `store/ranked.rs`
(new reasons), `tests/online.rs`. Client files listed above. Evidence in
`docs/media/p16f1/`.

## Do NOT

- Change regen, start Energy, Shop, units, Evolution or Stronghold numbers.
- Add Bounty, Reserve, kinds, Fortifications or Augments (later phases).
- Give the server or client any non-canonical ruleset (contract C10).
- Add a cap under another name (soft cap, decay of banked Energy, a
  maximum army). A `banker-N` failure goes to the owner, not to a new limit.
- Add floats, clocks or hash-ordered containers to `tbg-core` (C1).
- Switch canon before the owner's explicit yes at step 6.

## Steps (in order; check and commit each)

### 1. Baseline

Save to `docs/media/p16f1/baseline.txt`: HEAD; `gate --pairs 2000` on
discovery seeds and the P16-D confirmation `gate` output; C2 trace
goldens; `examples/army_load.rs` step mean/p99 at 10/40/100 units a side;
view bytes avg/max; `cargo test --workspace` count.
**Check:** a second run gives identical numbers (except wall time).
**Done when:** committed (`docs(p16f1): baseline`).

### 2. Ruleset fields and state, disabled

Add the `Ruleset` fields and `PlayerState.low` (initialized to the start
balance) and `spend()`; register every new field in `knobs.rs` (C5): band
600–4 800 step 600, per_band 60–300 step 30, max_bands 2–8, siege start
3 600–7 200 step 600, base/ramp 5–30 step 5, cap `None` or 6 000–30 000.
Canon values keep the old behavior.
Because `low` is serialized, every hash moves. Prove behavior is unchanged
**before** refreezing: for every golden seed, winner, reason, end tick,
the per-tick event list and the purchase/Evolution sequence equal the
step-1 values (write the comparison as a temporary test, run it, then
delete it), and `tbg-balance run` with fixed flags gives the same results
as step 1. Then refreeze C2 and `contract_determinism.rs` in this commit
and name the rule in the message.
**Check:** `cargo test --workspace`; wasm32 check.
**Done when:** committed (`refactor(p16f1): disabled economy and siege fields (BR-E04, BR-M07)`).

### 3. Uncapped Energy and Interest (behind the fields)

`regen_energy` caps only with `Some(cap)`; saturating adds. `pay_interest`
on rotation ticks after regen: `min(low / band, max_bands) × per_band`,
then `low = energy`. Every spend lowers `low`.
**Check:** tests 1–5 below pass with a lab ruleset (`with_rules`) that
enables them; canon goldens unchanged.
**Done when:** committed (`feat(p16f1): uncapped Energy and Interest (BR-E04–E07)`).

### 4. Siege, Collapse and the server limit (behind the field)

With `siege: Some`, `advance_phase` never decides on time: it enters
`Siege` at `RUNNING_START + start` (same phase index as overtime; until
step 7 the variant is still named `Overtime`), and ends the match as a draw
with `ServerLimit` at `SERVER_LIMIT`. `collapse` applies BR-M08/M09.
`headless` and `replay` stop at `SERVER_LIMIT + 5` instead of `MATCH_END + 5`.
**Check:** tests 6–10 pass with a lab ruleset; canon goldens unchanged.
**Done when:** committed (`feat(p16f1): Siege and Collapse (BR-M07–M11)`).

### 5. Lab: policies, metrics, gate rows, experiment

- Policies (generated, no copy-paste): `Banker(n)` for n in
  {10, 15, 20, 25, 30, 40} Energy — bank until `energy ≥ n`, then spend
  everything like `Heavy`; `Saver` becomes `Banker(cap)` and keeps its name
  in reports. `Rush` — buy the cheapest affordable offer at once, never
  evolve. `Eco` — never let `low` fall below 25 Energy; spend only the
  surplus, evolving the oldest unit when the surplus allows. Add them to
  the strategic pool.
- Metrics per match: Interest received, peak balance, Energy unspent at the
  end, reached Siege, ended by Collapse, longest Siege stretch with no unit
  damage to either Stronghold, running time at end; report p50/p90/p99/max.
- Gate rows: GAME-DESIGN §10.9 economy, duration and Siege rows (the
  `TimeUp` share row is replaced by "ended by Collapse").
- Experiment on discovery seeds (2 000 pairs per cell, then 20 000
  confirmation pairs for the best cell): rulesets (a) P16-D canon,
  (b) no cap, (c) no cap + Interest, (d) no cap + Siege, (e) no cap +
  Interest + Siege (the proposal). For (e) also screen band 4/5/6 Energy
  and max bands 4/5/6 (9 cells). Save `docs/media/p16f1/experiment.txt`
  (table of every gate row per cell with intervals) and `gate-proposal.txt`.
**Check:** C3 (1 vs 16 threads identical), C5, C7, C8, C11 still pass;
`banker-n` policies produce only legal intentions.
**Done when:** committed (`feat(p16f1): economy and siege lab experiment`).

### 6. Owner checkpoint (stop here)

Present the experiment table, the `banker-n` rows first. If no cell of
(e) passes the `banker-n` and duration rows, do **not** tune further than
the 9 screened cells: report it and ask the owner to choose between (c)/(d)
alone, a different Interest shape (a new DECISIONS entry), or dropping
Interest. Record the answer and the chosen cell under *Owner decision*.
**Done when:** the decision is committed (`docs(p16f1): owner decision`).

### 7. Switch canon

Set the chosen values in `content.rs` (each number once) and GAME-DESIGN
§5/§10 (still **Proposed**, "P16-F1 lab evidence, <date>"); rename
`Overtime` → `Siege`; append `EndReason::{Collapse, ServerLimit}`; delete
the regulation/overtime decision code, `OVERTIME_START`/`MATCH_END` uses
and the overtime UI text; `contract_constants.rs`, `contract_roster.rs`,
goldens, `contract_protocol*.rs` updated in this commit; append the
`MatchView` fields and bump `PROTOCOL_VERSION` by one. Server:
`DRAIN_LIMIT` ≥ `SERVER_LIMIT` running time + countdown + 60 s (720 s);
ranked settlement maps `Collapse` like `StrongholdDestroyed` and
`ServerLimit` to a draw (½); new migration adds `matches.ruleset_fp`.
GAME-DESIGN: mark BR-M05 and the cap part of BR-E02 replaced; §4 step 7
rewritten for Siege.
**Check:** full verification block; `gate` on confirmation seeds equals
the step-5 numbers for the chosen cell.
**Done when:** committed (`feat(p16f1): switch canon to uncapped Energy, Interest, Siege`).

### 8. HUD (desktop HTML and Bevy)

- Energy: whole Energy plus one decimal, no maximum shown; a five-pip
  Interest band meter (filled pips = `min(low / band, 5)`), the next
  payment as "+0.75" from `interest_next`, and a ring that empties toward
  the next payment (`next_rotation_in`). A short "+0.75 Interest" float on
  payment (client-side from the Energy jump on a rotation tick, no new
  event).
- Clock: in Running, "Siege in m:ss"; in Siege, a "SIEGE" banner and the
  Collapse rate ("−0.3 % / s") from `collapse_bp`; Stronghold bars pulse
  while Collapse applies.
- Result screen: reasons "Stronghold destroyed", "Collapsed in the Siege",
  "Draw — both collapsed", "Server limit (draw)", "Forfeit".
- Remove every client use of `ENERGY_CAP`/`data-energy-max`.
- Desktop: `battle_hud.rs` writes `data-interest`, `data-interest-next`,
  `data-collapse`; `hud.js`/`hud.css` render them. Bevy: `hud_update.rs`,
  `hud_top.rs`.
**Check:** client unit tests for the label functions; `trunk build
--release`; browser at 1920×1080, 1280×800 and 390×844.
**Done when:** committed (`feat(client): Interest meter and Siege clock`).

### 9. End-to-end and report

Online test (`tests/online.rs`, virtual clock): two bot clients that never
buy reach Siege and end by `Collapse`; the record has reason `Collapse`
and `ruleset_fp`; a reconnect during Siege resumes with the right clock.
C4a and C4b (`lab_equivalence`, real clock, `--ignored`) pass. Measure
`army_load` and view bytes again. Fill in the report.
**Done when:** committed (`docs(p16f1): phase report`).

## Acceptance tests

`tests/economy_siege.rs` (core), each citing its rule:

1. `br_e04_energy_grows_past_ten_without_spending` — 60 s idle, no cap.
2. `br_e06_interest_pays_on_the_lowest_balance` — spend to 9.9 Energy 1
   tick before payment: interest is 1 band (from `low`), not 2.
3. `br_e06_purchase_on_the_payment_tick_counts_first` — intention applied
   on the payment tick lowers `low` before payment.
4. `br_e05_both_seats_paid_on_the_same_tick` — equal balances, equal pay.
5. `br_e07_energy_saturates_without_overflow` — balance near `u32::MAX`.
6. `br_m07_no_result_at_three_or_four_minutes` — unequal HP at 3:00 and
   4:00 does not end the match.
7. `br_m08_collapse_schedule_is_exact` — a 4 500-HP Stronghold loses 4 per
   second for Siege seconds 1–10 and 9 for seconds 11–20.
8. `br_m09_collapse_on_both_sides_is_a_draw_with_reason_collapse`.
9. `br_m10_idle_match_ends_before_the_bound` — no purchases: ends by
   `Collapse` before 6:40 running time.
10. `br_m11_server_limit_is_unreachable` — for 1 000 seeds of every lab
    policy pair the end tick is below `SERVER_LIMIT`; plus an analytic test:
    with the largest Stronghold max the canon allows and zero unit damage,
    Collapse ends the match before 6:40 (extended by P16-F3/F5 when
    healing appears).

Server: the online Collapse match above; `ranked` settlement of a
`Collapse` win and a `ServerLimit` draw (store test). Lab: C3, C5, C11
with the new policies.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cargo test -p tbg-server --test lab_equivalence -- --ignored
cargo run --release -p tbg-balance -- gate --pairs 20000 --threads 16
cd crates/tbg-client && trunk build --release
```

Plus: a browser practice match reaching Siege; two clients online finish a
match, one reconnects during Siege.

## Performance

Report before/after: step mean and p99 at 10/40/100 units a side (budget:
≤ +5 %); view bytes avg/max (budget: +3 B); server RSS per match at 250
concurrent matches with `scripts/capacity.sh` (no budget change). Siege
lengthens the longest matches: report bytes per match at p50 and p99
duration. Never claim a number not measured in this phase.

## Acceptance checklist

- [ ] Steps 1–9 committed; verification block passes
- [ ] Owner decision recorded before the canon switch
- [ ] Gate on confirmation seeds: every §10.9 economy/duration/Siege row
      passes, and no P16-D row that passed now fails (or the owner accepted it)
- [ ] No time-up code path remains; `TimeUp` never produced
- [ ] HUD shows Interest and Siege on desktop and phone sizes
- [ ] Docs: GAME-DESIGN §4/§5/§10, ARCHITECTURE §4/§5/§8, roadmap

## Risks and what to do

| Problem | Action |
|---------|--------|
| `banker-n` dominates (banked Energy deployed at home beats trickles) | Stop at step 6; owner chooses; never add a disguised cap |
| Interest makes passive play optimal | `eco` row; compare (b) vs (c); drop Interest if (c) fails where (b) passes |
| Siege decides too many matches | Collapse row ≤ 15 %; screen Siege start 4:00–5:00 at step 5 |
| Hash refreeze hides a behavior change | Step-2 comparison of outcomes, events and purchases before refreezing |
| Server shutdown cuts long matches | `DRAIN_LIMIT` raised in step 7 and tested |

## Reviewer checklist

- Re-run `gate` (confirmation seeds) and C4b; compare with the report.
- `low` is lowered by every spend path and never raised by gains.
- Interest pays on rotation ticks only, both seats, after regen.
- Collapse ignores everything listed in BR-M09 (shields arrive in F5:
  leave a test hook comment naming BR-M09).
- No client use of `ENERGY_CAP`, `MATCH_END` or `OVERTIME_START` remains.
- Contract-test diffs each name the canon change in their commit.

## Owner checkpoint

Step 6: approve O-4 and O-5 with the experiment table, then O-11 (switch).
After step 9: play two practice matches and one online match into Siege;
confirm the Interest meter and the Siege are understandable.

## Phase report

### Owner revision 2026-10-29 (D-41), branch `phase/p16f-infinite-economy`

Executed as one combined change for P16-F1 (economy, open-ended matches)
and P16-F5–F7 (infinite Augments online), plus the owner's mandatory
**global pause during Augment selection** (BR-AG12–AG14). Not the spec
above: no lab gate, no Siege/Collapse, no `ServerLimit`.

**Starting state.** Base `origin/main` becd566. The previous agent left the
economy, the 40 Augments, protocol v10 and both HUDs uncommitted;
`cargo test -p tbg-core`, `-p tbg-client` (26), server `online` (29 of 30),
`ranked` (6) and `leaderboard` (4) passed; the new online Augment test did
not compile (it read `b_view` before defining it, then compared it with
itself).

**What changed in this pass.**
- Online test rewritten around the pause (below); it now compiles and
  checks real values.
- `MatchState::clock` (selection clock, every step) beside `tick` (combat
  clock, frozen while paused). `step()` during a round only advances the
  clock and resolves on both confirmations or the deadline; Interest,
  regen, Shop, combat, cooldowns, summons and Synergy do not run.
- `PlayerState::augment_pick`: a confirmed id stays hidden; Energy is paid
  on confirmation; both outcomes apply in seat order at resolution.
  `Forgo` rewards are paid at resolution too (canon value 0).
- `Rejection::Paused` for `Purchase`/`Evolve` during a pause.
- `ReplayOp.clock` replaces `ReplayOp.tick`; views and the bot cadence run
  on the selection clock in the server, the practice match and `headless`.
- `ServerMsg::View(Box<MatchView>)` (clippy `large_enum_variant`; same
  wire bytes). `MatchView.clock` added.
- Practice/reference bot forgoes every round (`ponytail:` comment).
- Desktop HTML War Council is a modal over a dim veil that blocks the
  field: four cards with descriptions and costs, time left, select then
  **Confirm choice**, **Forgo**, then "Augment selected! / Waiting for
  opponent...". Bevy fallback HUD: Council row only while paused, Shop and
  Evolution disabled, waiting text.

**Goldens refrozen** (`contract_determinism`, `contract_lab` C2): the new
serialized fields move every hash; outcomes and end combat ticks of all
golden seeds are unchanged (seed 1 A 2 297, 7 B 1 920, 42 B 3 233,
1000 → 2 160, 65535 → 2 637).

**Verification (2026-10-29, this machine).**

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace -- -D warnings` | exit 0 (also clean with `--all-targets`) |
| `cargo test --workspace` | exit 0; `tbg-balance` contract 7 passed in 193.34 s, value 12; `tbg-client` 26; `tbg-core` augment_pause 9, augment_effects 14, augments_infinite 10, economy_interest 13, contract_determinism 2, contract_lab 3; server online 30, ranked 6, leaderboard 4, lab_equivalence C4a 1 (C4b ignored) |
| `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown` | exit 0 |
| `trunk build --release` | success |
| Browser, desktop 1440×900, two real clients (desktop + second tab) on a local server at :8137 | paused at 1:00 for both; four cards, costs, descriptions, 0:55 timer; select + confirm → "Augment selected! Waiting for opponent..." with the clock frozen at 1:00 and Energy 16 → 13; the idle opponent never confirmed and the battle resumed at the deadline with "Reinforced Shields" shown; Shop clicks behind the veil did nothing |

**Owner-required pause tests** (`crates/tbg-core/tests/augment_pause.rs`,
`crates/tbg-server/tests/online.rs`): (1) both seats frozen on the same
round and combat tick; (2) units, attacks in flight and cooldowns
unchanged over 300 paused steps; (3) Energy, `low`, Interest, offers
unchanged; (4) one confirmation never resumes; (5) both confirmations
resume on the next step; (6) the deadline resumes with an idle player
after exactly 400 steps (20 s; first built as 60 s, shortened by the owner afterwards); (7) the pick is invisible to the other seat
until resolution; (8) repeat or changed choices rejected and charged once;
(9) online: the undecided seat disconnects and the deadline still resumes
the battle, then it reconnects into the resumed battle; a seat reconnecting
mid-pause gets the round, less time left and its confirmation; (10) the
next draw is 1 200 combat ticks later, 1 600 selection steps including the
20-s pause; (11) a replay with choices made 40 and 300 steps into pauses
reproduces the final hash, and the server-vs-headless C4a contract passes;
(12) desktop UI checked in the browser (phone not checked: owner rule,
desktop only).

**Deviations.** Phone (Bevy) War Council confirms on one tap, without a
separate Confirm step, and does not highlight the chosen card; not
browser-checked (desktop-only rule). The bot never buys Augments.

**Shop follow-up (owner request, same day).** Offers no longer rotate
every 10 s; a purchase refills its slot with a unit drawn uniformly from
the five not on screen (never the one just bought, BR-S04), from the
seat's server-seeded stream (online seeds come from the OS generator). The
card timer bar is gone from the desktop HUD; `INTEREST_TICKS` replaces
`ROTATION_TICKS` for the Interest period; `MatchView.next_rotation_in` is
removed. Bot goldens moved with this rule (seed 1 now B at 3 664, 7 A at
2 146, 42 B at 2 342). Test: `rules.rs`
`br_s04_a_purchase_refills_with_an_unseen_unit_and_nothing_rotates`
(400 purchases: never a shown unit, others untouched, all 10 units
reachable; offers unchanged over 1 000 steps without a purchase).

**Limitations (not measured or not done).**
- Balance is **not measured**: no `tbg-balance` run for uncapped Energy,
  Interest or infinite Augments; GAME-DESIGN §10.9 gates are open; P16-C's
  `saver` finding probably gets stronger.
- No `ruleset_fp` (ruleset fingerprint) is recorded with matches.
- Not every cap is visible in the UI (lifesteal, damage reduction, status,
  move and attack-speed caps; Bounty ceiling).
- Matches have no time bound at all; one long match costs one task.
- Augment picks are not in match records or telemetry yet.
