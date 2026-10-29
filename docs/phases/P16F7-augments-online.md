# P16-F7 — Augments online: server, desktop and phone UI, telemetry

**Depends on:** P16-F6 reviewed, its gate passed, owner decision O-8.
**Branch:** `phase/p16f7-augments-online`.
**Difficulty:** medium–high (two UIs with a non-modal panel during
combat; reconnect during a window).
**Owner decisions:** look and touch approval, O-11 (switch) at step 6.

## Goal

Players see a War Council round open, compare three options, buy one or
Forgo within 30 s while still buying units, evolving and Fortifying, and
read both players' Augments at any time. The server owns everything;
records let later balance work use real matches.

## Starting state (verify first)

Record HEAD, `PROTOCOL_VERSION`. Expected from P16-F5/F6: the 40
Augments, rounds disabled online (`Unavailable`), view fields `round`,
`augments`, `shield`, `evolve_cost`, events `RoundOpened`,
`AugmentChosen`, `Forgone`, `ShieldRaised`. HUD structure as in P16-F1/F4
(desktop HTML via `ui/battle_hud.rs` data attributes and `hud.js`
`data-action`s; Bevy fallback `ui/hud*.rs`; panel pattern `ui/evolve.rs`
and `ui/fortify.rs`).

## Rules

GAME-DESIGN §10.6 (BR-AG01–AG11), §10.8; BR-A01–A04; BR-M03 (reconnect);
D-24 (quality tiers never remove information: Augment icons and shields
always show).

## Technical decisions

