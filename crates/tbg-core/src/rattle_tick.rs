//! R05's independent primary-centered bomb: damage and ordinary Slow once per enemy.
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
            Ability::DoubleMayhem {
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
        if s.units[i].hp > 0 {
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
    fn arena() -> (MatchState, u32, u32, u32, u32) {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        let a = s.lab_spawn_at(Seat::A, UnitKind::RattleTick, 0, 0, 4_000);
        let b = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 4_900);
        let c = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_100);
        let ally = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 4_900);
        (s, a, b, c, ally)
    }
    #[test]
    fn r05_bomb_centers_on_primary_includes_carried_and_never_allies_or_towers() {
        let (mut s, a, b, c, ally) = arena();
        let i = s.unit_index(a).unwrap();
        let ci = s.unit_index(c).unwrap();
        s.units[ci].fx.carrier = Some(b);
        let ahp = s.units[s.unit_index(ally).unwrap()].hp;
        let chp = s.units[ci].hp;
        let towers = s.players.clone();
        let caster = s.units[i];
        resolve(&mut s, &caster, b, &mut Vec::new());
        assert_eq!(s.units[ci].hp, chp - 60);
        assert_eq!(
            s.units[ci].status,
            Status::Slowed {
                bp: 1500,
                ticks_left: 30
            }
        );
        assert_eq!(s.units[s.unit_index(ally).unwrap()].hp, ahp);
        assert_eq!(s.players, towers);
        assert_eq!(s.units[i].fx.attacks, 0);
    }
    #[test]
    fn r05_dead_primary_cancels_ticos_bomb_even_after_a_club_kill() {
        let (mut s, a, b, c, _) = arena();
        let i = s.unit_index(a).unwrap();
        s.units[i].fx.special = Some(PendingCast {
            target: Some(b),
            left: 1,
        });
        let bi = s.unit_index(b).unwrap();
        s.units[bi].hp = 0;
        let hp = s.units[s.unit_index(c).unwrap()].hp;
        let mut ev = Vec::new();
        specials::advance(&mut s, i, &mut ev);
        assert!(s.units[i].fx.special.is_none());
        assert!(ev.is_empty());
        assert_eq!(s.units[s.unit_index(c).unwrap()].hp, hp);
    }
    #[test]
    fn r05_branch_values_replace_and_ward_blocks_slow_but_not_bomb_damage() {
        let (mut s, a, b, c, _) = arena();
        let i = s.unit_index(a).unwrap();
        s.units[i].tier = 2;
        s.units[i].branch = 1;
        let ci = s.unit_index(c).unwrap();
        s.units[ci].x = 6_400;
        s.units[ci].fx.ward_until = 100;
        let hp = s.units[ci].hp;
        let caster = s.units[i];
        assert_eq!(specials::period(s.rules(), caster.kind, 2, 1), Some(150));
        resolve(&mut s, &caster, b, &mut Vec::new());
        assert_eq!(s.units[ci].hp, hp - 60);
        assert_eq!(s.units[ci].status, Status::None);
        let a = s.rules().effective_stats(UnitKind::RattleTick, 2, 0, 0);
        assert_eq!((a.max_hp, a.damage), (1080, 72));
    }
}
