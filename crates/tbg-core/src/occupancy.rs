//! PK-01: integer lane footprints, persistent lateral slots and capacity-limited deployment.
use crate::{content, MatchState, Unit};
mod deployment;

pub use crate::content::occupancy::{footprint, Footprint, CELL, COLUMNS, GAP, WIDTH};
fn cells(u: &Unit) -> i32 {
    (footprint(u.kind, u.tier).width + CELL - 1) / CELL
}
fn center(u: &Unit, slot: i32) -> i32 {
    -WIDTH / 2 + slot * CELL + cells(u) * CELL / 2
}
pub fn lateral(u: &Unit) -> i32 {
    if u.fx.lane_slot == 0 {
        0
    } else {
        center(u, i32::from(u.fx.lane_slot) - 1)
    }
}
fn overlaps(a: &Unit, slot: i32, b: &Unit) -> bool {
    let other = i32::from(b.fx.lane_slot) - 1;
    slot < other + cells(b) && other < slot + cells(a)
}
fn spacing(a: &Unit, b: &Unit) -> i32 {
    let a_size = footprint(a.kind, a.tier);
    let b_size = footprint(b.kind, b.tier);
    if a.seat == b.seat {
        a_size.front + b_size.back + GAP
    } else {
        a_size.body + b_size.body + GAP
    }
}
/// Preserve authored root reach, with only a body-contact floor for short melee attacks.
pub(crate) fn distance(a: &Unit, b: &Unit, range: i32) -> i32 {
    let contact = footprint(a.kind, a.tier).body + footprint(b.kind, b.tier).body + GAP;
    ((a.x - b.x).abs() - (contact - range).max(0)).max(0)
}

fn place(u: &mut Unit, placed: &[Unit]) -> bool {
    let dir = u.seat.dir();
    let preferred = if u.fx.lane_slot == 0 {
        let edge = (WIDTH - cells(u) * CELL) / 2;
        if u.id % 2 == 0 {
            edge
        } else {
            -edge
        }
    } else {
        lateral(u)
    };
    let mut best = None;
    let mut intervals = Vec::with_capacity(placed.len());
    for slot in 0..=COLUMNS - cells(u) {
        let mut progress = u.x * dir;
        intervals.clear();
        for other in placed.iter().filter(|v| v.hp > 0 && v.fx.carrier.is_none()) {
            if overlaps(u, slot, other) {
                intervals.push((
                    other.x * dir - spacing(u, other),
                    other.x * dir + spacing(other, u),
                ));
            }
        }
        intervals.sort_unstable_by_key(|&(_, end)| std::cmp::Reverse(end));
        for &(start, end) in &intervals {
            if start < progress && progress < end {
                progress = start;
            }
        }
        let z = center(u, slot);
        let side = if u.id % 2 == 0 { z * dir } else { -z * dir };
        let score = (progress, -(z - preferred).abs(), side);
        if best.is_none_or(|(old, _)| score > old) {
            best = Some((score, slot));
        }
    }
    let Some(((progress, _, _), slot)) = best else {
        return false;
    };
    let x = progress * dir;
    if !(content::SPAWN_OFFSET..=content::LANE_LEN - content::SPAWN_OFFSET).contains(&x) {
        return false;
    }
    u.x = x;
    u.fx.lane_slot = (slot + 1) as u8;
    if u.fx.lane_goal == 0 {
        u.fx.lane_goal = u.fx.lane_slot;
    }
    true
}

/// Stable front-to-back packing. Overflow waits outside the active combat vector, without a purchase cap.
pub(crate) fn arrange(state: &mut MatchState) {
    if !state.physical_lanes {
        return;
    }
    let mut all = std::mem::take(&mut state.units);
    state
        .deployments
        .retain(|u| u.hp > 0 && u.expires_at.is_none_or(|t| t > state.tick));
    all.append(&mut state.deployments);
    let mut carried = Vec::new();
    all.retain(|u| {
        if u.fx.carrier.is_some() {
            carried.push(*u);
            false
        } else {
            true
        }
    });
    all.sort_unstable_by_key(|u| {
        (
            u.fx.lane_slot == 0,
            u.seat.index(),
            std::cmp::Reverse(u.x * u.seat.dir()),
            u.id,
        )
    });
    let mut live = Vec::with_capacity(all.len());
    for mut u in all {
        // ponytail: forced teleports keep active units targetable even if no legal
        // projection exists; a future displacement rule must resolve that exceptional case.
        if u.hp <= 0
            || place(&mut u, &live)
            || u.fx.lane_slot > 0
            || deployment::admit(&mut u, &live)
        {
            live.push(u);
            live.sort_unstable_by_key(|v| {
                (v.seat.index(), std::cmp::Reverse(v.x * v.seat.dir()), v.id)
            });
        } else {
            u.x = if u.seat.dir() > 0 {
                content::SPAWN_OFFSET
            } else {
                content::LANE_LEN - content::SPAWN_OFFSET
            };
            u.windup = None;
            u.fx.lane_slot = 0;
            state.deployments.push(u);
        }
    }
    for mut u in carried {
        if let Some(parent) = live.iter().find(|p| Some(p.id) == u.fx.carrier) {
            u.x = parent.x;
            u.fx.lane_slot = parent.fx.lane_slot;
            live.push(u);
        } else if state.deployments.iter().any(|p| Some(p.id) == u.fx.carrier) {
            state.deployments.push(u);
        } else {
            u.fx.carrier = None;
            live.push(u);
        }
    }
    live.sort_unstable_by_key(|u| u.id);
    state.deployments.sort_unstable_by_key(|u| u.id);
    state.units = live;
}

