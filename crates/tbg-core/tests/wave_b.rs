//! Wave B (P45): shared capped stacks, exact pulse counts, fish movement,
//! ward rules, Antidote behavior. Micro-scenarios cite the §C rules.

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

fn poison_count(s: &MatchState, id: u32) -> usize {
    let i = s.unit_index(id).expect("victim must exist");
    s.units[i].fx.poison.iter().filter(|p| p.left > 0).count()
}

fn fish_count(s: &MatchState, id: u32) -> usize {
    let i = s.unit_index(id).expect("victim must exist");
    s.units[i].fx.fish.iter().filter(|f| f.left > 0).count()
}

// Rotcap (§C.1): stacks stay after the source dies and are shared across
// sources under one cap of 6 per victim.
#[test]
fn rotcap_stacks_are_shared_and_capped() {
    let mut s = state(11);
    let _r1 = s.lab_spawn_at(Seat::A, UnitKind::Rotcap, 0, 0, 6_000);
    let _r2 = s.lab_spawn_at(Seat::A, UnitKind::Rotcap, 0, 0, 6_200);
    let victim = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    step_n(&mut s, 150);
    assert!(
        s.unit_index(victim).is_some(),
        "the victim died before the cap was measurable"
    );
    assert_eq!(
        poison_count(&s, victim),
        6,
        "two Rotcaps must share one cap of 6, not stack to 12"
    );
}

// Rotcap (§C.1): one stack pulses exactly ten times, then expires; the dead
// source's tail deals its full remaining damage.
#[test]
fn rotcap_stack_pulses_exactly_ten_times() {
    let mut s = state(21);
    let rotcap = s.lab_spawn_at(Seat::A, UnitKind::Rotcap, 0, 0, 6_000);
    let victim = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    // Let exactly the first hit land, then remove the source.
    for _ in 0..500 {
        let applied = s
            .step()
            .iter()
            .any(|e| matches!(e, Event::StatusApplied { unit } if *unit == victim));
        if applied {
            break;
        }
    }
    assert_eq!(poison_count(&s, victim), 1, "one hit must add one stack");
    let i = s.unit_index(rotcap).expect("Rotcap must exist");
    s.units[i].hp = 0;
    let mut pulses = 0;
    let mut damage = 0;
    for _ in 0..400 {
        for e in s.step() {
            if let Event::PoisonTick { unit, amount } = e {
                if unit == victim {
                    pulses += 1;
                    damage += amount;
                }
            }
        }
    }
    assert_eq!(pulses, 10, "one stack must pulse exactly ten times");
    assert_eq!(damage, 60, "ten pulses of 6 must deal 60 after the tail");
}

// Rotcap B (§C.1): every 3rd attack also stacks the 2nd-nearest enemy.
// Farwatch victims hold still and survive past the 3rd attack.
#[test]
fn rotcap_echo_stacks_the_second_nearest() {
    let mut s = state(31);
    // tier 1, branch 1: Spore Split.
    let _rotcap = s.lab_spawn_at(Seat::A, UnitKind::Rotcap, 1, 1, 6_000);
    let _near = s.lab_spawn_at(Seat::B, UnitKind::Farwatch, 0, 0, 7_000);
    let far = s.lab_spawn_at(Seat::B, UnitKind::Farwatch, 0, 0, 7_500);
    // Two attacks (ticks 6 and 36: combat runs through the countdown):
    // no echo yet.
    step_n(&mut s, 40);
    assert_eq!(poison_count(&s, far), 0, "echo fired before the 3rd attack");
    // The 3rd attack (tick 66) echoes onto the 2nd-nearest enemy.
    step_n(&mut s, 120);
    assert!(
        poison_count(&s, far) >= 1,
        "the 3rd attack must echo onto the 2nd-nearest enemy"
    );
}

// Old Finn (§C.5): fish slow movement only, never the attack interval.
#[test]
fn fish_slows_movement_not_attacks() {
    let mut s = state(41);
    let _finn = s.lab_spawn_at(Seat::A, UnitKind::OldFinn, 0, 0, 5_000);
    let diver = s.lab_spawn_at(Seat::B, UnitKind::Sawbot, 0, 0, 9_000);
    let mut attacks = Vec::new();
    let mut fished_speeds = Vec::new();
    let mut prev_x = unit_x(&s, diver);
    for t in 0..220 {
        if s.unit_index(diver).is_none() {
            break;
        }
        for e in s.step() {
            if let Event::AttackStart {
                from: Target::Unit(a),
                ..
            } = e
            {
                if a == diver {
                    attacks.push(t);
                }
            }
        }
        if s.unit_index(diver).is_none() {
            break;
        }
        let x = unit_x(&s, diver);
        if fish_count(&s, diver) > 0 && x != prev_x {
            fished_speeds.push((x - prev_x).abs());
        }
        prev_x = x;
    }
    assert!(attacks.len() >= 3, "the diver must attack several times");
    for w in attacks.windows(2) {
        // Sawbot interval is 21 ticks; fish must not lengthen it.
        assert_eq!(w[1] - w[0], 21, "fish changed the attack interval");
    }
    assert!(
        !fished_speeds.is_empty(),
        "the diver never moved while fished (scenario broken)"
    );
    assert!(
        fished_speeds.iter().all(|&dx| dx < 65),
        "fished steps are not slowed: {fished_speeds:?}"
    );
}

fn unit_x(s: &MatchState, id: u32) -> i32 {
    let i = s.unit_index(id).expect("unit must exist");
    s.units[i].x
}

