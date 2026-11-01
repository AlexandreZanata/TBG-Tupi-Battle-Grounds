//! R07's primary-centered rend and snapshot-scaled single bleed slot on surviving enemies.
use crate::{
    content::Ability,
    hits, roster_slots,
    state::{Event, MatchState, Unit},
    types::UnitId,
};
pub(crate) fn resolve(s: &mut MatchState, a: &Unit, primary: UnitId, ev: &mut Vec<Event>) {
    let Some((radius, damage, bleed, pulses)) = s
        .rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::BloodyRend {
                radius,
                damage,
                bleed,
                pulses,
                ..
            } => Some((radius, damage, bleed, pulses)),
            _ => None,
        })
    else {
        return;
    };
    let Some(center) = s.unit_index(primary).map(|i| s.units[i].x) else {
        return;
    };
    let st = s.stats_of(a);
    let amount = hits::auxiliary_amount(&st, damage);
    let value = hits::auxiliary_amount(&st, bleed).clamp(0, i32::from(u16::MAX)) as u16;
    for i in 0..s.units.len() {
        let v = s.units[i];
        if v.hp <= 0 || v.seat == a.seat || (v.x - center).abs() > radius {
            continue;
        }
        hits::auxiliary_hit(s, a, v.id, amount, ev);
        if s.units[i].hp > 0 && s.tick >= s.units[i].fx.ward_until {
            roster_slots::apply_bleed(&mut s.units[i].fx, value, pulses, a.seat, s.tick);
        }
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{specials, ArcanaPreset, Seat, UnitKind};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::Trident, 0, 0, 4_000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 5_100);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_600);
        s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_100);
        s
    }
    #[test]
    fn r07_primary_center_bleeds_carried_enemies_without_ally_or_cut_effects() {
        let mut s = arena();
        let primary = s.units[1].id;
        s.units[2].fx.carrier = Some(primary);
        let caster = s.units[0];
        let hp = s.units[2].hp;
        let ally = s.units[3].hp;
        resolve(&mut s, &caster, primary, &mut Vec::new());
        assert_eq!(s.units[2].hp, hp - 75);
        assert_eq!(
            (
                s.units[2].fx.bleed.value,
                s.units[2].fx.bleed.left,
                s.units[2].fx.bleed.next
            ),
            (5, 4, 20)
        );
        assert_eq!(s.units[3].hp, ally);
        assert_eq!(roster_slots::heal_cut_now(&s.units[2], 0), 0);
    }
    #[test]
    fn r07_refresh_keeps_next_pulse_and_applied_value_survives_evolution_and_source_death() {
        let mut s = arena();
        let primary = s.units[1].id;
        let caster = s.units[0];
        resolve(&mut s, &caster, primary, &mut Vec::new());
        s.tick = 10;
        s.units[0].tier = 2;
        let caster = s.units[0];
        resolve(&mut s, &caster, primary, &mut Vec::new());
        assert_eq!(
            (s.units[1].fx.bleed.value, s.units[1].fx.bleed.next),
            (7, 20)
        );
        s.units[0].hp = 0;
        let hp = s.units[1].hp;
        let credited = s.lab.dealt[Seat::A.index()][UnitKind::Trident as usize];
        for tick in [20, 40, 60, 80] {
            s.tick = tick;
            roster_slots::tick_bleed(&mut s, &mut Vec::new());
        }
        assert_eq!(s.units[1].hp, hp - 28);
        assert_eq!(
            s.lab.dealt[Seat::A.index()][UnitKind::Trident as usize] - credited,
            56
        );
        assert_eq!(s.units[1].fx.bleed.left, 0);
    }
    #[test]
    fn r07_ward_blocks_bleed_only_and_branch_b_replaces_cadence_radius() {
        let mut s = arena();
        let primary = s.units[1].id;
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        s.units[2].x = 7_000;
        s.units[2].fx.ward_until = 100;
        let hp = s.units[2].hp;
        let caster = s.units[0];
        assert_eq!(specials::period(s.rules(), caster.kind, 2, 1), Some(150));
        resolve(&mut s, &caster, primary, &mut Vec::new());
        assert_eq!(s.units[2].hp, hp - 75);
        assert_eq!(s.units[2].fx.bleed.left, 0);
        assert_eq!(
            s.public_view(&[], 0).units[1]
                .effects
                .unwrap()
                .bleed
                .unwrap()
                .value,
            5
        );
    }
}