- **Server**: nothing new to validate beyond core (intentions go through
  `apply_intention` with the connection's seat). Rate limits unchanged
  (two intentions per round at most matter).
- **Canon switch** (step 6): `augment_rounds = Some([600, 2 400, 4 200])`.
- **Records** (seat JSON, no migration): `augments: [[round, id|null,
  tick]]` with `null` for Forgo and a flag for expiry vs explicit Forgo;
  the round's three options (identical for both seats) once per match;
  Interest total, Bounty total, Energy unspent at the end. `ruleset_fp`
  already identifies the rules (P16-F1).
- **Desktop HTML** (D-29 bridge): a compact *War Council* tray anchored
  above the Energy panel, never over the five Shop cards or the center of
  the BattleLine. States: closed (no round); *chip* ("War Council 0:24",
  pulsing once when a round opens, with a short horn sound from the
  existing `audio` module); *open* (three rarity-framed cards: icon, name,
  cost, one-line text, Buy; a Forgo button "+2 Energy"; a timer bar).
  Opening/closing the tray never blocks clicks elsewhere (no backdrop, no
  focus trap; Esc collapses). Bought or Forgone → tray collapses to a
  result chip for 3 s. Hover/long-press shows the full text.
  `battle_hud.rs` writes `data-round` (`round|ticks_left|resolved|id,id,id`),
  `data-augments-you`, `data-augments-enemy`; actions `aug:0..2`, `forgo`,
  `council` (toggle).
- **Equipped Augments**: three sockets next to your Energy panel and three
  under the enemy HP bar (icons, rarity rim, tooltip). A shield shows as a
  translucent segment on the HP bar with its remaining HP.
- **Bevy (phones and fallback)**: the chip sits above the Shop row; tapping
  opens a sheet over the upper arena (never over the Shop row or the
  player's Stronghold), three rows of 56 px with Buy buttons, a Forgo row,
  and a drag-down/X to collapse; the round keeps running. Equipped icons
  as small sockets by each HP bar; tap shows the text.
- **Icons**: generated SVG → WebP with the existing reproducible art
  script pattern (`scripts/`), one per Augment, under
  `assets/battle/augments/`, lazy-loaded after the menu (D-26).
- **Bot (practice)**: core `Bot` gains the lab's `Greedy` Augment rule
  (deterministic from its stream) so practice exercises the system; this
  changes bot-vs-bot goldens — refreeze with the canon switch commit.

## Read first (only these)

`AGENTS.md`; GAME-DESIGN §10.6, §10.8; DECISIONS D-24, D-26, D-29, D-38;
ART-DIRECTION (UI and icon sections); P16-F5 and P16-F6 reports; client
`ui/battle_hud.rs`, `ui/hud*.rs`, `ui/evolve.rs`, `ui/fortify.rs`,
`ui/actions.rs`, `assets/battle/*`, `index.html`, `audio.rs`; server
`match_task/report.rs`, `match_task/views.rs`, `tests/online.rs`; core
`bot.rs`.

## Files

Core `bot.rs`, `content.rs` (canon), contract tests. Server
`match_task/report.rs`, `match_task/views.rs`, `tests/online.rs`. Client
`ui/battle_hud.rs`, new `ui/council.rs` (Bevy sheet and chip), `ui/hud*.rs`,
`ui/actions.rs`, `assets/battle/hud.js`, `hud.css`, `index.html`,
`assets/battle/augments/*`, `scripts/augment-icons.sh` (new, reproducible).
Evidence `docs/media/p16f7/`.

## Do NOT

- Pause the match, dim the arena, or cover the Shop cards.
- Add a fourth round, reroll, sell, or automatic pick.
- Compute Augment effects in the client for anything but labels.
- Add dependencies or a UI framework (D-27).

## Steps (in order; check and commit each)

### 1. Records and server tests

Record fields; write online tests 1–6 below now, marked
`#[ignore = "enabled by the P16-F7 canon switch"]` (rounds are still off
online); step 6 removes the attribute.
**Done when:** committed (`test(p16f7): online Augment contracts`).

### 2. Icons

Script, 40 icons, size report (bytes per icon, total lazily loaded).
**Done when:** committed (`feat(client): Augment icons`).

### 3. Desktop War Council tray and sockets

As specified; works with keyboard (Tab to cards, Enter to buy).
**Check:** browser 1920×1080 and 1280×800 in a practice match with rounds
enabled through the local practice path after step 6 — until then a
`?council-demo` client-only preview fed by a fixed `RoundView` (removed in
step 6 if no longer needed; no orphan).
**Done when:** committed (`feat(client): desktop War Council`).

### 4. Bevy chip, sheet and sockets

As specified; touch targets ≥ 44 px; the Shop row stays usable while the
sheet is open.
**Check:** browser 390×844 and 844×390.
**Done when:** committed (`feat(client): War Council on phones`).

### 5. Practice bot

Greedy Augment rule in `Bot`; practice match shows the bot's choices.
**Done when:** committed (`feat(core): bot takes Augments`).

### 6. Owner checkpoint and switch

Captures of every state on desktop and phone sizes; a recorded practice
match. On approval: canon rounds on, goldens refrozen (bot + rounds),
`PROTOCOL_VERSION` + 1 if any wire field changed since P16-F5, contract
tests, GAME-DESIGN §10.6 "active since P16-F7", ARCHITECTURE §5/§6/§8;
un-ignore step-1 tests.
**Done when:** committed (`feat(p16f7): switch canon to Augment rounds`).

### 7. End-to-end and report

Two clients (desktop + phone viewport) online: round 1 buy, round 2 Forgo,
round 3 expire while disconnected (reconnect inside the window shows the
open round with the right timer; reconnect after it shows the +2 once);
both see both players' Augments; the record matches. Fill in the report.
**Done when:** committed (`docs(p16f7): phase report`).

## Acceptance tests

Server (`tests/online.rs`):

1. `br_ag03_both_clients_receive_the_same_options`
2. `br_ag05_second_choice_in_a_round_is_rejected`
3. `br_ag06_disconnect_through_expiry_grants_forgo_once`
4. `br_ag09_equipped_augments_reach_both_views`
5. `br_ag11_client_cannot_choose_for_the_other_seat` (seat from the
   connection; no seat field exists)
6. Record round-trip: offers, choices, Forgo vs expiry.

Client: unit tests for tray/sheet state functions (closed/chip/open/
resolved; timer text; affordable state from Energy); JS syntax check.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cd crates/tbg-client && trunk build --release
```

Plus the two-client run and captures at 1920×1080, 1280×800, 390×844,
844×390.

## Performance

Measured: view bytes during a window vs outside; client frame time when
the tray/sheet opens (no hitch > 1 frame at 60 FPS desktop in the
browser); icon bytes; WASM size delta.

## Acceptance checklist

- [ ] Tests pass; switch recorded; goldens refrozen with the bot change named
- [ ] Tray and sheet never block Shop, Evolution or Fortify
- [ ] Reconnect inside and after a window behaves per BR-AG06
- [ ] Captures at four sizes; owner approval
- [ ] Docs: GAME-DESIGN, ARCHITECTURE §5/§6/§8, roadmap

## Risks and what to do

| Problem | Action |
|---------|--------|
| Too many simultaneous decisions on phones | Chip-first design; sheet collapses itself after a choice; owner touch review |
| Players miss a round | Chip pulse + horn; the timer bar; result chip "Forgone +2" |
| Text too long for cards | One-line text on cards, full text in tooltip/long-press |
| Bot change hides a regression | Goldens refrozen only in the switch commit, with the lab gate re-run |

## Reviewer checklist

- Play a practice match with the tray open the whole round: Shop, Evolve,
  Fortify still work.
- Try `aug:` actions after the round closes and for a resolved round:
  only `RoundClosed` toasts, no client state drift.
- Compare a record with the match you watched.

## Owner checkpoint

Step 6: look and touch approval, O-11. After step 7: play online matches
with every round.

## Phase report

_Not executed._ Fill in: starting state; captures; icon sizes; switch and
goldens; two-client run with reconnects; performance; deviations; blocked
items; limitations.
