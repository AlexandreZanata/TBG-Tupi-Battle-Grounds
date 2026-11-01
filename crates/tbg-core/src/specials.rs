//! Independent special readiness and locked windups, after each unit's basic action (P58 §3.1).

use crate::{
    content::{Ability, WINDUP},
    state::{Event, MatchState, PendingCast, Unit},
    types::{Status, UnitKind},
    view::SpecialView,
    Ruleset,
};

pub(crate) fn period(rules: &Ruleset, kind: UnitKind, tier: u8, branch: u8) -> Option<u16> {
    rules
        .def(kind)
        .ability(tier, branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::StickyWeb { period, .. }
            | Ability::FearPulse { period, .. }
            | Ability::LivingWall { period, .. }
            | Ability::AreaCrush { period, .. }
            | Ability::RaisedCarapace { period, .. }
            | Ability::CelestialRain { period, .. }
            | Ability::HarmonicPulse { period, .. }
            | Ability::SolarBarrier { period, .. }
            | Ability::BloodyRend { period, .. }
            | Ability::SwampBelly { period, .. }
            | Ability::DoubleMayhem { period, .. }
            | Ability::CrowMark { period, .. } => Some(period.max(1)),
            _ => None,
        })
}

pub(crate) fn view(rules: &Ruleset, u: &Unit) -> Option<SpecialView> {
    if u.is_summon()
        || period(rules, u.kind, u.tier, u.branch)
            .or_else(|| crate::drillnest::period(rules, u.kind, u.tier, u.branch))
            .is_none()
    {
        return None;
    }
    Some(SpecialView {
        ready_in: u.action_cd,
        cast: u.fx.special,
    })
}

pub(crate) fn advance(state: &mut MatchState, i: usize, ev: &mut Vec<Event>) {
    let u = state.units[i];
    if u.hp <= 0 || u.is_summon() {
        return;
    }
    if crate::crowcaller::parameters(state.rules(), &u).is_some() {
        return;
    }
    let Some(every) = period(state.rules(), u.kind, u.tier, u.branch) else {
        return;
    };
    if matches!(u.status, Status::Rooted { .. }) {
        state.units[i].fx.special = None;
        return;
    }
    if let Some(cast) = u.fx.special {
        advance_cast(state, i, &u, cast, ev);
        return;
    }
    if u.action_cd == 0 {
        start(state, i, &u, every, ev);
    }
}

fn primary(state: &MatchState, u: &Unit) -> Option<u32> {
    if crate::solar_sentinel::parameters(state, u).is_some() {
        return crate::solar_sentinel::target(state, u, None);
    }
    let range = state.stats_of(u).range + crate::weaver::extra_range(state, u);
    state
        .units
        .iter()
        .filter(|v| {
            v.hp > 0
                && v.seat != u.seat
                && crate::targeting::targetable(v)
                && (v.x - u.x).abs() <= range
        })
        .map(|v| {
            let guard = state
                .rules()
                .def(v.kind)
                .ability(v.tier, v.branch)
                .contains(&Ability::Guard);
            ((!guard), (v.x - u.x).abs(), v.id)
        })
        .min()
        .map(|(_, _, id)| id)
}

fn center(state: &MatchState, u: &Unit, target: Option<u32>) -> i32 {
    if matches!(
        u.kind,
        UnitKind::Bulwark | UnitKind::Demolisher | UnitKind::Capybara | UnitKind::Jawguard
    ) {
        return u.x;
    }
    target
        .and_then(|id| state.unit_index(id))
        .map_or(u.x, |t| state.units[t].x)
}

fn start(state: &mut MatchState, i: usize, u: &Unit, every: u16, ev: &mut Vec<Event>) {
    let Some(target) = primary(state, u) else {
        return;
    };
    let center = center(state, u, Some(target));
    state.units[i].fx.special = Some(PendingCast {
        target: Some(target),
        left: WINDUP,
    });
    state.units[i].action_cd = every;
    ev.push(Event::AbilityStarted {
        unit: u.id,
        kind: u.kind,
        primary: Some(target),
        center,
        tick: state.tick,
    });
}

