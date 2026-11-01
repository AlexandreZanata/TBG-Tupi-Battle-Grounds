> Superseded 2026-10-31 by the owner's new-shop-charge design (D-69, GAME-DESIGN §10.3). The implementation/report is in SYSTEM-updates.md. CANON remains disabled.

# P16-F2 — Shop Reserve: hold one offer for later

**Depends on:** P16-F1 reviewed, canon switched, owner checkpoint done.
**Branch:** `phase/p16f2-shop-reserve`.
**Difficulty:** low–medium. Step 2 is the only tricky one (same-tick order).
**Owner decisions:** O-11 (switch canon) at step 5.

## Goal

A player can hold one of the five Shop offers out of rotation for 20 s,
free, to buy it when affordable — a planning tool for expensive units and
Synergies, with no second currency and no new randomness exposed.

## Starting state (verify first)

Record HEAD and `PROTOCOL_VERSION`. At planning time: `PlayerState.offers:
[UnitKind; 5]` drawn from the seat's own `Pcg32` (`sim.rs`, streams 1/2);
`shop::refill` picks uniformly among kinds not shown in the other four
slots; `shop::rotate` redraws all five of both seats on
`(t − RUNNING_START) % ROTATION_TICKS == 0`; `Purchase{offer: u8}` carries
no kind, so a purchase arriving after a rotation buys the new card; no
lock, hold or reroll exists.

## Rules

GAME-DESIGN §10.3 (BR-S06–S10). BR-S01 (five distinct), BR-S02 (shared
rotation timer), BR-S05 (hidden RNG) keep holding. The Shop still sells
units only (BR-S03).

## Technical decisions

- `Ruleset.reserve_ticks: Option<u32>` — canon `None` until step 5, then
  `Some(400)`; knob 200–800 step 100.
- `PlayerState.hold: Option<Hold>` with `Hold { slot: u8, until: Tick }`
  and `held_mask: u8` (bit per slot: "this card was held"; cleared when the
  slot is redrawn by rotation or refill).
- `PlayerIntention` appends `Reserve { offer: u8, kind: UnitKind }` and
  `Release`. `Rejection` appends `StaleOffer` (kind ≠ visible card),
  `Unavailable` (feature disabled, already holding, card already held,
  match not live), `NoHold` (Release without a hold). P16-F3 reuses
  `Unavailable` for disabled features.
- `shop::rotate` redraws only unheld slots; draws exclude every visible
  kind, the held one included (existing partial Fisher-Yates over the
  remaining kinds). `shop::refill` unchanged except clearing the slot's
  `held_mask` bit.
- Shop step order inside `step()`: `expire_hold` (if `tick ≥ until`, hold
  ends; card stays) runs immediately before `rotate`. Intentions are
  applied before `step()`, so BR-S10 holds without special cases.
- Purchase of the held slot: hold cleared, then refill as today.
- `MatchView` appends `hold: Option<(u8, u32)>` (slot, ticks left) and
  `held_mask: u8`. Bump `PROTOCOL_VERSION` by one in step 5.
- Core `Bot` unchanged. Lab: `Faction(f)` and `Favor(k)` gain an optional
  hold rule (hold their preferred card when it is unaffordable and nothing
  is held); a policy flag `hold: bool` so every comparison has a with/without
  pair on the same seeds.

## Read first (only these)

`AGENTS.md`; GAME-DESIGN §4, §10.3; DECISIONS D-32, D-34; core `shop.rs`,
`intent.rs`, `state.rs`, `sim.rs`, `view.rs`, `protocol.rs`,
`tests/contract_protocol.rs`; lab `policy.rs`, `metrics.rs`, `gate.rs`;
client `ui/battle_hud.rs`, `assets/battle/hud.js`, `hud.css`,
`ui/hud.rs`, `ui/hud_update.rs`, `ui/actions.rs`; `index.html` cards.

## Files

Core `ruleset.rs`, `state.rs`, `intent.rs`, `shop.rs`, `sim.rs`,
`view.rs`, `protocol.rs`; new `tests/shop_reserve.rs`. Lab `policy.rs`,
`metrics.rs`, `knobs.rs`, `gate.rs`. Client HUD files above. Evidence in
`docs/media/p16f2/`.

## Do NOT

- Charge for a hold, add a second hold, a reroll, or offer weighting.
- Reveal the next draw, the opponent's hold, or any RNG state.
- Let a hold survive a purchase of that card or be renewed on the same card.
- Change rotation timing or the number of offers.

## Steps (in order; check and commit each)

### 1. State and intentions, disabled

Add the fields, intentions and rejections; with `reserve_ticks: None`
every `Reserve` is `Unavailable`. New serialized fields move hashes: prove
unchanged behavior as in P16-F1 step 2 (outcomes, events, purchases equal
for the golden seeds), then refreeze in this commit.
**Check:** `cargo test --workspace`; wasm32 check.
**Done when:** committed (`refactor(p16f2): disabled Shop Reserve state (BR-S06)`).

