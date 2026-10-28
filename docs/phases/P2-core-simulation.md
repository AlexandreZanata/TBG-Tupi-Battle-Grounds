# P2 — Core simulation (`tbg-core`)

**Depends on:** P0. **May run in parallel with:** P1.
**Branch:** `phase/p2-core-simulation`.
**Difficulty:** high (steps 1, 6). Follow the types and algorithms below
exactly; they are the contract the client and server will rely on.

## Goal

Every MVP rule from `docs/GAME-DESIGN.md` §3–§5 as a deterministic,
headless Rust library with focused tests, a bot, the wire protocol, and a
bot-vs-bot balance harness.

## Deliverable

`cargo run --release -p tbg-core --example balance -- 1000` prints faction
win rates, median duration, end-reason rates, purchase share per unit,
evolutions per match, and mean/max step time.

## Read first (only these)

- `AGENTS.md`
- `docs/GAME-DESIGN.md` §2–§5
- `docs/ARCHITECTURE.md` §3 (core row), §4, §5 (messages)
- `crates/tbg-core/src/lib.rs`, `crates/tbg-core/src/protocol.rs`

## Do NOT

- Use floats (`f32`/`f64`) anywhere in `src/` (examples may use them for printing).
- Use `HashMap`/`HashSet`, threads, time, I/O, or `rand`.
- Add dependencies other than `serde` (features `derive`) and `postcard`
  (features `alloc`, `default-features = false`).
- Change any number. All numbers are in the tables below.

## Units and constants (integers only)

| Concept | Unit | Value |
|---------|------|-------|
| Tick | 1 tick = 50 ms | `TICK_HZ = 20` |
| Position | milli-tiles (`i32`) | `LANE_LEN = 16_000`; seat 0 Stronghold at `0`, seat 1 at `16_000` |
| Spawn position | milli-tiles | seat 0: `500`; seat 1: `15_500` |
| Energy | sub-energy (`u32`), `ENERGY_SCALE = 600` per 1 Energy | start `3_000`, cap `6_000` |
| Energy regen per tick | sub-energy | `20` (Running), `40` (Overtime) |
| Phases (ticks) | | `COUNTDOWN = 60`, `REGULATION = 3_600`, `OVERTIME = 1_200` |
| Shop | | `OFFERS = 3`, `ROTATION_TICKS = 200` |
| `SLOT_CAP` | slots | `8` |
| Stronghold | | HP `2_500`, damage `40`, every `20` ticks, range `4_000` |
| Attack windup | ticks | `WINDUP = 6` |
| Evolution cost | sub-energy | tier 1 `1_800`, tier 2 `2_400` |
| Percent modifiers | basis points (bp), `10_000` = 100% | |

Phase boundaries: Countdown for ticks `0..60`; Running `60..3_660`;
Overtime `3_660..4_860`; the match is Ended by tick `4_860` at the latest.

## Content table (put in `content.rs` exactly)

Base stats. `interval` and windup are in ticks; `range`, `speed` (per
tick), `splash` in milli-tiles; `cost` in sub-energy.

| Kind | Faction | Archetype | Slots | Cost | HP | Damage | Interval | Range | Speed | Splash | Heal | On hit |
|------|---------|-----------|-------|------|----|--------|----------|-------|-------|--------|------|--------|
| Shieldbearer | Humans | Tank | 1 | 1800 | 600 | 30 | 24 | 800 | 40 | 0 | 0 | — |
| Archer | Humans | Ranged | 1 | 1800 | 220 | 45 | 20 | 4500 | 50 | 0 | 0 | — |
| BoneBrute | Undead | Tank | 2 | 3000 | 1300 | 60 | 30 | 1000 | 30 | 0 | 0 | — |
| Hexer | Undead | Control | 1 | 2400 | 260 | 25 | 28 | 3500 | 50 | 0 | 0 | Slow 4000 bp, 40 ticks |
| Sawbot | Machines | Dps | 1 | 1800 | 300 | 70 | 18 | 800 | 65 | 0 | 0 | — |
| SiegeWalker | Machines | Ranged | 2 | 3600 | 700 | 120 | 50 | 6000 | 30 | 1500 | 0 | — |
| Bloomkeeper | Plants | Support | 1 | 1800 | 280 | 0 | 30 | 3000 | 45 | 0 | 50 | — |
| Rootcaller | Plants | Control | 1 | 2400 | 300 | 20 | 32 | 3000 | 45 | 0 | 0 | Root 20 ticks |

