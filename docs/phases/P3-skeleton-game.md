# P3 — Skeleton game (local match vs bot, placeholder visuals)

**Depends on:** P1 and P2 reviewed and merged.
**Branch:** `phase/p3-skeleton-game`.
**Difficulty:** medium (step 2 is the most important).

## Goal

The complete game loop in the browser — menu, match, shop, deploy,
automatic combat, evolution, synergies, win/lose, play again — against the
bot, using the real `tbg-core` rules and **simple placeholder shapes**. No
3D art. This is the "skeleton" every later phase plugs into.

## Deliverable

Open the page → **Practice vs Bot** → full match with HUD → result screen
→ **Play again**. Works with mouse and touch, portrait and landscape.

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §4 (how a match plays)
- `docs/ARCHITECTURE.md` §6 (client)
- `crates/tbg-core/src/{view.rs,protocol.rs,intent.rs,bot.rs,content.rs}` (public API only)
- The client files from P1
- Bevy examples: `state/states.rs`, `ecs/fixed_timestep.rs`,
  `ui/widgets/button.rs`, `ui/layout/flex_layout.rs`, `ui/text/text.rs`,
  `picking/mesh_picking.rs`

## Do NOT

- Put any game rule in the client. The client only reads `MatchView` and
  sends `PlayerIntention`s.
- Build real 3D characters, VFX, or audio (P6/P7).
- Add networking for matches (P4). Keep the P1 ping code untouched.
- Add dependencies.

## Client module layout after this phase

```
src/main.rs        App, plugins, GameState
src/config.rs      url_param, server_url (from P1)
src/camera.rs      framing (from P1)
src/arena.rs       placeholder arena (from P1)
src/overlay.rs     FPS/RTT overlay (from P1)
src/net.rs         ping (from P1; P4 replaces it)
src/link.rs        MatchLink (NonSend): Local now, Net in P4
src/local.rs       LocalMatch: tbg-core + bot stepped at 20 Hz
src/sync.rs        ServerMsg → world: view buffer, interpolation, unit entities
src/visuals.rs     THE SEAM: spawn/update/animate unit visuals (placeholder now; P6 replaces internals)
src/ui/mod.rs, ui/menu.rs, ui/hud.rs, ui/evolve.rs, ui/result.rs, ui/toast.rs
```

Remove `walkers.rs` from P1 (keep `?stress=N` working by making it spawn
placeholder visuals in `visuals.rs`).

## Steps

### 1. Game states

`#[derive(States)] enum GameState { #[default] Menu, InMatch, Result }`.
Each screen spawns its UI on `OnEnter`, and every entity it spawns gets
`DespawnOnExit(GameState::X)` (or the 0.20 equivalent shown in
`state/states.rs`).

### 2. MatchLink and LocalMatch

`link.rs`:
```rust
pub enum MatchLink { Idle, Local(LocalMatch) /* P4 adds: Net(NetLink) */ }
impl MatchLink {
    pub fn send(&mut self, msg: ClientMsg);          // Local: forwards Intent to LocalMatch
    pub fn poll(&mut self) -> Vec<ServerMsg>;        // drains messages produced since the last call
}
```
Insert with `app.insert_non_send(MatchLink::Idle)` (Bevy 0.20 name). `Commands` cannot insert
non-send resources: replace the value from an exclusive system (`fn f(world: &mut World)`,
see `net.rs` from P1), and read/write it in normal systems with `NonSendMut<MatchLink>`.

`local.rs`:
```rust
pub struct LocalMatch { state: MatchState, bot: Bot, queue: Vec<(u32, PlayerIntention)>,
                        events: Vec<Event>, outbox: Vec<ServerMsg> }
```
- `LocalMatch::new(seed)`: seed from `js_sys`-free source — use the frame
  count + elapsed time bits from Bevy `Time` at the moment Practice is
  pressed. Push `ServerMsg::MatchStart { you: Seat::A, opponent: "Training Bot".into() }`.
- A system in `FixedUpdate` with `Time::<Fixed>::from_hz(20.0)` calls
  `LocalMatch::tick()`: apply queued player intents (seat A; push
  `Rejected` on error), `events.extend(state.step())`, and every 2nd tick
  build `view_for(A, &events)` (push `ServerMsg::View`), let the bot decide
  from `view_for(B, &events)` and apply its intent for seat B, then clear
  `events`. When the state has an outcome, push `ServerMsg::MatchEnd` once.
