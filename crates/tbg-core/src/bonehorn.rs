//! R08's reactive carapace and printed short-range target bonus (GAME-DESIGN §13).

use crate::{
    content::Ability,
    roster_slots,
    state::{MatchState, Target, Unit},
    types::UnitId,
};

pub(crate) fn after_hp_damage(state: &mut MatchState, id: UnitId, removed: i32) {
    if removed <= 0 {
        return;
    }
    let Some(i) = state.unit_index(id) else {
        return;
    };
    let u = state.units[i];
    if u.hp <= 0 || u.is_summon() {
        return;
    }
    for ability in state.rules().def(u.kind).ability(u.tier, u.branch) {
        if let Ability::PrimordialCarapace {
            step_bp,
            cap_bp,
            gap,
            ..
        } = ability
        {
            let bp = (roster_slots::self_dr_now(&u.fx, state.tick) as u16)
                .saturating_add(step_bp)
                .min(cap_bp);
            roster_slots::apply_self_dr(&mut state.units[i].fx, bp, gap, state.tick);
        }
    }
}

pub(crate) fn short_range_bonus(state: &MatchState, attacker: &Unit, target: Target) -> i32 {
    let Target::Unit(id) = target else {
        return 0;
    };
    let Some(i) = state.unit_index(id) else {
        return 0;
    };
    let rules = state.rules();
    for ability in rules
        .def(attacker.kind)
        .ability(attacker.tier, attacker.branch)
    {
        if let Ability::PrimordialCarapace {
            short_range,
            short_bonus_bp,
            ..
        } = ability
        {
            if rules.def(state.units[i].kind).range <= short_range {
                return i32::from(short_bonus_bp);
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hits, state::UnitFx, ArcanaPreset, MatchState, Seat, Status, UnitKind};

    fn bonehorn() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.units.push(Unit {
            id: 1,
            seat: Seat::A,
            kind: UnitKind::Bonehorn,
            tier: 0,
            branch: 0,
            x: 0,
            hp: 1_600,
            max_hp: 1_600,
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
        s
    }

    #[test]
    fn r08_hp_receipts_stack_after_the_hit_and_expire_together() {
        let mut s = bonehorn();
        let mut ev = Vec::new();
        // §13.2: first hit has no reduction; the second sees exactly one stack.
        assert_eq!(hits::damage(&mut s, Target::Unit(1), 100, &mut ev), 100);
        assert_eq!(hits::damage(&mut s, Target::Unit(1), 100, &mut ev), 97);
        for _ in 0..8 {
            hits::damage(&mut s, Target::Unit(1), 1, &mut ev);
        }
        assert_eq!(roster_slots::self_dr_now(&s.units[0].fx, s.tick), 1_500);
        s.tick = 60;
        assert_eq!(roster_slots::self_dr_now(&s.units[0].fx, s.tick), 0);
        assert_eq!(hits::damage(&mut s, Target::Unit(1), 100, &mut ev), 100);
    }

    #[test]
    fn r08_shield_only_and_zero_damage_never_stack_or_refresh() {
        let mut s = bonehorn();
        let mut ev = Vec::new();
        roster_slots::apply_shield(&mut s.units[0].fx, 200, 100, 0);
        assert_eq!(hits::damage(&mut s, Target::Unit(1), 100, &mut ev), 0);
        hits::damage(&mut s, Target::Unit(1), 0, &mut ev);
        assert_eq!(s.units[0].fx.self_dr_until, 0);
        // Overflow removes HP once, and starts one stack, not two.
        assert_eq!(hits::damage(&mut s, Target::Unit(1), 110, &mut ev), 10);
        assert_eq!(s.units[0].fx.self_dr_bp, 300);
    }

    #[test]
    fn r08_public_and_seat_views_restore_armor_and_clear_it_at_expiry() {
        let mut s = bonehorn();
        after_hp_damage(&mut s, 1, 1);
        let seat = s.view_for(Seat::B, &[]);
        let public = s.public_view(&[], 0);
        assert_eq!(seat.units[0].reduction, 300);
        assert_eq!(
            seat.units, public.units,
            "both audiences receive the same armor fact"
        );
        s.tick = 60;
        assert_eq!(s.public_view(&[], 0).units[0].reduction, 0);
    }

    #[test]
    fn r08_refreshes_at_cap_but_still_shares_the_global_reduction_cap() {
        let mut s = bonehorn();
        let mut ev = Vec::new();
        for _ in 0..5 {
            after_hp_damage(&mut s, 1, 1);
        }
        s.tick = 30;
        s.units[0].fx.march_dr_bp = 1_000;
        assert_eq!(hits::damage(&mut s, Target::Unit(1), 100, &mut ev), 80);
        assert_eq!(
            s.units[0].fx.self_dr_until, 90,
            "even a capped receipt refreshes the shared expiry"
        );
        assert_eq!(s.units[0].fx.self_dr_bp, 1_500);
    }

    #[test]
    fn r08_forms_use_absolute_bonuses_and_printed_range_not_modified_range() {
        let mut s = bonehorn();
        let mut target = s.units[0];
        target.id = 2;
        target.seat = Seat::B;
        target.kind = UnitKind::Sawbot;
        s.units.push(target);
        let mut a = s.units[0];
        a.tier = 2;
        a.branch = 1;
        assert_eq!(short_range_bonus(&s, &a, Target::Unit(2)), 2_500);
        assert_eq!(short_range_bonus(&s, &a, Target::Stronghold(Seat::B)), 0);
        s.units[1].kind = UnitKind::Archer;
        assert_eq!(short_range_bonus(&s, &a, Target::Unit(2)), 0);
        s.units[0].tier = 2;
        s.units[0].branch = 0;
        after_hp_damage(&mut s, 1, 1);
        assert_eq!(s.units[0].fx.self_dr_until, 80);
        assert_eq!(
            s.rules()
                .effective_stats(UnitKind::Bonehorn, 2, 0, 0)
                .max_hp,
            1_920
        );
    }
}