Evolution forms. Branch A = `0`, B = `1`. Columns are additive
modifiers; `on_hit` **replaces** the unit's on-hit effect when present.
Modifiers of tier 1 and tier 2 of the same branch are summed.

| Kind | Form (tier, branch) | hp bp | dmg bp | atk speed bp | speed bp | range + | splash + | heal bp | dmg + | on_hit |
|------|--------------------|-------|--------|--------------|----------|---------|----------|---------|-------|--------|
| Shieldbearer | Bulwark (1,A) | 5000 | | | | | | | | |
| Shieldbearer | Fortress (2,A) | 5000 | | | | | | | | Slow 3000 bp, 30 |
| Shieldbearer | Vanguard (1,B) | | 4000 | | 2000 | | | | | |
| Shieldbearer | Champion (2,B) | | 4000 | | | | 1000 | | | |
| Archer | Longbow (1,A) | | 2000 | | | 1500 | | | | |
| Archer | Marksman (2,A) | | 6000 | | | | | | | |
| Archer | Volley (1,B) | | | 4000 | | | | | | |
| Archer | Arrow Rain (2,B) | | | | | | 1000 | | | |
| BoneBrute | Ossified (1,A) | 5000 | | | | | | | | |
| BoneBrute | Bone Wall (2,A) | 5000 | | | | | | | | Slow 3000 bp, 30 |
| BoneBrute | Butcher (1,B) | | 5000 | | | | | | | |
| BoneBrute | Reaper (2,B) | | 3000 | | | | 1200 | | | |
| Hexer | Withering (1,A) | | | | | | | | | Slow 6000 bp, 40 |
| Hexer | Doom (2,A) | | 6000 | | | | | | | |
| Hexer | Plague (1,B) | | | | | | 1000 | | | |
| Hexer | Pestilence (2,B) | | 4000 | | | | 500 | | | |
| Sawbot | Overclock (1,A) | | | 4000 | | | | | | |
| Sawbot | Shredder (2,A) | | | 4000 | 3000 | | | | | |
| Sawbot | Plating (1,B) | 6000 | | | | | | | | |
| Sawbot | Juggernaut (2,B) | 6000 | | | | | 800 | | | |
| SiegeWalker | Longshot (1,A) | | | | | 2000 | | | | |
| SiegeWalker | Artillery (2,A) | | 5000 | | | | | | | |
| SiegeWalker | Shrapnel (1,B) | | | | | | 1000 | | | |
| SiegeWalker | Concussion (2,B) | | | | | | | | | Root 16 |
| Bloomkeeper | Blossom (1,A) | | | | | | | 5000 | | |
| Bloomkeeper | Grove (2,A) | | | | | | 1500 | | | |
| Bloomkeeper | Bramble (1,B) | 5000 | | | | | | | 25 | |
| Bloomkeeper | Thornwarden (2,B) | | | | | | | | | Root 12 |
| Rootcaller | Deep Roots (1,A) | | | | | | | | | Root 32 |
| Rootcaller | Strangle (2,A) | | 8000 | | | | | | | |
| Rootcaller | Spores (1,B) | | | | | | 1000 | | | |
| Rootcaller | Overgrowth (2,B) | | | | | 1000 | | | | Root 28 |

Synergies (living units of that faction owned by the same seat; a
two-slot unit counts as one): level 1 at ≥ 2 units, level 2 at ≥ 4.

| Faction | Level 1 | Level 2 |
|---------|---------|---------|
| Humans | hp +1500 bp | hp +3500 bp |
| Undead | atk speed +1500 bp | atk speed +3500 bp |
| Machines | dmg +1500 bp | dmg +3500 bp |
| Plants | regen 100 bp of max HP every 20 ticks | regen 250 bp every 20 ticks |

## Effective stats formula (`content.rs` or `combat.rs`)

