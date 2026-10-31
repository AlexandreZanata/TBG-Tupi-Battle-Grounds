//! R04's fourth completed enemy hit and one nearest distinct authoritative secondary.
use crate::{
    content::{Ability, Stats},
    hits,
    state::{Event, MatchState, Target, Unit},
    types::{UnitId, UnitKind},
};
pub(crate) fn counter(u: &Unit) -> u8 {
    if u.kind == UnitKind::Trigger && !u.is_summon() {
        u.fx.attacks.min(3)
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
    let Some((every, ratio, radius)) = s
        .rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::Crossfire {
                every,
                ratio,
                radius,
            } => Some((every, ratio, radius)),
            _ => None,
        })
    else {
        return;
    };
    let Target::Unit(primary) = target else {
        return;
    };
    let Some(pi) = s.unit_index(primary) else {
        return;
    };
    if s.units[pi].seat == a.seat {
        return;
    }
    let Some(i) = s.unit_index(a.id) else {
        return;
    };
    let n = s.units[i].fx.attacks.saturating_add(1);
    if n < every.max(1) {
        s.units[i].fx.attacks = n;
        return;
    }
    s.units[i].fx.attacks = 0;
    let center = s.units[pi].x;
    let id = crate::secondary_targets::nearest(s, a, primary, center, radius)[0];
    if id == UnitId::MAX {
        return;
    }
    let amount =
        (i64::from(st.damage) * i64::from(ratio) / 10_000).clamp(0, i64::from(i32::MAX)) as i32;
    hits::auxiliary_hit(s, a, id, amount, ev);
    ev.push(Event::Crossfire { from: a.id, to: id });
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state::UnitFx, ArcanaPreset, Seat, Status};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        for id in 1..=5 {
            s.units.push(Unit {
                id,
                kind: if id == 1 {
                    UnitKind::Trigger
                } else {
                    UnitKind::Archer
                },
                seat: if id == 1 { Seat::A } else { Seat::B },
                tier: 0,
                branch: 0,
                x: 4_000 + if id > 2 { 100 } else { 0 },
                hp: 1000,
                max_hp: 1000,
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
            });
        }
        s.refresh_army();
        s
    }
    fn hit(s: &mut MatchState, t: Target) -> Vec<Event> {
        let a = s.units[0];
        let st = s.stats_of(&a);
        let mut ev = Vec::new();
        hits::resolve_hit(s, &a, t, &st, &mut ev);
        ev
    }
    #[test]
    fn r04_fourth_hit_uses_primary_center_and_distance_id_ties_without_recursion() {
        let mut s = arena();
        for _ in 0..3 {
            assert!(!hit(&mut s, Target::Unit(2))
                .iter()
                .any(|e| matches!(e, Event::Crossfire { .. })));
        }
        s.units[1].hp = 1;
        let ev = hit(&mut s, Target::Unit(2));
        assert!(ev.contains(&Event::Crossfire { from: 1, to: 3 }));
        assert_eq!(s.units[2].hp, 985);
        assert_eq!(s.units[3].hp, 1000);
        assert_eq!(counter(&s.units[0]), 0);
    }
    #[test]
    fn r04_no_secondary_spends_proc_and_stronghold_does_not_advance() {
        let mut s = arena();
        s.units[0].fx.attacks = 3;
        hit(&mut s, Target::Stronghold(Seat::B));
        assert_eq!(counter(&s.units[0]), 3);
        s.units[2].fx.carrier = Some(99);
        s.units[3].seat = Seat::A;
        s.units[4].x = 9_000;
        let ev = hit(&mut s, Target::Unit(2));
        assert!(!ev.iter().any(|e| matches!(e, Event::Crossfire { .. })));
        assert_eq!(counter(&s.units[0]), 0);
    }
    #[test]
    fn r04_forms_replace_ratio_radius_and_keep_static_secondary_scaling() {
        let mut s = arena();
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        s.units[0].fx.attacks = 3;
        s.units[2].x = 5_900;
        s.units[3].x = 7_000;
        s.units[4].hp = 0;
        let ev = hit(&mut s, Target::Unit(2));
        assert!(ev.contains(&Event::Crossfire { from: 1, to: 3 }));
        assert_eq!(s.units[2].hp, 982);
        let a = s.view_for(Seat::A, &[]);
        assert_eq!(a.units[0].ability_counter(), 0);
        let st = s.rules().effective_stats(UnitKind::Trigger, 2, 0, 0);
        assert!(st.interval < 17);
        assert_eq!(st.speed, 58);
    }
}
