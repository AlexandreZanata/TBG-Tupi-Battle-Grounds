//! R06's caster-centered belly impact and A2-only post-impact protection.
use crate::{
    content::Ability,
    hits, roster_slots,
    state::{Event, MatchState, Unit},
    types::OnHit,
};
pub(crate) fn resolve(s: &mut MatchState, a: &Unit, ev: &mut Vec<Event>) {
    let Some((radius, damage, bp, ticks, protection, protection_ticks)) = s
        .rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::SwampBelly {
                radius,
                damage,
                bp,
                ticks,
                protection,
                protection_ticks,
                ..
            } => Some((radius, damage, bp, ticks, protection, protection_ticks)),
            _ => None,
        })
    else {
        return;
    };
    let amount = hits::auxiliary_amount(&s.stats_of(a), damage);
    for i in 0..s.units.len() {
        let v = s.units[i];
        if v.hp <= 0 || v.seat == a.seat || (v.x - a.x).abs() > radius {
            continue;
        }
        hits::auxiliary_hit(s, a, v.id, amount, ev);
        if s.units[i].hp > 0 {
            let _applied = hits::apply_on_hit(s, v.id, OnHit::Slow { bp, ticks }, a.seat, ev);
            #[cfg(feature = "lab")]
            crate::lab::status(s, a, _applied);
        }
    }
    if protection > 0 {
        if let Some(i) = s.unit_index(a.id) {
            roster_slots::apply_self_dr(&mut s.units[i].fx, protection, protection_ticks, s.tick);
        }
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{specials, ArcanaPreset, Seat, Status, UnitKind};
    fn arena() -> (MatchState, usize, usize, usize) {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::Capybara, 0, 0, 4_000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 5_500);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 5_501);
        (s, 0, 1, 2)
    }
    #[test]
    fn r06_base_flop_is_caster_centered_without_passive_protection() {
        let (mut s, a, b, c) = arena();
        let caster = s.units[a];
        let hp = s.units[b].hp;
        let out = s.units[c].hp;
        resolve(&mut s, &caster, &mut Vec::new());
        assert_eq!(s.units[b].hp, hp - 65);
        assert_eq!(
            s.units[b].status,
            Status::Slowed {
                bp: 1500,
                ticks_left: 40
            }
        );
        assert_eq!(s.units[c].hp, out);
        assert_eq!(roster_slots::self_dr_now(&s.units[a].fx, s.tick), 0);
    }
    #[test]
    fn r06_stone_belly_protection_starts_only_after_impact_and_expires_normally() {
        let (mut s, a, _, _) = arena();
        s.units[a].tier = 2;
        let caster = s.units[a];
        assert_eq!(s.stats_of(&caster).max_hp, 1560);
        assert_eq!(roster_slots::self_dr_now(&caster.fx, 0), 0);
        resolve(&mut s, &caster, &mut Vec::new());
        assert_eq!(s.view_for(Seat::A, &[]).units[a].reduction, 1500);
        s.units[a].status = Status::Rooted { ticks_left: 100 };
        s.tick = 40;
        assert_eq!(s.view_for(Seat::A, &[]).units[a].reduction, 0);
    }
    #[test]
    fn r06_swamp_wave_replaces_radius_period_and_slow_without_upgrading_damage() {
        let (mut s, a, _, c) = arena();
        s.units[a].tier = 2;
        s.units[a].branch = 1;
        s.units[c].x = 5_900;
        let hp = s.units[c].hp;
        let caster = s.units[a];
        assert_eq!(specials::period(s.rules(), caster.kind, 2, 1), Some(160));
        resolve(&mut s, &caster, &mut Vec::new());
        assert_eq!(s.units[c].hp, hp - 65);
        assert_eq!(
            s.units[c].status,
            Status::Slowed {
                bp: 2000,
                ticks_left: 40
            }
        );
        assert_eq!(roster_slots::self_dr_now(&s.units[a].fx, 0), 0);
    }
}