Sum bp from the unit's forms and its seat's active synergy, then (integer
division, floor):

```
max_hp   = base_hp * (10_000 + hp_bp) / 10_000
damage   = (base_dmg + dmg_add) * (10_000 + dmg_bp) / 10_000
interval = max(5, base_interval * 10_000 / (10_000 + atk_bp))
speed    = base_speed * (10_000 + speed_bp) / 10_000
range    = base_range + range_add
splash   = base_splash + splash_add
heal     = base_heal * (10_000 + heal_bp) / 10_000
on_hit   = tier-2 on_hit, else tier-1 on_hit, else base on_hit
```

When `max_hp` changes for a living unit (evolution, synergy change):
`hp = max(1, hp * new_max / old_max)` using `i64` for the multiplication.

## Types (create exactly these public names)

```rust
// types.rs
pub type Tick = u32;
pub type UnitId = u32;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Seat { A, B }            // A = seat 0 (x = 0 side), B = seat 1
impl Seat { pub fn index(self) -> usize; pub fn other(self) -> Seat; pub fn dir(self) -> i32 /* A: +1, B: -1 */ }
// Keep the existing Archetype, Faction, SlotCount (add serde derives).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitKind { Shieldbearer, Archer, BoneBrute, Hexer, Sawbot, SiegeWalker, Bloomkeeper, Rootcaller }
pub const UNIT_KINDS: [UnitKind; 8];
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnHit { Slow { bp: u16, ticks: u16 }, Root { ticks: u16 } }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status { None, Slowed { bp: u16, ticks_left: u16 }, Rooted { ticks_left: u16 } }

// state.rs
pub enum Phase { Countdown, Running, Overtime, Ended }
pub struct Unit { pub id: UnitId, pub seat: Seat, pub kind: UnitKind, pub tier: u8, pub branch: u8,
                  pub x: i32, pub hp: i32, pub max_hp: i32, pub cooldown: u16,
                  pub windup: Option<(Target, u16)>, pub status: Status }
pub enum Target { Unit(UnitId), Stronghold(Seat) }
pub struct PlayerState { pub energy: u32, pub offers: [UnitKind; 3], pub stronghold_hp: i32,
                         pub stronghold_cooldown: u16, pub synergy: [u8; 4] /* level per Faction index */,
                         pub last_seq: u32, pub rng: Pcg32 }
pub struct MatchState { pub tick: Tick, pub phase: Phase, pub players: [PlayerState; 2],
                        pub units: Vec<Unit> /* always sorted by id */, pub next_unit_id: UnitId,
                        pub outcome: Option<Outcome> }
pub struct Outcome { pub winner: Option<Seat> /* None = draw */, pub reason: EndReason }
pub enum EndReason { StrongholdDestroyed, TimeUp, Forfeit }

// intent.rs
pub enum PlayerIntention { Purchase { offer: u8 }, Evolve { unit: UnitId, branch: u8 } }
pub enum Rejection { NotRunning, BadOffer, NotEnoughEnergy, SlotCapReached, NotYourUnit, UnitGone, MaxTier, WrongBranch }

// events (state.rs)
pub enum Event { Spawn { unit: UnitId, seat: Seat, kind: UnitKind }, AttackStart { from: Target, to: Target },
                 Hit { to: Target, amount: i32 }, Heal { to: UnitId, amount: i32 },
                 StatusApplied { unit: UnitId }, Death { unit: UnitId },
                 Evolved { unit: UnitId, tier: u8, branch: u8 },
                 SynergyChanged { seat: Seat, faction: Faction, level: u8 }, PhaseChanged { phase: Phase } }
```

All of these derive `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`
(`Copy` where they are small). `MatchState` methods:

```rust
pub fn new(seed: u64) -> MatchState;
pub fn apply_intention(&mut self, seat: Seat, seq: u32, intent: PlayerIntention) -> Result<(), Rejection>;
pub fn step(&mut self) -> Vec<Event>;          // advances exactly one tick
pub fn forfeit(&mut self, seat: Seat);          // sets outcome: other seat wins, Forfeit; phase Ended
pub fn view_for(&self, seat: Seat, events: &[Event]) -> MatchView;
pub fn hash(&self) -> u64;                      // FNV-1a 64 over postcard::to_allocvec(self)
```

