//! Deterministic fusion inheritance; merging never cleanses a harmful family or sums shields.
use crate::{
    state::{Bleed, QueuedSlow, Stack, Unit, UnitFx},
    types::{Status, Tick},
};
use std::cmp::Reverse;
fn stacks<const N: usize>(a: [Stack; N], b: [Stack; N]) -> [Stack; N] {
    let mut slots: Vec<_> = a
        .into_iter()
        .chain(b)
        .enumerate()
        .filter(|(_, s)| s.left > 0)
        .collect();
    slots.sort_by_key(|(i, s)| (Reverse(s.value), Reverse(s.left), *i));
    let mut out = [Stack::default(); N];
    for (slot, (_, s)) in out.iter_mut().zip(slots) {
        *slot = s;
    }
    out
}
fn dot(a: Bleed, b: Bleed) -> Bleed {
    if a.left == 0 {
        return b;
    }
    if b.left == 0 {
        return a;
    }
    let winner = if b.value > a.value { b } else { a };
    Bleed {
        value: winner.value,
        source: winner.source,
        next: a.next.min(b.next),
        left: a.left.max(b.left),
    }
}
fn timed(a: (u16, Tick), b: (u16, Tick), now: Tick) -> (u16, Tick) {
    let a = if a.1 > now { a } else { (0, 0) };
    let b = if b.1 > now { b } else { (0, 0) };
    (a.0.max(b.0), a.1.max(b.1))
}
pub(super) fn status(a: Status, b: Status) -> Status {
    match (a, b) {
        (Status::Rooted { ticks_left: a }, Status::Rooted { ticks_left: b }) => Status::Rooted {
            ticks_left: a.max(b),
        },
        (Status::Rooted { ticks_left: a }, Status::Slowed { ticks_left: b, .. })
        | (Status::Slowed { ticks_left: b, .. }, Status::Rooted { ticks_left: a }) => {
            Status::Rooted {
                ticks_left: a.max(b),
            }
        }
        (a @ Status::Rooted { .. }, Status::None) => a,
        (Status::None, b @ Status::Rooted { .. }) => b,
        (
            Status::Slowed {
                bp: a,
                ticks_left: at,
            },
            Status::Slowed {
                bp: b,
                ticks_left: bt,
            },
        ) => Status::Slowed {
            bp: a.max(b),
            ticks_left: at.max(bt),
        },
        (Status::None, b) => b,
        (a, Status::None) => a,
    }
}
pub(super) fn inherit(a: &Unit, b: &Unit, now: Tick) -> UnitFx {
    let (weaken_bp, weaken_until) = timed(
        (a.fx.weaken_bp, a.fx.weaken_until),
        (b.fx.weaken_bp, b.fx.weaken_until),
        now,
    );
    let (ability_cut_bp, ability_cut_until) = timed(
        (a.fx.ability_cut_bp, a.fx.ability_cut_until),
        (b.fx.ability_cut_bp, b.fx.ability_cut_until),
        now,
    );
    let (self_dr_bp, self_dr_until) = timed(
        (a.fx.self_dr_bp, a.fx.self_dr_until),
        (b.fx.self_dr_bp, b.fx.self_dr_until),
        now,
    );
    let shield_a = crate::roster_slots::shield_now(&a.fx, now);
    let shield_b = crate::roster_slots::shield_now(&b.fx, now);
    let (shield, shield_until) = if shield_b > shield_a {
        (shield_b, b.fx.shield_until)
    } else if shield_a > shield_b {
        (shield_a, a.fx.shield_until)
    } else {
        (shield_a, a.fx.shield_until.max(b.fx.shield_until))
    };
    let queued_slow = match (a.fx.queued_slow, b.fx.queued_slow) {
        (Some(a), Some(b)) => Some(QueuedSlow {
            bp: a.bp.max(b.bp),
            ticks: a.ticks.max(b.ticks),
        }),
        (a, None) => a,
        (None, b) => b,
    };
    UnitFx {
        poison: stacks(a.fx.poison, b.fx.poison),
        fish: stacks(a.fx.fish, b.fx.fish),
        bleed: dot(a.fx.bleed, b.fx.bleed),
        venom: dot(a.fx.venom, b.fx.venom),
        weaken_bp,
        weaken_until,
        ability_cut_bp,
        ability_cut_until,
        self_dr_bp,
        self_dr_until,
        shield,
        shield_until,
        queued_slow,
        confused_until: a.fx.confused_until.max(b.fx.confused_until),
        immune_until: a.fx.immune_until.max(b.fx.immune_until),
        ward_until: a.fx.ward_until.max(b.fx.ward_until),
        ..UnitFx::default()
    }
}
pub(super) fn pair_cut(a: &Unit, b: &Unit, now: Tick) -> (i32, Tick) {
    let a = if a.heal_cut_until > now {
        (a.heal_cut_bp, a.heal_cut_until)
    } else {
        (0, 0)
    };
    let b = if b.heal_cut_until > now {
        (b.heal_cut_bp, b.heal_cut_until)
    } else {
        (0, 0)
    };
    (a.0.max(b.0), a.1.max(b.1))
}
