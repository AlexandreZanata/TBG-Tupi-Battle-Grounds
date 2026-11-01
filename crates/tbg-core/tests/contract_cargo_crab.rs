//! Cargo Crab contract (D-56, §C.15):
//! - Boarding picks the nearest eligible ally within 0.5 tile, tiebreak by lowest ID.
//! - The pair moves at the speed of the faster of the two + 20% carry bonus.
//! - Single-target attacks target and damage the crab first; passenger is untargetable.
//! - Area/splash damage hits both the crab and the passenger.
//! - Crab death dismounts the passenger behind it with intact HP and 3s re-board lock.
//! - `UnitView.fx` packs `FX_CARRIED` (`is_carried()`) for the passenger.
//! - Symmetrical and deterministic across seeds and seats.
//! - The carry bonus is the larger of the +20 % base and the branch bonus,
//!   never their sum, and shares the +50 % move cap with every other move
//!   bonus (D-60, BR-X12, BR-X03).
//! - A Slow on the passenger lowers the pair only down to the crab's own
//!   speed (D-60).
//!
//! The `PROTOCOL_VERSION` pin lives only in `contract_protocol.rs`.

#![cfg(feature = "lab")]

use tbg_core::content::TICK_HZ;
use tbg_core::state::Target;
use tbg_core::view::{FX_CARRIED, FX_SUMMON};
use tbg_core::{ArcanaPreset, Event, MatchState, Ruleset, Seat, Status, UnitId, UnitKind};

fn rules() -> &'static Ruleset {
    Ruleset::canon()
}

fn state(seed: u64) -> MatchState {
    MatchState::lab(seed, rules(), [ArcanaPreset::NEUTRAL; 2])
}

fn step_n(s: &mut MatchState, n: u32) {
    for _ in 0..n {
        s.step();
    }
}

#[test]
fn boarding_picks_nearest_eligible_ally_tiebreak_lowest_id() {
    let mut s = state(42);
    // Spawn two allies at the exact same distance (5_100, crab at 5_000)
    let ally_a = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 5_100);
    let ally_b = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 5_100);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 5_000);

    step_n(&mut s, 20);

    let ci = s.unit_index(crab).expect("crab lives");
    assert_eq!(
        s.units[ci].fx.passenger,
        Some(ally_a),
        "equal distance tie-break must choose lowest unit id"
    );
    let pi = s.unit_index(ally_a).expect("passenger lives");
    assert_eq!(s.units[pi].fx.carrier, Some(crab));
    assert_eq!(s.units[pi].x, s.units[ci].x);

    let other_pi = s.unit_index(ally_b).expect("other ally lives");
    assert_eq!(s.units[other_pi].fx.carrier, None);
}

#[test]
fn pair_speed_uses_faster_unit_speed_plus_carry_bonus() {
    let mut s = state(43);
    // Cargo Crab base speed: 25 millitiles/tick.
    // Archer base speed: 50 millitiles/tick.
    let archer = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 5_000);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 5_050);

    step_n(&mut s, 10);
    let ci = s.unit_index(crab).expect("crab lives");
    assert_eq!(s.units[ci].fx.passenger, Some(archer));

    let x_before = s.units[ci].x;
    s.step();
    let x_after = s.units[ci].x;
    let dx = x_after - x_before;

    // Archer speed 50 > crab 25. Faster speed is 50.
    // Carry bonus is +20% (2_000 bp): 50 * 12_000 / 10_000 = 60.
    assert_eq!(
        dx, 60,
        "carried pair moves at max(crab, passenger) * (1 + 0.20)"
    );

    let pi = s.unit_index(archer).expect("archer lives");
    assert_eq!(
        s.units[pi].x, x_after,
        "passenger x is exactly synced with crab"
    );
}