## Algorithms

**PCG32** (`rng.rs`, copy exactly):

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pcg32 { state: u64, inc: u64 }
impl Pcg32 {
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut r = Pcg32 { state: 0, inc: (stream << 1) | 1 };
        r.next_u32();
        r.state = r.state.wrapping_add(seed);
        r.next_u32();
        r
    }
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(6364136223846793005).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        xorshifted.rotate_right((old >> 59) as u32)
    }
    pub fn below(&mut self, n: u32) -> u32 { self.next_u32() % n }
}
```

`MatchState::new(seed)`: seat A rng = `Pcg32::new(seed, 1)`, seat B rng =
`Pcg32::new(seed, 2)`; each seat draws 3 offers (`UNIT_KINDS[rng.below(8)]`);
energy 3 000; Strongholds 2 500 HP; phase Countdown; tick 0.

**`apply_intention`** (called by the caller before `step`). Always set
`players[seat].last_seq = seq` first, then validate in this order:
- `Purchase{offer}`: phase must be Running or Overtime → else `NotRunning`;
  `offer < 3` → else `BadOffer`; energy ≥ cost → else `NotEnoughEnergy`;
  slots used by living own units + kind slots ≤ 8 → else `SlotCapReached`.
  Then: subtract energy; create the unit (id = `next_unit_id`, increment;
  x = spawn position; hp = max_hp from effective stats; cooldown 0); push
  to `units`; replace that offer with `UNIT_KINDS[rng.below(8)]`; queue
  `Event::Spawn` to be returned by the next `step()`.
- `Evolve{unit, branch}`: phase Running/Overtime → else `NotRunning`; unit
  exists → else `UnitGone`; unit seat == seat → else `NotYourUnit`;
  tier 2 → `MaxTier`; tier 0 needs `branch ≤ 1`, tier 1 needs
  `branch == unit.branch` → else `WrongBranch`; energy ≥ cost (1 800 for
  tier 0→1, 2 400 for 1→2) → else `NotEnoughEnergy`. Then subtract,
  set tier/branch, recompute max_hp preserving the HP ratio, queue
  `Event::Evolved`.

Store events created by `apply_intention` in a private
`pending: Vec<Event>` field (`#[serde(skip)]`); `step()` returns them first.

**`step()` order** (each tick, `tick` is incremented at the very end):
1. If phase is Ended: return pending events and do nothing else.
2. Phase transitions by tick: at tick 60 → Running; at 3 660 → compare
   Stronghold HP: different → Ended with the higher HP as winner,
   `TimeUp`; equal → Overtime; at 4 860 → same comparison, equal → draw
   (`winner: None`, `TimeUp`). Emit `PhaseChanged`.
3. Energy: Running +20, Overtime +40, capped at 6 000.
4. Shop: when Running/Overtime and `(tick - 60) % 200 == 0` and
   `tick > 60`, redraw all 3 offers for both seats (seat A first).
5. Synergy: count living units per seat per faction; compute levels;
   when a level changes, emit `SynergyChanged` and recompute max_hp of
   that faction's units (ratio rule). Plants regen: every 20 ticks, each
   Plants unit heals `max_hp * bp / 10_000` (not above max_hp).
6. Statuses: decrement `ticks_left`; at 0 set `Status::None`.
   Decrement every unit's `cooldown` if > 0. Decrement Stronghold cooldowns.
