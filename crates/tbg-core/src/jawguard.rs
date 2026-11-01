//! R16's timed carapace and one expiry-earned retaliation charge, reusing bounded progress.
use crate::{
    content::Ability,
    roster_slots,
    state::{MatchState, Target, Unit},
    types::{Tick, UnitKind},
    Ruleset,
};
fn parameters(rules: &Ruleset, u: &Unit) -> Option<(u16, u16, u16)> {
    if u.kind != UnitKind::Jawguard || u.is_summon() {
        return None;
    }
    rules
        .def(u.kind)
        .ability(u.tier, u.branch)
        .into_iter()
        .find_map(|a| match a {
            Ability::RaisedCarapace {
                bp,
                ticks,
                retaliation,
                ..
            } => Some((bp, ticks, retaliation)),
            _ => None,
        })
}
pub(crate) fn counter(u: &Unit) -> u8 {
    if u.kind == UnitKind::Jawguard && !u.is_summon() {
        u.fx.attacks.min(1)
    } else {
        0
    }
}
pub(crate) fn tick(rules: &Ruleset, u: &mut Unit, now: Tick) {
    let Some((_, _, retaliation)) = parameters(rules, u) else {
        return;
    };
    if u.fx.self_dr_bp > 0 && u.fx.self_dr_until > 0 && now >= u.fx.self_dr_until {
        u.fx.self_dr_bp = 0;
        u.fx.self_dr_until = 0;
        u.fx.attacks = u8::from(retaliation > 0);
    }
}
pub(crate) fn resolve(s: &mut MatchState, u: &Unit) {
    let Some((bp, ticks, _)) = parameters(s.rules(), u) else {
        return;
    };
    if let Some(i) = s.unit_index(u.id) {
        roster_slots::apply_self_dr(&mut s.units[i].fx, bp, ticks, s.tick);
        s.units[i].fx.attacks = 0;
    }
}
pub(crate) fn bonus(s: &MatchState, u: &Unit, target: Target) -> u16 {
    let Some((_, _, retaliation)) = parameters(s.rules(), u) else {
        return 0;
    };
    let Target::Unit(id) = target else {
        return 0;
    };
    if counter(u) > 0
        && s.unit_index(id)
            .is_some_and(|i| s.units[i].hp > 0 && s.units[i].seat != u.seat)
    {
        retaliation
    } else {
        0
    }
}
pub(crate) fn completed(s: &mut MatchState, u: &Unit, target: Target) {
    if counter(u) == 0 {
        return;
    }
    let Target::Unit(id) = target else {
        return;
    };
    if s.unit_index(id).is_some_and(|i| s.units[i].seat != u.seat) {
        if let Some(i) = s.unit_index(u.id) {
            s.units[i].fx.attacks = 0;
        }
    }
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{hits, specials, ArcanaPreset, Seat, Status};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::Jawguard, 0, 0, 4_000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 4_700);
        s
    }
    fn hit(s: &mut MatchState, t: Target) {
        let a = s.units[0];
        let st = s.stats_of(&a);
        hits::resolve_hit(s, &a, t, &st, &mut Vec::new());
    }
    fn expiry(s: &mut MatchState, t: Tick) {
        s.tick = t;
        let rules = s.rules();
        tick(rules, &mut s.units[0], t);
    }
    #[test]
    fn r16_protection_is_self_only_non_anchoring_and_a2_keeps_applied_expiry() {
        let mut s = arena();
        assert_eq!(s.units[0].action_cd, 200);
        s.units[0].tier = 2;
        let a = s.units[0];
        resolve(&mut s, &a);
        assert_eq!(s.view_for(Seat::A, &[]).units[0].reduction, 1500);
        assert_eq!(roster_slots::self_dr_now(&s.units[1].fx, 0), 0);
        assert_eq!(s.units[0].fx.self_dr_until, 80);
        assert_eq!(s.stats_of(&a).max_hp, 1680);
        s.units[1].x = 7_000;
        let before = s.units[0].x;
        crate::combat::run_units(&mut s, &mut Vec::new());
        assert!(s.units[0].x > before);
        s.units[0].status = Status::Rooted { ticks_left: 100 };
        expiry(&mut s, 80);
        assert_eq!(s.view_for(Seat::A, &[]).units[0].reduction, 0);
        assert_eq!(counter(&s.units[0]), 0);
        assert_eq!(specials::period(s.rules(), a.kind, 2, 0), Some(200));
    }
    #[test]
    fn r16_expiry_arms_one_charge_and_only_next_enemy_unit_primary_spends_it() {
        let mut s = arena();
        s.units[0].tier = 1;
        s.units[0].branch = 1;
        let a = s.units[0];
        resolve(&mut s, &a);
        expiry(&mut s, 70);
        assert_eq!(counter(&s.units[0]), 1);
        expiry(&mut s, 71);
        assert_eq!(counter(&s.units[0]), 1);
        s.units[0].x = 15_000;
        hit(&mut s, Target::Stronghold(Seat::B));
        assert_eq!(counter(&s.units[0]), 1);
        let id = s.units[1].id;
        let hp = s.units[1].hp;
        hit(&mut s, Target::Unit(id));
        assert_eq!(s.units[1].hp, hp - 85);
        assert_eq!(counter(&s.units[0]), 0);
        hit(&mut s, Target::Unit(id));
        assert_eq!(s.units[1].hp, hp - 130);
    }
    #[test]
    fn r16_new_protection_overwrites_unspent_charge_and_shielded_hit_still_consumes() {
        let mut s = arena();
        s.units[0].tier = 2;
        s.units[0].branch = 1;
        s.units[0].fx.attacks = 1;
        let a = s.units[0];
        resolve(&mut s, &a);
        assert_eq!(counter(&s.units[0]), 0);
        expiry(&mut s, 60);
        assert_eq!(counter(&s.units[0]), 1);
        let id = s.units[1].id;
        roster_slots::apply_shield(&mut s.units[1].fx, 1000, 80, 60);
        let hp = s.units[1].hp;
        hit(&mut s, Target::Unit(id));
        assert_eq!((s.units[1].hp, s.units[1].fx.shield), (hp, 903));
        assert_eq!(s.public_view(&[], 0).units[0].ability_counter(), 0);
    }
}