### 2. Hold, release, expiry, rotation

Implement BR-S06–S10 behind `Some(ticks)`.
**Check:** tests 1–8 below with a lab ruleset.
**Done when:** committed (`feat(p16f2): Shop Reserve (BR-S06–S10)`).

### 3. Lab measurement

Run the strategic pool with `hold` off/on (same seeds, 2 000 pairs, then
20 000 confirmation pairs for the final comparison). Metrics: holds per
match, share of holds converted to purchases, mean wait, purchase share of
5-cost units, faction-majority scores, `favor-<unit>` rows. Gate rows: every
§6/§10.9 row that passed in P16-F1 still passes; faction-majority spread
(max − min) not wider with holds; seat bias ±1.5 pp. Save
`docs/media/p16f2/hold-on-off.txt`.
**Check:** C3, C5, C11 pass.
**Done when:** committed (`feat(p16f2): Reserve lab comparison`).

### 4. HUD

- Desktop cards: a lock button in each card's corner
  (`data-action="hold:i"`), a held card shows a gold frame, a "HELD" ribbon
  and a ring with the seconds left; clicking the lock again sends
  `Release`; disabled locks on cards whose `held_mask` bit is set, with the
  tooltip "Already held". Rejections show the existing toast.
- Bevy (phones): a 44 px lock button under each card (tap), same states.
  Long-press inspect keeps working.
- `battle_hud.rs` writes `data-hold` (`slot|ticks_left`) and
  `data-held-mask`.
**Check:** client tests for label/state functions; `trunk build
--release`; browser at 1920×1080, 1280×800, 390×844.
**Done when:** committed (`feat(client): Shop Reserve controls`).

### 5. Owner checkpoint and switch

Show the lab comparison and the HUD captures. On the owner's yes: canon
`Some(400)`, GAME-DESIGN §5/§10.3 noted "active since P16-F2", goldens
and `contract_protocol*.rs` updated, `PROTOCOL_VERSION` + 1, ARCHITECTURE
§5 message list updated.
**Check:** full verification block.
**Done when:** committed (`feat(p16f2): switch canon to Shop Reserve`).

### 6. End-to-end and report

Two clients online: hold, wait through a rotation, buy the held card; hold
then reconnect (the hold and its timer survive); hold until expiry.
Fill in the report.
**Done when:** committed (`docs(p16f2): phase report`).

## Acceptance tests

`tests/shop_reserve.rs`:

1. `br_s06_hold_survives_rotation_and_lasts_twenty_seconds`
2. `br_s06_hold_with_a_stale_kind_is_rejected`
3. `br_s07_no_duplicate_of_the_held_card_after_rotation_or_refill` (1 000
   seeded rotations)
4. `br_s08_buying_the_held_card_consumes_the_hold_and_refills`
5. `br_s08_release_ends_the_hold_and_the_card_rotates_next_time`
6. `br_s09_a_held_card_cannot_be_held_again_until_redrawn`
7. `br_s10_purchase_on_the_expiry_tick_succeeds`
8. `br_s10_expiry_and_rotation_on_the_same_tick_redraws_the_card`
9. `only_one_hold_per_player` and `holds_are_private_to_the_seat` (the
   opponent's view never contains the hold)

Server: `online.rs` hold → reconnect → buy. Lab: C11 with holding policies.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cargo run --release -p tbg-balance -- gate --pairs 20000 --threads 16
cd crates/tbg-client && trunk build --release
```

## Performance

View bytes +2–3 B; step time unchanged within noise (one branch in
rotation). Report both, measured.

## Acceptance checklist

- [ ] BR-S06–S10 tests pass; gate rows unchanged or better
- [ ] Owner switch recorded; protocol bumped once
- [ ] Desktop and phone controls work; reconnect keeps the hold
- [ ] Docs: GAME-DESIGN §10.3 status, ARCHITECTURE §5, roadmap

## Risks and what to do

| Problem | Action |
|---------|--------|
| Holding makes one-unit spam (`favor-siegewalker`) stronger | Compare `favor-*` rows on/off; if one passes its band only without holds, report it to the owner before the switch |
| Players hold forever by chaining | `held_mask` forbids re-holding the same card |
| Lock buttons crowd phone cards | Lock under the card, 44 px target; owner reviews captures |

## Reviewer checklist

- RNG draws: a hold changes how many values rotation consumes; verify
  determinism with C2 and trace tests, not by reasoning.
- No path lets two offers show the same kind.
- `Unavailable` covers every refused case; no panic on any input.

## Owner checkpoint

Step 5: approve the switch with the comparison table and captures.

## Phase report

_Not executed._ Fill in: starting state; refreeze proof; lab on/off table;
HUD captures (desktop, phone); two-client checks; performance; deviations;
blocked items; limitations.
