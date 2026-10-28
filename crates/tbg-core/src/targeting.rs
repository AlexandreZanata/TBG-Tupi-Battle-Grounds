//! Who a unit acts on this tick: nearest enemy (unit or Stronghold), or
//! for Support the most damaged ally in range (GAME-DESIGN §4).

use crate::content::{Stats, BP};
use crate::state::{stronghold_x, MatchState, Target, Unit};
use crate::types::UnitId;

pub(crate) enum Engage {
    /// Attack an enemy or heal an ally.
    Act(Target),
    /// Support with nothing to heal and an enemy in range: stand still.
    Hold,
}

pub(crate) fn engage(state: &MatchState, u: &Unit, st: &Stats) -> Option<Engage> {
    if st.heal > 0 {
        if let Some(ally) = most_damaged_ally(state, u, st.range) {
            return Some(Engage::Act(Target::Unit(ally)));
        }
        if st.damage == 0 {
            return enemy_in_range(state, u, st.range).map(|_| Engage::Hold);
        }
    }
    enemy_in_range(state, u, st.range).map(Engage::Act)
}

/// Nearest living enemy unit by |dx| (ties: lowest id), or the enemy
/// Stronghold when it is strictly nearer or no enemy unit exists.
pub(crate) fn nearest_enemy(state: &MatchState, u: &Unit) -> (Target, i32) {
    let enemy = u.seat.other();
    let best = state
        .units
        .iter()
        .filter(|e| e.seat == enemy && e.hp > 0)
        .map(|e| ((e.x - u.x).abs(), e.id))
        .min();
    let base = (stronghold_x(enemy) - u.x).abs();
    match best {
        Some((d, id)) if d <= base => (Target::Unit(id), d),
        _ => (Target::Stronghold(enemy), base),
    }
}

fn enemy_in_range(state: &MatchState, u: &Unit, range: i32) -> Option<Target> {
    let (target, dist) = nearest_enemy(state, u);
    (dist <= range).then_some(target)
}

/// Lowest HP ratio (below full) among own living units within range.
fn most_damaged_ally(state: &MatchState, u: &Unit, range: i32) -> Option<UnitId> {
    state
        .units
        .iter()
        .filter(|a| a.seat == u.seat && a.hp > 0 && a.hp < a.max_hp)
        .filter(|a| (a.x - u.x).abs() <= range)
        .map(|a| (i64::from(a.hp) * BP / i64::from(a.max_hp), a.id))
        .min()
        .map(|(_, id)| id)
}

/// Is the target still there to be hit or healed?
pub(crate) fn alive(state: &MatchState, target: Target) -> bool {
    match target {
        Target::Unit(id) => state.unit_index(id).is_some_and(|i| state.units[i].hp > 0),
        Target::Stronghold(seat) => state.player(seat).stronghold_hp > 0,
    }
}

pub(crate) fn target_x(state: &MatchState, target: Target) -> i32 {
    match target {
        Target::Unit(id) => state.unit_index(id).map_or(0, |i| state.units[i].x),
        Target::Stronghold(seat) => stronghold_x(seat),
    }
}
