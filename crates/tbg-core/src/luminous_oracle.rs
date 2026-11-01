//! R13's single-target weakening: no healing, aura, on-hit or control-immunity shortcut.
use crate::{
    content::Ability,
    roster_slots,
    state::{MatchState, Unit},
    types::UnitId,
};
pub(crate) fn resolve(s: &mut MatchState, a: &Unit, target: UnitId) {
    let Some((bp, ticks)) = s
        .rules()
        .def(a.kind)
        .ability(a.tier, a.branch)
        .into_iter()
        .find_map(|v| match v {
            Ability::HarmonicPulse { bp, ticks, .. } => Some((bp, ticks)),
            _ => None,
        })
    else {
        return;
    };
    let Some(i) = s.unit_index(target) else {
        return;
    };
    if s.units[i].hp <= 0 || s.units[i].seat == a.seat || s.tick < s.units[i].fx.ward_until {
        return;
    }
    roster_slots::apply_weaken(&mut s.units[i].fx, bp, ticks, s.tick);
}
#[cfg(all(test, feature = "lab"))]
mod tests {
    use super::*;
    use crate::{afflictions, hits, specials, ArcanaPreset, Seat, Target, UnitKind};
    fn arena() -> MatchState {
        let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
        s.lab_spawn_at(Seat::A, UnitKind::LuminousOracle, 0, 0, 4_000);
        s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 4_500);
        s
    }
    #[test]
    fn r13_weakening_changes_basic_at_resolution_but_never_auxiliary_damage() {
        let mut s = arena();
        let a = s.units[0];
        let victim = s.units[1].id;
        resolve(&mut s, &a, victim);
        let enemy = s.units[1];
        let st = s.stats_of(&enemy);
        let hp = s.units[0].hp;
        hits::resolve_hit(&mut s, &enemy, Target::Unit(a.id), &st, &mut Vec::new());
        assert_eq!(s.units[0].hp, hp - 38);
        assert_eq!(hits::auxiliary_amount(&st, 65), 65);
        assert_eq!(s.stats_of(&a).heal, 0);
    }
    #[test]
    fn r13_ward_blocks_weaken_but_control_only_immunity_does_not() {
        let mut s = arena();
        s.units[1].kind = UnitKind::Oilpress;
        s.units[1].tier = 2;
        s.units[1].fx.ward_until = 100;
        let a = s.units[0];
        let victim = s.units[1].id;
        let hp = s.units[1].hp;
        resolve(&mut s, &a, victim);
        assert_eq!(roster_slots::weaken_now(&s.units[1].fx, 0), 0);
        s.units[1].fx.ward_until = 0;
        resolve(&mut s, &a, victim);
        assert_eq!(roster_slots::weaken_now(&s.units[1].fx, 0), 1500);
        assert_eq!(s.units[1].hp, hp);
    }
    #[test]
    fn r13_absolute_duration_cadence_and_brew_clear_public_weakening() {
        let mut s = arena();
        s.units[0].tier = 2;
        let a = s.units[0];
        let victim = s.units[1].id;
        resolve(&mut s, &a, victim);
        assert_eq!(
            s.public_view(&[], 0).units[1]
                .effects
                .unwrap()
                .weaken
                .unwrap()
                .ticks,
            80
        );
        let brewer_id = s.lab_spawn_at(Seat::B, UnitKind::AntidoteBrewer, 0, 0, 4_700);
        let brewer = s.units[s.unit_index(brewer_id).unwrap()];
        let (strip, ward, second) = afflictions::brew_of(&s, &brewer).unwrap();
        afflictions::brew_cast(
            &mut s,
            &brewer,
            3_000,
            victim,
            strip,
            ward,
            second,
            &mut Vec::new(),
        );
        assert_eq!(roster_slots::weaken_now(&s.units[1].fx, 0), 0);
        assert_eq!(
            specials::period(s.rules(), UnitKind::LuminousOracle, 2, 1),
            Some(160)
        );
    }
}
