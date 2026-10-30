//! Display layout: where each unit is drawn along its lane so that bodies do
//! not overlap. Presentation only: the server's `x` is never changed here and
//! the simulation stays one-dimensional (D-57).

use std::collections::HashMap;

use bevy::prelude::*;
use tbg_core::{Seat, UnitId};

/// Lane depth of the five files, centre out. Files are 0.6 tiles apart, closer
/// than two radii, so neighbours in one rank collide and the pass steps back.
const FILES: [f32; 5] = [0.0, -0.6, 0.6, -1.2, 1.2];
/// Ranks a unit may step back toward its own Stronghold. Past this cap units
/// overlap, but they still take the least crowded file of the last rank.
const MAX_RANKS: usize = 8;
/// Footprint radius of a figure at scale 1, in tiles (the ring's radius).
pub const BASE_RADIUS: f32 = 0.36;
/// Back step of one rank for `BASE_RADIUS`, in tiles.
const RANK_TILES: f32 = 0.62;
/// Two discs may touch this much closer than the sum of their radii.
const CLEARANCE: f32 = 0.02;
/// Easing rate (per second) of the displayed `back` and `z`.
const EASE: f32 = 10.0;
/// Neighbour grid: buckets of `BUCKET_TILES` from `GRID_MIN`, `GRID_LEN` of
/// them. A bucket is wider than the largest pair of discs can reach (two
/// large bodies touch at 1.06 tiles), so a disc only meets its neighbours.
const GRID_MIN: f32 = -8.0;
const GRID_LEN: usize = 32;
const BUCKET_TILES: f32 = 1.1;
const REACH_BUCKETS: usize = 1;

/// What the layout needs of one live unit.
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub id: UnitId,
    pub seat: Seat,
    /// True lane position in tiles (`x / 1000`), interpolated for this frame.
    pub x: f32,
    /// Footprint radius in tiles: `BASE_RADIUS × plan.scale`.
    pub radius: f32,
}

/// Where one unit is drawn this frame, in lane tiles.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Slot {
    pub id: UnitId,
    /// Lane x of the drawn centre: the true `x` minus the eased `back`.
    pub x: f32,
    /// Lane depth (across the lane), eased.
    pub z: f32,
    /// Tiles behind the true `x`, toward the unit's own Stronghold (≥ 0).
    back: f32,
    /// Index into `FILES`, kept so the unit prefers its file next frame.
    file: usize,
}

#[derive(Clone, Copy)]
struct Disc {
    x: f32,
    z: f32,
    r: f32,
}

/// Per-frame layout state. Buffers are reused, so a steady frame allocates
/// nothing; `prev` and `next` swap each frame.
#[derive(Resource, Default)]
pub struct Layout {
    bodies: Vec<Body>,
    slots: Vec<Slot>,
    order: Vec<usize>,
    grid: [Vec<Disc>; GRID_LEN],
    prev: HashMap<UnitId, Slot>,
    next: HashMap<UnitId, Slot>,
}

impl Layout {
    /// Forget the previous frame, e.g. when a match ends.
    pub fn clear(&mut self) {
        self.bodies.clear();
        self.prev.clear();
    }

    /// Start a new frame: bodies are pushed, then `solve` runs once.
    pub fn begin(&mut self) {
        self.bodies.clear();
    }

    pub fn push(&mut self, body: Body) {
        self.bodies.push(body);
    }

    /// Slot of the `i`-th body pushed since `begin`.
    pub fn slot(&self, i: usize) -> Slot {
        self.slots.get(i).copied().unwrap_or_default()
    }

    /// Lay out the pushed bodies of both seats, easing by `dt` seconds.
    pub fn solve(&mut self, dt: f32) {
        self.slots.clear();
        self.slots.resize(self.bodies.len(), Slot::default());
        self.next.clear();
        for seat in [Seat::A, Seat::B] {
            self.solve_seat(seat, dt);
        }
        std::mem::swap(&mut self.prev, &mut self.next);
    }

    /// Seats are laid out independently; each seat's bodies go front first.
    fn solve_seat(&mut self, seat: Seat, dt: f32) {
        let dir = seat.dir() as f32;
        let mut order = std::mem::take(&mut self.order);
        order.clear();
        order.extend((0..self.bodies.len()).filter(|&i| self.bodies[i].seat == seat));
        let bodies = &self.bodies;
        // Furthest walked first, ties by id: the result never depends on the
        // order the bodies were pushed in.
        order.sort_unstable_by(|&a, &b| {
            let (p, q) = (bodies[a], bodies[b]);
            (q.x * dir).total_cmp(&(p.x * dir)).then(p.id.cmp(&q.id))
        });
        for cell in &mut self.grid {
            cell.clear();
        }
        for &i in &order {
            let body = self.bodies[i];
            let prev = self.prev.get(&body.id).copied();
            let target = self.place(dir, body, prev.map(|p| p.file));
            let slot = match prev {
                Some(p) => ease(p, target, body.x, dir, dt),
                None => target,
            };
            self.slots[i] = slot;
            self.next.insert(body.id, slot);
        }
        self.order = order;
    }

