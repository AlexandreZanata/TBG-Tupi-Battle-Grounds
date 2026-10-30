//! Stack-list afflictions (P45): Rotcap poison and Old Finn fish.
//! Fixed arrays per victim with a shared cap; at the cap a new application
//! replaces the entry with the least time left. Poison pulses ride the
//! 20-tick period; every stack pulses exactly ten times.

use crate::content::REGEN_PERIOD;
use crate::state::{Event, MatchState, Stack, Target};
use crate::types::UnitId;

/// Add one stack: the first empty slot, else the entry with the least time
/// left. `cap` is the victim's shared cap for this list.
pub(crate) fn add_stack(slots: &mut [Stack], cap: usize, left: u16, value: u16) {
    let cap = cap.min(slots.len());
    if cap == 0 {
        return;
    }
    if let Some(s) = slots[..cap].iter_mut().find(|s| s.left == 0) {
        *s = Stack { left, value };
        return;
    }
    if let Some(s) = slots[..cap].iter_mut().min_by_key(|s| s.left) {
        *s = Stack { left, value };
    }
}

/// Countdown, then poison pulses on 20-tick boundaries: one summed,
/// non-hit damage call per victim (damage reduction and Second Wind apply;
/// no on-hit effects, no lifesteal) plus one `PoisonTick` event.
pub(crate) fn tick(state: &mut MatchState, ev: &mut Vec<Event>) {
    for u in &mut state.units {
        for s in u.fx.poison.iter_mut().chain(u.fx.fish.iter_mut()) {
            s.left = s.left.saturating_sub(1);
        }
    }
    if state.tick % REGEN_PERIOD != 0 {
        return;
    }
    let mut pulses: Vec<(UnitId, i32)> = Vec::new();
    for u in &state.units {
        if u.hp <= 0 {
            continue;
        }
        let total: i64 = u
            .fx
            .poison
            .iter()
            .filter(|s| s.left > 0)
            .map(|s| i64::from(s.value))
            .sum();
        if total > 0 {
            pulses.push((u.id, total.clamp(0, i64::from(i32::MAX)) as i32));
        }
    }
    for (id, total) in pulses {
        let removed = crate::hits::damage(state, Target::Unit(id), total, ev);
        ev.push(Event::PoisonTick {
            unit: id,
            amount: removed,
        });
    }
}
