//! OG-02: authoritative swept rolling barrels, finite travel and collision-centered area healing.
#[cfg(any(test, feature = "lab"))]
use crate::UnitKind;
use crate::{completion, content::Stats, Event, MatchState, Seat, Target, Tick, Unit, UnitId};
use serde::{Deserialize, Serialize};

/// Public immutable launch facts plus the current integer travel position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Barrel {
    pub source: UnitId,
    pub seat: Seat,
    pub launched: Tick,
    pub tier: u8,
    pub branch: u8,
    pub origin_x: i32,
    pub origin_z: i32,
    pub target: UnitId,
    pub x: i32,
    pub direction: i32,
    pub remaining: i32,
    pub speed: i32,
    pub radius: i32,
    pub heal: i32,
    pub damage: i32,
}
/// A swept contact radius, independent of cosmetic mesh spacing.
pub const CONTACT_RADIUS: i32 = 180;

pub(crate) fn launch(state: &mut MatchState, u: &Unit, target: UnitId, st: &Stats) -> bool {
    let Some(completion::Ability::Barrel {
        speed,
        radius,
        damage,
    }) = completion::of(state, u)
    else {
        return false;
    };
    let Some(v) = state.unit_index(target).map(|i| state.units[i]) else {
        return true;
    };
    if v.id == u.id
        || v.hp <= 0
        || v.seat != u.seat
        || state.barrels.iter().any(|b| b.source == u.id)
    {
        return true;
    }
    state.barrels.push(Barrel {
        source: u.id,
        seat: u.seat,
        launched: state.tick,
        tier: u.tier,
        branch: u.branch,
        origin_x: u.x,
        origin_z: crate::occupancy::lateral(u),
        target,
        x: u.x,
        direction: if v.x < u.x { -1 } else { 1 },
        remaining: st.range,
        speed,
        radius,
        heal: st.heal,
        damage: crate::hits::auxiliary_amount(st, damage),
    });
    true
}

pub(crate) fn advance(state: &mut MatchState, ev: &mut Vec<Event>) {
    let mut active = std::mem::take(&mut state.barrels);
    active.retain_mut(|b| {
        let step = b.speed.min(b.remaining).max(0);
        let contact = state
            .units
            .iter()
            .filter(|u| {
                if u.hp <= 0 || u.id == b.source || !crate::targeting::targetable(u) {
                    return false;
                }
                let along = (u.x - b.x) * b.direction;
                (-CONTACT_RADIUS..=step + CONTACT_RADIUS).contains(&along)
            })
            .map(|u| (((u.x - b.x) * b.direction).max(0), u.id))
            .min();
        if let Some((distance, id)) = contact {
            b.x += b.direction * distance.saturating_sub(CONTACT_RADIUS).max(0).min(step);
            burst(state, b, id, ev);
            return false;
        }
        b.x = (b.x + b.direction * step).clamp(0, crate::content::LANE_LEN);
        b.remaining -= step;
        b.remaining > 0
    });
    state.barrels = active;
}

fn burst(state: &mut MatchState, b: &Barrel, contact: UnitId, ev: &mut Vec<Event>) {
    let enemy = state
        .unit_index(contact)
        .is_some_and(|i| state.units[i].seat != b.seat);
    if enemy {
        crate::training_hooks::scoped(
            state,
            crate::training_hooks::Source::Unit(b.source),
            crate::training_hooks::Effect::Direct,
            |s| {
                let _removed = crate::hits::damage(s, Target::Unit(contact), b.damage, ev);
                #[cfg(feature = "lab")]
                crate::lab::dot_dealt(s, Some(b.seat), UnitKind::GustDrummer, _removed);
            },
        );
    }
    for i in 0..state.units.len() {
        let u = state.units[i];
        if u.hp <= 0 || u.seat != b.seat || (u.x - b.x).abs() > b.radius {
            continue;
        }
        let _actual = crate::training_hooks::scoped(
            state,
            crate::training_hooks::Source::Unit(b.source),
            crate::training_hooks::Effect::Splash,
            |s| crate::hits::heal(s, b.seat, u.id, b.heal, true, ev),
        );
        #[cfg(feature = "lab")]
        crate::lab::healed_kind(state, b.seat, UnitKind::GustDrummer, _actual);
    }
    ev.push(Event::BarrelBurst {
        source: b.source,
        seat: b.seat,
        contact,
        center: b.x,
        center_z: state
            .unit_index(contact)
            .map_or(0, |i| crate::occupancy::lateral(&state.units[i])),
        radius: b.radius,
        tier: b.tier,
        branch: b.branch,
        tick: state.tick,
    });
}

#[cfg(all(test, feature = "lab"))]
mod tests;
