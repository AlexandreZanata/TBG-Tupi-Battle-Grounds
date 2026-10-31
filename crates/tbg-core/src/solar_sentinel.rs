//! R12's wounded-other-ally selection and strongest/equal/weak temporary shield grant.
use crate::{
    content::Ability,
    roster_slots,
    state::{MatchState, Unit},
    types::UnitId,
};
pub(crate) fn parameters(s: &MatchState, u: &Unit) -> Option<(i32, u16, u16, u16)> {
    s.rules()
        .def(u.kind)
        .ability(u.tier, u.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::SolarBarrier {
                range,
                amount,
                ticks,
                second,
                ..
            } => Some((range, amount, ticks, second)),
            _ => None,
        })
}
pub(crate) fn target(s: &MatchState, u: &Unit, skip: Option<UnitId>) -> Option<UnitId> {
    let (range, _, _, _) = parameters(s, u)?;
    s.units
        .iter()
        .filter(|v| {
            v.id != u.id
                && Some(v.id) != skip
                && v.seat == u.seat
                && v.hp > 0
                && v.hp < v.max_hp
                && (v.x - u.x).abs() <= range
        })
        .min_by(|a, b| {
            (i64::from(a.hp) * i64::from(b.max_hp))
                .cmp(&(i64::from(b.hp) * i64::from(a.max_hp)))
                .then_with(|| (a.x - u.x).abs().cmp(&(b.x - u.x).abs()))
                .then_with(|| a.id.cmp(&b.id))
        })
        .map(|v| v.id)
}
pub(crate) fn resolve(s: &mut MatchState, u: &Unit, primary: UnitId) {
    let Some((_, amount, ticks, second)) = parameters(s, u) else {
        return;
    };
    let Some(i) = s
        .unit_index(primary)
        .filter(|&i| s.units[i].hp > 0 && s.units[i].seat == u.seat && s.units[i].id != u.id)
    else {
        return;
    };
    roster_slots::apply_shield(&mut s.units[i].fx, amount, ticks, s.tick);
    if second > 0 {
        if let Some(id) = target(s, u, Some(primary)) {
            if let Some(j) = s.unit_index(id) {
                roster_slots::apply_shield(&mut s.units[j].fx, second, ticks, s.tick);
            }
        }
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{specials, state::PendingCast, ArcanaPreset, Seat, UnitKind};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::SolarSentinel, 0, 0, 4_000);
        s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_000);
        s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 4_500);
        s.lab_spawn_at(Seat::A, UnitKind::Beetling, 0, 0, 5_000);
        for i in 1..4 {
            s.units[i].hp = s.units[i].max_hp / 2;
        }
        s
    }
    #[test]
    fn r12_lowest_hp_fraction_then_distance_id_and_never_self_or_full_allies() {
        let mut s = arena();
        let u = s.units[0];
        assert_eq!(target(&s, &u, None), Some(s.units[2].id));
        s.units[1].x = 4_500;
        assert_eq!(target(&s, &u, None), Some(s.units[1].id));
        s.units[3].hp = 1;
        assert_eq!(target(&s, &u, None), Some(s.units[3].id));
        for i in 1..4 {
            s.units[i].hp = s.units[i].max_hp;
        }
        assert_eq!(target(&s, &u, None), None);
    }
    #[test]
    fn r12_locked_primary_no_range_recheck_but_dead_primary_cancels_without_retarget() {
        let mut s = arena();
        let id = s.units[2].id;
        s.units[0].fx.special = Some(PendingCast {
            target: Some(id),
            left: 1,
        });
        s.units[2].x = 10_000;
        specials::advance(&mut s, 0, &mut Vec::new());
        assert_eq!(roster_slots::shield_now(&s.units[2].fx, 0), 100);
        s.units[0].fx.special = Some(PendingCast {
            target: Some(id),
            left: 1,
        });
        s.units[2].hp = 0;
        specials::advance(&mut s, 0, &mut Vec::new());
        assert_eq!(roster_slots::shield_now(&s.units[1].fx, 0), 0);
    }
    #[test]
    fn r12_group_shields_are_distinct_and_snapshot_expiry_survives_source_death() {
        let mut s = arena();
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        let u = s.units[0];
        let id = target(&s, &u, None).unwrap();
        resolve(&mut s, &u, id);
        assert_eq!(
            (
                s.view_for(Seat::A, &[]).units[2].shield,
                s.public_view(&[], 0).units[1].shield
            ),
            (100, 50)
        );
        assert_eq!(roster_slots::shield_now(&s.units[0].fx, 0), 0);
        s.units[0].hp = 0;
        s.tick = 80;
        assert!(s.public_view(&[], 0).units.iter().all(|v| v.shield == 0));
        let st = s.rules().effective_stats(UnitKind::SolarSentinel, 2, 0, 0);
        assert_eq!(st.max_hp, 1260);
    }
}
