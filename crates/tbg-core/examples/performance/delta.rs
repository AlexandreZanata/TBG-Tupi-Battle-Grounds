//! Lossless delta prototype, intentionally outside the real protocol.
//! The complete per-seat header/events remain; only unchanged units vanish.

use serde::{Deserialize, Serialize};
use tbg_core::{MatchView, UnitId, UnitView};

#[derive(Serialize, Deserialize)]
pub struct Delta {
    baseline: u64,
    header: MatchView,
    changed: Vec<UnitView>,
    removed: Vec<UnitId>,
}

pub fn build(baseline: u64, previous: &MatchView, current: &MatchView) -> Delta {
    let mut old = previous.units.iter().peekable();
    let (mut changed, mut removed) = (Vec::new(), Vec::new());
    for unit in &current.units {
        while old.peek().is_some_and(|u| u.id < unit.id) {
            removed.push(old.next().unwrap().id);
        }
        if old.peek().is_some_and(|u| u.id == unit.id) {
            if old.next().unwrap() != unit {
                changed.push(unit.clone());
            }
        } else {
            changed.push(unit.clone());
        }
    }
    removed.extend(old.map(|u| u.id));
    // Conservative prototype cost: cloning the header also clones its
    // units before clearing them. A production codec would need a header
    // type and a versioned protocol; neither is introduced by this example.
    let mut header = current.clone();
    header.units.clear();
    Delta {
        baseline,
        header,
        changed,
        removed,
    }
}

pub fn apply(expected: u64, previous: &MatchView, mut delta: Delta) -> Option<MatchView> {
    if delta.baseline != expected {
        return None;
    }
    let mut changed = delta.changed.into_iter().peekable();
    for unit in &previous.units {
        if delta.removed.binary_search(&unit.id).is_ok() {
            continue;
        }
        while changed.peek().is_some_and(|u| u.id < unit.id) {
            delta.header.units.push(changed.next().unwrap());
        }
        if changed.peek().is_some_and(|u| u.id == unit.id) {
            delta.header.units.push(changed.next().unwrap());
        } else {
            delta.header.units.push(unit.clone());
        }
    }
    delta.header.units.extend(changed);
    Some(delta.header)
}
