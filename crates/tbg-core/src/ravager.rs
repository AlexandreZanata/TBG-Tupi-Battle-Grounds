//! R09's bounded same-prey basic damage sequence (GAME-DESIGN §13.2–13.3).

use crate::{
    content::Ability,
    roster_slots,
    state::{MatchState, Target, Unit},
    types::{Tick, UnitKind},
};

fn parameters(state: &MatchState, u: &Unit) -> Option<(u16, u8, u16)> {
    if u.is_summon() {
        return None;
    }
    state
        .rules()
        .def(u.kind)
        .ability(u.tier, u.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::PredatoryFrenzy {
                step_bp,
                increments,
                gap,
            } => Some((step_bp, increments, gap)),
            _ => None,
        })
}

pub(crate) fn counter(u: &Unit, now: Tick) -> u8 {
    if u.kind == UnitKind::Ravager && u.fx.mark_target.is_some() && now < u.fx.mark_until {
        u.fx.attacks.min(7)
    } else {
        0
    }
}

fn clear(u: &mut Unit) {
    u.fx.attacks = 0;
    u.fx.mark_target = None;
    u.fx.mark_until = 0;
}

/// Even a cancelled new-target windup cannot carry the old prey's sequence.
pub(crate) fn retarget(state: &mut MatchState, i: usize, target: Target) {
    let u = state.units[i];
    if parameters(state, &u).is_none() {
        return;
    }
    let same_prey = matches!(target, Target::Unit(id) if u.fx.mark_target == Some(id));
    if !same_prey {
        clear(&mut state.units[i]);
    }
}

pub(crate) fn bonus(state: &MatchState, attacker: &Unit, target: Target) -> i32 {
    let Some((step, cap, _)) = parameters(state, attacker) else {
        return 0;
    };
    let Target::Unit(id) = target else {
        return 0;
    };
    let valid = state
        .unit_index(id)
        .is_some_and(|i| state.units[i].hp > 0 && state.units[i].seat != attacker.seat);
    if valid && attacker.fx.mark_target == Some(id) {
        i32::from(counter(attacker, state.tick).min(cap)) * i32::from(step)
    } else {
        0
    }
}

/// Only `resolve_hit` calls this: no auxiliary hit, DoT, or canceled windup counts.
pub(crate) fn completed(state: &mut MatchState, attacker: &Unit, target: Target) {
    let Some((_, cap, gap)) = parameters(state, attacker) else {
        return;
    };
    let Some(i) = state.unit_index(attacker.id) else {
        return;
    };
    let valid = match target {
        Target::Unit(id) => state
            .unit_index(id)
            .filter(|&t| state.units[t].seat != attacker.seat && state.units[t].hp > 0)
            .map(|_| id),
        Target::Stronghold(_) => None,
    };
    let Some(id) = valid else {
        clear(&mut state.units[i]);
        return;
    };
    let previous = if attacker.fx.mark_target == Some(id) {
        counter(attacker, state.tick)
    } else {
        0
    };
    state.units[i].fx.attacks = previous.saturating_add(1).min(cap);
    roster_slots::set_mark(
        &mut state.units[i].fx,
        id,
        state.tick.saturating_add(Tick::from(gap)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hits, state::UnitFx, ArcanaPreset, Seat, Status};

    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        for (id, kind, seat) in [
            (1, UnitKind::Ravager, Seat::A),
            (2, UnitKind::Archer, Seat::B),
            (3, UnitKind::Beetling, Seat::B),
        ] {
            s.units.push(Unit {
                id,
                kind,
                seat,
                tier: 0,
                branch: 0,
                x: 1000,
                hp: 10_000,
                max_hp: 10_000,
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

    fn hit(s: &mut MatchState, target: Target) -> i32 {
        let before = match target {
            Target::Unit(id) => s.units[s.unit_index(id).unwrap()].hp,
            Target::Stronghold(seat) => s.player(seat).stronghold_hp,
        };
        let a = s.units[0];
        let st = s.stats_of(&a);
        hits::resolve_hit(s, &a, target, &st, &mut Vec::new());
        before
            - match target {
                Target::Unit(id) => s.units[s.unit_index(id).unwrap()].hp,
                Target::Stronghold(seat) => s.player(seat).stronghold_hp,
            }
    }

    #[test]
    fn r09_first_hit_is_plain_then_ramps_only_same_enemy_primaries() {
        let mut s = arena();
        let damage: Vec<_> = (0..6).map(|_| hit(&mut s, Target::Unit(2))).collect();
        assert_eq!(
            damage,
            [48, 51, 55, 59, 59, 59],
            "§13.2: floor once after the additive bonus"
        );
        assert_eq!(
            hit(&mut s, Target::Unit(3)),
            48,
            "summons are legal new prey, no old sequence"
        );
        assert_eq!(hit(&mut s, Target::Stronghold(Seat::B)), 48);
        assert_eq!(
            counter(&s.units[0], s.tick),
            0,
            "Strongholds clear the sequence"
        );
    }

    #[test]
    fn r09_cancelled_windup_target_death_and_inactivity_reset_without_free_hits() {
        let mut s = arena();
        hit(&mut s, Target::Unit(2));
        let before = counter(&s.units[0], 0);
        retarget(&mut s, 0, Target::Unit(2));
        assert_eq!(
            counter(&s.units[0], 0),
            before,
            "starting is not completing"
        );
        retarget(&mut s, 0, Target::Unit(3));
        assert_eq!(
            counter(&s.units[0], 0),
            0,
            "new prey clears even if the new windup is canceled"
        );
        hit(&mut s, Target::Unit(2));
        s.tick = 60;
        roster_slots::tick_marks(&mut s);
        assert_eq!(s.units[0].fx.attacks, 0);
        assert_eq!(hit(&mut s, Target::Unit(2)), 48);
        s.units[1].hp = 1;
        hit(&mut s, Target::Unit(2));
        assert_eq!(
            counter(&s.units[0], s.tick),
            0,
            "killing prey leaves no ramp"
        );
    }

    #[test]
    fn r09_deep_frenzy_replaces_cap_and_gap_and_views_restore_it() {
        let mut s = arena();
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        let damage: Vec<_> = (0..6).map(|_| hit(&mut s, Target::Unit(2))).collect();
        assert_eq!(damage, [48, 51, 55, 59, 63, 63]);
        s.tick = 79;
        let a = s.view_for(Seat::A, &[]);
        let p = s.public_view(&[], 0);
        assert_eq!(a.units[0].ability_counter(), 4);
        assert_eq!(a.units, p.units);
        s.tick = 80;
        assert_eq!(s.public_view(&[], 0).units[0].ability_counter(), 0);
    }
}