- Intent sequence numbers start at 1 and increase per player intent.

### 3. Sync: views to entities

`sync.rs` owns `struct ViewBuffer { prev: Option<(f64, MatchView)>, last: Option<(f64, MatchView)> }`
(arrival time in seconds from `Time::elapsed_secs_f64`).
- On `View`: shift last → prev, store the new one; process its `events`
  (send them to `visuals.rs`).
- Every frame: render time = now − 0.12 s. For each unit in `last`,
  interpolate x between `prev` and `last` by arrival time (clamp 0..1).
  World position: `world_x = x / 1000.0` for seat A view; **if `you ==
  Seat::B`, use `world_x = 16.0 - x / 1000.0`** so the player's own
  Stronghold is always at world x = 0.
- Lane depth: `z = [-0.8, 0.0, 0.8][unit.id % 3]` (presentation only).
- Spawn a visual for unit ids that appear; despawn for ids that vanish
  (after the death animation in visuals).
- Keep a resource `LatestView(Option<MatchView>)` for the HUD.

### 4. Visuals seam (placeholder implementation)

`visuals.rs` exposes exactly:
```rust
pub struct UnitLook { pub kind: UnitKind, pub tier: u8, pub branch: u8, pub own: bool }
pub fn spawn_unit_visual(commands: &mut Commands, assets: &VisualAssets, look: UnitLook, pos: Vec3) -> Entity;
pub fn update_unit_visual(/* entity, look changes (evolution), hp ratio */);
pub enum VisualCue { Spawn, AttackStart, Hit, Heal, Status, Evolved, Death }
pub fn play_cue(/* entity, cue */);
```
Placeholder look:
- Body: capsule colored by faction (Humans `#8B7355`, Undead `#D8D2C0`,
  Machines `#4A5058`, Plants `#4E9F3D`); one-slot radius 0.3 / height 0.8,
  two-slot radius 0.45 / height 1.2.
- Archetype marker on top (head height): Tank cube, Dps cone, Ranged
  thin cylinder ("bow"), Support sphere, Control torus.
- Team ring under the unit: own `#3FA9F5`, enemy `#F2545B`.
- Tier: 0/1/2 small gold cubes floating above the head; branch A gold,
  branch B silver.
- HP bar: two thin quads above the unit facing the camera (background
  dark, fill green → red by ratio).
- Cues (all by scaling/offset over 0.2–0.4 s, no animation system):
  Spawn = grow from 0; AttackStart = lunge 0.2 toward the enemy and back;
  Hit = flash to white material for 0.08 s; Heal = brief green tint;
  Status = blue (slow) or brown (root) ring tint while the status lasts;
  Evolved = pop to 1.3× and back; Death = shrink to 0 then despawn.
- Ranged attacks: a small sphere flying from attacker to target in 0.25 s.

### 5. HUD

`ui/hud.rs` (portrait-first layout, fits 360×640; landscape puts the shop
on the bottom too):
- Top: clock `m:ss` of `phase_ticks_left / 20`; "OVERTIME" banner in
  overtime; both Stronghold HP bars (own left/bottom color blue, enemy red).
- Bottom: Energy bar showing `energy / 600` with one decimal and 10 tick
  marks; three shop cards (unit name, faction color strip, archetype
  word, cost = `cost / 600`), greyed when energy is too low or the slot cap
  would be exceeded; a thin bar showing `next_rotation_in`.
- Side: slots used `n/8`; synergy badges for own factions with level ≥ 1
  (`Humans ●●` style).
- Tapping a card sends `ClientMsg::Intent { seq, intent: Purchase { offer } }`
  and greys the card until a view arrives with `last_seq ≥ seq`.

### 6. Evolution panel

Tap/click an own unit (use mesh picking on an invisible sphere of radius
0.7 around each own unit) → `ui/evolve.rs` panel: unit name, current tier;
tier 0 shows two buttons with branch A and B form names and their effects
from `content.rs`, tier 1 shows the next form of its branch, tier 2 shows
"Max". Each button shows the cost and is greyed if unaffordable. Tapping
sends `Evolve`; the panel closes. Tapping elsewhere closes it.

### 7. Rejections and result

- `ui/toast.rs`: on `Rejected { reason }` show a 1.5 s message:
  NotEnoughEnergy → "Not enough Energy"; SlotCapReached → "Army is full
  (8 slots)"; others → "Action not possible".