// Old Finn (§C.5): the fish cap is shared across sources. The Farwatch
// victim holds still and outlives the sampling window.
#[test]
fn fish_cap_is_shared() {
    let mut s = state(51);
    let _f1 = s.lab_spawn_at(Seat::A, UnitKind::OldFinn, 0, 0, 6_000);
    let _f2 = s.lab_spawn_at(Seat::A, UnitKind::OldFinn, 0, 0, 6_200);
    let victim = s.lab_spawn_at(Seat::B, UnitKind::Farwatch, 0, 0, 7_000);
    step_n(&mut s, 200);
    assert_eq!(
        fish_count(&s, victim),
        3,
        "two Finns must share one cap of 3 fish"
    );
}

// Antidote (§C.9): with nobody afflicted it waits ready — no brew, no attack.
#[test]
fn antidote_waits_when_nothing_to_cleanse() {
    let mut s = state(61);
    let brewer = s.lab_spawn_at(Seat::A, UnitKind::AntidoteBrewer, 0, 0, 5_000);
    let _ally = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_200);
    let evs = step_n(&mut s, 400);
    assert!(
        evs.iter().all(|e| !matches!(e, Event::Cleansed { .. })),
        "brewed with nobody afflicted"
    );
    assert!(
        evs.iter().all(|e| !matches!(
            e,
            Event::AttackStart { from: Target::Unit(a), .. } if *a == brewer
        )),
        "a unit with no attack started an attack"
    );
}

// Antidote (§C.9): the ward blocks slows, poison and fish, but not damage.
#[test]
fn ward_blocks_effects_not_damage() {
    let mut s = state(71);
    let _brewer = s.lab_spawn_at(Seat::A, UnitKind::AntidoteBrewer, 2, 0, 5_000);
    let victim = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_200);
    let _hexer = s.lab_spawn_at(Seat::B, UnitKind::Hexer, 0, 0, 8_000);
    let _rotcap = s.lab_spawn_at(Seat::B, UnitKind::Rotcap, 0, 0, 8_200);
    let _finn = s.lab_spawn_at(Seat::B, UnitKind::OldFinn, 0, 0, 8_400);
    // Wait for the first brew, then watch the 70-tick ward (A2) in the same
    // pass: the ward covers [brewed_at, brewed_at + 70).
    let mut brewed_at = None;
    let mut blocked_status = true;
    let mut damage_taken = false;
    for t in 0..800 {
        for e in s.step() {
            match e {
                Event::Cleansed { unit } if unit == victim && brewed_at.is_none() => {
                    brewed_at = Some(t);
                }
                Event::StatusApplied { unit } if Some(unit) == Some(victim) => {
                    if brewed_at.is_some_and(|b| t >= b && t < b + 70) {
                        blocked_status = false;
                    }
                }
                Event::Hit {
                    to: Target::Unit(id),
                    amount,
                } if id == victim
                    && amount > 0
                    && brewed_at.is_some_and(|b| t >= b && t < b + 70) =>
                {
                    damage_taken = true;
                }
                _ => {}
            }
        }
        if brewed_at.is_some_and(|b| t >= b + 70) {
            break;
        }
    }
    assert!(
        brewed_at.is_some(),
        "the Antidote never brewed (scenario broken)"
    );
    assert!(
        blocked_status,
        "a slow, poison or fish landed inside the ward"
    );
    assert_eq!(fish_count(&s, victim), 0, "fish stacked inside the ward");
    assert_eq!(
        poison_count(&s, victim),
        0,
        "poison stacked inside the ward"
    );
    assert!(
        damage_taken,
        "no damage landed inside the ward (scenario broken)"
    );
}

// Antidote (§C.9): one brew strips up to 3 poison stacks, all fish, status.
// The brewer spawns late, so the victim already holds a full cap of 6.
#[test]
fn brew_strips_three_poison_all_fish_and_status() {
    let mut s = state(82);
    let _r1 = s.lab_spawn_at(Seat::B, UnitKind::Rotcap, 0, 0, 7_000);
    let _r2 = s.lab_spawn_at(Seat::B, UnitKind::Rotcap, 0, 0, 7_200);
    let _finn = s.lab_spawn_at(Seat::B, UnitKind::OldFinn, 0, 0, 7_400);
    let victim = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_000);
    step_n(&mut s, 250);
    assert_eq!(
        poison_count(&s, victim),
        6,
        "the victim must hold a full cap before the brew (scenario broken)"
    );
    let i = s.unit_index(victim).expect("victim must exist");
    let _brewer = s.lab_spawn_at(Seat::A, UnitKind::AntidoteBrewer, 0, 0, s.units[i].x);
    let mut brewed = false;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::Cleansed { unit } = e {
                if unit == victim {
                    brewed = true;
                }
            }
        }
        if brewed {
            break;
        }
    }
    assert!(brewed, "the Antidote never brewed (scenario broken)");
    // Freeze new applications: remove every enemy, then read the brew result.
    for u in s.units.iter_mut() {
        if u.seat == Seat::B {
            u.hp = 0;
        }
    }
    step_n(&mut s, 5);
    assert_eq!(
        poison_count(&s, victim),
        3,
        "one brew must strip exactly 3 of a full 6-stack"
    );
    assert_eq!(fish_count(&s, victim), 0, "one brew must clear all fish");
    let i = s.unit_index(victim).expect("victim must exist");
    assert_eq!(s.units[i].status, tbg_core::Status::None);
}