    /// Greedy placement: the first free spot, rank by rank from the true `x`.
    /// The unit's previous file is tried first on every rank.
    fn place(&mut self, dir: f32, body: Body, prev_file: Option<usize>) -> Slot {
        let step = RANK_TILES * body.radius / BASE_RADIUS;
        let files = file_order(prev_file);
        for rank in 0..=MAX_RANKS {
            let back = rank as f32 * step;
            let x = body.x - dir * back;
            let free = files
                .iter()
                .copied()
                .find(|&f| !self.blocked(x, FILES[f], body.radius));
            if let Some(file) = free {
                return self.occupy(dir, body, back, file);
            }
        }
        let back = MAX_RANKS as f32 * step;
        let x = body.x - dir * back;
        let file = files
            .into_iter()
            .min_by_key(|&f| self.crowd(x, FILES[f], body.radius))
            .unwrap_or(0);
        self.occupy(dir, body, back, file)
    }

    fn occupy(&mut self, dir: f32, body: Body, back: f32, file: usize) -> Slot {
        let z = FILES[file];
        let x = body.x - dir * back;
        self.grid[bucket(x)].push(Disc {
            x,
            z,
            r: body.radius,
        });
        Slot {
            id: body.id,
            x,
            z,
            back,
            file,
        }
    }

    /// Grid cells that can hold a disc touching lane x `x`.
    fn cells(x: f32) -> std::ops::RangeInclusive<usize> {
        let b = bucket(x);
        b.saturating_sub(REACH_BUCKETS)..=(b + REACH_BUCKETS).min(GRID_LEN - 1)
    }

    /// Whether a candidate at (`x`, `z`) would overlap a placed disc.
    fn blocked(&self, x: f32, z: f32, r: f32) -> bool {
        Self::cells(x).any(|cell| self.grid[cell].iter().any(|&d| overlaps(d, x, z, r)))
    }

    /// Number of placed discs a candidate at (`x`, `z`) would overlap.
    fn crowd(&self, x: f32, z: f32, r: f32) -> usize {
        Self::cells(x)
            .map(|cell| {
                self.grid[cell]
                    .iter()
                    .filter(|&&d| overlaps(d, x, z, r))
                    .count()
            })
            .sum()
    }
}

fn overlaps(d: Disc, x: f32, z: f32, r: f32) -> bool {
    let reach = d.r + r - CLEARANCE;
    let (dx, dz) = (d.x - x, d.z - z);
    dx * dx + dz * dz < reach * reach
}

/// Exponential easing of the displayed `back` and `z`; the true `x` is
/// applied at once, so the layout never delays movement.
fn ease(prev: Slot, target: Slot, true_x: f32, dir: f32, dt: f32) -> Slot {
    let k = 1.0 - (-EASE * dt).exp();
    let back = prev.back + (target.back - prev.back) * k;
    let z = prev.z + (target.z - prev.z) * k;
    Slot {
        x: true_x - dir * back,
        z,
        back,
        ..target
    }
}

/// Files centre out; the unit's previous file goes first when it has one.
fn file_order(prev: Option<usize>) -> [usize; 5] {
    let mut order = [0, 1, 2, 3, 4];
    if let Some(p) = prev.filter(|&p| p < FILES.len()) {
        order[..=p].rotate_right(1);
    }
    order
}