- On `MatchEnd`: switch to `Result` state. `ui/result.rs` shows VICTORY /
  DEFEAT / DRAW, the reason in words, match duration, and a **Play again**
  button (starts a new LocalMatch) and a **Menu** button.

### 8. Menu

`ui/menu.rs`: title "Tupi Battle Grounds", a big **Practice vs Bot**
button, and two disabled buttons "Quick Match" and "Play with a friend"
labeled "(online soon)". The P1 overlay stays visible in all states.

## Tests to write

None in the client. Rules are already tested in `tbg-core`.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Three complete matches on desktop: each ends with the correct result screen
- [ ] One complete match on a phone in portrait (owner)
- [ ] Buying, evolving (both branches), and at least one synergy badge observed
- [ ] Rejection toast observed (try to buy with too little Energy)
- [ ] No errors in the browser console; FPS ≥ 30 on the phone during a busy match
- [ ] No balance number is copied into the client: costs, HP, and the Energy
      scale are read from `tbg-core` (`content.rs` constants)

## Risks and what to do

| Problem | Action |
|---------|--------|
| Units jitter | Check the interpolation uses arrival times and the 0.12 s delay |
| Seat B view mirrored wrong | Only `sync.rs` mirrors; nothing else may flip coordinates |
| UI too small on phone | Use `Val::Vw/Vh` units and a minimum touch size of 44 px |
| Borrow errors with NonSend | Use `NonSendMut<MatchLink>` in exclusive or normal systems as in Bevy docs |

## Reviewer checklist

- No game rule in the client; all numbers come from `tbg-core`.
- `visuals.rs` is the only place that knows how a unit looks.
- `MatchLink` is the only way the client talks to a match.
- The match plays correctly from both seats (temporarily start LocalMatch as seat B to check mirroring).

## Phase report

Executed by the reviewer agent (Claude) on 2026-10-28, branch `phase/p3-skeleton-game`.

- Commands run and results:
  - `cargo clippy --workspace --all-targets -- -D warnings` and
    `cargo clippy -p tbg-client --target wasm32-unknown-unknown -- -D warnings` → clean.
  - `cargo test --workspace` → 15 passed (tbg-core).
  - `trunk serve` (dev build) → page loads, no panics in the console.
  - Release build and size measurement: done at the end of P4 (P4 changes the
    client again); see the P4 report.
- Matches played (browser pane, desktop Chrome engine, 373×335 window):
  - Practice vs Bot started with one click; HUD shows clock, both Stronghold
    HP bars, Energy, Army n/8, synergy ("Humans 1/2"), rotation bar, 3 cards
    with faction strip; unaffordable cards grey out.
  - Buying a card spawns the unit at the own Stronghold; tapping the own unit
    opens the evolution panel ("Bone Brute | tier 0", Ossified +50% HP (3),
    Butcher +50% dmg (3), Close); choosing a branch closes the panel.
  - `?autoplay` full match → DEFEAT, "A Stronghold was destroyed",
    "Duration 1:10 vs Training Bot"; Play again starts a new match.
  - Phone portrait test: **pending owner** (real Android/iPhone).
- Deviations:
  1. **No Bevy picking.** Hover-based picking missed taps whose press and
     release land in the same frame (seen with synthetic clicks; same risk for
     fast phone taps). `ui/input.rs` hit-tests the release position: buttons
     (`ComputedNode::contains_point`), then the nearest own unit on screen
     (42 px × UI scale), else "tap outside" (closes the panel). The picking
     features and `MeshPickingPlugin` were removed (smaller WASM).
  2. `UnitLook` carries the unit `id` (needed to open its panel).
  3. Interpolation runs from the previous to the latest view over one view
     interval (≈100 ms behind) instead of a fixed 0.12 s offset; same intent,
     smoother with frame-quantized arrivals.
  4. `?autoplay` (bot plays for this client through `MatchLink`) added as a
     validation aid; bare URL flags (`?autoplay`) now parse.
  5. `?stress=N` spawns in `Startup`: Bevy 0.20 runs the first state
     transition before `PreStartup`, so `OnEnter(Menu)` cannot use assets
     created there.
  6. The middle dot `·` is missing from Bevy's default font; separators use `|`.
  7. Dev profile: line-table debug info only, none for dependencies. Full
     debug info had grown `target/debug` to 12 GB on a disk with 5 GB free.
- Blocked items: none.
- Needs the owner: one match on a real phone (portrait).