7. Units, in ascending id order, skipping dead ones (hp ≤ 0):
   - If `Rooted`: clear `windup`, skip the unit.
   - Find the target: Support with heal > 0 first looks for the living
     ally (own seat, including itself) with the lowest `hp * 10_000 / max_hp`
     below 10 000 within `range` (|dx| ≤ range); ties → lowest id.
     Otherwise (and for everyone else, and for Support with damage > 0
     when no ally needs healing) the target is the nearest living enemy
     unit by |dx|, ties → lowest id, or the enemy Stronghold if it is
     nearer or no enemy unit exists. A target is *in range* when
     |dx| ≤ range (Stronghold x is 0 or 16 000).
   - A Support whose damage is 0 and that has no ally to heal never
     attacks: if an enemy is in range it stands still (no windup);
     otherwise it moves.
   - If a windup is active: decrement it; when it reaches 0, resolve the
     hit (below) if the target is still alive, then set
     `cooldown = effective_interval - WINDUP` (min 0; if Slowed, first
     multiply interval by `10_000 / (10_000 - bp)`); clear windup.
   - Else if a target is in range: if `cooldown == 0`, start windup
     `(target, WINDUP)` and emit `AttackStart`. Do not move.
   - Else (nothing in range): move `x += dir * speed` (if Slowed,
     `speed * (10_000 - bp) / 10_000`), clamped to `0..=16_000`.
8. Resolving a hit: for an enemy target, subtract `damage` and emit
   `Hit`; for an ally target (Support), add `heal` capped at max_hp and
   emit `Heal`. If `splash > 0`, also
   damage every *other* enemy (units and the enemy Stronghold) with
   |x − target_x| ≤ splash (heals for Support: every other damaged ally in
   radius). Apply `on_hit` to the primary unit target only: Root replaces
   anything; Slow replaces Slow only if its bp is higher, otherwise
   refreshes `ticks_left` to the larger value; Slow never replaces Root.
   Emit `StatusApplied`.
9. Strongholds: for each seat whose cooldown is 0, if the nearest enemy
   unit is within 4 000 of its Stronghold x, deal 40 damage to it, emit
   `AttackStart{from: Stronghold(seat)}` and `Hit`, set cooldown 20.
10. Remove units with hp ≤ 0 (emit `Death` for each, ascending id).
11. Outcome: if a Stronghold HP ≤ 0: both → draw, one → the other seat
    wins (`StrongholdDestroyed`); set phase Ended.
12. `tick += 1`; return all events (pending first).

## Steps

1. **State skeleton** — `types.rs`, `rng.rs`, `state.rs`, `sim.rs` with
   `new`, an empty `step` doing items 1, 2, 12, and `hash`. Move the
   existing enums from `lib.rs` into `types.rs`; `lib.rs` only declares
   modules and re-exports. Add serde/postcard dependencies.
   *Done when* a test calls `step()` on an empty match until
   `outcome.is_some()` and gets a draw with reason `TimeUp` after exactly
   4 861 calls (the check runs when `tick == 4 860`).
2. **Content tables** — `content.rs` with `UnitDef`, `FormDef`, `SynergyDef`,
   the three tables above, the constants, and
   `fn effective_stats(kind, tier, branch, synergy_level) -> Stats`.
   *Done when* no other file contains a balance number.
3. **Energy, phases, shop** — items 2–4, plus initial offers.
4. **Intentions** — `intent.rs` with `apply_intention`.
5. **Synergy** — `synergy.rs`, item 5.
6. **Combat** — `combat.rs`, items 6–10. Keep functions under 80 lines:
   split into `find_target`, `advance_unit`, `resolve_hit`,
   `apply_on_hit`, `stronghold_fire`, `remove_dead`.
7. **Outcome** — item 11 and `forfeit`.
8. **View** — `view.rs`:
   ```rust
   pub struct MatchView { pub tick: Tick, pub phase: Phase, pub phase_ticks_left: u32, pub you: Seat,
       pub energy: u32, pub offers: [UnitKind; 3], pub next_rotation_in: u32, pub slots_used: u8,
       pub stronghold_hp: [i32; 2], pub units: Vec<UnitView>, pub synergy: [[u8; 4]; 2],
       pub last_seq: u32, pub events: Vec<Event>, pub outcome: Option<Outcome> }
   pub struct UnitView { pub id: UnitId, pub seat: Seat, pub kind: UnitKind, pub tier: u8, pub branch: u8,
       pub x: i32, pub hp: i32, pub max_hp: i32, pub status: u8 /* 0 none, 1 slowed, 2 rooted */ }
   ```
   `energy`, `offers`, `next_rotation_in`, `slots_used`, `last_seq` are
   for `you` only. Never include the rng or the opponent's energy/offers.
