//! R02's single caster-centered pressure hit, with no movement or interruption.

use crate::{
    content::Ability,
    hits,
    state::{Event, MatchState, Unit},
};
pub(crate) fn resolve(state: &mut MatchState, caster: &Unit, ev: &mut Vec<Event>) {
    let Some((radius, damage)) = state
        .rules()
        .def(caster.kind)
        .ability(caster.tier, caster.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::AreaCrush { radius, damage, .. } => Some((radius, damage)),
            _ => None,
        })
    else {
        return;
    };
    let amount = hits::auxiliary_amount(&state.stats_of(caster), damage);
    for i in 0..state.units.len() {
        let v = state.units[i];
        if v.hp > 0 && v.seat != caster.seat && (v.x - caster.x).abs() <= radius {
            hits::auxiliary_hit(state, caster, v.id, amount, ev);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hits, specials, state::UnitFx, ArcanaPreset, Seat, Status, Target, UnitKind};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        for (id, seat, x) in [
            (1, Seat::A, 4_000),
            (2, Seat::B, 4_500),
            (3, Seat::B, 6_400),
            (4, Seat::B, 6_600),
            (5, Seat::B, 1_500),
            (6, Seat::A, 4_200),
        ] {
            s.units.push(Unit {
                id,
                seat,
                kind: if id == 1 {
                    UnitKind::Demolisher
                } else {
                    UnitKind::Archer
                },
                tier: 0,
                branch: 0,
                x,
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
    #[test]
    fn r02_pressure_hits_from_caster_including_carried_enemies_without_displacement() {
        let mut s = arena();
        s.units[2].fx.carrier = Some(2);
        let caster = s.units[0];
        let before: Vec<_> = s.units.iter().map(|u| u.x).collect();
        resolve(&mut s, &caster, &mut Vec::new());
        assert_eq!(
            s.units.iter().map(|u| u.hp).collect::<Vec<_>>(),
            [1_000, 880, 880, 1_000, 880, 1_000]
        );
        assert_eq!(s.units.iter().map(|u| u.x).collect::<Vec<_>>(), before);
        assert!(s.units.iter().all(|u| u.status == Status::None));
    }
    #[test]
    fn r02_reinforced_fists_replaces_special_damage_then_grows_radius() {
        let mut s = arena();
        s.units[0].tier = 2;
        let caster = s.units[0];
        resolve(&mut s, &caster, &mut Vec::new());
        assert_eq!(
            s.units[3].hp, 856,
            "§13.3: 120 × 1.20, within the A2 radius of 3,000"
        );
        assert_eq!(
            s.stats_of(&caster).damage,
            80,
            "special upgrade never raises the basic"
        );
    }
    #[test]
    fn r02_shock_system_changes_cadence_and_basic_slow_only() {
        let mut s = arena();
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        let caster = s.units[0];
        assert_eq!(specials::period(s.rules(), caster.kind, 2, 1), Some(160));
        resolve(&mut s, &caster, &mut Vec::new());
        assert_eq!(s.units[1].status, Status::None);
        let st = s.stats_of(&caster);
        hits::resolve_hit(&mut s, &caster, Target::Unit(2), &st, &mut Vec::new());
        assert_eq!(
            s.units[1].status,
            Status::Slowed {
                bp: 1_500,
                ticks_left: 40
            }
        );
        assert_eq!(
            s.units[0].fx.attacks, 0,
            "auxiliary pressure cannot proc counters"
        );
    }
}
