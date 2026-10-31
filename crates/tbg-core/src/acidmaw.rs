//! R10's completed-shot counter and fixed best-three secondary selection (P58 §3).

use crate::{
    content::{Ability, Stats},
    hits,
    state::{Event, MatchState, Target, Unit},
    types::{UnitId, UnitKind},
};

pub(crate) fn counter(u: &Unit) -> u8 {
    if u.kind == UnitKind::Acidmaw && !u.is_summon() {
        u.fx.attacks.min(7)
    } else {
        0
    }
}

pub(crate) fn completed(
    state: &mut MatchState,
    a: &Unit,
    target: Target,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    if a.is_summon() {
        return;
    }
    let Some((every, damage, radius, targets)) = state
        .rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|ability| match ability {
            Ability::CorrosiveSpit {
                every,
                damage,
                radius,
                targets,
            } => Some((every, damage, radius, targets)),
            _ => None,
        })
    else {
        return;
    };
    let Target::Unit(primary) = target else {
        return;
    };
    let Some(pi) = state.unit_index(primary) else {
        return;
    };
    if state.units[pi].seat == a.seat {
        return;
    }
    let Some(i) = state.unit_index(a.id) else {
        return;
    };
    let count = state.units[i].fx.attacks.saturating_add(1);
    if count < every.max(1) {
        state.units[i].fx.attacks = count;
        return;
    }
    state.units[i].fx.attacks = 0;
    let center = state.units[pi].x;
    let ids = crate::secondary_targets::nearest(state, a, primary, center, radius);
    let amount = hits::auxiliary_amount(st, damage);
    for id in ids
        .into_iter()
        .take(usize::from(targets.min(3)))
        .filter(|&id| id != UnitId::MAX)
    {
        hits::auxiliary_hit(state, a, id, amount, ev);
        ev.push(Event::CorrosiveSplash {
            from: a.id,
            primary,
            to: id,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state::UnitFx, ArcanaPreset, Seat, Status};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        for id in 1..=7 {
            s.units.push(Unit {
                id,
                kind: if id == 1 {
                    UnitKind::Acidmaw
                } else {
                    UnitKind::Archer
                },
                seat: if id == 1 { Seat::A } else { Seat::B },
                tier: 0,
                branch: 0,
                x: 1_000 + if id > 2 { (id as i32 - 3) * 100 } else { 0 },
                hp: 1_000,
                max_hp: 1_000,
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
    fn hit(s: &mut MatchState, target: Target) -> Vec<Event> {
        let a = s.units[0];
        let st = s.stats_of(&a);
        let mut ev = Vec::new();
        hits::resolve_hit(s, &a, target, &st, &mut ev);
        ev
    }
    #[test]
    fn r10_third_shot_retains_primary_damage_and_selects_nearest_distinct_secondaries() {
        let mut s = arena();
        hit(&mut s, Target::Unit(2));
        hit(&mut s, Target::Stronghold(Seat::B));
        assert_eq!(
            s.units[0].fx.attacks, 1,
            "Strongholds never advance the counter"
        );
        hit(&mut s, Target::Unit(2));
        assert_eq!(s.units[2].hp, 1_000);
        let ev = hit(&mut s, Target::Unit(2));
        assert_eq!(
            s.units[1].hp, 874,
            "§13.2: three ordinary 42-damage primaries"
        );
        assert_eq!(
            [s.units[2].hp, s.units[3].hp, s.units[4].hp],
            [982, 982, 1_000]
        );
        let to: Vec<_> = ev
            .iter()
            .filter_map(|e| {
                if let Event::CorrosiveSplash { to, .. } = e {
                    Some(*to)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(to, [3, 4]);
        assert_eq!(counter(&s.units[0]), 0);
    }
    #[test]
    fn r10_best_k_respects_carriers_allies_death_radius_and_tie_id() {
        let mut s = arena();
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        s.units[0].fx.attacks = 2;
        s.units[2].fx.carrier = Some(2);
        s.units[3].seat = Seat::A;
        s.units[4].hp = 0;
        s.units[5].x = 1_200;
        s.units[6].x = 1_200;
        hit(&mut s, Target::Unit(2));
        assert_eq!(s.units[2].hp, 1_000);
        assert_eq!(s.units[3].hp, 1_000);
        assert_eq!([s.units[5].hp, s.units[6].hp], [976, 976]);
        assert_eq!(
            crate::secondary_targets::nearest(&s, &s.units[0], 2, 1_000, 150),
            [UnitId::MAX; 3]
        );
    }
    #[test]
    fn r10_killing_primary_still_splashes_and_enemy_summons_count() {
        let mut s = arena();
        s.units[1].kind = UnitKind::Beetling;
        s.units[0].fx.attacks = 2;
        s.units[1].hp = 1;
        hit(&mut s, Target::Unit(2));
        assert_eq!(s.units[1].hp, 0);
        assert_eq!(s.units[2].hp, 982);
    }

    #[test]
    fn r10_auxiliary_static_modifiers_are_applied_once_without_weaken_or_on_hit() {
        let mut s = arena();
        s.units[0].tier = 1;
        s.units[0].fx.attacks = 2;
        crate::roster_slots::apply_weaken(&mut s.units[0].fx, 2_000, 60, 0);
        hit(&mut s, Target::Unit(2));
        assert_eq!(
            s.units[2].hp, 980,
            "18 × 1.15 floors to 20, not weakened basic ratio"
        );
        assert_eq!(
            s.units[2].fx.poison.iter().filter(|p| p.left > 0).count(),
            0
        );
        assert_eq!(
            counter(&s.units[0]),
            0,
            "auxiliary hits do not count recursively"
        );
    }
}