9. **Protocol** — replace `protocol.rs` with:
   ```rust
   pub const PROTOCOL_VERSION: u16 = 1;
   pub const MAX_CLIENT_MSG_BYTES: usize = 1024;
   pub type Token = [u8; 16];
   pub type RoomCode = [u8; 5];   // uppercase letters/digits from "ABCDEFGHJKMNPQRSTUVWXYZ23456789"
   pub enum ClientMsg { Hello { version: u16, resume: Option<Token> }, QuickMatch, CreateRoom,
       JoinRoom { code: RoomCode }, Cancel, Intent { seq: u32, intent: PlayerIntention }, Pong { id: u32 } }
   pub enum ServerMsg { Welcome { token: Token, name: String }, VersionMismatch { server: u16 },
       Queued, RoomCreated { code: RoomCode }, RoomNotFound,
       MatchStart { you: Seat, opponent: String }, View(MatchView),
       Rejected { seq: u32, reason: Rejection }, MatchEnd { outcome: Outcome },
       Ping { id: u32 }, NetStats { rtt_ms: u16 }, Error { code: ErrorCode } }
   pub enum ErrorCode { BadMessage, RateLimited, TooManyConnections, NotInMatch, ServerFull }
   pub fn encode<T: Serialize>(msg: &T) -> Vec<u8>;                 // postcard::to_allocvec
   pub fn decode_client(bytes: &[u8]) -> Result<ClientMsg, DecodeError>; // reject len > MAX_CLIENT_MSG_BYTES
   pub fn decode_server(bytes: &[u8]) -> Result<ServerMsg, DecodeError>;
   pub struct DecodeError;
   ```
10. **Bot** — `bot.rs`: `pub struct Bot { rng: Pcg32, wait: u32 }`,
    `Bot::new(seed: u64)`, `fn decide(&mut self, view: &MatchView) -> Option<PlayerIntention>`.
    Called once per view (10 per second). If `wait > 0`: decrement, return
    None. Otherwise set `wait = 3 + rng.below(6)` and choose:
    (a) if no own Tank alive and an affordable Tank offer fits the slot
    cap → buy it; (b) else buy the most expensive affordable offer that
    fits if energy ≥ cost + `rng.below(3) * 600`; (c) else if energy ≥ 4 800
    or the slot cap is full, evolve the own living unit with the lowest
    id and tier < 2 (branch: tier 0 → `rng.below(2)`, tier 1 → same branch)
    if affordable; (d) else None.
11. **Tests** (`crates/tbg-core/tests/rules.rs`, 10–15 tests, each naming
    the rule ID it protects in a comment):
    `empty_match_ends_in_draw_at_time_up` (BR-M05) ·
    `same_seed_and_script_same_hash` (D-04; run 2 bot matches with seed 42 → equal `hash()`) ·
    `energy_regenerates_and_caps` (BR-E02) ·
    `overspend_is_rejected_and_energy_unchanged` (BR-E03) ·
    `purchase_rejected_during_countdown` · `slot_cap_rejects_purchase` (BR-U06) ·
    `purchased_offer_refills_and_rotation_is_shared` (BR-S02, BR-S04) ·
    `evolution_branch_gating` (BR-V01: tier-2 with other branch → `WrongBranch`; third evolve → `MaxTier`) ·
    `evolve_other_seats_unit_rejected` (BR-A02) ·
    `synergy_levels_at_two_and_four_and_drop_on_death` (BR-Y01) ·
    `stronghold_destroyed_wins` (BR-M04) · `forfeit_gives_win_to_other_seat` (BR-M03) ·
    `protocol_roundtrip_and_garbage_rejected` (BR-A04) ·
    `view_hides_opponent_energy_and_offers` (BR-S05).
12. **Balance harness** — `examples/balance.rs`: args `[N=200]`. For
    `i in 0..N`: `MatchState::new(i)`, bots with seeds `2i+1`, `2i+2`;
    loop: every 2nd tick build both views (with the events gathered since
    the last view), let each bot decide, apply intents, then `step`, until
    Ended. Measure each `step()` with `std::time::Instant`. Print: matches,
    win rate by the faction most bought by the winner, median and p90
    duration in seconds, % ended by Stronghold / time / draw, purchase share
    per unit kind, mean evolutions per match, mean and max step µs, max
    units alive. Exit with code 1 if any match did not end.