/// An upgrade may reflow deployed allies, but cannot remove one from combat to create room.
pub(crate) fn evolution_fits(state: &MatchState, id: u32) -> bool {
    if !state.physical_lanes {
        return true;
    }
    let mut units: Vec<_> = state
        .units
        .iter()
        .filter(|u| u.hp > 0 && u.fx.carrier.is_none())
        .copied()
        .collect();
    if let Some(u) = units.iter_mut().find(|u| u.id == id) {
        u.tier = u.tier.saturating_add(1);
    }
    units.sort_unstable_by_key(|u| (u.seat.index(), std::cmp::Reverse(u.x * u.seat.dir()), u.id));
    let mut placed = Vec::new();
    for mut u in units {
        if !place(&mut u, &placed) {
            return false;
        }
        placed.push(u);
    }
    true
}

/// Returns a swept reservation's allowed forward position, or rejects a blocked lateral strip.
fn progress_in(state: &MatchState, u: &Unit, slot: i32, new_x: i32) -> Option<i32> {
    let old = i32::from(u.fx.lane_slot) - 1;
    let dir = u.seat.dir();
    let mut progress = new_x * dir;
    let low = slot.min(old);
    let high = (slot + cells(u)).max(old + cells(u));
    for other in state
        .units
        .iter()
        .filter(|v| v.id != u.id && v.hp > 0 && v.fx.carrier.is_none() && v.fx.lane_slot > 0)
    {
        let at = i32::from(other.fx.lane_slot) - 1;
        let ahead = other.x * dir;
        if slot != old
            && low < at + cells(other)
            && at < high
            && u.x * dir > ahead - spacing(u, other)
            && u.x * dir < ahead + spacing(other, u)
        {
            return None;
        }
        if (other.x - u.x) * dir > 0 && overlaps(u, slot, other) {
            progress = progress.min((ahead - spacing(u, other)).max(u.x * dir));
        }
    }
    Some(progress)
}
fn goal(state: &MatchState, u: &Unit, new_x: i32) -> i32 {
    let old = i32::from(u.fx.lane_slot) - 1;
    if u.fx.lane_goal > 0 && i32::from(u.fx.lane_goal) - 1 != old {
        return (i32::from(u.fx.lane_goal) - 1).clamp(0, COLUMNS - cells(u));
    }
    if state.tick < u.fx.lane_lock_until {
        return old;
    }
    let mut best = (
        progress_in(state, u, old, new_x).unwrap_or(u.x * u.seat.dir()),
        0,
        old,
    );
    for slot in 0..=COLUMNS - cells(u) {
        if let Some(progress) = progress_in(state, u, slot, new_x) {
            let score = (progress, -(slot - old).abs(), slot);
            if score > best {
                best = score;
            }
        }
    }
    best.2
}
/// Lateral travel is bounded to one 100-milli-tile cell per tick; completed bypasses lock for one second.
pub(crate) fn move_unit(state: &mut MatchState, index: usize, new_x: i32) {
    if !state.physical_lanes {
        state.units[index].x = new_x;
        return;
    }
    let u = state.units[index];
    if u.fx.lane_slot == 0 {
        return;
    }
    let old = i32::from(u.fx.lane_slot) - 1;
    let target = goal(state, &u, new_x);
    let next = old + (target - old).signum();
    let Some(progress) = progress_in(state, &u, next, new_x) else {
        state.units[index].fx.lane_goal = u.fx.lane_slot;
        state.units[index].fx.lane_lock_until = state
            .tick
            .saturating_add(content::occupancy::LANE_CHANGE_LOCK_TICKS);
        return;
    };
    let placed = &mut state.units[index];
    placed.x = progress * u.seat.dir();
    placed.fx.lane_slot = (next + 1) as u8;
    placed.fx.lane_goal = (target + 1) as u8;
    if next == target && next != old {
        placed.fx.lane_lock_until = state
            .tick
            .saturating_add(content::occupancy::LANE_CHANGE_LOCK_TICKS);
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests;
