//! R17's no-timeout same-enemy sequence and one integrated fourth-hit bonus.
use crate::{
    content::Ability,
    state::{Event, MatchState, Target, Unit},
    types::UnitKind,
};
fn parameters(s: &MatchState, u: &Unit) -> Option<(u8, u16)> {
    if u.kind != UnitKind::Mantis || u.is_summon() {
        return None;
    }
    s.rules()
        .def(u.kind)
        .ability(u.tier, u.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::PreciseStrike { every, damage } => Some((every.max(1), damage)),
            _ => None,
        })
}
pub(crate) fn counter(u: &Unit) -> u8 {
    if u.kind == UnitKind::Mantis && !u.is_summon() && u.fx.mark_target.is_some() {
        u.fx.attacks.min(3)
    } else {
        0
    }
}
fn clear(u: &mut Unit) {
    u.fx.attacks = 0;
    u.fx.mark_target = None;
    u.fx.mark_until = 0;
}
pub(crate) fn retarget(s: &mut MatchState, i: usize, target: Target) {
    let u = s.units[i];
    if parameters(s, &u).is_none() {
        return;
    }
    if !matches!(target,Target::Unit(id)if u.fx.mark_target==Some(id)) {
        clear(&mut s.units[i]);
    }
}
pub(crate) fn bonus(s: &MatchState, u: &Unit, target: Target) -> u16 {
    let Some((every, damage)) = parameters(s, u) else {
        return 0;
    };
    let Target::Unit(id) = target else {
        return 0;
    };
    if u.fx.mark_target == Some(id)
        && counter(u).saturating_add(1) >= every
        && s.unit_index(id)
            .is_some_and(|i| s.units[i].hp > 0 && s.units[i].seat != u.seat)
    {
        damage
    } else {
        0
    }
}
pub(crate) fn completed(s: &mut MatchState, u: &Unit, target: Target, ev: &mut Vec<Event>) {
    let Some((every, _)) = parameters(s, u) else {
        return;
    };
    let Some(i) = s.unit_index(u.id) else {
        return;
    };
    let Target::Unit(id) = target else {
        clear(&mut s.units[i]);
        return;
    };
    let Some(t) = s.unit_index(id).filter(|&t| s.units[t].seat != u.seat) else {
        return;
    };
    let n = if u.fx.mark_target == Some(id) {
        counter(u).saturating_add(1)
    } else {
        1
    };
    let center = s.units[t].x;
    let dead = s.units[t].hp <= 0;
    if n >= every || dead {
        clear(&mut s.units[i]);
    } else {
        s.units[i].fx.attacks = n;
        s.units[i].fx.mark_target = Some(id);
        s.units[i].fx.mark_until = 0;
    }
    if n >= every {
        ev.push(Event::AbilityResolved {
            unit: u.id,
            kind: u.kind,
            primary: Some(id),
            center,
            tick: s.tick,
        });
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{combat, hits, roster_slots, ArcanaPreset, Seat, Status};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::Mantis, 0, 0, 4_000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 4_500);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 4_600);
        s
    }
    fn hit(s: &mut MatchState, id: u32) {
        let a = s.units[0];
        let st = s.stats_of(&a);
        hits::resolve_hit(s, &a, Target::Unit(id), &st, &mut Vec::new());
    }
    #[test]
    fn r17_fourth_same_enemy_hit_adds_once_and_resets_without_timeout() {
        let mut s = arena();
        let id = s.units[1].id;
        let hp = s.units[1].hp;
        for _ in 0..3 {
            hit(&mut s, id);
        }
        assert_eq!(s.units[1].hp, hp - 150);
        s.tick = 1000;
        roster_slots::tick_marks(&mut s);
        assert_eq!(counter(&s.units[0]), 3);
        assert_eq!(
            s.public_view(&[], 0).units[0]
                .effects
                .unwrap()
                .mark
                .unwrap()
                .target,
            id
        );
        assert!(!crate::crowcaller::marked(&s, id));
        hit(&mut s, id);
        assert_eq!(s.units[1].hp, hp - 260);
        assert_eq!(counter(&s.units[0]), 0);
    }
    #[test]
    fn r17_target_switch_and_death_reset_but_same_target_root_cancellation_does_not_advance() {
        let mut s = arena();
        let id = s.units[1].id;
        for _ in 0..3 {
            hit(&mut s, id);
        }
        s.units[0].windup = Some((Target::Unit(id), 1));
        s.units[0].status = Status::Rooted { ticks_left: 20 };
        combat::run_units(&mut s, &mut Vec::new());
        assert_eq!(counter(&s.units[0]), 3);
        assert!(s.units[0].windup.is_none());
        let other = s.units[2].id;
        retarget(&mut s, 0, Target::Unit(other));
        assert_eq!(counter(&s.units[0]), 0);
        s.units[0].status = Status::None;
        hit(&mut s, other);
        s.units[2].hp = 0;
        roster_slots::tick_marks(&mut s);
        assert_eq!(counter(&s.units[0]), 0);
    }
    #[test]
    fn r17_forms_replace_bonus_and_stronghold_target_change_clears_sequence() {
        let mut s = arena();
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        let id = s.units[1].id;
        s.units[0].fx.mark_target = Some(id);
        s.units[0].fx.attacks = 3;
        let hp = s.units[1].hp;
        hit(&mut s, id);
        assert_eq!(s.units[1].hp, hp - 135);
        s.units[0].fx.mark_target = Some(id);
        s.units[0].fx.attacks = 3;
        retarget(&mut s, 0, Target::Stronghold(Seat::B));
        assert_eq!(counter(&s.units[0]), 0);
        let st = s.rules().effective_stats(UnitKind::Mantis, 2, 0, 0);
        assert!(st.interval < 24);
        assert_eq!(st.speed, 66);
    }
    #[test]
    fn r17_fourth_bonus_is_one_primary_against_second_wind() {
        let mut s = arena();
        let id = s.units[1].id;
        s.players[Seat::B.index()].augments.add(33);
        s.units[1].hp = 80;
        s.units[0].fx.mark_target = Some(id);
        s.units[0].fx.attacks = 3;
        let a = s.units[0];
        let st = s.stats_of(&a);
        let mut ev = Vec::new();
        hits::resolve_hit(&mut s, &a, Target::Unit(id), &st, &mut ev);
        assert_eq!((s.units[1].hp, s.units[1].second_wind_used), (1, 1));
        assert_eq!(
            ev.iter()
                .filter(|e| matches!(e,Event::Hit{to:Target::Unit(t),..}if *t==id))
                .count(),
            1
        );
        assert_eq!(counter(&s.units[0]), 0);
    }
}
