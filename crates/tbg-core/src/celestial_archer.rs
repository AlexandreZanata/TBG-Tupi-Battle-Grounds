//! R14's counted empowered basic and pre-damage temporary Unit shield removal.
use crate::{
    content::Ability,
    state::{Event, MatchState, Target, Unit},
    types::UnitKind,
};
fn parameters(s: &MatchState, a: &Unit) -> Option<(u8, u16)> {
    if a.is_summon() {
        return None;
    }
    s.rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::PurifyingArrow { every, damage } => Some((every.max(1), damage)),
            _ => None,
        })
}
pub(crate) fn counter(a: &Unit) -> u8 {
    if a.kind == UnitKind::CelestialArcher && !a.is_summon() {
        a.fx.attacks.min(3)
    } else {
        0
    }
}
pub(crate) fn empowered(s: &MatchState, a: &Unit, target: Target) -> Option<u16> {
    let (every, damage) = parameters(s, a)?;
    let Target::Unit(id) = target else {
        return None;
    };
    let i = s.unit_index(id)?;
    (s.units[i].hp > 0 && s.units[i].seat != a.seat && a.fx.attacks.saturating_add(1) >= every)
        .then_some(damage)
}
pub(crate) fn strip(s: &mut MatchState, target: Target) {
    let Target::Unit(id) = target else {
        return;
    };
    if let Some(i) = s.unit_index(id) {
        s.units[i].fx.shield = 0;
        s.units[i].fx.shield_until = 0;
    }
}
pub(crate) fn completed(s: &mut MatchState, a: &Unit, target: Target, ev: &mut Vec<Event>) {
    let Some((every, _)) = parameters(s, a) else {
        return;
    };
    let Target::Unit(id) = target else {
        return;
    };
    let Some(i) = s.unit_index(id).filter(|&i| s.units[i].seat != a.seat) else {
        return;
    };
    let center = s.units[i].x;
    let Some(c) = s.unit_index(a.id) else {
        return;
    };
    let n = s.units[c].fx.attacks.saturating_add(1);
    if n < every {
        s.units[c].fx.attacks = n;
        return;
    }
    s.units[c].fx.attacks = 0;
    ev.push(Event::AbilityResolved {
        unit: a.id,
        kind: a.kind,
        primary: Some(id),
        center,
        tick: s.tick,
    });
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{hits, roster_slots, ArcanaPreset, Seat};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::CelestialArcher, 0, 0, 4_000);
        s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 4_500);
        s
    }
    fn hit(s: &mut MatchState, t: Target) -> Vec<Event> {
        let a = s.units[0];
        let st = s.stats_of(&a);
        let mut ev = Vec::new();
        hits::resolve_hit(s, &a, t, &st, &mut ev);
        ev
    }
    #[test]
    fn r14_fourth_shot_strips_before_damage_and_needs_no_existing_shield() {
        let mut s = arena();
        let id = s.units[1].id;
        roster_slots::apply_shield(&mut s.units[1].fx, 1000, 80, 0);
        let hp = s.units[1].hp;
        s.units[1].fx.ward_until = 100;
        for _ in 0..3 {
            hit(&mut s, Target::Unit(id));
        }
        assert_eq!((s.units[1].hp, s.units[1].fx.shield), (hp, 835));
        let ev = hit(&mut s, Target::Unit(id));
        assert_eq!((s.units[1].hp, s.units[1].fx.shield), (hp - 80, 0));
        assert_eq!(s.units[1].fx.ward_until, 100);
        assert!(ev.iter().any(|e| matches!(
            e,
            Event::AbilityResolved {
                kind: UnitKind::CelestialArcher,
                ..
            }
        )));
        s.units[0].fx.attacks = 3;
        hit(&mut s, Target::Unit(id));
        assert_eq!(s.units[1].hp, hp - 160);
    }
    #[test]
    fn r14_purification_never_removes_plates_or_stronghold_shields() {
        let mut s = arena();
        s.units[1].kind = UnitKind::Shatterback;
        s.units[1].fx.plates = 3;
        s.units[1].fx.plate_hp = 200;
        roster_slots::apply_shield(&mut s.units[1].fx, 100, 80, 0);
        s.units[0].fx.attacks = 3;
        let id = s.units[1].id;
        let hp = s.units[1].hp;
        hit(&mut s, Target::Unit(id));
        assert_eq!(
            (s.units[1].hp, s.units[1].fx.plates, s.units[1].fx.plate_hp),
            (hp, 3, 120)
        );
        s.units[0].fx.attacks = 3;
        s.units[0].x = 15_000;
        s.players[Seat::B.index()].shield = 1000;
        s.players[Seat::B.index()].shield_until = 100;
        hit(&mut s, Target::Stronghold(Seat::B));
        assert_eq!(s.players[Seat::B.index()].shield, 945);
        assert_eq!(counter(&s.units[0]), 3);
    }
    #[test]
    fn r14_forms_replace_frequency_and_amount_while_static_damage_scales_once() {
        let mut s = arena();
        let id = s.units[1].id;
        s.units[0].tier = 2;
        s.units[0].fx.attacks = 3;
        let hp = s.units[1].hp;
        hit(&mut s, Target::Unit(id));
        assert_eq!(s.units[1].hp, hp - 96);
        s.units[0].branch = 1;
        s.units[0].fx.attacks = 2;
        let hp = s.units[1].hp;
        hit(&mut s, Target::Unit(id));
        assert_eq!(s.units[1].hp, hp - 90);
        assert_eq!(s.view_for(Seat::A, &[]).units[0].ability_counter(), 0);
    }
}
