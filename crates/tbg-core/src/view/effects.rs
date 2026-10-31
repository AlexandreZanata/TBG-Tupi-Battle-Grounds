//! Bounded public roster-effect facts reconstructed for seat and spectator reconnects.

use crate::{
    state::{Bleed, Unit},
    types::{Status, Tick, UnitId},
    Ruleset,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedBp {
    pub bp: u16,
    pub ticks: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DotView {
    pub value: u16,
    pub pulses: u8,
    pub next_in: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkView {
    pub target: UnitId,
    pub ticks: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectsView {
    pub mark: Option<MarkView>,
    pub bleed: Option<DotView>,
    pub venom: Option<DotView>,
    pub weaken: Option<TimedBp>,
    pub heal_cut: Option<TimedBp>,
    pub queued_slow: Option<crate::state::QueuedSlow>,
    pub slow_bp: u16,
    pub control_ticks: u16,
    pub ward_ticks: u16,
    pub reduction_ticks: u16,
    pub shield_ticks: u16,
    /// Zero means no temporary lifetime; expired summons are removed before views.
    pub life: u16,
}
fn left(until: Tick, now: Tick) -> u16 {
    until.saturating_sub(now).min(u32::from(u16::MAX)) as u16
}
fn timed(bp: u16, until: Tick, now: Tick) -> Option<TimedBp> {
    (bp > 0 && now < until).then(|| TimedBp {
        bp,
        ticks: left(until, now),
    })
}
fn dot(slot: &Bleed, now: Tick) -> Option<DotView> {
    (slot.left > 0).then(|| DotView {
        value: slot.value,
        pulses: slot.left,
        next_in: left(slot.next, now),
    })
}
fn cut(u: &Unit, now: Tick) -> Option<TimedBp> {
    let a = timed(
        u.heal_cut_bp.clamp(0, i32::from(u16::MAX)) as u16,
        u.heal_cut_until,
        now,
    );
    let b = timed(u.fx.ability_cut_bp, u.fx.ability_cut_until, now);
    match (a, b) {
        (Some(a), Some(b)) if a.bp == b.bp => Some(TimedBp {
            bp: a.bp,
            ticks: a.ticks.max(b.ticks),
        }),
        (Some(a), Some(b)) => Some(if a.bp > b.bp { a } else { b }),
        (a, b) => a.or(b),
    }
}
pub(super) fn project(rules: &Ruleset, u: &Unit, now: Tick) -> Option<EffectsView> {
    let (slow_bp, control_ticks) = match u.status {
        Status::Slowed { bp, ticks_left } => (bp, ticks_left),
        Status::Rooted { ticks_left } => (0, ticks_left),
        Status::None => (0, 0),
    };
    let mark = if crate::crowcaller::parameters(rules, u).is_some() && now < u.fx.mark_until {
        u.fx.mark_target.map(|target| MarkView {
            target,
            ticks: left(u.fx.mark_until, now),
        })
    } else {
        None
    };
    let effects = EffectsView {
        mark,
        bleed: dot(&u.fx.bleed, now),
        venom: dot(&u.fx.venom, now),
        weaken: timed(u.fx.weaken_bp, u.fx.weaken_until, now),
        heal_cut: cut(u, now),
        queued_slow: u.fx.queued_slow,
        slow_bp,
        control_ticks,
        ward_ticks: left(u.fx.ward_until, now),
        reduction_ticks: if crate::roster_slots::self_dr_now(&u.fx, now) > 0 {
            left(u.fx.self_dr_until, now)
        } else {
            0
        },
        shield_ticks: if crate::roster_slots::shield_now(&u.fx, now) > 0 {
            left(u.fx.shield_until, now)
        } else {
            0
        },
        life: u.expires_at.map_or(0, |until| left(until, now)),
    };
    (effects != EffectsView::default()).then_some(effects)
}
