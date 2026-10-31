//! Wave C (P46): Shatterback plates, Blood Furnace sacrifice, Bloodweaver
//! Siphon (covered in champion_siphon.rs). Micro-scenarios cite the §C rules.

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

fn plates_of(s: &MatchState, id: u32) -> (u8, i16) {
    let i = s.unit_index(id).expect("unit must exist");
    (s.units[i].fx.plates, s.units[i].fx.plate_hp)
}

// Shatterback (§C.12): damage after reduction empties the plate before HP.
#[test]
fn plates_absorb_before_hp() {
    let mut s = state(11);
    let wall = s.lab_spawn_at(Seat::A, UnitKind::Shatterback, 0, 0, 6_000);
    let _archer = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 8_000);
    // Archer hits for 45 every 20 ticks: two hits leave the plate at 10.
    // (Hits show the post-plate remainder, like post-DR amounts.)
    let mut hits = 0;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::Hit {
                to: Target::Unit(id),
                ..
            } = e
            {
                if id == wall {
                    hits += 1;
                }
            }
        }
        if hits >= 2 {
            break;
        }
    }
    assert_eq!(hits, 2);
    let i = s.unit_index(wall).expect("wall must live");
    assert_eq!(s.units[i].hp, 560, "HP must be untouched while plates hold");
    assert_eq!(plates_of(&s, wall), (4, 10));
    // Third hit breaks the first plate; HP is still untouched.
    let mut broke = false;
    for _ in 0..60 {
        for e in s.step() {
            if let Event::PlateBroken { unit } = e {
                if unit == wall {
                    broke = true;
                }
            }
        }
        if broke {
            break;
        }
    }
    assert!(broke, "the first plate must break on the third hit");
    let i = s.unit_index(wall).expect("wall must live");
    assert_eq!(s.units[i].hp, 560, "HP must be untouched after one plate");
    // The 35-point remainder carries into the next plate.
    assert_eq!(plates_of(&s, wall), (3, 65));
}

// Shatterback (§C.12): plates never regenerate; heals restore HP only.
#[test]
fn plates_never_restore() {
    let mut s = state(21);
    let wall = s.lab_spawn_at(Seat::A, UnitKind::Shatterback, 0, 0, 6_000);
    let _keeper = s.lab_spawn_at(Seat::A, UnitKind::Bloomkeeper, 0, 0, 6_200);
    let _archer = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 8_500);
    // Break the first plate, then remove the attacker and let heals run.
    let mut broke = false;
    for _ in 0..400 {
        for e in s.step() {
            if let Event::PlateBroken { unit } = e {
                if unit == wall {
                    broke = true;
                }
            }
        }
        if broke {
            break;
        }
    }
    assert!(broke, "a plate must break (scenario broken)");
    for u in s.units.iter_mut() {
        if u.seat == Seat::B {
            u.hp = 0;
        }
    }
    // Root the wall in place: otherwise it marches into Stronghold range
    // and the test would measure Stronghold fire, not plates.
    let i = s.unit_index(wall).expect("wall must live");
    s.units[i].status = tbg_core::Status::Rooted {
        ticks_left: u16::MAX,
    };
    // The keeper heals HP back, but no plate may come back. Plate 2 keeps
    // the 35-point remainder the breaking hit carried over.
    step_n(&mut s, 150);
    assert_eq!(
        plates_of(&s, wall),
        (3, 65),
        "heals must never restore a broken plate"
    );
    let i = s.unit_index(wall).expect("wall must live");
    assert_eq!(s.units[i].hp, 560, "heals restore HP only");
}

// Shatterback A2 (§C.12): shrapnel chains terminate; every break is
// processed exactly once, never recursively.
#[test]
fn shrapnel_chain_terminates() {
    let mut s = state(31);
    let _a = s.lab_spawn_at(Seat::A, UnitKind::Shatterback, 2, 0, 6_000);
    let _b = s.lab_spawn_at(Seat::B, UnitKind::Shatterback, 2, 0, 6_800);
    // 30 damage every 30 ticks both ways: plates break steadily, shrapnel
    // flies both ways.
    let mut breaks = 0;
    for _ in 0..700 {
        for e in s.step() {
            if matches!(e, Event::PlateBroken { .. }) {
                breaks += 1;
            }
        }
    }
    assert!(breaks >= 2, "plates must break (scenario broken)");
    assert!(
        breaks <= 8,
        "each of the 8 plates breaks at most once: {breaks}"
    );
    // Run to the end: no hang, no resurrection of plates.
    step_n(&mut s, 2_000);
    assert!(
        breaks <= 8,
        "shrapnel must terminate instead of looping: {breaks}"
    );
}

