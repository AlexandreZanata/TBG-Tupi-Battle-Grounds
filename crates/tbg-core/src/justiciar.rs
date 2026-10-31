//! R15's single locked-primary light pulse, with B2-only ordinary Slow.
use crate::{
    content::Ability,
    hits,
    state::{Event, MatchState, Unit},
    types::{OnHit, UnitId},
};
pub(crate) fn resolve(s: &mut MatchState, a: &Unit, primary: UnitId, ev: &mut Vec<Event>) {
    let Some((radius, damage, bp, ticks)) = s
        .rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::CelestialRain {
                radius,
                damage,
                bp,
                ticks,
                ..
            } => Some((radius, damage, bp, ticks)),
            _ => None,
        })
    else {
        return;
    };
    let Some(center) = s.unit_index(primary).map(|i| s.units[i].x) else {
        return;
    };
    let amount = hits::auxiliary_amount(&s.stats_of(a), damage);
    for i in 0..s.units.len() {
        let v = s.units[i];
        if v.hp <= 0 || v.seat == a.seat || (v.x - center).abs() > radius {
            continue;
        }
        hits::auxiliary_hit(s, a, v.id, amount, ev);
        if s.units[i].hp > 0 && bp > 0 {
            let _applied = hits::apply_on_hit(s, v.id, OnHit::Slow { bp, ticks }, a.seat, ev);
            #[cfg(feature = "lab")]
            crate::lab::status(s, a, _applied);
        }
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{specials, state::PendingCast, ArcanaPreset, Seat, Status, UnitKind};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::Justiciar, 0, 0, 4_000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 5_000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_300);
        s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_000);
        s
    }
    #[test]
    fn r15_one_primary_centered_pulse_includes_carried_but_never_allies_or_towers() {
        let mut s = arena();
        let primary = s.units[1].id;
        s.units[2].fx.carrier = Some(primary);
        let hp = s.units[2].hp;
        let ally = s.units[3].hp;
        let towers = s.players.clone();
        let a = s.units[0];
        resolve(&mut s, &a, primary, &mut Vec::new());
        assert_eq!(s.units[2].hp, hp - 65);
        assert_eq!(s.units[3].hp, ally);
        assert_eq!(s.players, towers);
        assert!(s.units.iter().all(|u| u.status == Status::None));
    }
    #[test]
    fn r15_judgment_has_one_wider_pulse_with_warded_raw_damage_and_unwarded_slow() {
        let mut s = arena();
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        s.units[1].fx.ward_until = 100;
        s.units[2].x = 6_600;
        let hp = s.units[2].hp;
        let primary = s.units[1].id;
        let a = s.units[0];
        resolve(&mut s, &a, primary, &mut Vec::new());
        assert_eq!(s.units[2].hp, hp - 65);
        assert_eq!(s.units[1].status, Status::None);
        assert_eq!(
            s.units[2].status,
            Status::Slowed {
                bp: 1500,
                ticks_left: 20
            }
        );
    }
    #[test]
    fn r15_fury_changes_special_not_basic_and_dead_primary_cancels_the_entire_pulse() {
        let mut s = arena();
        s.units[0].tier = 2;
        let a = s.units[0];
        let primary = s.units[1].id;
        let hp = s.units[1].hp;
        resolve(&mut s, &a, primary, &mut Vec::new());
        assert_eq!(s.units[1].hp, hp - 78);
        assert_eq!((s.stats_of(&a).max_hp, s.stats_of(&a).damage), (770, 35));
        s.units[0].fx.special = Some(PendingCast {
            target: Some(primary),
            left: 1,
        });
        s.units[1].hp = 0;
        let hp = s.units[2].hp;
        let mut ev = Vec::new();
        specials::advance(&mut s, 0, &mut ev);
        assert_eq!(s.units[2].hp, hp);
        assert!(ev.is_empty());
    }
}
