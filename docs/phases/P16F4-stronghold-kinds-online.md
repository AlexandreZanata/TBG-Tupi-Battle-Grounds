# P16-F4 — Stronghold kinds online: selection, persistence, lobby, HUD, 3D

**Depends on:** P16-F3 reviewed, owner decisions O-2, O-9, O-12 recorded.
**Branch:** `phase/p16f4-stronghold-kinds-online`.
**Difficulty:** high (server lifecycle, store migration, two UIs, four
models). Each step is independently testable.
**Owner decisions:** O-3 (default kind) at step 1; look approval and O-11
(switch) at step 8.

## Goal

Players choose a `StrongholdKind` before a match, see both kinds in the
match, Fortify their Stronghold by click or tap, and recognize every kind
and Fortification on sight. The choice is free, frozen at queue or room
entry, kept across reconnect and takeover, remembered per account (guests:
per device), and recorded with results.

## Starting state (verify first)

Record HEAD, `PROTOCOL_VERSION`, migration level. Expected from P16-F3:
`MatchState::new_with_kinds`, `Fortify` disabled online (`Unavailable`),
view fields `stronghold_max`, `fort`, `kinds`. Server facts at planning
time: the connection's seat comes from `c.matched` (`ws.rs`), intentions
are stamped in `ws/client_msg.rs`, matches start in
`lobby/matchmaking.rs`, reconnect in `lobby/sessions.rs` and
`match_task/command.rs`, records in `match_task/report.rs` and
`store/matches.rs`, accounts in `store/` with numbered migrations
(`store/schema.rs`). Client: lobby HTML `index.html` `#tbg-lobby` +
`assets/lobby/lobby.js`/`.css`, bridge `ui/desktop.rs`,
`ui/desktop_actions.rs`; Bevy menu `ui/menu.rs`; HUD as in P16-F1; the
Stronghold mesh `arena/towers.rs` (`tower(own)`, `damage_states`); local
practice `local.rs`; preferences `config.rs` (`load_flag`/`save_flag`).

## Rules

GAME-DESIGN §10.4 (BR-K01–K08), §10.5 (BR-F04), BR-A01–A04, BR-P03
(takeover), BR-R01 (private rooms are casual), BR-U05 (cosmetics never
change stats).

## Technical decisions

