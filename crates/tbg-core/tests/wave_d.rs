//! Wave D (P47): Oilpress formation, Drumguard bonds and march bonuses,
//! B2 splash carrying slow, A2 control immunity. Micro-scenarios cite §C.

#![cfg(feature = "lab")]

use tbg_core::{ArcanaPreset, Event, MatchState, Ruleset, Seat, Target, UnitKind};

fn rules() -> &'static Ruleset {
    Ruleset::canon()
}

fn state(seed: u64) -> MatchState {
    MatchState::lab(seed, rules(), [ArcanaPreset::NEUTRAL; 2])
}

/// Step `n` ticks, collecting every event.
fn step_n(s: &mut MatchState, n: u32) -> Vec<Event> {
    let mut out = Vec::new();
    for _ in 0..n {
        out.extend(s.step());
    }
    out
}

fn unit_x(s: &MatchState, id: u32) -> i32 {
    let i = s.unit_index(id).expect("unit must exist");
    s.units[i].x
}

// Oilpress (§C.8): while engaged, allies behind it stop 0.2 tiles behind
// and cannot walk past it.
#[test]
fn formation_holds_allies_behind_the_wall() {
    let mut s = state(11);
    let wall = s.lab_spawn_at(Seat::A, UnitKind::Oilpress, 0, 0, 6_000);
    // Root the wall in place: it is the anchor, and a marching wall would
    // move the assertion window.
    let i = s.unit_index(wall).expect("wall must exist");
    s.units[i].status = tbg_core::Status::Rooted { ticks_left: u16::MAX };
    let ally = s.lab_spawn_at(Seat::A, UnitKind::Sawbot, 0, 0, 4_000);
    let enemy = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 9_000);
    step_n(&mut s, 200);
    // The wall is engaged (enemy within 4 tiles in front); the ally holds.
    assert_eq!(
        unit_x(&s, ally),
        5_800,
        "the ally must stop 0.2 tiles behind the engaged wall"
    );
    // No cross-seat clamp: the enemy walks to contact, not to the wall line.
    let i = s.unit_index(enemy).expect("enemy must live");
    assert!(
        s.units[i].x >= 6_700,
        "enemies are unaffected by the formation"
    );
}

// Oilpress (§C.8): allies already ahead are unaffected.
#[test]
fn allies_ahead_are_unaffected() {
    let mut s = state(21);
    let _wall = s.lab_spawn_at(Seat::A, UnitKind::Oilpress, 0, 0, 6_000);
    // A tanky ally ahead of the wall: survives the walk into contact.
    let ally = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 7_500);
    let _enemy = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 9_000);
    step_n(&mut s, 200);
    assert!(
        unit_x(&s, ally) >= 7_700,
        "an ally ahead of the wall marches freely"
    );
}

// Oilpress (§C.8): the formation only applies while engaged; without an
// enemy near the wall, allies pass freely.
#[test]
fn formation_needs_engagement() {
    let mut s = state(22);
    let _wall = s.lab_spawn_at(Seat::A, UnitKind::Oilpress, 0, 0, 6_000);
    let ally = s.lab_spawn_at(Seat::A, UnitKind::Sawbot, 0, 0, 4_000);
    // No enemies at all: the wall never engages, the ally marches through.
    step_n(&mut s, 120);
    assert!(
        unit_x(&s, ally) > 6_000,
        "without engagement the ally passes the wall"
    );
}

