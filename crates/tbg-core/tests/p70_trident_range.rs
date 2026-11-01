//! Proposed P70 range boundaries and inheritance, independent of renderer geometry.
#![cfg(feature = "lab")]

use tbg_core::{
    content, state::stronghold_x, ArcanaPreset, Event, MatchState, Phase, Ruleset, Seat, Target,
    UnitKind,
};
const FORMS: [(u8, u8); 5] = [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)];
fn arena() -> MatchState {
    let mut s = MatchState::lab(70, Ruleset::canon(), [ArcanaPreset::NEUTRAL; 2]);
    s.tick = content::RUNNING_START;
    s.phase = Phase::Running;
    s
}
fn attacks(events: &[Event], id: u32, target: Target) -> bool {
    events.iter().any(|e| matches!(e, Event::AttackStart {from:Target::Unit(from),to} if *from==id && *to==target))
}
#[test]
fn p70_trident_inherits_the_proposed_range_and_respects_both_seat_edges() {
    // GAME-DESIGN Proposed P70 / BR-U01: closed distance boundary, no client-derived range.
    for seat in Seat::BOTH {
        for (tier, branch) in FORMS {
            assert_eq!(
                content::effective_stats(UnitKind::Trident, tier, branch, 0).range,
                2_000
            );
            for distance in [1_100, 1_600, 2_000, 2_001] {
                let mut s = arena();
                let x = 8_000 - seat.dir() * 3_000;
                let attacker = s.lab_spawn_at(seat, UnitKind::Trident, tier, branch, x);
                let target = s.lab_spawn_at(
                    seat.other(),
                    UnitKind::BoneBrute,
                    0,
                    0,
                    x + seat.dir() * distance,
                );
                let events = s.step();
                assert_eq!(
                    attacks(&events, attacker, Target::Unit(target)),
                    distance <= 2_000,
                    "{seat:?} {tier}/{branch} at {distance}"
                );
            }
        }
    }
}
#[test]
fn p70_trident_keeps_guard_priority_and_stronghold_boundaries() {
    // BR-U09 / BR-U01: increased reach does not bypass Guard or invent a tower targeting rule.
    for seat in Seat::BOTH {
        for distance in [2_000, 2_001] {
            let mut s = arena();
            let x = stronghold_x(seat.other()) - seat.dir() * distance;
            let attacker = s.lab_spawn_at(seat, UnitKind::Trident, 0, 0, x);
            assert_eq!(
                attacks(&s.step(), attacker, Target::Stronghold(seat.other())),
                distance <= 2_000
            );
        }
        let mut s = arena();
        let x = 8_000 - seat.dir() * 3_000;
        let attacker = s.lab_spawn_at(seat, UnitKind::Trident, 0, 0, x);
        s.lab_spawn_at(
            seat.other(),
            UnitKind::BoneBrute,
            0,
            0,
            x + seat.dir() * 1_200,
        );
        let guard = s.lab_spawn_at(
            seat.other(),
            UnitKind::Rampart,
            0,
            0,
            x + seat.dir() * 1_800,
        );
        assert!(attacks(&s.step(), attacker, Target::Unit(guard)));
    }
}
