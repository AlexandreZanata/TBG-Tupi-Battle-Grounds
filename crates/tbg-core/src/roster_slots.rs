//! W0 slots of the roster expansion (P58 §3): one bounded field each for a
//! victim's bleed, a weakening of basic attacks, a Slow queued behind a Root,
//! a caster's mark, a timed self-reduction and an ability heal-cut. No unit
//! uses them yet, so an empty slot changes nothing; the rules here are the
//! contract that the unit waves build on.
#![allow(dead_code)] // ponytail: the appliers have no caller until the first unit that uses each slot (W2–W6); drop this allow then.

use crate::hits;
use crate::state::{Bleed, Event, MatchState, QueuedSlow, Target, Unit, UnitFx};
use crate::types::{Seat, Status, Tick, UnitId};

/// Ticks between bleed pulses: one second, at 20 simulation steps a second.
pub(crate) const BLEED_PERIOD: Tick = 20;

/// The most that the weakening family may take off basic attacks (R13).
pub(crate) const WEAKEN_CAP_BP: u16 = 2_000;

/// The stronger, equal or weaker rule for a timed basis-point slot: a stronger
/// value replaces it and restarts the timer, an equal one refreshes the timer,
/// and a weaker one is ignored until the slot expires.
fn apply_timed(bp_slot: &mut u16, until: &mut Tick, bp: u16, ticks: u16, now: Tick) {
    if now < *until && bp < *bp_slot {
        return;
    }
    *bp_slot = bp;
    *until = now.saturating_add(Tick::from(ticks));
}

fn timed_now(bp: u16, until: Tick, now: Tick) -> u16 {
    if now < until {
        bp
    } else {
        0
    }
}

/// R13: basic attacks this unit makes lose `bp`, capped at the family limit.
pub(crate) fn apply_weaken(fx: &mut UnitFx, bp: u16, ticks: u16, now: Tick) {
    apply_timed(
        &mut fx.weaken_bp,
        &mut fx.weaken_until,
        bp.min(WEAKEN_CAP_BP),
        ticks,
        now,
    );
}

/// The weakening in force, in basis points.
pub(crate) fn weaken_now(fx: &UnitFx, now: Tick) -> i32 {
    i32::from(timed_now(fx.weaken_bp, fx.weaken_until, now))
}

/// A timed self-reduction (R01, R06, R08, R16). It joins the damage reduction
/// sum in `hits::damage`, so it shares that sum's 20 % cap.
pub(crate) fn apply_self_dr(fx: &mut UnitFx, bp: u16, ticks: u16, now: Tick) {
    apply_timed(&mut fx.self_dr_bp, &mut fx.self_dr_until, bp, ticks, now);
}

/// The self-reduction in force, in basis points.
pub(crate) fn self_dr_now(fx: &UnitFx, now: Tick) -> i32 {
    i32::from(timed_now(fx.self_dr_bp, fx.self_dr_until, now))
}

/// The unit shield (R12, R14; D-63): one amount and one expiry per unit. A
/// stronger grant replaces the amount, an equal one refreshes the expiry, and a
/// weaker one is ignored until the held shield runs out. A shield never heals
/// or revives; `hits::damage` consumes it before HP.
pub(crate) fn apply_shield(fx: &mut UnitFx, amount: u16, ticks: u16, now: Tick) {
    if amount == 0 || amount < shield_now(fx, now) {
        return;
    }
    fx.shield = amount;
    fx.shield_until = now.saturating_add(Tick::from(ticks));
}

/// The shield still standing at `now`; an expired one counts as none.
pub(crate) fn shield_now(fx: &UnitFx, now: Tick) -> u16 {
    if now < fx.shield_until {
        fx.shield
    } else {
        0
    }
}

/// The ability heal-cut slot (R03, R20): a second, bounded cut beside the pair
/// that lives in `Unit`.
pub(crate) fn apply_ability_cut(fx: &mut UnitFx, bp: u16, ticks: u16, now: Tick) {
    apply_timed(
        &mut fx.ability_cut_bp,
        &mut fx.ability_cut_until,
        bp,
        ticks,
        now,
    );
}

