//! R03's ready-on-next-hit mark, caster-only follow-up bonus and ward/cleanse boundary.

use crate::{
    content::Ability,
    hits, roster_slots,
    state::{Event, MatchState, Target, Unit},
    types::{OnHit, UnitId},
    Ruleset,
};

pub(crate) fn parameters(rules: &Ruleset, u: &Unit) -> Option<(u16, u16, u16, u16, u16)> {
    if u.is_summon() {
        return None;
    }
    rules
        .def(u.kind)
        .ability(u.tier, u.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::CrowMark {
                period,
                ticks,
                slow_bp,
                cut_bp,
                bonus_bp,
            } => Some((period, ticks, slow_bp, cut_bp, bonus_bp)),
            _ => None,
        })
}
pub(crate) fn marked(state: &MatchState, id: UnitId) -> bool {
    state.units.iter().any(|u| {
        u.hp > 0
            && u.fx.mark_target == Some(id)
            && state.tick < u.fx.mark_until
            && parameters(state.rules(), u).is_some()
    })
}
pub(crate) fn clear_marks_on(state: &mut MatchState, id: UnitId) {
    for i in 0..state.units.len() {
        let u = state.units[i];
        if u.fx.mark_target == Some(id) && parameters(state.rules(), &u).is_some() {
            state.units[i].fx.mark_target = None;
            state.units[i].fx.mark_until = 0;
        }
    }
}
pub(crate) fn bonus(state: &MatchState, a: &Unit, target: Target) -> i32 {
    let Some((_, _, _, _, bp)) = parameters(state.rules(), a) else {
        return 0;
    };
    if matches!(target,Target::Unit(id) if a.fx.mark_target==Some(id))
        && state.tick < a.fx.mark_until
    {
        i32::from(bp)
    } else {
        0
    }
}
pub(crate) fn completed(state: &mut MatchState, a: &Unit, target: Target, ev: &mut Vec<Event>) {
    let Some((period, ticks, slow_bp, cut_bp, _)) = parameters(state.rules(), a) else {
        return;
    };
    if a.action_cd > 0 {
        return;
    }
    let Target::Unit(id) = target else {
        return;
    };
    let Some(v) = state.unit_index(id) else {
        return;
    };
    if state.units[v].seat == a.seat {
        return;
    }
    let Some(c) = state.unit_index(a.id) else {
        return;
    };
    state.units[c].action_cd = period.max(1);
    // A completed killing or warded hit spends readiness but cannot place a payload.
    if state.units[v].hp <= 0 || state.tick < state.units[v].fx.ward_until {
        return;
    }
    roster_slots::set_mark(
        &mut state.units[c].fx,
        id,
        state.tick.saturating_add(u32::from(ticks)),
    );
    roster_slots::apply_ability_cut(&mut state.units[v].fx, cut_bp, ticks, state.tick);
    let _applied = hits::apply_on_hit(state, id, OnHit::Slow { bp: slow_bp, ticks }, a.seat, ev);
    #[cfg(feature = "lab")]
    crate::lab::status(state, a, _applied);
    ev.push(Event::AbilityResolved {
        unit: a.id,
        kind: a.kind,
        primary: Some(id),
        center: state.units[v].x,
        tick: state.tick,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        afflictions, combat, specials, state::UnitFx, ArcanaPreset, Seat, Status, UnitKind,
    };
    fn unit(id: u32, kind: UnitKind, seat: Seat) -> Unit {
        Unit {
            id,
            kind,
            seat,
            tier: 0,
            branch: 0,
            x: 4_000 + if seat == Seat::B { 500 } else { 0 },
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
        }
    }
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.units = vec![
            unit(1, UnitKind::Crowcaller, Seat::A),
            unit(2, UnitKind::Archer, Seat::B),
            unit(3, UnitKind::Crowcaller, Seat::A),
        ];
        s.refresh_army();
        s
    }
    fn hit(s: &mut MatchState, i: usize, target: Target) -> i32 {
        let before = match target {
            Target::Unit(id) => s.units[s.unit_index(id).unwrap()].hp,
            Target::Stronghold(seat) => s.player(seat).stronghold_hp,
        };
        let a = s.units[i];
        let st = s.stats_of(&a);
        hits::resolve_hit(s, &a, target, &st, &mut Vec::new());
        before
            - match target {
                Target::Unit(id) => s.units[s.unit_index(id).unwrap()].hp,
                Target::Stronghold(seat) => s.player(seat).stronghold_hp,
            }
    }
    #[test]
    fn r03_mark_is_after_damage_and_bonus_is_owned_not_a_team_amplifier() {
        let mut s = arena();
        assert_eq!(hit(&mut s, 0, Target::Unit(2)), 58);
        assert_eq!(
            hit(&mut s, 2, Target::Unit(2)),
            58,
            "another caster cannot borrow the first mark"
        );
        assert_eq!(hit(&mut s, 0, Target::Unit(2)), 66);
        assert_eq!(
            s.units[1].fx.ability_cut_bp, 3_000,
            "two marks do not sum cuts"
        );
        assert_eq!(
            s.units[1].status,
            Status::Slowed {
                bp: 2_000,
                ticks_left: 80
            }
        );
        assert_eq!(s.units[0].action_cd, 140);
    }
    #[test]
    fn r03_ready_waits_for_completed_enemy_hit_and_ward_blocks_payload() {
        let mut s = arena();
        specials::advance(&mut s, 0, &mut Vec::new());
        assert!(s.units[0].fx.special.is_none());
        assert_eq!(hit(&mut s, 0, Target::Stronghold(Seat::B)), 58);
        assert_eq!(s.units[0].action_cd, 0);
        s.units[0].windup = Some((Target::Unit(2), 1));
        s.units[0].status = Status::Rooted { ticks_left: 20 };
        combat::run_units(&mut s, &mut Vec::new());
        assert_eq!(
            s.units[0].action_cd, 0,
            "Root cancellation spends no ready mark"
        );
        s.units[0].status = Status::None;
        s.units[1].fx.ward_until = 100;
        hit(&mut s, 0, Target::Unit(2));
        assert_eq!(s.units[0].action_cd, 140);
        assert_eq!(s.units[0].fx.mark_target, None);
        assert_eq!(s.units[1].fx.ability_cut_bp, 0);
        assert_eq!(s.units[1].status, Status::None);
    }
    #[test]
    fn r03_brew_clears_mark_and_new_cut_without_erasing_predatory_sequence() {
        let mut s = arena();
        hit(&mut s, 0, Target::Unit(2));
        let mut predator = unit(4, UnitKind::Ravager, Seat::A);
        predator.fx.mark_target = Some(2);
        predator.fx.mark_until = 80;
        predator.fx.attacks = 1;
        s.units.push(predator);
        let brewer = unit(5, UnitKind::AntidoteBrewer, Seat::B);
        s.units.push(brewer);
        s.units[1].heal_cut_bp = 2_000;
        s.units[1].heal_cut_until = 80;
        assert!(afflictions::affliction_score(&s, 1) > 0);
        afflictions::brew_cast(&mut s, &brewer, 3_000, 2, 1, 20, false, &mut Vec::new());
        assert_eq!(s.units[0].fx.mark_target, None);
        assert_eq!(s.units[1].status, Status::None);
        assert_eq!(s.units[1].fx.ability_cut_bp, 0);
        assert_eq!(
            roster_slots::heal_cut_now(&s.units[1], s.tick),
            2_000,
            "legacy Augment cut is unchanged"
        );
        assert_eq!(
            (s.units[3].fx.mark_target, s.units[3].fx.attacks),
            (Some(2), 1)
        );
    }
    #[test]
    fn r03_control_immunity_is_not_immunity_to_the_mark_or_heal_cut() {
        let mut s = arena();
        s.units[1].kind = UnitKind::Oilpress;
        s.units[1].tier = 2;
        hit(&mut s, 0, Target::Unit(2));
        assert_eq!(s.units[1].status, Status::None);
        assert_eq!(s.units[0].fx.mark_target, Some(2));
        assert_eq!(s.units[1].fx.ability_cut_bp, 3_000);
    }
    #[test]
    fn r03_forms_and_public_snapshot_restore_exact_own_mark_and_cut() {
        let mut s = arena();
        s.units[0].tier = 2;
        hit(&mut s, 0, Target::Unit(2));
        let a = s.view_for(Seat::A, &[]);
        let b = s.public_view(&[], 0);
        assert_eq!(a.units, b.units);
        assert_eq!(a.units[0].effects.unwrap().mark.unwrap().ticks, 110);
        assert_eq!(a.units[1].effects.unwrap().heal_cut.unwrap().bp, 4_000);
        let wire = crate::protocol::ServerMsg::View(Box::new(a));
        assert_eq!(
            crate::protocol::decode_server(&crate::protocol::encode(&wire)),
            Ok(wire)
        );
        s.units[0].branch = 1;
        s.units[0].fx.mark_target = None;
        s.units[0].action_cd = 0;
        assert_eq!(hit(&mut s, 0, Target::Unit(2)), 66);
        assert_eq!(hit(&mut s, 0, Target::Unit(2)), 82);
        s.tick = 80;
        roster_slots::tick_marks(&mut s);
        assert_eq!(s.units[0].fx.mark_target, None);
    }
    #[test]
    fn r03_a_killing_completed_hit_spends_readiness_without_marking_a_corpse() {
        let mut s = arena();
        s.units[1].hp = 1;
        hit(&mut s, 0, Target::Unit(2));
        assert_eq!(s.units[0].action_cd, 140);
        assert_eq!(s.units[0].fx.mark_target, None);
    }
    #[cfg(feature = "lab")]
    #[test]
    fn r03_real_spawn_has_full_initial_mark_cooldown() {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        let id = s.lab_spawn(Seat::A, UnitKind::Crowcaller, 0, 0);
        assert_eq!(s.units[s.unit_index(id).unwrap()].action_cd, 140);
    }
}
