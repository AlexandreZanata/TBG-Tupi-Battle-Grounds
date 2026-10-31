//! BR-U08a: future summoners must join these enemy-independent timer checks.
#![cfg(feature = "lab")]
use tbg_core::{ArcanaPreset, MatchState, Ruleset, Seat, UnitKind};

fn timers(kind: UnitKind, enemies: bool) -> Vec<u32> {
    let mut rules = Ruleset::canon().clone();
    // Isolate timer behavior from combat deaths, movement and summon caps.
    for u in &mut rules.units {
        u.speed = 0;
        u.damage = 0;
    }
    let mut s = MatchState::lab(1, Box::leak(Box::new(rules)), [ArcanaPreset::NEUTRAL; 2]);
    let id = s.lab_spawn_at(Seat::A, kind, 0, 0, 2_000);
    if enemies {
        s.lab_spawn_at(Seat::B, UnitKind::Rampart, 0, 0, 14_000);
    }
    let mut ticks = Vec::new();
    for _ in 0..500 {
        let before = s.units.iter().filter(|u| u.summoner == Some(id)).count();
        s.step();
        let after = s.units.iter().filter(|u| u.summoner == Some(id)).count();
        if after > before {
            ticks.push(s.tick);
        }
    }
    ticks
}
#[test]
fn summoners_never_need_an_enemy() {
    for kind in [UnitKind::Nester, UnitKind::Smuggler] {
        let empty = timers(kind, false);
        assert!(!empty.is_empty(), "BR-U08a: {kind:?}");
        assert_eq!(empty, timers(kind, true), "BR-U08a: {kind:?}");
    }
}
#[test]
fn crate_is_dropped_behind_without_stopping() {
    for seat in Seat::BOTH {
        let mut s = MatchState::lab(1, Ruleset::canon(), [ArcanaPreset::NEUTRAL; 2]);
        let id = s.lab_spawn_at(seat, UnitKind::Smuggler, 0, 0, 8_000);
        s.units[0].action_cd = 1;
        let x = s.units[0].x;
        s.step();
        let krate = s.units.iter().find(|u| u.kind == UnitKind::Crate).unwrap();
        assert_eq!(krate.x, x - seat.dir() * 400);
        assert_eq!(s.units[s.unit_index(id).unwrap()].action_cd, 400);
        assert!(
            (s.units[s.unit_index(id).unwrap()].x - x) * seat.dir() > 0,
            "dropping cargo must not stop movement"
        );
        assert_eq!(
            tbg_core::content::def(UnitKind::Smuggler).range,
            tbg_core::content::STRONGHOLD_RANGE
        );
    }
}