- **Protocol** (bump once, step 2): `ClientMsg` appends
  `SelectStronghold { kind: StrongholdKind }`; `ServerMsg` appends
  `StrongholdSelected { kind }` (ack, also sent after `Welcome`/`LoggedIn`
  with the session's current kind); `MatchStart` appends `you_kind` and
  `opponent_kind`. `Classic` is refused (`Error{code: BadRequest}` or the
  existing equivalent). Size stays far under 1 KiB.
- **Session lifecycle** (server): `Session.kind: StrongholdKind`, default
  O-3 (Redoubt proposed) or the account's saved kind. `SelectStronghold`
  is accepted only when the session is idle (not queued, not in a room, not
  in a match); otherwise ignored with `Error{code: Busy}` (append a code if
  none fits). The queue ticket and the room store a copy at entry; the
  match is built with `MatchState::new_with_kinds(seed, [kind_a, kind_b])`.
  Reconnect and takeover resend `MatchStart` with the match's kinds.
- **Store**: additive migration `accounts.stronghold_kind INTEGER NULL`
  (enum index); `SelectStronghold` by a logged-in session sends a
  fire-and-forget save to the writer task (never awaited on the lobby path);
  login loads it with the profile. Records: the per-seat JSON gains `kind`
  and `fortify: [[tick, branch, tier], …]`; no rating-table change.
- **Guests**: `config.rs` stores the kind in `localStorage`
  (`tbg.stronghold`, validated on load; unknown → default) and sends
  `SelectStronghold` after each `Welcome`.
- **Practice** (`local.rs`): player's stored kind; bot's kind
  `SELECTABLE_KINDS[Pcg32::new(seed, 5).below(4)]` (BR-K05); bot never
  fortifies in this phase (core `Bot` unchanged; P16-F8 may add it).
- **Canon switch** (step 8): `fortify_enabled = true`; online matches never
  use `Classic`.
- **3D** (`arena/towers.rs`, code-built, ART-DIRECTION; no Blender): one
  shared base plinth, then a kind silhouette readable at match distance:
  Treasury — gilded dome and coin chests, gold trim; Redoubt — squat, wide,
  buttressed stone, double wall line; Ballista — tall narrow tower with a
  mounted bolt-thrower on top; Banner — tower with a large war banner and
  drums on the parapet. Fortification overlays shared by every kind:
  Walls 1 stone skirt, Walls 2 crenellated curtain ring; Arms 1 one
  ballista mount, Arms 2 twin engines and a brazier. Team colors and
  existing damage states stay (they read per-seat max HP). One material,
  merged meshes, triangle counts logged per kind and overlay (D-30).
  A `?gallery=stronghold-<kind>[-walls2|-arms2]` entry exists for each.
- **Portraits** for the lobby cards come from the D-29 gallery capture
  tool (`?gallery=…&portrait`), saved as WebP under
  `assets/lobby/strongholds/`.

## Read first (only these)

`AGENTS.md`; GAME-DESIGN §10.4, §10.5; DECISIONS D-17, D-19, D-23, D-27,
D-29, D-37; ARCHITECTURE §5, §6, §8; ART-DIRECTION (Stronghold and team
color sections); P16-F3 report; the server, store and client files named
under *Starting state*; `tests/online.rs`, `tests/contract_protocol*.rs`.

## Files

Core `protocol.rs`, `tests/contract_protocol.rs`. Server `ws/client_msg.rs`,
`lobby.rs`, `lobby/sessions.rs`, `lobby/matchmaking.rs` (+ rooms),
`match_task.rs`, `match_task/command.rs`, `match_task/report.rs`,
`store/schema.rs` (migration), `store/` account load/save,
`tests/online.rs`, `tests/contract_protocol_server.rs`. Client `config.rs`,
`net/` (send after `Welcome`), `local.rs`, `ui/desktop.rs`,
`ui/desktop_actions.rs`, `ui/menu.rs`, `ui/battle_hud.rs`, `ui/hud*.rs`,
new `ui/fortify.rs` (Bevy panel, modeled on `ui/evolve.rs`),
`assets/lobby/*`, `assets/battle/*`, `index.html`, `arena/towers.rs`
(split into `arena/towers/{kinds,fortify}.rs` if it grows two concerns),
`gallery.rs`. Evidence `docs/media/p16f4/`.

## Do NOT

- Sell, unlock, or rank-gate kinds; add cosmetic styles (P18 stays archived).
- Let the client send stats, a seat, or a kind during a match.
- Await the store on the lobby or match path.
- Change any number from P16-F3.

## Steps (in order; check and commit each)

### 1. Default kind (owner) and contracts

Ask O-3 if not yet answered; record it in GAME-DESIGN BR-K04. Write the
server tests 1–6 below first, failing.
**Done when:** committed (`test(p16f4): selection lifecycle contracts (BR-K01–K05)`).

### 2. Protocol and server lifecycle

Messages, session field, idle-only rule, freeze at queue/room entry,
`new_with_kinds`, reconnect/takeover `MatchStart`, records JSON.
**Check:** tests 1–6 pass; `contract_protocol*` updated; `PROTOCOL_VERSION` + 1.
**Done when:** committed (`feat(p16f4): authoritative Stronghold selection`).

### 3. Persistence

Migration, save on select (accounts), load on login, guest
`localStorage`. Store tests: round-trip, NULL → default, old databases
migrate.
**Done when:** committed (`feat(p16f4): remember the Stronghold kind (BR-K01)`).

### 4. 3D kinds and Fortification overlays

Models, gallery entries, damage states from per-seat max, triangle counts.
**Check:** gallery screenshots of 4 kinds × {T0, Walls 2, Arms 2} at
desktop and phone size in `docs/media/p16f4/`; a match with two different
kinds.
**Done when:** committed (`feat(client): four Stronghold kinds and Fortification overlays`).

### 5. Lobby selection

Desktop HTML: a "Stronghold" panel beside the Arcana placeholder with four
cards (portrait, name, HP and damage as "+15 % HP / −15 % damage" and the
resulting numbers, one-line passive, a "Selected" state); the panel is
disabled with "Locked while searching" when queued. Bevy menu (phones and
portrait): a four-button row with the same information in a detail line.
Both call one `UiAction::SelectStronghold(kind)`.
**Check:** browser at 1920×1080, 1280×800, 390×844; select, queue
(locked), cancel (unlocked), reload (remembered).
**Done when:** committed (`feat(client): Stronghold selection in the lobby`).

### 6. Match HUD and Fortify control

- Both HP bars show the kind icon and name and use `stronghold_max`.
- Fortify: click/tap your own Stronghold (canvas pick, like units) opens a
  panel with the two branches (tier 1) or the next tier (tier 2), cost,
  effect line, disabled state live with Energy (the P16-A Evolution panel
  pattern: buttons keep identity and refresh from `LatestView`); desktop
  also shows a "Fortify" button on the Energy panel. Rejections reuse the
  toast. A `Fortified` event plays the existing evolve burst on the tower.
- Opponent Fortifications are visible on the model and as small tier pips
  under their HP bar.
**Check:** client tests for panel state functions; browser checks.
**Done when:** committed (`feat(client): Fortify panel and kind indicators`).

### 7. Practice parity

`local.rs` uses the stored kind and the seeded bot kind; the practice
match shows both at start.
**Done when:** committed (`feat(client): Stronghold kinds in practice`).

### 8. Owner checkpoint and switch

Show gallery captures, lobby and HUD captures, and a recorded match.
On approval (look + O-11): `fortify_enabled = true`, GAME-DESIGN §10.4/
§10.5 "active since P16-F4", goldens if canon changed, ARCHITECTURE §5/§8.
**Done when:** committed (`feat(p16f4): switch canon to Stronghold kinds and Fortifications`).

### 9. End-to-end and report

Two real clients (desktop + phone viewport) with different kinds finish a
ranked match on a local server; one Fortifies, one reconnects mid-match
(kinds and tiers intact), then a takeover from another tab. The record has
both kinds and the Fortification timeline. Fill in the report.
**Done when:** committed (`docs(p16f4): phase report`).

## Acceptance tests

Server (`tests/online.rs` and store tests):

1. `br_k02_selection_is_refused_while_queued_in_a_room_or_in_a_match`
2. `br_k02_queue_entry_freezes_the_kind_until_cancel_or_match_end`
3. `br_k02_reconnect_and_takeover_keep_the_match_kinds`
4. `br_k03_both_players_receive_both_kinds_in_match_start`
5. `br_k04_new_guest_gets_the_default_kind`
6. `classic_is_never_selectable_online`
7. `br_k01_account_kind_round_trips_and_old_databases_migrate`
8. `br_f04_fortify_online_is_applied_and_recorded` (after step 8)
9. Seat ownership: a `Fortify` always applies to the sender's seat (seat
   from the connection; no field to forge).

