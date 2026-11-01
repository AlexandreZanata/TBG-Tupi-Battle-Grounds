//! Fixed best-three enemy selection shared by completed-hit secondary abilities.
use crate::{
    state::{MatchState, Unit},
    targeting,
    types::UnitId,
};

/// Three is the structural maximum of the approved roster, not a target cap knob.
pub(crate) fn nearest(
    state: &MatchState,
    attacker: &Unit,
    primary: UnitId,
    center: i32,
    radius: i32,
) -> [UnitId; 3] {
    let mut best = [(i32::MAX, UnitId::MAX); 3];
    for u in &state.units {
        if u.id == primary || u.hp <= 0 || u.seat == attacker.seat || !targeting::targetable(u) {
            continue;
        }
        let key = ((u.x - center).abs(), u.id);
        if key.0 > radius {
            continue;
        }
        if let Some(at) = best.iter().position(|&current| key < current) {
            for j in (at + 1..best.len()).rev() {
                best[j] = best[j - 1];
            }
            best[at] = key;
        }
    }
    best.map(|(_, id)| id)
}
