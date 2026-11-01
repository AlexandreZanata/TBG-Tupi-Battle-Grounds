//! R18's third completed sting applies one separate refresh-only venom family.
use crate::{
    content::{Ability, Stats},
    hits, roster_slots,
    state::{Event, MatchState, Target, Unit},
    types::UnitKind,
};
pub(crate) fn counter(u: &Unit) -> u8 {
    if u.kind == UnitKind::Hornet && !u.is_summon() {
        u.fx.attacks.min(2)
    } else {
        0
    }
}
pub(crate) fn completed(
    s: &mut MatchState,
    a: &Unit,
    target: Target,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    if a.is_summon() {
        return;
    }
    let Some((every, damage, pulses)) = s
        .rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::VenomousSting {
                every,
                damage,
                pulses,
            } => Some((every.max(1), damage, pulses)),
            _ => None,
        })
    else {
        return;
    };
    let Target::Unit(id) = target else {
        return;
    };
    let Some(t) = s.unit_index(id).filter(|&t| s.units[t].seat != a.seat) else {
        return;
    };
    let Some(i) = s.unit_index(a.id) else {
        return;
    };
    let n = s.units[i].fx.attacks.saturating_add(1);
    if n < every {
        s.units[i].fx.attacks = n;
        return;
    }
    s.units[i].fx.attacks = 0;
    if s.units[t].hp <= 0 || s.tick < s.units[t].fx.ward_until {
        return;
    }
    let value = hits::auxiliary_amount(st, damage).clamp(0, i32::from(u16::MAX)) as u16;
    roster_slots::apply_venom(&mut s.units[t].fx, value, pulses, a.seat, s.tick);
    ev.push(Event::AbilityResolved {
        unit: a.id,
        kind: a.kind,
        primary: Some(id),
        center: s.units[t].x,
        tick: s.tick,
    });
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{afflictions, state::Stack, ArcanaPreset, Seat};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::Hornet, 0, 0, 4_000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 4_500);
        s
    }
    fn hit(s: &mut MatchState, i: usize, t: Target) {
        let a = s.units[i];
        let st = s.stats_of(&a);
        hits::resolve_hit(s, &a, t, &st, &mut Vec::new());
    }
    #[test]
    fn r18_third_completed_unit_hit_applies_one_delayed_venom_and_towers_never_count() {
        let mut s = arena();
        let id = s.units[1].id;
        for _ in 0..2 {
            hit(&mut s, 0, Target::Unit(id));
        }
        hit(&mut s, 0, Target::Stronghold(Seat::B));
        assert_eq!(counter(&s.units[0]), 2);
        assert_eq!(s.units[1].fx.venom.left, 0);
        hit(&mut s, 0, Target::Unit(id));
        assert_eq!(
            (
                s.units[1].fx.venom.value,
                s.units[1].fx.venom.left,
                s.units[1].fx.venom.next
            ),
            (5, 4, 20)
        );
        assert_eq!(counter(&s.units[0]), 0);
        assert_eq!(s.units[1].fx.poison, [Stack::default(); 6]);
    }
    #[test]
    fn r18_stronger_refresh_keeps_phase_and_four_pulses_survive_both_sources_death() {
        let mut s = arena();
        let id = s.units[1].id;
        for _ in 0..3 {
            hit(&mut s, 0, Target::Unit(id));
        }
        let other = s.lab_spawn_at(Seat::A, UnitKind::Hornet, 2, 0, 4_100);
        let i = s.unit_index(other).unwrap();
        s.tick = 10;
        for _ in 0..3 {
            hit(&mut s, i, Target::Unit(id));
        }
        assert_eq!(
            (s.units[1].fx.venom.value, s.units[1].fx.venom.next),
            (7, 20)
        );
        s.units[0].hp = 0;
        s.units[i].hp = 0;
        let hp = s.units[1].hp;
        let credited = s.lab.dealt[Seat::A.index()][UnitKind::Hornet as usize];
        for tick in [20, 40, 60, 80] {
            s.tick = tick;
            roster_slots::tick_venom(&mut s, &mut Vec::new());
        }
        assert_eq!(s.units[1].hp, hp - 28);
        assert_eq!(s.units[1].fx.venom.left, 0);
        assert_eq!(
            s.lab.dealt[Seat::A.index()][UnitKind::Hornet as usize] - credited,
            28
        );
    }
    #[test]
    fn r18_ward_spends_proc_without_payload_and_brew_strips_bleed_then_venom_before_legacy() {
        let mut s = arena();
        let id = s.units[1].id;
        s.units[1].fx.ward_until = 100;
        for _ in 0..3 {
            hit(&mut s, 0, Target::Unit(id));
        }
        assert_eq!(s.units[1].fx.venom.left, 0);
        assert_eq!(counter(&s.units[0]), 0);
        s.units[1].fx.ward_until = 0;
        roster_slots::apply_bleed(&mut s.units[1].fx, 5, 4, Seat::A, 0);
        roster_slots::apply_venom(&mut s.units[1].fx, 7, 4, Seat::A, 0);
        s.units[1].fx.poison[0] = Stack {
            left: 100,
            value: 9,
        };
        let brewer = s.lab_spawn_at(Seat::B, UnitKind::AntidoteBrewer, 0, 0, 4_700);
        let b = s.units[s.unit_index(brewer).unwrap()];
        let (_, ward, _) = afflictions::brew_of(&s, &b).unwrap();
        afflictions::brew_cast(&mut s, &b, 3000, id, 1, ward, false, &mut Vec::new());
        assert_eq!(s.units[1].fx.bleed.left, 0);
        assert_eq!(s.units[1].fx.venom.left, 4);
        assert_eq!(s.units[1].fx.poison[0].value, 9);
        afflictions::brew_cast(&mut s, &b, 3000, id, 1, ward, false, &mut Vec::new());
        assert_eq!(s.units[1].fx.venom.left, 0);
        assert_eq!(s.units[1].fx.poison[0].value, 9);
    }
    #[test]
    fn r18_control_immunity_does_not_stop_venom_and_swoop_changes_only_cadence_range() {
        let mut s = arena();
        s.units[1].kind = UnitKind::Oilpress;
        s.units[1].tier = 2;
        let id = s.units[1].id;
        for _ in 0..3 {
            hit(&mut s, 0, Target::Unit(id));
        }
        assert_eq!(s.units[1].fx.venom.left, 4);
        let st = s.rules().effective_stats(UnitKind::Hornet, 2, 1, 0);
        assert!(st.interval < 30);
        assert_eq!(st.range, 4200);
        assert!(crate::targeting::targetable(&s.units[0]));
    }
}
