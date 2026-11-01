//! R01's timed self-protection anchors movement while leaving the ordinary basic intact.

use crate::{
    content::Ability,
    roster_slots,
    state::{MatchState, Unit},
};

pub(crate) fn anchored(state: &MatchState, u: &Unit) -> bool {
    !u.is_summon()
        && roster_slots::self_dr_now(&u.fx, state.tick) > 0
        && state
            .rules()
            .def(u.kind)
            .ability(u.tier, u.branch)
            .iter()
            .any(|a| matches!(a, Ability::LivingWall { .. }))
}

pub(crate) fn resolve(state: &mut MatchState, u: &Unit) {
    let Some(i) = state.unit_index(u.id) else {
        return;
    };
    for a in state.rules().def(u.kind).ability(u.tier, u.branch) {
        if let Ability::LivingWall { bp, ticks, .. } = a {
            roster_slots::apply_self_dr(&mut state.units[i].fx, bp, ticks, state.tick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        combat, content::WINDUP, hits, specials, state::UnitFx, ArcanaPreset, Seat, Status, Target,
        UnitKind,
    };
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        for (id, kind, seat, x) in [
            (1, UnitKind::Bulwark, Seat::A, 4_000),
            (2, UnitKind::Archer, Seat::B, 4_500),
        ] {
            s.units.push(Unit {
                id,
                kind,
                seat,
                tier: 0,
                branch: 0,
                x,
                hp: 1_400,
                max_hp: 1_400,
                cooldown: 0,
                windup: None,
                status: Status::None,
                summoner: None,
                expires_at: None,
                action_cd: if id == 1 { 160 } else { 0 },
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
    fn r01_living_wall_starts_only_with_enemy_unit_and_anchors_without_guard() {
        let mut s = arena();
        s.units[0].action_cd = 0;
        s.units[1].hp = 0;
        specials::advance(&mut s, 0, &mut Vec::new());
        assert!(s.units[0].fx.special.is_none(), "no enemy unit: wait ready");
        s.units[1].hp = 1_400;
        specials::advance(&mut s, 0, &mut Vec::new());
        assert!(s.units[0].fx.special.is_some());
        for _ in 0..WINDUP {
            specials::advance(&mut s, 0, &mut Vec::new());
        }
        assert!(anchored(&s, &s.units[0]));
        s.units[1].x = 10_000;
        let x = s.units[0].x;
        combat::run_units(&mut s, &mut Vec::new());
        assert_eq!(s.units[0].x, x);
        assert!(!s
            .rules()
            .def(UnitKind::Bulwark)
            .ability(0, 0)
            .contains(&Ability::Guard));
        s.tick = 60;
        combat::run_units(&mut s, &mut Vec::new());
        assert!(s.units[0].x > x, "window expiry releases movement");
    }
    #[test]
    fn r01_protection_caps_normally_and_root_never_extends_it() {
        let mut s = arena();
        let u = s.units[0];
        resolve(&mut s, &u);
        assert_eq!(
            hits::damage(&mut s, Target::Unit(1), 100, &mut Vec::new()),
            85
        );
        let until = s.units[0].fx.self_dr_until;
        s.units[0].status = Status::Rooted { ticks_left: 30 };
        combat::run_units(&mut s, &mut Vec::new());
        assert_eq!(s.units[0].fx.self_dr_until, until);
        s.units[0].fx.march_dr_bp = 1_000;
        assert_eq!(
            hits::damage(&mut s, Target::Unit(1), 100, &mut Vec::new()),
            80
        );
    }
    #[test]
    fn r01_basic_still_finishes_while_anchored_and_forms_keep_catalog_values() {
        let mut s = arena();
        let u = s.units[0];
        resolve(&mut s, &u);
        let mut ev = Vec::new();
        for _ in 0..=WINDUP {
            combat::tick_timers(&mut s);
            combat::run_units(&mut s, &mut ev);
            s.tick += 1;
        }
        assert_eq!(s.units[1].hp, 1_370);
        let a = s.rules().effective_stats(UnitKind::Bulwark, 2, 0, 0);
        assert_eq!(a.max_hp, 1_680);
        s.units[0].tier = 2;
        let u = s.units[0];
        resolve(&mut s, &u);
        assert_eq!(s.units[0].fx.self_dr_until, s.tick + 80);
        let b = s.rules().effective_stats(UnitKind::Bulwark, 2, 1, 0);
        assert_eq!(b.damage, 34);
        assert_eq!(
            b.on_hit,
            Some(crate::OnHit::Slow {
                bp: 1_500,
                ticks: 20
            })
        );
    }
}
