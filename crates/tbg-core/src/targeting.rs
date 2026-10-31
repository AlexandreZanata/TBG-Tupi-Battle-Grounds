//! Who a unit acts on this tick: nearest enemy (unit or Stronghold), a
//! Rampart in range first (Guard, BR-U09), or for Support the most damaged
//! ally in range (GAME-DESIGN §4). One pass over the units per decision.

use crate::content::{Ability, Stats, BP};
use crate::state::{stronghold_x, MatchState, Target, Unit};
use crate::types::UnitId;

pub(crate) enum Engage {
    /// Attack an enemy or heal an ally.
    Act(Target),
    /// Support with nothing to heal and an enemy in range: stand still.
    Hold,
}

pub(crate) fn engage(state: &MatchState, u: &Unit, st: &Stats) -> Option<Engage> {
    // Action units (the Antidote) brew instead of attacking when an ally
    // needs it.
    if crate::afflictions::brew_of(state, u).is_some() {
        if let Some(ally) = crate::afflictions::brew_target(state, u, st.range, None) {
            return Some(Engage::Act(Target::Unit(ally)));
        }
    }
    // Puppetmasters confuse instead of attacking when a victim exists.
    if crate::support::puppet_of(state, u).is_some() {
        if let Some(victim) = crate::support::puppet_target(state, u, st.range) {
            return Some(Engage::Act(Target::Unit(victim)));
        }
    }
    // Units with no attack and no heal hold when an enemy is in range and
    // never emit `Hit{amount: 0}` (P43; unreachable today: every kind
    // attacks or heals).
    if st.damage == 0 && st.heal == 0 {
        return enemy_in_range(state, u, st.range).map(|_| Engage::Hold);
    }
    if st.heal > 0 {
        if let Some(ally) = most_damaged_ally(state, u, st.range) {
            return Some(Engage::Act(Target::Unit(ally)));
        }
        if st.damage == 0 {
            return enemy_in_range(state, u, st.range).map(|_| Engage::Hold);
        }
    }
    let no_stronghold = state
        .rules()
        .def(u.kind)
        .ability(u.tier, u.branch)
        .contains(&Ability::NoStronghold);
    match enemy_in_range(state, u, st.range) {
        // A unit that never targets Strongholds holds when the Stronghold
        // is the only thing in range (Farwatch, §C.2).
        Some(Target::Stronghold(_)) if no_stronghold => Some(Engage::Hold),
        Some(t) => Some(Engage::Act(t)),
        None => None,
    }
}

/// A carried unit cannot be chosen by any targeted effect (P48-A): enemy
/// attacks, Stronghold fire, echoes, betrayal, Farwatch. Area damage,
/// cleanse, transfers, bonds (and the passenger's own actions) still
/// reach it. Carried units are never Tanks, so Guard never sees one.
pub(crate) fn targetable(unit: &Unit) -> bool {
    unit.fx.carrier.is_none()
}

/// Guard first: the nearest living enemy Rampart within range. Otherwise
/// the nearest living enemy unit by |dx| (ties: lowest id), or the enemy
/// Stronghold when it is strictly nearer or no enemy unit exists. A unit
/// that never targets Strongholds (Farwatch) takes the nearest enemy unit
/// in range instead, and holds when there is none.
pub(crate) fn enemy_in_range(state: &MatchState, u: &Unit, range: i32) -> Option<Target> {
    let enemy = u.seat.other();
    let no_stronghold = state
        .rules()
        .def(u.kind)
        .ability(u.tier, u.branch)
        .contains(&Ability::NoStronghold);
    let mut nearest: Option<(i32, UnitId)> = None;
    let mut guard: Option<(i32, UnitId)> = None;
    for e in state
        .units
        .iter()
        // The candidate is already borrowed from this sorted unit vector;
        // looking its id up again adds log(n) work to every candidate.
        .filter(|e| e.seat == enemy && e.hp > 0 && targetable(e))
    {
        let key = ((e.x - u.x).abs(), e.id);
        #[cfg(feature = "bench-profile")]
        crate::profiling::TARGET_CANDIDATES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        nearest = Some(nearest.map_or(key, |n| n.min(key)));
        let is_guard = state
            .rules
            .def(e.kind)
            .ability(e.tier, e.branch)
            .contains(&Ability::Guard);
        if is_guard && key.0 <= range {
            guard = Some(guard.map_or(key, |g| g.min(key)));
        }
    }
    if let Some((_, id)) = guard {
        return Some(Target::Unit(id));
    }
    let base = (stronghold_x(enemy) - u.x).abs();
    let (target, dist) = match nearest {
        Some((d, id)) if d <= base || no_stronghold => (Target::Unit(id), d),
        _ => (Target::Stronghold(enemy), base),
    };
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