/// Grid cell of a lane x. Float-to-int casts saturate, so this cannot panic.
fn bucket(x: f32) -> usize {
    (((x - GRID_MIN) / BUCKET_TILES) as usize).min(GRID_LEN - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(id: UnitId, seat: Seat, x: f32, scale: f32) -> Body {
        Body {
            id,
            seat,
            x,
            radius: BASE_RADIUS * scale,
        }
    }

    /// Lay out `bodies` from a cold start (no previous frame, so no easing).
    fn solve(bodies: &[Body]) -> Vec<(Body, Slot)> {
        let mut layout = Layout::default();
        layout.begin();
        for &b in bodies {
            layout.push(b);
        }
        layout.solve(1.0 / 60.0);
        bodies
            .iter()
            .enumerate()
            .map(|(i, &b)| (b, layout.slot(i)))
            .collect()
    }

    /// Pairs of one seat whose drawn discs overlap beyond the clearance.
    fn overlapping_pairs(drawn: &[(Body, Slot)]) -> usize {
        let mut count = 0;
        for (i, (p, a)) in drawn.iter().enumerate() {
            for (q, b) in &drawn[i + 1..] {
                if p.seat != q.seat {
                    continue;
                }
                let reach = p.radius + q.radius - CLEARANCE;
                let d2 = (a.x - b.x).powi(2) + (a.z - b.z).powi(2);
                if d2 < reach * reach {
                    count += 1;
                }
            }
        }
        count
    }

    /// Small deterministic generator: the tests must not depend on the clock.
    fn lcg(state: &mut u32) -> f32 {
        *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (*state >> 8) as f32 / 16_777_216.0
    }

    fn army(seed: u32, per_seat: u32) -> Vec<Body> {
        let mut s = seed;
        (0..per_seat * 2)
            .map(|k| {
                let seat = if k % 2 == 0 { Seat::A } else { Seat::B };
                let x = 1.0 + 14.0 * lcg(&mut s);
                body(k, seat, x, 1.0)
            })
            .collect()
    }

    #[test]
    fn no_overlap_while_the_lane_has_room() {
        // Twenty units engaging the same enemy at the same spot per side.
        let mut bodies: Vec<Body> = (0..20).map(|k| body(k, Seat::A, 6.0, 1.0)).collect();
        bodies.extend((0..20).map(|k| body(100 + k, Seat::B, 10.0, 1.0)));
        assert_eq!(overlapping_pairs(&solve(&bodies)), 0);
        // A realistic spread with some large bodies mixed in.
        let mut spread = army(7, 14);
        for b in spread.iter_mut().step_by(5) {
            b.radius = BASE_RADIUS * 1.5;
        }
        assert_eq!(overlapping_pairs(&solve(&spread)), 0);
    }

    #[test]
    fn front_unit_stays_at_its_true_x_and_nothing_is_ahead() {
        let bodies = army(3, 25);
        let drawn = solve(&bodies);
        for seat in [Seat::A, Seat::B] {
            let dir = seat.dir() as f32;
            let front = drawn
                .iter()
                .filter(|(b, _)| b.seat == seat)
                .max_by(|(p, _), (q, _)| (p.x * dir).total_cmp(&(q.x * dir)))
                .expect("every seat has units");
            assert_eq!(front.1.x, front.0.x, "front line is drawn at its true x");
        }
        for (b, s) in &drawn {
            let dir = b.seat.dir() as f32;
            assert!(
                (s.x - b.x) * dir <= 0.0,
                "unit {} drawn ahead of its x",
                b.id
            );
        }
    }

    #[test]
    fn same_input_gives_the_same_slots_in_any_order() {
        let bodies = army(11, 20);
        let mut reversed = bodies.clone();
        reversed.reverse();
        let (a, b) = (solve(&bodies), solve(&reversed));
        for (p, s) in &a {
            let (_, t) = b.iter().find(|(q, _)| q.id == p.id).expect("same ids");
            assert_eq!(s, t, "unit {} moved with input order", p.id);
        }
    }

    #[test]
    fn surviving_units_keep_their_file_when_one_dies() {
        let bodies = army(5, 12);
        let mut layout = Layout::default();
        layout.begin();
        bodies.iter().for_each(|&b| layout.push(b));
        layout.solve(10.0);
        let before: Vec<Slot> = (0..bodies.len()).map(|i| layout.slot(i)).collect();
        // The front unit of seat A dies; everyone else keeps their file.
        let dead = bodies
            .iter()
            .filter(|b| b.seat == Seat::A)
            .max_by(|p, q| p.x.total_cmp(&q.x))
            .map(|b| b.id);
        layout.begin();
        bodies
            .iter()
            .filter(|b| Some(b.id) != dead)
            .for_each(|&b| layout.push(b));
        layout.solve(10.0);
        let survivors = bodies
            .iter()
            .enumerate()
            .filter(|(_, b)| Some(b.id) != dead);
        for (j, (i, _)) in survivors.enumerate() {
            assert_eq!(
                layout.slot(j).file,
                before[i].file,
                "unit {} changed file",
                bodies[i].id
            );
        }
    }

    #[test]
    fn three_hundred_units_within_budget() {
        let bodies = army(9, 150);
        let mut layout = Layout::default();
        // The fastest of many frames: the machine is shared, so the mean is noise.
        let mut per_frame = std::time::Duration::MAX;
        for _ in 0..50 {
            let start = std::time::Instant::now();
            layout.begin();
            bodies.iter().for_each(|&b| layout.push(b));
            layout.solve(1.0 / 60.0);
            per_frame = per_frame.min(start.elapsed());
        }
        eprintln!("layout of 300 units: {per_frame:?} per frame (fastest of 50)");
        assert!(per_frame.as_secs_f64() < 0.002, "{per_frame:?} per frame");
    }

    /// Overlap tally of drawn discs: (lane x, lane z, radius, seat) per unit.
    #[derive(Default, Debug)]
    struct Tally {
        samples: u32,
        samples_with_overlap: u32,
        pairs: u64,
        same_seat_pairs: u64,
        units: u64,
        units_overlapping: u64,
        units_in_stack_of_3: u64,
        worst_stack: u32,
        solve_ns: u128,
    }

    impl Tally {
        fn add(&mut self, drawn: &[(f32, f32, f32, Seat)]) {
            self.samples += 1;
            let mut touching = vec![0u32; drawn.len()];
            for (i, &(ax, az, ar, aseat)) in drawn.iter().enumerate() {
                for (j, &(bx, bz, br, bseat)) in drawn.iter().enumerate().skip(i + 1) {
                    let reach = ar + br - CLEARANCE;
                    if (ax - bx).powi(2) + (az - bz).powi(2) < reach * reach {
                        self.pairs += 1;
                        self.same_seat_pairs += u64::from(aseat == bseat);
                        touching[i] += 1;
                        touching[j] += 1;
                    }
                }
            }
            self.samples_with_overlap += u32::from(touching.iter().any(|&n| n > 0));
            self.units += drawn.len() as u64;
            self.units_overlapping += touching.iter().filter(|&&n| n > 0).count() as u64;
            self.units_in_stack_of_3 += touching.iter().filter(|&&n| n >= 2).count() as u64;
            let worst = touching.iter().map(|&n| n + 1).max().unwrap_or(0);
            self.worst_stack = self.worst_stack.max(worst);
        }
    }

    /// Measurement, not a check. Run with
    /// `cargo test --release -p tbg-client probe_bot_matches -- --ignored --nocapture`.
    /// Plays 40 offline bot × bot matches (tbg-core, no network) and samples
    /// every half second: the old lane rule against this layout.
    #[test]
    #[ignore = "measurement probe, see docs/phases/P57-unit-spacing.md"]
    fn probe_bot_matches() {
        use crate::art::roster::figure_spec;
        use tbg_core::bot::Bot;
        use tbg_core::headless::{self, Moment};
        use tbg_core::state::MatchState;

        let mut old = Tally::default();
        let mut new = Tally::default();
        let mut cross_front = 0u64;
        let mut cross_front_small = 0u64;
        for seed in 1..=40u64 {
            let mut state = MatchState::new(seed);
            let mut bots = [Bot::new(seed * 2), Bot::new(seed * 2 + 1)];
            let mut layout = Layout::default();
            headless::play(
                &mut state,
                |seat, view| bots[seat.index()].decide(view),
                |m| {
                    let Moment::Stepped { state } = m else {
                        return;
                    };
                    if state.clock % 10 != 0 {
                        return;
                    }
                    let view = state.view_for(Seat::A, &[]);
                    if view.units.is_empty() {
                        return;
                    }
                    let radius = |u: &tbg_core::view::UnitView| {
                        BASE_RADIUS * figure_spec(u.kind, u.tier, u.branch).plan.scale
                    };
                    let lane = |u: &tbg_core::view::UnitView| (u.x as f32 / 1000.0, u.seat);
                    let before: Vec<_> = view
                        .units
                        .iter()
                        .map(|u| {
                            let (x, seat) = lane(u);
                            let z = [-0.8, 0.0, 0.8][(u.id % 3) as usize];
                            (x, z, radius(u), seat)
                        })
                        .collect();
                    old.add(&before);
                    layout.begin();
                    for u in &view.units {
                        let (x, seat) = lane(u);
                        layout.push(Body {
                            id: u.id,
                            seat,
                            x,
                            radius: radius(u),
                        });
                    }
                    let start = std::time::Instant::now();
                    layout.solve(0.5);
                    new.solve_ns += start.elapsed().as_nanos();
                    let after: Vec<_> = view
                        .units
                        .iter()
                        .enumerate()
                        .map(|(i, u)| {
                            let s = layout.slot(i);
                            (s.x, s.z, radius(u), u.seat)
                        })
                        .collect();
                    new.add(&after);
                    // Opposing units that both stand on the front line (true x).
                    for (i, p) in after.iter().enumerate() {
                        for (j, q) in after.iter().enumerate().skip(i + 1) {
                            let reach = p.2 + q.2 - CLEARANCE;
                            let d2 = (p.0 - q.0).powi(2) + (p.1 - q.1).powi(2);
                            let front = layout.slot(i).back == 0.0 && layout.slot(j).back == 0.0;
                            if p.3 != q.3 && front && d2 < reach * reach {
                                cross_front += 1;
                                cross_front_small += u64::from(p.2 < 0.5 && q.2 < 0.5);
                            }
                        }
                    }
                },
            );
        }
        eprintln!("old lane rule: {old:?}");
        eprintln!("new layout:    {new:?}");
        eprintln!("cross-seat pairs with both units on the front line: {cross_front} (both small bodies: {cross_front_small})");
    }
}
