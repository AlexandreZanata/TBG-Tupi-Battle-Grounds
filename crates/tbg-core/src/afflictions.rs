//! Stack-list afflictions (P45): Rotcap poison and Old Finn fish.
//! Fixed arrays per victim with a shared cap; at the cap a new application
//! replaces the entry with the least time left. Poison pulses ride the
//! 20-tick period; every stack pulses exactly ten times.

use std::cmp::Reverse;

use crate::content::{Ability, Stats, REGEN_PERIOD};
use crate::state::{Event, MatchState, Stack, Target, Unit};
use crate::types::{Status, UnitId, UnitKind};

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
    if !state.tick.is_multiple_of(REGEN_PERIOD) {
        return;
    }
    let mut pulses: Vec<(UnitId, i32)> = Vec::new();
    for u in &state.units {
        if u.hp <= 0 {
            continue;
        }
        let total: i64 =
            u.fx.poison
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

/// The attacker's ability pair: form slots are absolute (F.1), so the
/// current tier's slot is the whole answer — no cross-tier fallback.
pub(crate) fn abilities_of(state: &MatchState, attacker: &Unit) -> [Ability; 2] {
    state
        .rules()
        .def(attacker.kind)
        .ability(attacker.tier, attacker.branch)
}

/// Highest active Brew params, if any.
pub(crate) fn brew_of(state: &MatchState, attacker: &Unit) -> Option<(u8, u16, bool)> {
    abilities_of(state, attacker)
        .into_iter()
        .find_map(|ability| match ability {
            Ability::Brew {
                strip_poison,
                ward,
                second,
            } => Some((strip_poison, ward, second)),
            _ => None,
        })
}

/// Apply the attacker's Poison/Fish stacks to one enemy unit: warded units
/// and corpses take nothing. Emits `StatusApplied` per applied stack type.
fn apply_types(state: &mut MatchState, attacker: &Unit, st: &Stats, id: u32, ev: &mut Vec<Event>) {
    let Some(i) = state.unit_index(id) else {
        return;
    };
    if state.units[i].hp <= 0 || state.tick < state.units[i].fx.ward_until {
        return;
    }
    for ability in abilities_of(state, attacker) {
        match ability {
            Ability::Poison { dps, pulses, cap } => {
                // Fixed when applied: the static damage bonus scales the stack, so
                // later buffs never rewrite it (§C.1).
                let base = state.rules().def(attacker.kind).damage.max(1);
                let pulse = (i64::from(dps) * i64::from(st.damage) / i64::from(base)) as i32;
                let pulse = pulse.clamp(0, i32::MAX) as u16;
                // The tail covers the tenth pulse on the 20-tick grid.
                let since = state.tick % REGEN_PERIOD;
                let delta = if since == 0 {
                    REGEN_PERIOD
                } else {
                    REGEN_PERIOD - since
                };
                let left = u32::from(pulses)
                    .saturating_mul(REGEN_PERIOD)
                    .saturating_add(delta)
                    .min(u32::from(u16::MAX)) as u16;
                add_stack(&mut state.units[i].fx.poison, usize::from(cap), left, pulse);
                ev.push(Event::StatusApplied { unit: id });
            }
            Ability::Fish {
                bp,
                ticks,
                cap,
                per_hit,
            } => {
                for _ in 0..per_hit {
                    add_stack(&mut state.units[i].fx.fish, usize::from(cap), ticks, bp);
                }
                ev.push(Event::StatusApplied { unit: id });
            }
            _ => {}
        }
    }
}

/// Leechroot Essence (P48-A, §C.3): HP removed by hits from enemy
/// conventional units fills Essence; the n-th stack costs `base + 2 × n`.
/// Stacks never cap; all stacks are lost on death (fresh `UnitFx`).
/// Not Strongholds, summons, betrayals, or poison pulses: those never pass
/// through the primary-hit hook that calls this.
pub(crate) fn feed_essence(state: &mut MatchState, attacker: &Unit, removed: i32) {
    let [a, b] = abilities_of(state, attacker);
    let essence = [a, b].into_iter().find_map(|ability| match ability {
        Ability::Essence {
            base,
            step,
            dmg: _,
            atk_bp: _,
        } => Some((base, step)),
        _ => None,
    });
    let Some((base, step)) = essence else {
        return;
    };
    let Some(ai) = state.unit_index(attacker.id) else {
        return;
    };
    let mut stacks = u32::from(state.units[ai].fx.essence);
    let mut progress = state.units[ai]
        .fx
        .essence_progress
        .saturating_add(removed.max(0) as u32);
    // The cost rises with every stack, so the loop always terminates.
    let mut cost = u32::from(base).saturating_add(u32::from(step).saturating_mul(stacks));
    while progress >= cost.max(1) && stacks < u32::from(u16::MAX) {
        progress -= cost.max(1);
        stacks += 1;
        cost = u32::from(base).saturating_add(u32::from(step).saturating_mul(stacks));
    }
    state.units[ai].fx.essence = stacks.min(u32::from(u16::MAX)) as u16;
    state.units[ai].fx.essence_progress = progress;
}

/// Apply stacks for a primary hit, then count the attack toward the echo and
/// fire it (Rotcap B, Old Finn B): every `every`-th attack also stacks up to
/// `extra` nearest other enemies in range.
pub(crate) fn apply_stacks(
    state: &mut MatchState,
    attacker: &Unit,
    st: &Stats,
    id: u32,
    ev: &mut Vec<Event>,
) {
    apply_types(state, attacker, st, id, ev);
    let echo = abilities_of(state, attacker)
        .into_iter()
        .find_map(|ability| match ability {
            Ability::Echo { every, extra } => Some((every, extra)),
            _ => None,
        });
    let Some((every, extra)) = echo else {
        return;
    };
    let every = every.max(1);
    let Some(ai) = state.unit_index(attacker.id) else {
        return;
    };
    let n = state.units[ai].fx.attacks.saturating_add(1);
    if n.is_multiple_of(every) {
        state.units[ai].fx.attacks = 0;
        let mut others: Vec<(i32, UnitId)> = state
            .units
            .iter()
            .filter(|e| {
                e.seat != attacker.seat
                    && e.hp > 0
                    && e.id != id
                    && crate::targeting::targetable(state, e.id)
                    && (e.x - attacker.x).abs() <= st.range
            })
            .map(|e| ((e.x - attacker.x).abs(), e.id))
            .collect();
        others.sort();
        for (_, eid) in others.into_iter().take(usize::from(extra)) {
            apply_types(state, attacker, st, eid, ev);
        }
    } else {
        state.units[ai].fx.attacks = n;
    }
}

/// Oilpress A2 Immovable Mass (§C.8): cannot be slowed, rooted or
/// confused. Poison and fish stacks still apply (fish only stops slowing
/// it); the ward and immunity compose independently.
pub(crate) fn is_immune_to_control(u: &Unit) -> bool {
    u.kind == UnitKind::Oilpress && u.tier == 2 && u.branch == 0
}

/// Affliction score for the Antidote's brew choice (§C.9): Rooted or
/// Confused 4, +1 per poison stack, Slowed 2, +1 per fish.
pub(crate) fn affliction_score(state: &MatchState, idx: usize) -> u32 {
    let u = &state.units[idx];
    let mut score = match u.status {
        Status::Rooted { .. } => 4,
        Status::Slowed { .. } => 2,
        Status::None => 0,
    };
    if state.tick < u.fx.confused_until {
        score += 4;
    }
    score += u.fx.poison.iter().filter(|s| s.left > 0).count() as u32;
    score += u.fx.fish.iter().filter(|s| s.left > 0).count() as u32;
    score
}

/// The Antidote's brew target: the most afflicted ally in range
/// (conventional or summon, itself included); ties nearest, then lowest id.
/// `exclude` skips one unit (the Double Flask second brew). Nobody
/// afflicted means no target: the Antidote waits ready.
pub(crate) fn brew_target(
    state: &MatchState,
    u: &Unit,
    range: i32,
    exclude: Option<UnitId>,
) -> Option<UnitId> {
    state
        .units
        .iter()
        .enumerate()
        .filter(|(_, a)| {
            a.seat == u.seat && a.hp > 0 && Some(a.id) != exclude && (a.x - u.x).abs() <= range
        })
        .map(|(i, a)| (affliction_score(state, i), (a.x - u.x).abs(), a.id))
        .filter(|(score, _, _)| *score > 0)
        .min()
        .map(|(_, _, id)| id)
}

/// One cleanse: strip slows, roots and confusion, up to `strip` poison
/// stacks (strongest first), all fish, then optionally ward.
fn cleanse(
    state: &mut MatchState,
    id: u32,
    strip: u8,
    ward_ticks: u16,
    ward: bool,
    ev: &mut Vec<Event>,
) {
    let Some(i) = state.unit_index(id) else {
        return;
    };
    let until = state.tick.saturating_add(u32::from(ward_ticks));
    let u = &mut state.units[i];
    u.status = Status::None;
    u.fx.confused_until = 0;
    let mut order: Vec<usize> = (0..u.fx.poison.len()).collect();
    order.sort_by_key(|&k| (Reverse(u.fx.poison[k].value), u.fx.poison[k].left));
    for k in order.into_iter().take(usize::from(strip)) {
        u.fx.poison[k] = Stack::default();
    }
    for s in u.fx.fish.iter_mut() {
        *s = Stack::default();
    }
    if ward {
        u.fx.ward_until = until;
    }
    ev.push(Event::Cleansed { unit: id });
}

/// The Antidote's brew (§C.9): cleanse the most afflicted ally; Double Flask
/// also cleanses the next one, without the ward.
#[allow(clippy::too_many_arguments)] // one call site; reads as a table
pub(crate) fn brew_cast(
    state: &mut MatchState,
    attacker: &Unit,
    range: i32,
    id: u32,
    strip: u8,
    ward_ticks: u16,
    second: bool,
    ev: &mut Vec<Event>,
) {
    cleanse(state, id, strip, ward_ticks, true, ev);
    if second {
        if let Some(id2) = brew_target(state, attacker, range, Some(id)) {
            cleanse(state, id2, strip, ward_ticks, false, ev);
        }
    }
}