// Drumguard (§C.6): the bond goes to the nearest ally in reach, and a
// challenger less than 0.5 tiles closer does not steal it — but one at
// least 0.5 tiles closer does. Deterministic, ties to the lowest id.
#[test]
fn bond_hysteresis_is_deterministic() {
    let mut s = state(31);
    let drummer = s.lab_spawn_at(Seat::A, UnitKind::Drumguard, 0, 0, 5_000);
    let ally1 = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 6_500);
    for id in [drummer, ally1] {
        let i = s.unit_index(id).expect("unit must exist");
        s.units[i].status = tbg_core::Status::Rooted { ticks_left: u16::MAX };
    }
    step_n(&mut s, 20);
    let bond_of = |s: &MatchState| s.units[s.unit_index(1).unwrap()].fx.bond;
    assert_eq!(bond_of(&s), Some(ally1), "bond must go to the only ally");
    // A challenger 350 closer (1150 vs 1500) does not steal the bond.
    let ally2 = s.lab_spawn_at(Seat::A, UnitKind::Sawbot, 0, 0, 6_150);
    let i = s.unit_index(ally2).expect("ally must exist");
    s.units[i].status = tbg_core::Status::Rooted { ticks_left: u16::MAX };
    step_n(&mut s, 20);
    assert_eq!(
        bond_of(&s),
        Some(ally1),
        "a challenger 350 closer must not steal the bond"
    );
    // ...but one 600 closer does, deterministically.
    let i = s.unit_index(ally2).expect("ally must exist");
    s.units[i].x = 5_900;
    step_n(&mut s, 20);
    assert_eq!(
        bond_of(&s),
        Some(ally2),
        "a challenger 600 closer must take the bond"
    );
}

// Drumguard (§C.6): with two drummers on one ally, the best value of each
// bonus applies (no stacking).
#[test]
fn best_bonus_wins_with_two_drummers() {
    let mut s = state(41);
    // A2 War March (+45 % move) and B1 Iron Rhythm (−18 % damage taken).
    let _d1 = s.lab_spawn_at(Seat::A, UnitKind::Drumguard, 2, 0, 5_000);
    let _d2 = s.lab_spawn_at(Seat::A, UnitKind::Drumguard, 1, 1, 5_100);
    let ally = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_500);
    for id in [1, 2, ally] {
        let i = s.unit_index(id).expect("unit must exist");
        s.units[i].status = tbg_core::Status::Rooted { ticks_left: u16::MAX };
    }
    step_n(&mut s, 20);
    let i = s.unit_index(ally).expect("ally must exist");
    assert_eq!(
        s.units[i].fx.march_move_bp, 4_500,
        "best move bonus wins (+45 %, not +25 %)"
    );
    assert_eq!(
        s.units[i].fx.march_dr_bp, 1_800,
        "best reduction wins (−18 %, not −15 %)"
    );
}

// Drumguard (§C.6) joined with BR-X05: the bonded ally takes the capped
// reduction (B1 −18 % joins the 20 % cap).
#[test]
fn march_damage_reduction_applies() {
    let mut s = state(42);
    let _d = s.lab_spawn_at(Seat::A, UnitKind::Drumguard, 1, 1, 5_000);
    let ally = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_500);
    let _hexer = s.lab_spawn_at(Seat::B, UnitKind::Hexer, 0, 0, 8_000);
    let mut first = 0;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::Hit { to: Target::Unit(id), amount } = e {
                if id == ally && first == 0 {
                    first = amount;
                }
            }
        }
        if first > 0 {
            break;
        }
    }
    // 29 × (1 − 0.18) = 23.78 → 23.
    assert_eq!(first, 23, "bonded ally takes −18 % damage");
}

// Drumguard B2 (§C.6): the bonded ally ignores slows and fish for
// movement (the stacks still land). The drummer is rooted in place; the
// ally marches at full speed through the first 40 bonded ticks.
#[test]
fn b2_ally_ignores_slow_and_fish_for_movement() {
    let mut s = state(43);
    // B2 Unbreakable Rhythm.
    let drummer = s.lab_spawn_at(Seat::A, UnitKind::Drumguard, 2, 1, 5_000);
    let ally = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_400);
    let _hexer = s.lab_spawn_at(Seat::B, UnitKind::Hexer, 0, 0, 9_000);
    let _finn = s.lab_spawn_at(Seat::B, UnitKind::OldFinn, 0, 0, 9_200);
    let i = s.unit_index(drummer).expect("drummer must exist");
    s.units[i].status = tbg_core::Status::Rooted { ticks_left: u16::MAX };
    let mut slowed = false;
    let mut positions = Vec::new();
    for _ in 0..60 {
        for e in s.step() {
            if let Event::StatusApplied { unit } = e {
                if unit == ally {
                    slowed = true;
                }
            }
        }
        positions.push(unit_x(&s, ally));
    }
    assert!(slowed, "slow was never applied (scenario broken)");
    // Ticks 10..40, bonded and afflicted: every step covers the bonded
    // speed (30 × 1.25 = 37; the +25 % march of the B2 drummer applies,
    // slows and fish are ignored).
    for w in positions[10..40].windows(2) {
        assert_eq!(
            w[1] - w[0],
            37,
            "the bonded ally must keep full speed through slows and fish"
        );
    }
}

