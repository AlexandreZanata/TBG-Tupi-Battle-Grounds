//! P40 conditional Arcana effects in the real simulation: lab-spawned
//! units on a quiet arena, fixed seeds, no Shop or Augment randomness.

use tbg_core::{
    ArcanaPreset, Event, MatchState, Phase, Ruleset, Seat, Status, Target, UnitId, UnitKind,
};

const DEFAULT: ArcanaPreset = ArcanaPreset::DEFAULT;
const WAR_FULL: ArcanaPreset = ArcanaPreset {
    ranks: [2, 2, 2, 2, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0],
};
const PRECISION_FULL: ArcanaPreset = ArcanaPreset {
    ranks: [2, 2, 2, 2, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
};
const VITALITY_FULL: ArcanaPreset = ArcanaPreset {
    ranks: [2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 1, 1, 1],
};

fn arena(seed: u64, a: ArcanaPreset, b: ArcanaPreset) -> MatchState {
    let mut rules = Ruleset::canon().clone();
    rules.stronghold_damage = 0;
    rules.stronghold_range = 0;
    rules.regen_running = 0;
    rules.interest.max_bands = 0;
    rules.augment.round_ticks = 0;
    rules.synergy_thresholds = [99, 99];
    let rules: &'static Ruleset = Box::leak(Box::new(rules));
    let mut s = MatchState::lab(seed, rules, [a, b]);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

fn spawn(s: &mut MatchState, seat: Seat, kind: UnitKind, x: i32) -> UnitId {
    s.lab_spawn_at(seat, kind, 0, 0, x)
}

fn set_status(s: &mut MatchState, id: UnitId, status: Status) {
    let i = s.unit_index(id).expect("unit");
    s.units[i].status = status;
}

fn set_hp(s: &mut MatchState, id: UnitId, hp: i32) {
    let i = s.unit_index(id).expect("unit");
    s.units[i].hp = hp;
}

fn first_hit(s: &mut MatchState, target: UnitId) -> (u32, i32) {
    for _ in 0..2_000 {
        let events = s.step();
        if let Some(Event::Hit {
            to: Target::Unit(_),
            amount,
        }) = events
            .iter()
            .find(|e| matches!(e, Event::Hit { to: Target::Unit(id), .. } if *id == target))
        {
            return (s.tick, *amount);
        }
    }
    panic!("no hit on {target}");
}

fn hit_ticks(s: &mut MatchState, target: UnitId, count: usize) -> Vec<u32> {
    let mut out = Vec::new();
    for _ in 0..10_000 {
        let events = s.step();
        for event in events {
            if matches!(event, Event::Hit { to: Target::Unit(id), .. } if id == target) {
                out.push(s.tick);
                if out.len() == count {
                    return out;
                }
            }
        }
    }
    panic!("only {} hits on {target}", out.len());
}

#[test]
fn shieldwall_and_combined_arms_reduce_tank_damage_and_stop_without_dps() {
    let baseline = {
        let mut s = arena(1, DEFAULT, ArcanaPreset::NEUTRAL);
        let tank = spawn(&mut s, Seat::A, UnitKind::Shieldbearer, 7_000);
        spawn(&mut s, Seat::A, UnitKind::Sawbot, 6_000);
        spawn(&mut s, Seat::B, UnitKind::BoneBrute, 8_000);
        first_hit(&mut s, tank).1
    };
    let shielded = {
        let mut s = arena(1, WAR_FULL, ArcanaPreset::NEUTRAL);
        let tank = spawn(&mut s, Seat::A, UnitKind::Shieldbearer, 7_000);
        let dps = spawn(&mut s, Seat::A, UnitKind::Sawbot, 6_000);
        spawn(&mut s, Seat::B, UnitKind::BoneBrute, 8_000);
        let first = first_hit(&mut s, tank).1;
        set_hp(&mut s, dps, 0);
        let second = first_hit(&mut s, tank).1;
        (first, second)
    };
    assert_eq!(baseline, 75);
    assert_eq!(shielded.0, 66, "Shieldwall + Combined Arms: 11% less");
    assert_eq!(shielded.1, 75, "without the DPS the condition turns off");
}

#[test]
fn battle_rush_moves_dps_faster_only_with_a_living_tank() {
    let run = |preset: ArcanaPreset, tank: bool| {
        let mut s = arena(2, preset, ArcanaPreset::NEUTRAL);
        let dps = spawn(&mut s, Seat::A, UnitKind::Sawbot, 1_000);
        if tank {
            spawn(&mut s, Seat::A, UnitKind::Shieldbearer, 1_000);
        }
        for _ in 0..100 {
            s.step();
        }
        s.unit_index(dps)
            .map(|i| s.units[i].x)
            .expect("sawbot alive")
    };
    let alone = run(DEFAULT, false);
    let protected = run(WAR_FULL, true);
    assert!(protected > alone, "{protected} vs {alone}");
}

#[test]
fn hunters_focus_adds_damage_only_against_a_controlled_target() {
    let run = |preset: ArcanaPreset, status: Status| {
        let mut s = arena(3, preset, ArcanaPreset::NEUTRAL);
        spawn(&mut s, Seat::A, UnitKind::Archer, 6_000);
        let target = spawn(&mut s, Seat::B, UnitKind::Shieldbearer, 7_000);
        set_status(&mut s, target, status);
        first_hit(&mut s, target).1
    };
    let plain = run(PRECISION_FULL, Status::None);
    let slowed = run(
        PRECISION_FULL,
        Status::Slowed {
            bp: 3_000,
            ticks_left: 200,
        },
    );
    let rooted = run(PRECISION_FULL, Status::Rooted { ticks_left: 200 });
    assert_eq!(plain, 47);
    assert_eq!(slowed, 52);
    assert_eq!(rooted, 52);
}

#[test]
fn quickdraw_and_deadeye_change_attack_timing_in_combat() {
    let run = |preset: ArcanaPreset, target_x: i32| {
        let mut s = arena(4, preset, ArcanaPreset::NEUTRAL);
        spawn(&mut s, Seat::A, UnitKind::Archer, 6_000);
        let target = spawn(&mut s, Seat::B, UnitKind::Rampart, target_x);
        hit_ticks(&mut s, target, 2)
    };
    let normal = run(DEFAULT, 7_000);
    let quick = run(PRECISION_FULL, 7_000);
    assert!(
        quick[1] - quick[0] < normal[1] - normal[0],
        "Quickdraw interval {quick:?} vs {normal:?}"
    );
    let far_normal = run(DEFAULT, 10_800);
    let far_deadeye = run(PRECISION_FULL, 10_800);
    assert!(
        far_deadeye[0] < far_normal[0],
        "Deadeye first hit {far_deadeye:?} vs {far_normal:?}"
    );
}

#[test]
fn emergency_care_boosts_low_hp_heals_and_purification_cleanses_control() {
    let run = |preset: ArcanaPreset| {
        let mut s = arena(5, preset, ArcanaPreset::NEUTRAL);
        spawn(&mut s, Seat::A, UnitKind::Bloomkeeper, 7_000);
        let target = spawn(&mut s, Seat::A, UnitKind::Shieldbearer, 7_200);
        let max = s
            .unit_index(target)
            .map(|i| s.units[i].max_hp)
            .expect("unit");
        set_hp(&mut s, target, max / 5);
        set_status(&mut s, target, Status::Rooted { ticks_left: 20 });
        for _ in 0..500 {
            let events = s.step();
            if let Some(Event::Heal { amount, .. }) = events
                .iter()
                .find(|e| matches!(e, Event::Heal { to, .. } if *to == target))
            {
                let status = s.unit_index(target).map(|i| s.units[i].status);
                return (*amount, status);
            }
        }
        panic!("no heal");
    };
    let (plain_heal, plain_status) = run(DEFAULT);
    let (care_heal, care_status) = run(VITALITY_FULL);
    assert_eq!(plain_heal, 44);
    assert_eq!(care_heal, 50, "Emergency Care adds 15% below 40% HP");
    let plain_root = match plain_status {
        Some(Status::Rooted { ticks_left }) => ticks_left,
        other => panic!("root lost: {other:?}"),
    };
    let care_root = match care_status {
        Some(Status::Rooted { ticks_left }) => ticks_left,
        other => panic!("root lost: {other:?}"),
    };
    assert!(
        care_root < plain_root,
        "Purification halves the root: {care_root} vs {plain_root}"
    );
}