/// The heal cut in force: the stronger of the pair and the ability slot. The
/// two are never summed (P58 §3.2).
pub(crate) fn heal_cut_now(u: &Unit, now: Tick) -> i32 {
    let pair = if now < u.heal_cut_until {
        u.heal_cut_bp
    } else {
        0
    };
    let ability = i32::from(timed_now(u.fx.ability_cut_bp, u.fx.ability_cut_until, now));
    pair.max(ability)
}

/// R07: a victim's bleed. An empty slot starts its first pulse one period after
/// application. An active slot takes an equal or stronger value and restarts
/// its count without moving the next pulse; a weaker value is ignored.
pub(crate) fn apply_bleed(fx: &mut UnitFx, value: u16, pulses: u8, source: Seat, now: Tick) {
    let slot = &mut fx.bleed;
    if slot.left == 0 {
        *slot = Bleed {
            value,
            next: now.saturating_add(BLEED_PERIOD),
            left: pulses,
            source: Some(source),
        };
    } else if value >= slot.value {
        slot.value = value;
        slot.left = pulses;
        slot.source = Some(source);
    }
}

/// Bleed pulses for this tick. A due slot deals its value once through the
/// ordinary damage path (reduction and shields apply; no on-hit effects, no
/// lifesteal), then counts down. The source's death changes nothing.
pub(crate) fn tick_bleed(state: &mut MatchState, ev: &mut Vec<Event>) {
    for i in 0..state.units.len() {
        let u = state.units[i];
        let slot = u.fx.bleed;
        if u.hp <= 0 || slot.left == 0 || state.tick != slot.next {
            continue;
        }
        hits::damage(state, Target::Unit(u.id), i32::from(slot.value), ev);
        let slot = &mut state.units[i].fx.bleed;
        slot.left -= 1;
        if slot.left == 0 {
            *slot = Bleed::default();
        } else {
            slot.next = slot.next.saturating_add(BLEED_PERIOD);
        }
    }
}

/// A cleanse takes the bleed before any ordinary poison stack (P58 §3.2).
/// Returns true when a bleed was taken.
pub(crate) fn cleanse_bleed(fx: &mut UnitFx) -> bool {
    if fx.bleed.left == 0 {
        return false;
    }
    fx.bleed = Bleed::default();
    true
}

/// R19: one Slow waiting for the victim's Root to end. As with an active Slow,
/// a weaker one keeps the queued amount and only extends the duration.
pub(crate) fn queue_slow(fx: &mut UnitFx, bp: u16, ticks: u16) {
    fx.queued_slow = Some(match fx.queued_slow {
        Some(q) if bp <= q.bp => QueuedSlow {
            bp: q.bp,
            ticks: q.ticks.max(ticks),
        },
        _ => QueuedSlow { bp, ticks },
    });
}

/// When a Root has just ended in `tick_timers`, the queued Slow starts in its
/// place. A Root that is still running, or one that was refreshed, releases
/// nothing.
pub(crate) fn release_queued_slow(u: &mut Unit, before: Status) {
    let was_rooted = matches!(before, Status::Rooted { .. });
    if was_rooted && !matches!(u.status, Status::Rooted { .. }) {
        if let Some(q) = u.fx.queued_slow.take() {
            u.status = Status::Slowed {
                bp: q.bp,
                ticks_left: q.ticks,
            };
        }
    }
}

/// R03: a caster's mark. A new mark moves it; moving it does not strip the old
/// target's Slow or cut.
pub(crate) fn set_mark(fx: &mut UnitFx, target: UnitId, until: Tick) {
    fx.mark_target = Some(target);
    fx.mark_until = until;
}