fn advance_cast(
    state: &mut MatchState,
    i: usize,
    u: &Unit,
    mut cast: PendingCast,
    ev: &mut Vec<Event>,
) {
    let alive = cast
        .target
        .is_none_or(|id| state.unit_index(id).is_some_and(|t| state.units[t].hp > 0));
    if !alive {
        state.units[i].fx.special = None;
        return;
    }
    if cast.left > 1 {
        cast.left -= 1;
        state.units[i].fx.special = Some(cast);
        return;
    }
    state.units[i].fx.special = None;
    if let Some(target) = cast.target {
        crate::training_hooks::scoped(
            state,
            crate::training_hooks::Source::Unit(u.id),
            crate::training_hooks::Effect::Direct,
            |state| {
                // The primary stays locked: no range recheck, replacement or extra hit.
                crate::weaver::resolve(state, u, target, ev);
                crate::whisperer::resolve(state, u, target, ev);
                crate::bulwark::resolve(state, u);
                crate::demolisher::resolve(state, u, ev);
                crate::jawguard::resolve(state, u);
                crate::justiciar::resolve(state, u, target, ev);
                crate::luminous_oracle::resolve(state, u, target);
                crate::solar_sentinel::resolve(state, u, target);
                crate::trident::resolve(state, u, target, ev);
                crate::capybara::resolve(state, u, ev);
                crate::rattle_tick::resolve(state, u, target, ev);
            },
        );
    }
    let center = center(state, u, cast.target);
    ev.push(Event::AbilityResolved {
        unit: u.id,
        kind: u.kind,
        primary: cast.target,
        center,
        tick: state.tick,
    });
}