Client: label/state unit tests for the lobby card text and the Fortify
panel; JS syntax check for `lobby.js`/`hud.js` as in D-29.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cd crates/tbg-client && trunk build --release
```

Plus gallery screenshots at desktop and phone size, and the two-client
match with reconnect and takeover.

## Performance

Measured, reported: triangles per kind and overlay; desktop FPS in a busy
match with both towers at tier 2; phone-size FPS in the browser pane (real
phones are the owner's check, P30 matrix); WASM brotli size delta; lobby
first-load bytes (portraits lazy after menu, D-26); `MatchStart` bytes.

## Acceptance checklist

- [ ] Tests 1–9 pass; protocol bumped once
- [ ] Kinds remembered for accounts and guests; frozen correctly
- [ ] Four kinds and both branches distinguishable in captures
- [ ] Lobby and HUD work at desktop and phone sizes
- [ ] Owner look approval and switch recorded
- [ ] Docs: GAME-DESIGN, ARCHITECTURE §5/§6/§8, ART-DIRECTION (kind cues), roadmap

## Risks and what to do

| Problem | Action |
|---------|--------|
| Kinds unreadable at match distance | Silhouette first (height, width, top shape), color second; owner review in step 8 |
| Selection races with queue entry | Server snapshot at entry is the only source; client lock is presentation |
| Store latency delays the lobby | Fire-and-forget saves; ack does not wait for durability |
| Phone lobby too dense | Row of four icons + one detail line; full stats on tap |

## Reviewer checklist

- Try every message out of order (select while queued, during countdown,
  after takeover); nothing changes a running match.
- Records and `MatchStart` agree on kinds for both seats.
- No client code computes Stronghold stats for authority; displays may use
  canon helpers.

## Owner checkpoint

Step 1 (O-3) and step 8 (look approval, O-11 switch). After step 9: play a
few matches with each kind.

## Phase report

_Not executed._ Fill in: starting state; protocol diff; migration; captures
(gallery, lobby, HUD; desktop and phone); triangle counts; FPS and size;
two-client run with reconnect and takeover; owner decisions; deviations;
blocked items; limitations.