/// Clears a mark once its deadline passes or its target is dead or gone.
pub(crate) fn tick_marks(state: &mut MatchState) {
    for i in 0..state.units.len() {
        let fx = state.units[i].fx;
        let Some(target) = fx.mark_target else {
            continue;
        };
        let alive = state
            .unit_index(target)
            .is_some_and(|t| state.units[t].hp > 0);
        if state.tick >= fx.mark_until || !alive {
            let fx = &mut state.units[i].fx;
            fx.mark_target = None;
            fx.mark_until = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat;
    use crate::types::UnitKind;

    fn unit(id: UnitId, seat: Seat) -> Unit {
        Unit {
            id,
            seat,
            kind: UnitKind::Archer,
            tier: 0,
            branch: 0,
            x: 0,
            hp: 1_000,
            max_hp: 1_000,
            cooldown: 0,
            windup: None,
            status: Status::None,
            summoner: None,
            expires_at: None,
            action_cd: 0,
            haste_until: 0,
            heal_cut_until: 0,
            heal_cut_bp: 0,
            second_wind_used: 0,
            fx: UnitFx::default(),
        }
    }

    fn state_with(units: Vec<Unit>) -> MatchState {
        let mut s = MatchState::new(1);
        s.units = units;
        s
    }

    #[test]
    fn bleed_is_one_slot_that_refreshes_and_takes_the_stronger_value() {
        let mut fx = UnitFx::default();
        apply_bleed(&mut fx, 5, 4, Seat::A, 100);
        assert_eq!((fx.bleed.value, fx.bleed.next, fx.bleed.left), (5, 120, 4));
        // A weaker application is ignored until the slot empties.
        apply_bleed(&mut fx, 3, 4, Seat::B, 110);
        assert_eq!((fx.bleed.value, fx.bleed.source), (5, Some(Seat::A)));
        // An equal one refreshes the count and keeps the next pulse.
        fx.bleed.left = 2;
        apply_bleed(&mut fx, 5, 4, Seat::B, 110);
        assert_eq!((fx.bleed.left, fx.bleed.next), (4, 120));
        // A stronger one takes the value, still without moving the pulse.
        apply_bleed(&mut fx, 7, 4, Seat::B, 111);
        assert_eq!((fx.bleed.value, fx.bleed.next), (7, 120));
    }

    #[test]
    fn timed_slots_replace_when_stronger_and_refresh_when_equal() {
        let mut fx = UnitFx::default();
        apply_weaken(&mut fx, 1_500, 60, 100);
        assert_eq!(weaken_now(&fx, 159), 1_500);
        assert_eq!(weaken_now(&fx, 160), 0, "gone after its ticks");
        apply_weaken(&mut fx, 1_000, 60, 150);
        assert_eq!(weaken_now(&fx, 155), 1_500, "weaker while active: ignored");
        apply_weaken(&mut fx, 1_500, 60, 150);
        assert_eq!(weaken_now(&fx, 209), 1_500, "equal refreshes the timer");
        apply_weaken(&mut fx, 9_000, 30, 180);
        assert_eq!(
            weaken_now(&fx, 180),
            2_000,
            "stronger, capped at the family limit"
        );
    }

    #[test]
    fn the_ability_heal_cut_is_the_stronger_of_the_pair_and_its_slot() {
        let mut u = unit(1, Seat::A);
        u.heal_cut_until = 50;
        u.heal_cut_bp = 3_000;
        apply_ability_cut(&mut u.fx, 3_500, 40, 10);
        assert_eq!(
            heal_cut_now(&u, 20),
            3_500,
            "the stronger wins, never summed"
        );
        u.heal_cut_bp = 5_000;
        assert_eq!(heal_cut_now(&u, 20), 5_000);
        assert_eq!(heal_cut_now(&u, 60), 0, "both have expired");
    }

    #[test]
    fn bleed_pulses_four_times_one_second_apart() {
        let mut s = state_with(vec![unit(1, Seat::B)]);
        let mut ev = Vec::new();
        apply_bleed(&mut s.units[0].fx, 5, 4, Seat::A, 0);
        for t in 1..=100 {
            s.tick = t;
            tick_bleed(&mut s, &mut ev);
        }
        assert_eq!(s.units[0].hp, 980, "four pulses of 5");
        assert_eq!(s.units[0].fx.bleed, Bleed::default(), "the slot is empty");
    }

    #[test]
    fn a_self_reduction_shares_the_damage_cap() {
        let mut victim = unit(1, Seat::B);
        victim.fx.march_dr_bp = 1_000;
        let mut s = state_with(vec![victim]);
        s.tick = 100;
        apply_self_dr(&mut s.units[0].fx, 1_500, 60, 100);
        let mut ev = Vec::new();
        // 1 000 less 20 % (the sum is 2 500 bp, capped at 2 000 bp), not 25 %.
        hits::damage(&mut s, Target::Unit(1), 1_000, &mut ev);
        assert_eq!(s.units[0].hp, 1_000 - 800);
    }

    #[test]
    fn a_stronger_shield_replaces_an_equal_one_refreshes_and_a_weaker_one_is_ignored() {
        let mut fx = UnitFx::default();
        apply_shield(&mut fx, 300, 60, 100);
        assert_eq!((fx.shield, fx.shield_until), (300, 160));
        // A weaker grant is ignored, its expiry included.
        apply_shield(&mut fx, 200, 60, 110);
        assert_eq!((fx.shield, fx.shield_until), (300, 160));
        // An equal one refreshes the expiry only.
        apply_shield(&mut fx, 300, 60, 120);
        assert_eq!((fx.shield, fx.shield_until), (300, 180));
        // A stronger one takes the amount.
        apply_shield(&mut fx, 500, 60, 130);
        assert_eq!((fx.shield, fx.shield_until), (500, 190));
    }

    #[test]
    fn an_expired_shield_counts_as_none_and_a_zero_grant_does_nothing() {
        let mut fx = UnitFx::default();
        apply_shield(&mut fx, 300, 60, 100);
        assert_eq!(shield_now(&fx, 159), 300);
        assert_eq!(shield_now(&fx, 160), 0);
        apply_shield(&mut fx, 0, 60, 200);
        assert_eq!(shield_now(&fx, 200), 0);
        // After expiry any grant starts from zero, even a weaker one.
        apply_shield(&mut fx, 100, 60, 200);
        assert_eq!((fx.shield, fx.shield_until), (100, 260));
    }

    #[test]
    fn a_shield_takes_damage_before_hp_and_is_gone_at_its_expiry() {
        let mut s = state_with(vec![unit(1, Seat::B)]);
        s.tick = 100;
        apply_shield(&mut s.units[0].fx, 300, 60, 100);
        let mut ev = Vec::new();
        // The shield takes 300 of 500; HP takes the other 200.
        hits::damage(&mut s, Target::Unit(1), 500, &mut ev);
        assert_eq!((s.units[0].hp, s.units[0].fx.shield), (800, 0));
        // A shield that expires at tick 210 absorbs nothing at tick 210.
        apply_shield(&mut s.units[0].fx, 300, 10, 200);
        s.tick = 210;
        hits::damage(&mut s, Target::Unit(1), 100, &mut ev);
        assert_eq!(s.units[0].hp, 700);
    }

    #[test]
    fn queued_slow_starts_when_the_root_ends() {
        let mut u = unit(1, Seat::B);
        u.status = Status::Rooted { ticks_left: 2 };
        queue_slow(&mut u.fx, 2_000, 40);
        let mut s = state_with(vec![u]);
        combat::tick_timers(&mut s);
        assert!(matches!(
            s.units[0].status,
            Status::Rooted { ticks_left: 1 }
        ));
        combat::tick_timers(&mut s);
        assert_eq!(
            s.units[0].status,
            Status::Slowed {
                bp: 2_000,
                ticks_left: 40
            }
        );
        assert_eq!(s.units[0].fx.queued_slow, None);
    }

    #[test]
    fn a_mark_clears_when_its_target_dies_or_its_deadline_passes() {
        let mut s = state_with(vec![unit(1, Seat::A), unit(2, Seat::B)]);
        set_mark(&mut s.units[0].fx, 2, 100);
        s.tick = 50;
        tick_marks(&mut s);
        assert_eq!(s.units[0].fx.mark_target, Some(2));
        s.units[1].hp = 0;
        tick_marks(&mut s);
        assert_eq!(s.units[0].fx.mark_target, None, "dead target clears it");
        s.units[1].hp = 1_000;
        set_mark(&mut s.units[0].fx, 2, 100);
        s.tick = 100;
        tick_marks(&mut s);
        assert_eq!(s.units[0].fx.mark_target, None, "deadline clears it");
    }

    #[test]
    fn a_cleanse_takes_the_bleed_first_and_only_once() {
        let mut fx = UnitFx::default();
        assert!(!cleanse_bleed(&mut fx), "an empty slot has nothing to take");
        apply_bleed(&mut fx, 5, 4, Seat::A, 0);
        assert!(cleanse_bleed(&mut fx));
        assert_eq!(fx.bleed.left, 0);
        assert!(!cleanse_bleed(&mut fx));
    }
}