/// A death later in id order must not leave a stale pending target in a view.
pub(crate) fn cancel_gone(state: &mut MatchState) {
    for i in 0..state.units.len() {
        let target = state.units[i].fx.special.and_then(|c| c.target);
        if target.is_some_and(|id| state.unit_index(id).is_none_or(|t| state.units[t].hp <= 0)) {
            state.units[i].fx.special = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{combat, content::WINDUP, state::UnitFx, ArcanaPreset, Seat, UnitKind};

    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        for (id, kind, seat, x) in [
            (1, UnitKind::Whisperer, Seat::A, 4_000),
            (2, UnitKind::Archer, Seat::B, 5_000),
            (3, UnitKind::Archer, Seat::B, 6_000),
        ] {
            s.units.push(Unit {
                id,
                seat,
                kind,
                tier: 0,
                branch: 0,
                x,
                hp: 1_000,
                max_hp: 1_000,
                cooldown: 0,
                windup: None,
                status: Status::None,
                summoner: None,
                expires_at: None,
                action_cd: if id == 1 { 160 } else { 0 },
                haste_until: 0,
                heal_cut_until: 0,
                heal_cut_bp: 0,
                second_wind_used: 0,
                fx: UnitFx::default(),
            });
        }
        s.refresh_army();
        s
    }
    fn cast_step(s: &mut MatchState) -> Vec<Event> {
        combat::tick_timers(s);
        let mut ev = Vec::new();
        advance(s, 0, &mut ev);
        s.tick += 1;
        ev
    }
    fn start_now(s: &mut MatchState) {
        s.units[0].action_cd = 0;
        cast_step(s);
    }

    #[test]
    fn r11_full_initial_cooldown_ready_wait_and_six_tick_locked_cast() {
        let mut s = arena();
        for _ in 0..159 {
            assert!(cast_step(&mut s).is_empty());
        }
        let starts = cast_step(&mut s);
        assert!(matches!(
            starts.as_slice(),
            [Event::AbilityStarted {
                unit: 1,
                primary: Some(2),
                ..
            }]
        ));
        s.units[1].x = 20_000;
        for _ in 1..WINDUP {
            assert!(cast_step(&mut s).is_empty());
        }
        let resolved = cast_step(&mut s);
        assert!(resolved.iter().any(|e| matches!(
            e,
            Event::AbilityResolved {
                primary: Some(2),
                ..
            }
        )));
        assert_eq!(
            s.units[1].status,
            Status::Slowed {
                bp: 2_500,
                ticks_left: 40
            }
        );
        assert_eq!(
            s.units[1].hp, 1_000,
            "§13.2: special has no separate damage"
        );
        assert_eq!(
            s.units[0].action_cd,
            160 - WINDUP,
            "readiness is counted from start"
        );
        s.units[2].x = 20_000;
        s.units[0].action_cd = 0;
        assert!(cast_step(&mut s).is_empty());
        assert_eq!(s.units[0].action_cd, 0);
    }
    #[test]
    fn r11_root_and_primary_death_cancel_without_refund_or_retarget() {
        let mut s = arena();
        start_now(&mut s);
        s.units[0].status = Status::Rooted { ticks_left: 20 };
        cast_step(&mut s);
        assert!(s.units[0].fx.special.is_none());
        assert_eq!(s.units[0].action_cd, 159);
        s.units[0].status = Status::None;
        start_now(&mut s);
        s.units[1].hp = 0;
        cast_step(&mut s);
        assert!(s.units[0].fx.special.is_none());
        assert_eq!(s.units[2].status, Status::None, "no substitute primary");
    }
    #[test]
    fn r11_special_and_basic_windups_advance_independently() {
        let mut s = arena();
        s.units[0].action_cd = 0;
        let mut ev = Vec::new();
        for _ in 0..=WINDUP {
            combat::tick_timers(&mut s);
            combat::run_units(&mut s, &mut ev);
            s.tick += 1;
        }
        assert!(ev.iter().any(|e| matches!(
            e,
            Event::AttackStart {
                from: crate::Target::Unit(1),
                ..
            }
        )));
        assert!(ev
            .iter()
            .any(|e| matches!(e, Event::AbilityResolved { unit: 1, .. })));
        assert_eq!(s.units[1].hp, 985, "only one ordinary 15-damage basic");
    }
    #[test]
    fn r11_guard_priority_and_ward_reuse_the_existing_hostile_status_path() {
        let mut s = arena();
        s.units[2].kind = UnitKind::Rampart;
        start_now(&mut s);
        assert_eq!(s.units[0].fx.special.unwrap().target, Some(3));
        s.units[2].fx.ward_until = 1_000;
        for _ in 0..WINDUP {
            cast_step(&mut s);
        }
        assert_eq!(
            s.units[2].status,
            Status::None,
            "ward blocks Slow but not ordinary damage"
        );
    }
    #[test]
    fn r02_dead_caster_cannot_finish_an_independent_special() {
        let mut s = arena();
        s.units[0].kind = UnitKind::Demolisher;
        s.units[0].action_cd = 0;
        advance(&mut s, 0, &mut Vec::new());
        assert!(s.units[0].fx.special.is_some());
        s.units[0].hp = 0;
        let mut events = Vec::new();
        for _ in 0..=WINDUP {
            advance(&mut s, 0, &mut events);
        }
        assert!(events.is_empty());
        assert_eq!(s.units[1].hp, 1_000);
    }

    #[test]
    fn r11_pause_freezes_cast_cooldown_and_public_reconnect_facts() {
        let mut s = arena();
        start_now(&mut s);
        let special = s.view_for(Seat::B, &[]).units[0].special.unwrap();
        assert_eq!(special.ready_in, 160);
        assert_eq!(special.cast.unwrap().left, WINDUP);
        assert_eq!(s.public_view(&[], 0).units[0].special, Some(special));
        s.augments.round = 1;
        s.augments.deadline = 100;
        let tick = s.tick;
        for _ in 0..10 {
            s.step();
        }
        assert_eq!(s.tick, tick);
        assert_eq!(s.public_view(&[], 0).units[0].special, Some(special));
    }
}
