# P18 — Strongholds: visual upgrade and three selectable styles

**Depends on:** P17 reviewed.
**Branch:** `phase/p18-strongholds`.
**Difficulty:** medium.

## Goal

The two bases become the visual anchors of every match: detailed medieval
fortifications that defend themselves visibly, crumble in stages, and
collapse at defeat. Each player picks one of three free styles — Stone
Keep, Timber Fort, Gatehouse — and the opponent sees it. Style is purely
cosmetic (BR-U05, BR-C04).

## Deliverable

- Three Stronghold models (≤ 4 000 triangles each) with team banners, a
  defender figure, 4 damage states, hit shake, and a collapse sequence.
- `StrongholdStyle` chosen in the lobby, stored on the account (guests:
  on the device), sent to both players in `MatchStart`.
- Practice mode uses the player's style and a random style for the bot.

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §2, §3, §6, §9
- `docs/GAME-DESIGN.md` BR-U05, BR-C04 and glossary `StrongholdStyle`
- Core: `src/protocol.rs`, `src/types.rs`; Server: `src/lobby.rs`, `src/store/*`
- Client: `src/arena/` (from P17), `src/sync/messages.rs`, `src/local.rs`, `src/ui/menu.rs`

## Do NOT

- Change Stronghold HP, range, damage, position, or hitbox.
- Lock any style behind progress, accounts, or payment.

## Steps

### 1. Core type and protocol v5
- `types.rs`: `pub enum StrongholdStyle { StoneKeep, TimberFort, Gatehouse }`
  (`Default` = `StoneKeep`), serde derives.
- `protocol.rs`: `PROTOCOL_VERSION = 5`; append
  `ClientMsg::SetStrongholdStyle { style: StrongholdStyle }`; change
  `MatchStart` to add `styles: [StrongholdStyle; 2]` (indexed by `Seat::index()`).
- **Check:** `cargo test -p tbg-core` (protocol round-trip test updated).

### 2. Server
- Session field `style` (default `StoneKeep`); `SetStrongholdStyle` sets it
  any time except in a match (ignored there). Logged-in accounts persist
  it: store migration v4 `ALTER TABLE accounts ADD COLUMN stronghold_style
  INTEGER NOT NULL DEFAULT 0`; `Profile` gains `stronghold_style`; on
  `LoggedIn` the session style becomes the stored one.
- Match start sends both styles. Integration test: two clients with
  different styles see `styles` in the right seat order.

### 3. Models
`arena/strongholds/` with `keep.rs`, `fort.rs`, `gatehouse.rs`,
`common.rs` (banners, defender, damage-state logic):
- **Stone Keep:** round tower, crenellations, arrow slits, wooden hoarding,
  conical roof with banner.
- **Timber Fort:** palisade ring of sharpened logs, wooden watchtower,
  shields hung on the wall, torches.
- **Gatehouse:** twin square towers, portcullis arch, flag line, braziers.
- Team color on banners, flags, and the defender's tabard.
- Damage states at 75/50/25 % HP: missing crenellations and planks →
  cracks and tilted parts → broken top, fire and smoke. State changes
  spawn a debris burst (simple cubes now; P21 replaces them).

### 4. Life
- Defender on top (rig v2 Humanoid, crossbowman) plays a shoot clip on the
  Stronghold's `AttackStart` toward its target; bolt projectile.
- On `Hit` to the Stronghold: 120 ms shake (±0.04 world units), dust puff.
- Defeat: 1.6 s collapse — top pieces fall with simple ballistic motion,
  dust cloud, banner falls; the camera does not move.

### 5. Selection UI (temporary until P20)
Menu → **Stronghold** button → three cards (rendered previews, name) →
select sends `SetStrongholdStyle` and stores the choice in `localStorage`
(`tbg_stronghold_style`) for guests (sent after `Welcome`).

### 6. Gallery
`?gallery=strongholds`: three styles × four damage states, plus the
collapse on loop.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Style integration test passes; online match shows each player's style
- [ ] ≤ 4 000 triangles per style (log); FPS unchanged within 5 % vs P17
- [ ] Screenshots in `docs/media/p18/`; owner approves the three styles

## Reviewer checklist

- No gameplay field depends on `StrongholdStyle`.
- Opponent's style comes from the server, never from local storage.

## Phase report

_Executor fills this in._