#[test]
fn single_target_hits_crab_while_aoe_hits_passenger() {
    let mut s = state(44);
    let passenger = s.lab_spawn_at(Seat::A, UnitKind::Bloomkeeper, 0, 0, 6_000);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 6_050);
    // Farwatch is single-target (no splash).
    let _farwatch = s.lab_spawn_at(Seat::B, UnitKind::Farwatch, 0, 0, 10_000);

    step_n(&mut s, 30);
    let ci = s.unit_index(crab).expect("crab lives");
    assert_eq!(s.units[ci].fx.passenger, Some(passenger));

    let initial_passenger_hp = s.units[s.unit_index(passenger).unwrap()].hp;

    // 100 ticks of Farwatch fire: every single-target attack and hit lands on the crab
    let mut attacks_on_crab = 0;
    let mut attacks_on_passenger = 0;
    let mut hits_on_crab = 0;
    let mut hits_on_passenger = 0;
    for _ in 0..100 {
        for ev in s.step() {
            match ev {
                Event::AttackStart {
                    to: Target::Unit(uid),
                    ..
                } => {
                    if uid == crab {
                        attacks_on_crab += 1;
                    }
                    if uid == passenger {
                        attacks_on_passenger += 1;
                    }
                }
                Event::Hit {
                    to: Target::Unit(uid),
                    ..
                } => {
                    if uid == crab {
                        hits_on_crab += 1;
                    }
                    if uid == passenger {
                        hits_on_passenger += 1;
                    }
                }
                _ => {}
            }
        }
    }

    assert!(attacks_on_crab > 0, "Farwatch must aim at crab");
    assert_eq!(
        attacks_on_passenger, 0,
        "Farwatch must never aim at the passenger"
    );
    assert!(hits_on_crab > 0, "direct hits must land on the crab");
    assert_eq!(
        hits_on_passenger, 0,
        "single-target hits must never reach the carried passenger"
    );

    // Now spawn SiegeWalker (area splash damage):
    let _siege = s.lab_spawn_at(Seat::B, UnitKind::SiegeWalker, 0, 0, 8_000);
    step_n(&mut s, 60);

    let pi = s.unit_index(passenger).expect("passenger lives");
    assert!(
        s.units[pi].hp < initial_passenger_hp,
        "area/splash damage must reach the carried passenger"
    );
}

#[test]
fn crab_death_dismounts_passenger_with_intact_hp_and_lock() {
    let mut s = state(45);
    let passenger = s.lab_spawn_at(Seat::A, UnitKind::Bloomkeeper, 0, 0, 6_000);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 6_050);

    step_n(&mut s, 20);
    let ci = s.unit_index(crab).expect("crab lives");
    assert_eq!(s.units[ci].fx.passenger, Some(passenger));

    let pi = s.unit_index(passenger).expect("passenger lives");
    let hp_before_crab_death = s.units[pi].hp;

    // Eliminate the crab directly
    let _crab_x_at_death = s.units[ci].x;
    s.units[ci].hp = 0;
    let mut dismount_pos = 0;
    let mut dismounted = false;
    for _ in 0..10 {
        for ev in s.step() {
            if let Event::Dismounted { unit } = ev {
                if unit == passenger {
                    dismounted = true;
                    dismount_pos = s.units[s.unit_index(passenger).unwrap()].x;
                }
            }
        }
        if dismounted {
            break;
        }
    }

    assert!(dismounted, "Dismounted event emitted on crab death");
    let pi = s.unit_index(passenger).expect("passenger still lives");
    assert_eq!(
        s.units[pi].hp, hp_before_crab_death,
        "passenger HP must remain intact when crab dies"
    );
    assert_eq!(
        s.units[pi].fx.carrier, None,
        "carrier must be cleared after dismount"
    );
    assert_eq!(
        s.units[pi].x, dismount_pos,
        "passenger x must match dismount position"
    );
    assert!(
        s.units[pi].fx.board_lock >= s.tick + 50,
        "3-second re-boarding lock must be applied"
    );
}

#[test]
fn unit_view_packs_fx_carried_bit() {
    let mut s = state(46);
    let passenger = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 5_000);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 5_050);

    step_n(&mut s, 20);
    let view = s.view_for(Seat::A, &[]);

    let crab_v = view.units.iter().find(|u| u.id == crab).expect("crab view");
    let pass_v = view
        .units
        .iter()
        .find(|u| u.id == passenger)
        .expect("passenger view");

    assert!(!crab_v.is_carried(), "crab is not carried");
    assert_eq!(crab_v.fx & FX_CARRIED, 0);

    assert!(pass_v.is_carried(), "passenger must have is_carried() true");
    assert_ne!(
        pass_v.fx & FX_CARRIED,
        0,
        "FX_CARRIED bit must be set in UnitView.fx"
    );
    assert_eq!(pass_v.fx & FX_SUMMON, 0, "not a summon");
}

#[test]
fn cargo_crab_determinism_and_symmetry() {
    for seed in [101u64, 202, 303] {
        let mut s1 = state(seed);
        let mut s2 = state(seed);

        // Seat A Crab + Archer vs Seat B Crab + Archer
        s1.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 4_000);
        s1.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 4_100);
        s1.lab_spawn_at(Seat::B, UnitKind::CargoCrab, 0, 0, 16_000);
        s1.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 15_900);

        s2.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 4_000);
        s2.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 4_100);
        s2.lab_spawn_at(Seat::B, UnitKind::CargoCrab, 0, 0, 16_000);
        s2.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 15_900);

        for _ in 0..(10 * TICK_HZ) {
            let ev1 = s1.step();
            let ev2 = s2.step();
            assert_eq!(ev1, ev2, "event logs must match identically on same seed");
            assert_eq!(s1.units.len(), s2.units.len());
            for (u1, u2) in s1.units.iter().zip(s2.units.iter()) {
                assert_eq!(u1.hp, u2.hp);
                assert_eq!(u1.x, u2.x);
                assert_eq!(u1.fx.carrier, u2.fx.carrier);
                assert_eq!(u1.fx.passenger, u2.fx.passenger);
            }
        }
    }
}

