//! Cargo Crab contract (D-56, §C.15):
//! - Boarding picks the nearest eligible ally within 0.5 tile, tiebreak by lowest ID.
//! - The pair moves at the speed of the faster of the two + 20% carry bonus.
//! - Single-target attacks target and damage the crab first; passenger is untargetable.
//! - Area/splash damage hits both the crab and the passenger.
//! - Crab death dismounts the passenger behind it with intact HP and 3s re-board lock.
//! - `UnitView.fx` packs `FX_CARRIED` (`is_carried()`) for the passenger.
//! - Symmetrical and deterministic across seeds and seats.

#![cfg(feature = "lab")]

use tbg_core::content::TICK_HZ;
use tbg_core::protocol::PROTOCOL_VERSION;
use tbg_core::state::Target;
use tbg_core::view::{FX_CARRIED, FX_SUMMON};
use tbg_core::{ArcanaPreset, Event, MatchState, Ruleset, Seat, UnitKind};

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
fn protocol_version_is_16_for_cargo_crab_carried_bit() {
    assert_eq!(
        PROTOCOL_VERSION, 16,
        "D-56: PROTOCOL_VERSION bumped to 16 for FX_CARRIED bit in UnitView.fx"
    );
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