// Oilpress A2 (§C.8): immune to slow and root. Fish still stacks, but the
// movement factor ignores the immune unit.
#[test]
fn a2_ignores_slow_root_and_fish_movement() {
    let mut s = state(44);
    // tier 2, branch 0: Immovable Mass.
    let wall = s.lab_spawn_at(Seat::A, UnitKind::Oilpress, 2, 0, 6_000);
    let _hexer = s.lab_spawn_at(Seat::B, UnitKind::Hexer, 0, 0, 8_000);
    let _rootcaller = s.lab_spawn_at(Seat::B, UnitKind::Rootcaller, 0, 0, 8_200);
    let _finn = s.lab_spawn_at(Seat::B, UnitKind::OldFinn, 0, 0, 8_400);
    let mut hits = 0;
    for _ in 0..150 {
        for e in s.step() {
            if let Event::Hit { to: Target::Unit(id), .. } = e {
                if id == wall {
                    hits += 1;
                }
            }
        }
        let i = s.unit_index(wall).expect("wall must live");
        assert_eq!(
            s.units[i].status,
            tbg_core::Status::None,
            "Immovable Mass ignores slow and root"
        );
    }
    assert!(hits > 0, "hits must land (scenario broken)");
    // Fish stacks land normally...
    let i = s.unit_index(wall).expect("wall must live");
    assert!(
        s.units[i].fx.fish.iter().any(|f| f.left > 0),
        "fish still stacks on A2 (scenario broken?)"
    );
    // ...but movement ignores them: 60 ticks × 15 mdt = 900 while walking
    // into range (the wall stops at 800 from the Hexer).
    let mut s = state(45);
    let wall = s.lab_spawn_at(Seat::A, UnitKind::Oilpress, 2, 0, 6_000);
    let _finn = s.lab_spawn_at(Seat::B, UnitKind::OldFinn, 0, 0, 8_400);
    let x0 = unit_x(&s, wall);
    step_n(&mut s, 60);
    assert_eq!(
        unit_x(&s, wall) - x0,
        60 * 15,
        "fish must not slow an immune wall"
    );
    // Control: a base Oilpress in the same spot is slowed and rooted.
    let mut s = state(46);
    let wall = s.lab_spawn_at(Seat::A, UnitKind::Oilpress, 0, 0, 6_000);
    let _hexer = s.lab_spawn_at(Seat::B, UnitKind::Hexer, 0, 0, 8_000);
    let x0 = unit_x(&s, wall);
    let mut ever_afflicted = false;
    for _ in 0..150 {
        for e in s.step() {
            if let Event::StatusApplied { unit } = e {
                if unit == wall {
                    ever_afflicted = true;
                }
            }
        }
    }
    assert!(ever_afflicted, "control hits must land (scenario broken)");
    assert!(
        unit_x(&s, wall) - x0 < 150 * 15,
        "a non-immune wall is slowed by control"
    );
}

// Oilpress B2 (§C.8): the Black Mire splash carries the slow to every
// splashed enemy. Statuses expire, so poll every tick.
#[test]
fn b2_splash_carries_the_slow() {
    let mut s = state(46);
    // tier 2, branch 1: Black Mire.
    let _wall = s.lab_spawn_at(Seat::A, UnitKind::Oilpress, 2, 1, 6_000);
    let near = s.lab_spawn_at(Seat::B, UnitKind::Sawbot, 0, 0, 6_700);
    let far = s.lab_spawn_at(Seat::B, UnitKind::Sawbot, 0, 0, 7_200);
    let mut near_slowed = false;
    let mut far_slowed = false;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::StatusApplied { unit } = e {
                if unit == near {
                    near_slowed = true;
                }
                if unit == far {
                    far_slowed = true;
                }
            }
        }
    }
    assert!(near_slowed, "primary target must be slowed by Black Mire");
    assert!(far_slowed, "splashed target must be slowed by Black Mire");
}