/// A crab of the given form that has just boarded an Archer on an empty
/// lane. The Archer's speed 50 is above the crab's 25, so the pair moves at
/// 50 and every percentage below is exact: 50 × (10 000 + bp) / 10 000.
fn boarded_pair(seed: u64, tier: u8, branch: u8) -> (MatchState, UnitId) {
    let mut s = state(seed);
    let archer = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 1_000);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, tier, branch, 1_050);
    s.step(); // boarding runs before movement
    let ci = s.unit_index(crab).expect("crab lives");
    assert_eq!(s.units[ci].fx.passenger, Some(archer), "crab boarded");
    (s, crab)
}

/// How far the crab walks in the next step.
fn crab_dx(s: &mut MatchState, crab: UnitId) -> i32 {
    let before = s.units[s.unit_index(crab).expect("crab lives")].x;
    s.step();
    s.units[s.unit_index(crab).expect("crab lives")].x - before
}

#[test]
fn carry_bonus_is_the_larger_of_base_and_branch_for_every_form() {
    // BR-X12 (D-60, Proposed): the +20 % base and a branch bonus never add, so
    // base, A1, A2, B1 and B2 all move at +20 % (50 -> 60). The old sum made
    // B1 and B2 +40 % (70). B2's Overdrive surge has lapsed after 80 ticks.
    for (tier, branch, form) in [
        (0, 0, "base"),
        (1, 0, "A1"),
        (2, 0, "A2"),
        (1, 1, "B1"),
        (2, 1, "B2"),
    ] {
        let (mut s, crab) = boarded_pair(60, tier, branch);
        step_n(&mut s, 80);
        assert_eq!(crab_dx(&mut s, crab), 60, "{form}: max(+20 %, branch)");
    }
}

#[test]
fn overdrive_surge_replaces_the_base_bonus_for_four_seconds() {
    // BR-X12 (D-60, Proposed), ROSTER-EXPANSION §C.15: B2 Overdrive is +40 %
    // for 4 s from boarding, then the steady +20 %. The surge is
    // max(+20 %, +40 %) = +40 % (50 -> 70); the old sum was +60 %, which the
    // +50 % cap clipped to 75.
    let (mut s, crab) = boarded_pair(61, 2, 1);
    let ci = s.unit_index(crab).expect("crab lives");
    assert!(
        s.tick < s.units[ci].fx.surge_until,
        "surge armed at boarding"
    );
    assert_eq!(crab_dx(&mut s, crab), 70, "surge: 50 x 1.40");
    step_n(&mut s, 80);
    assert_eq!(crab_dx(&mut s, crab), 60, "steady: 50 x 1.20");
}

#[test]
fn carry_bonus_shares_the_plus_50_percent_move_cap() {
    // BR-X03: haste, march and carry bonuses add up to at most +50 %.
    // Forced March (Augment 0, +40 %) plus the base carry +20 % is +60 %, and
    // plus a surging B2's +40 % is +80 %: both clip to +50 % (50 -> 75; the
    // uncapped sums would be 80 and 90).
    for (tier, branch, form) in [(0, 0, "base crab"), (2, 1, "B2 in surge")] {
        let (mut s, crab) = boarded_pair(62, tier, branch);
        s.players[0].augments.add(0);
        let ci = s.unit_index(crab).expect("crab lives");
        s.units[ci].haste_until = s.tick + 1_000;
        assert_eq!(crab_dx(&mut s, crab), 75, "{form} + Forced March");
    }
}

#[test]
fn slowed_passenger_only_drags_the_pair_down_to_the_crab_own_speed() {
    // D-60, as coded since D-56: each unit's speed is read after its own
    // Slow and the pair takes the larger (a crab Slow, Root or fish is not
    // part of this test). Slowed Archer 50 -> 40 still beats the crab's 25
    // (x1.20 = 48); at 80 % it is 10, so the crab's own 25 wins (x1.20 = 30).
    let (mut s, crab) = boarded_pair(63, 0, 0);
    let ci = s.unit_index(crab).expect("crab lives");
    let archer = s.units[ci].fx.passenger.expect("boarded");
    assert_eq!(crab_dx(&mut s, crab), 60, "no Slow: 50 x 1.20");
    for (bp, expected) in [(2_000, 48), (8_000, 30)] {
        let pi = s.unit_index(archer).expect("passenger lives");
        s.units[pi].status = Status::Slowed {
            bp,
            ticks_left: 1_000,
        };
        assert_eq!(crab_dx(&mut s, crab), expected, "passenger slowed {bp} bp");
    }
}
