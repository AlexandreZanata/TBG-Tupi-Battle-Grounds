//! Tick's independent medium-range pistol; Rattle keeps ordinary movement and club combat.
use crate::{
    content::Ability,
    hits,
    state::{Event, MatchState, Unit},
    types::UnitId,
};

pub(crate) fn parameters(s: &MatchState, a: &Unit) -> Option<(u16, i32, u16)> {
    s.rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::ShoulderPistol {
                period,
                range,
                damage,
            } => Some((period, range, damage)),
            _ => None,
        })
}

pub(crate) fn resolve(s: &mut MatchState, a: &Unit, primary: UnitId, ev: &mut Vec<Event>) {
    let Some((_, _, damage)) = parameters(s, a) else {
        return;
    };
    let Some(i) = s.unit_index(primary) else {
        return;
    };
    if s.units[i].hp <= 0 || s.units[i].seat == a.seat {
        return;
    }
    let mut st = s.stats_of(a);
    // Branch A belongs to Rattle: its basic-damage bonus must not buff Tick.
    if a.branch == 0 {
        for tier in 1..=a.tier.min(2) {
            if let Some(form) = s.rules().form_of(a.kind, tier, a.branch) {
                st.damage_bp -= i64::from(form.dmg_bp);
            }
        }
    }
    hits::auxiliary_hit(s, a, primary, hits::auxiliary_amount(&st, damage), ev);
}

#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{specials, ArcanaPreset, Seat, Status, UnitKind};
    #[test]
    fn p71_tick_fires_beyond_club_range_without_changing_rattle_action_or_movement() {
        for seat in Seat::BOTH {
            let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
            let id = s.lab_spawn_at(seat, UnitKind::RattleTick, 0, 0, 8_000);
            let target = s.lab_spawn_at(seat.other(), UnitKind::BoneBrute, 0, 0, 10_500);
            let i = s.unit_index(id).unwrap();
            s.units[i].action_cd = 0;
            let before = s.units[i];
            let mut ev = Vec::new();
            specials::advance(&mut s, i, &mut ev);
            assert_eq!(s.units[i].fx.special.unwrap().target, Some(target));
            for _ in 0..crate::content::WINDUP {
                specials::advance(&mut s, i, &mut ev);
            }
            assert_eq!(
                (s.units[i].x, s.units[i].cooldown, s.units[i].windup),
                (before.x, before.cooldown, before.windup)
            );
            assert!(ev.iter().any(|e| matches!(e, Event::Hit { to: crate::Target::Unit(t), amount:12 } if *t == target)));
            assert_eq!(
                s.units[i].fx.attacks, 0,
                "pistol never advances club counters"
            );
        }
    }
    #[test]
    fn p71_tick_hits_one_enemy_without_splash_slow_or_friendly_fire() {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        let id = s.lab_spawn_at(Seat::A, UnitKind::RattleTick, 0, 0, 4_000);
        let b = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_000);
        let c = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_050);
        let ally = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 6_000);
        let caster = s.units[s.unit_index(id).unwrap()];
        let mut ev = Vec::new();
        resolve(&mut s, &caster, b, &mut ev);
        assert_eq!(
            ev.iter().filter(|e| matches!(e, Event::Hit { .. })).count(),
            1
        );
        assert!(s.units.iter().all(|u| u.status == Status::None));
        let hp = s.units[s.unit_index(ally).unwrap()].hp;
        resolve(&mut s, &caster, ally, &mut ev);
        assert_eq!(s.units[s.unit_index(ally).unwrap()].hp, hp);
        assert_eq!(
            s.units[s.unit_index(c).unwrap()].hp,
            s.units[s.unit_index(c).unwrap()].max_hp
        );
    }
    #[test]
    fn p71_each_brothers_evolution_changes_only_his_attack_channel() {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        let id = s.lab_spawn_at(Seat::A, UnitKind::RattleTick, 2, 0, 4_000);
        let target = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_000);
        let i = s.unit_index(id).unwrap();
        let a = s.units[i];
        assert_eq!(s.stats_of(&a).damage, 72);
        let mut ev = Vec::new();
        resolve(&mut s, &a, target, &mut ev);
        assert!(ev.contains(&Event::Hit {
            to: crate::Target::Unit(target),
            amount: 12
        }));
        s.units[i].branch = 1;
        let b = s.units[i];
        assert_eq!(s.stats_of(&b).damage, 60);
        assert_eq!(parameters(&s, &b), Some((24, 3_000, 18)));
    }
}