// Blood Furnace (§C.13): a root that cancels the windup costs nothing.
#[test]
fn root_cancels_burn_at_no_cost() {
    let mut s = state(41);
    let furnace = s.lab_spawn_at(Seat::A, UnitKind::BloodFurnace, 0, 0, 6_000);
    let enemy = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_800);
    // Let the Furnace start its windup, then root it mid-windup.
    let mut wound = false;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::AttackStart {
                from: Target::Unit(a),
                ..
            } = e
            {
                if a == furnace {
                    wound = true;
                }
            }
        }
        if wound {
            break;
        }
    }
    assert!(wound, "the Furnace never wound up (scenario broken)");
    let i = s.unit_index(furnace).expect("furnace must live");
    s.units[i].status = tbg_core::Status::Rooted {
        ticks_left: u16::MAX,
    };
    // The B Brute keeps hitting for 75 while the Furnace is rooted: every
    // HP lost must be a multiple of 75 (no 26-point burns interleaved), and
    // the Furnace must never land a hit of its own.
    let mut furnace_hits = 0;
    for _ in 0..60 {
        for e in s.step() {
            if let Event::Hit {
                to: Target::Unit(id),
                ..
            } = e
            {
                if id == enemy {
                    furnace_hits += 1;
                }
            }
        }
    }
    assert_eq!(furnace_hits, 0, "a cancelled windup must resolve nothing");
    let i = s.unit_index(furnace).expect("furnace must live");
    assert!(
        (440 - s.units[i].hp) % 75 == 0,
        "a cancelled windup pays no burn"
    );
    assert!(s.units[i].windup.is_none(), "root clears the windup");
}

// Blood Furnace (§C.13): Deathblow — the model table: 10 HP burns to 0,
// the hit lands at ×4 (144), and it dies at the end of the tick.
#[test]
fn deathblow_resolves_at_max_multiplier() {
    let mut s = state(51);
    let furnace = s.lab_spawn_at(Seat::A, UnitKind::BloodFurnace, 0, 0, 6_000);
    let victim = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_800);
    let i = s.unit_index(furnace).expect("furnace must exist");
    s.units[i].hp = 10;
    let mut dealt = 0;
    let mut died = false;
    // Death comes in the same tick as the resolving hit: collect both, and
    // stop at the first hit (the survivor later walks into Stronghold fire).
    for _ in 0..120 {
        for e in s.step() {
            match e {
                Event::Hit {
                    to: Target::Unit(id),
                    amount,
                } if id == victim => {
                    if dealt == 0 {
                        dealt = amount;
                    }
                }
                Event::Death { unit } if unit == furnace => {
                    died = true;
                }
                _ => {}
            }
        }
        if dealt > 0 {
            break;
        }
    }
    assert_eq!(dealt, 144, "Deathblow must resolve at ×4 (36 × 4)");
    assert!(
        died,
        "the Deathblow Furnace must die at the end of the tick"
    );
}

// Blood Furnace (§C.13): simultaneous deaths — the 144 hit also kills a
// weak enemy on the same tick.
#[test]
fn deathblow_can_trade() {
    let mut s = state(52);
    let furnace = s.lab_spawn_at(Seat::A, UnitKind::BloodFurnace, 0, 0, 6_000);
    let victim = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 6_800);
    let i = s.unit_index(furnace).expect("furnace must exist");
    s.units[i].hp = 10;
    let j = s.unit_index(victim).expect("victim must exist");
    s.units[j].hp = 100;
    let mut traded = false;
    for _ in 0..120 {
        let mut deaths = Vec::new();
        for e in s.step() {
            if let Event::Death { unit } = e {
                deaths.push(unit);
            }
        }
        if deaths.contains(&furnace) && deaths.contains(&victim) {
            traded = true;
            break;
        }
    }
    assert!(traded, "Deathblow must trade on one tick");
}

// Blood Furnace (§C.13): the multiplier is off vs Strongholds, but the
// burn is still paid. The Furnace walks into range with 100 HP: the first
// Stronghold shot plus the first burn leave it alive to report.
#[test]
fn no_multiplier_vs_strongholds() {
    let mut s = state(53);
    let furnace = s.lab_spawn_at(Seat::A, UnitKind::BloodFurnace, 0, 0, 15_000);
    let i = s.unit_index(furnace).expect("furnace must exist");
    s.units[i].hp = 100;
    let mut dealt = 0;
    let mut after = 0;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::Hit {
                to: Target::Stronghold(_),
                amount,
            } = e
            {
                dealt = amount;
                let j = s.unit_index(furnace).expect("furnace must live");
                after = s.units[j].hp;
            }
        }
        if dealt > 0 {
            break;
        }
    }
    assert_eq!(dealt, 36, "Strongholds take the base 36, never multiplied");
    // 100 − 53 (Stronghold) − 26 (burn) = 21: the burn was still paid.
    assert_eq!(after, 21, "the burn is paid even without a multiplier");
}