## Acceptance checklist

- [ ] `cargo test -p tbg-core` passes (paste the summary line)
- [ ] `cargo run --release -p tbg-core --example balance -- 1000` finishes; paste its output
- [ ] Mean step time ≤ 50 µs
- [ ] `grep -rnE "f32|f64|HashMap" crates/tbg-core/src` prints nothing
- [ ] `cargo check -p tbg-core --target wasm32-unknown-unknown` passes
- [ ] clippy clean; no file in `src/` over 200 lines except `content.rs`

Balance numbers are **not** expected to hit GAME-DESIGN §6 targets yet;
only report them.

## Risks and what to do

| Problem | Action |
|---------|--------|
| Matches never end (units stuck) | Print the first stuck match's units each 200 ticks; most likely a range/target bug in step 7 |
| `hash()` differs between runs | Something iterates in non-deterministic order or uses uninitialized/pending data; check `#[serde(skip)]` fields |
| A rule seems ambiguous | Choose the simplest reading, write it under Deviations |

## Reviewer checklist

- Tick order matches the list exactly; ids are processed in ascending order.
- Integer math everywhere; `i64` used where `hp * max` could overflow.
- View redaction: opponent energy/offers/rng never leave `MatchState`.
- Tests assert rules (with IDs), not internals.
- Harness output is plausible (median duration 1–5 min, no 100% draws).

## Phase report

Executed by the reviewer agent (Claude) on 2026-10-27, branch `phase/p2-core-simulation`.

- Commands run and results:
  - `cargo test -p tbg-core` → `test result: ok. 15 passed; 0 failed`
    (14 rule tests listed in step 11 plus `time_up_higher_stronghold_wins`).
  - `cargo clippy --workspace --all-targets -- -D warnings` → clean.
  - `cargo check -p tbg-core -p tbg-client --target wasm32-unknown-unknown` → ok.
  - `grep -rnE "f32|f64|HashMap" crates/tbg-core/src` → nothing.
  - Largest files: `content.rs` 274 (table, exempt), `state.rs` 170, `types.rs` 138.
- Balance harness output (`cargo run --release -p tbg-core --example balance -- 1000`):
  ```
  matches: 1000
  duration s: median 93 p90 139
  ended by: stronghold 97.7% | time 2.3% | forfeit 0.0%
  draws: 1.1%
  faction-majority Humans: 1035 armies, win 50.3%
  faction-majority Undead: 147 armies, win 57.8%
  faction-majority Machines: 122 armies, win 55.7%
  faction-majority Plants: 696 armies, win 45.3%
  purchase share Shieldbearer 21.0% · Archer 16.1% · BoneBrute 6.3% · Hexer 12.1%
                 Sawbot 15.7% · SiegeWalker 0.7% · Bloomkeeper 16.1% · Rootcaller 12.0%
  evolutions per match: 8.5
  step µs: mean 0.44 max 120.4 | max units alive 16
  ```
  All 1000 matches end. Mean step 0.44 µs (budget 50 µs). Not tuned (P7):
  median 93 s is below the 2.5–4 min target and Siege Walker is almost never
  bought by the bot (cost 6 + random reserve rarely affordable).
- Deviations:
  1. Root on an already rooted unit keeps the longer duration (spec: "Root
     replaces anything"; equal reading, avoids shortening a root).
  2. `combat.rs` split into `combat.rs` + `hits.rs` + `targeting.rs` to stay
     under 200 lines per file.
  3. The old one-line vocabulary test in `lib.rs` was removed; the rule tests
     cover the vocabulary through the content table.
  4. Harness "faction-majority" picks the faction a seat bought most; ties go
     to the later faction index. Good enough for trends, not for balance sign-off.
  5. Added `content::form_of`, `MatchState::is_live`, `slots_used`,
     `stats_of`, `protocol::valid_room_code`, `ROOM_ALPHABET` (helpers the
     client and server need; no new rules).
- Blocked items: none.
- Needs the owner: nothing for this phase.
