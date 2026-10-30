//! Wave E1 (P48-A): Leechroot Essence and the Cargo Crab's protected carry.
//! Micro-scenarios cite §C.3 and §C.15.

#![cfg(feature = "lab")]

use tbg_core::{ArcanaPreset, Event, MatchState, Ruleset, Seat, Status, Target, UnitKind};

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

fn essence_of(s: &MatchState, id: u32) -> u16 {
    let i = s.unit_index(id).expect("unit must exist");
    s.units[i].fx.essence
}

// Leechroot (§C.3): HP removed by its hits becomes permanent stacks; no
// maximum, lost on death. The Oilpress victim outlives the sampling.
#[test]
fn leechroot_stacks_grow_and_scale_damage() {
    let mut s = state(11);
    let leech = s.lab_spawn_at(Seat::A, UnitKind::Leechroot, 0, 0, 6_000);
    let _victim = s.lab_spawn_at(Seat::B, UnitKind::Oilpress, 0, 0, 6_800);
    step_n(&mut s, 250);
    let stacks = essence_of(&s, leech);
    assert!(stacks >= 1, "hits must fill Essence (scenario broken)");
    let i = s.unit_index(leech).expect("leech must live");
    let st = s.stats_of(&s.units[i]);
    assert_eq!(
        st.damage,
        10 + i32::from(stacks),
        "each stack adds +1 damage"
    );
    // BR-X02 shared cap: attack speed never exceeds +100 %.
    assert!(
        st.atk_bp <= 10_000,
        "Essence attack speed joins the BR-X02 cap"
    );
}

// Leechroot (§C.3): nothing from Strongholds or summons.
#[test]
fn leechroot_ignores_strongholds_and_summons() {
    // Strongholds: walk up and hit one; Essence stays 0. Spawned close so
    // the walk ends before Stronghold fire lands.
    let mut s = state(21);
    let leech = s.lab_spawn_at(Seat::A, UnitKind::Leechroot, 0, 0, 14_000);
    let mut hit_sh = false;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::Hit {
                to: Target::Stronghold(_),
                ..
            } = e
            {
                hit_sh = true;
            }
        }
        if hit_sh {
            break;
        }
    }
    assert!(hit_sh, "the Leechroot must reach the Stronghold");
    assert_eq!(
        essence_of(&s, leech),
        0,
        "Stronghold HP must never fill Essence"
    );
    // Summons: only Beetlings in range; Essence stays 0 while it fights them.
    let mut s = state(22);
    let leech = s.lab_spawn_at(Seat::A, UnitKind::Leechroot, 0, 0, 6_000);
    let nester = s.lab_spawn_at(Seat::B, UnitKind::Nester, 0, 0, 9_000);
    let mut beetlings = Vec::new();
    let mut hit_summon = false;
    for _ in 0..300 {
        for e in s.step() {
            match e {
                Event::Spawn { unit, kind, .. } if kind == UnitKind::Beetling => {
                    beetlings.push(unit);
                }
                Event::Hit {
                    to: Target::Unit(id),
                    ..
                } if beetlings.contains(&id) => {
                    hit_summon = true;
                }
                _ => {}
            }
        }
        // Keep the swarm to one: no reinforcements after the first hatch.
        if beetlings.len() == 1 {
            if let Some(i) = s.unit_index(nester) {
                s.units[i].hp = 0;
            }
        }
        if hit_summon {
            break;
        }
    }
    assert!(hit_summon, "it must hit Beetlings (scenario broken)");
    assert_eq!(
        essence_of(&s, leech),
        0,
        "summon HP must never fill Essence"
    );
}

// Leechroot (§C.3): the BR-X02 attack-speed cap binds at high stacks.
#[test]
fn essence_attack_speed_caps_at_plus_100() {
    let mut s = state(23);
    let leech = s.lab_spawn_at(Seat::A, UnitKind::Leechroot, 0, 0, 6_000);
    for stacks in [60u16, 200u16] {
        let i = s.unit_index(leech).expect("leech must live");
        s.units[i].fx.essence = stacks;
        let st = s.stats_of(&s.units[s.unit_index(leech).unwrap()]);
        assert_eq!(
            st.interval, 16,
            "attack speed caps at +100 % (interval {stacks} stacks)"
        );
    }
}

// Cargo Crab (§C.15): boarding by purchase order — the nearest eligible
// ally, ties to the lowest id (spawned = bought first here).
#[test]
fn crab_boards_nearest_ally_ties_lowest_id() {
    let mut s = state(31);
    let first = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 5_000);
    let _second = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 5_200);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 5_100);
    step_n(&mut s, 20);
    let i = s.unit_index(crab).expect("crab must live");
    assert_eq!(
        s.units[i].fx.passenger,
        Some(first),
        "the nearest ally boards; ties go to the lowest id"
    );
    let j = s.unit_index(first).expect("passenger must live");
    assert_eq!(s.units[j].fx.carrier, Some(crab));
    assert_eq!(
        s.units[j].x, s.units[i].x,
        "the passenger rides at the crab's x"
    );
}

// Cargo Crab (§C.15): no targeted effect can choose a carried unit —
// enemy units and Stronghold fire hit the crab instead, even when the
// passenger would win every tie-break. Farwatch holds rather than
// shooting what it cannot target.
#[test]
fn carried_units_cannot_be_targeted() {
    let mut s = state(32);
    // Spawn order matters: the passenger has the lowest id, so without the
    // filter every tie would go to it.
    let passenger = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 10_000);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 10_050);
    let _farwatch = s.lab_spawn_at(Seat::B, UnitKind::Farwatch, 0, 0, 14_000);
    step_n(&mut s, 60);
    // Boarded despite the enemy in range.
    let i = s.unit_index(crab).expect("crab must live");
    assert_eq!(s.units[i].fx.passenger, Some(passenger));
    // 200 ticks of Farwatch fire: every Hit lands on the crab, none on
    // the passenger (which shares the crab's x).
    let mut on_crab = 0;
    let mut on_passenger = 0;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::Hit {
                to: Target::Unit(id),
                ..
            } = e
            {
                if id == crab {
                    on_crab += 1;
                }
                if id == passenger {
                    on_passenger += 1;
                }
            }
        }
    }
    assert!(on_crab > 0, "the crab must draw fire (scenario broken)");
    assert_eq!(on_passenger, 0, "no targeted hit may reach a carried unit");
}

// Cargo Crab (§C.15): splash and poison reach the passenger at distance 0
// from the crab. Out of both Strongholds' range; the Bloomkeeper passenger
// survives the sampling window.
#[test]
fn area_damage_reaches_the_passenger() {
    let mut s = state(33);
    let passenger = s.lab_spawn_at(Seat::A, UnitKind::Bloomkeeper, 0, 0, 6_000);
    let _crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 6_050);
    let _siege = s.lab_spawn_at(Seat::B, UnitKind::SiegeWalker, 0, 0, 10_000);
    step_n(&mut s, 60);
    let i = s.unit_index(passenger).expect("passenger must live");
    assert!(
        s.units[i].fx.carrier.is_some(),
        "the archer must be carried (scenario broken)"
    );
    let before = s.units[i].hp;
    step_n(&mut s, 100);
    let i = s.unit_index(passenger).expect("passenger must live");
    assert!(
        s.units[i].hp < before,
        "splash on the crab must reach the passenger"
    );
}

// Cargo Crab (§C.15): an attack winding up on the boarder hits the crab.
#[test]
fn windup_redirects_to_the_crab_on_boarding() {
    let mut s = state(34);
    let future = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 6_000);
    let _enemy = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 8_000);
    // Let the enemy start winding up on the archer, then drop a crab next
    // to it mid-windup.
    let mut winding = false;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::AttackStart {
                to: Target::Unit(id),
                ..
            } = e
            {
                if id == future {
                    winding = true;
                }
            }
        }
        if winding {
            break;
        }
    }
    assert!(winding, "the enemy must wind up (scenario broken)");
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 6_050);
    let mut hit_crab = false;
    let mut hit_passenger = false;
    for _ in 0..30 {
        for e in s.step() {
            if let Event::Hit {
                to: Target::Unit(id),
                ..
            } = e
            {
                if id == crab {
                    hit_crab = true;
                }
                if id == future {
                    hit_passenger = true;
                }
            }
        }
    }
    assert!(hit_crab, "the in-flight attack must hit the crab");
    assert!(!hit_passenger, "nothing may hit the unit mid-boarding");
}

// Cargo Crab (§C.15): the crab's death dismounts the passenger 0.2 tiles
// behind with a 3 s re-board lock; the passenger's death frees the crab.
#[test]
fn crab_death_dismounts_behind_with_lock() {
    let mut s = state(35);
    let passenger = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 6_000);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 6_050);
    step_n(&mut s, 20);
    let i = s.unit_index(crab).expect("crab must live");
    assert_eq!(s.units[i].fx.passenger, Some(passenger));
    let cx = s.units[i].x;
    // Kill the crab directly.
    s.units[i].hp = 0;
    let mut dismounted = false;
    for _ in 0..10 {
        for e in s.step() {
            if let Event::Dismounted { unit } = e {
                if unit == passenger {
                    dismounted = true;
                }
            }
        }
        if dismounted {
            break;
        }
    }
    assert!(dismounted, "the passenger must dismount");
    let i = s.unit_index(passenger).expect("passenger must live");
    assert_eq!(s.units[i].fx.carrier, None);
    assert_eq!(s.units[i].x, cx - 200, "0.2 tiles behind, toward its base");
    assert!(
        s.units[i].fx.board_lock > s.tick,
        "a 3 s re-board lock must hold"
    );
    // Root it in place so the lock (not distance) is what prevents
    // re-boarding over the next 50 ticks.
    s.units[i].status = tbg_core::Status::Rooted {
        ticks_left: u16::MAX,
    };
    let mut reboarded = false;
    for _ in 0..50 {
        for e in s.step() {
            if let Event::Boarded { .. } = e {
                reboarded = true;
            }
        }
    }
    assert!(!reboarded, "the lock must prevent instant re-boarding");
}

// Cargo Crab (§C.15): a dead passenger frees the crab at once.
#[test]
fn passenger_death_frees_the_crab() {
    let mut s = state(36);
    let passenger = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 10_000);
    let crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 10_050);
    step_n(&mut s, 20);
    let i = s.unit_index(crab).expect("crab must live");
    assert_eq!(s.units[i].fx.passenger, Some(passenger));
    let j = s.unit_index(passenger).expect("passenger must live");
    s.units[j].hp = 0;
    step_n(&mut s, 5);
    let i = s.unit_index(crab).expect("crab must live");
    assert_eq!(
        s.units[i].fx.passenger, None,
        "a dead passenger frees its crab"
    );
}

// Cargo Crab (§C.15): the passenger acts from the crab's x (its own range
// measured there).
#[test]
fn passenger_acts_from_the_crab() {
    let mut s = state(37);
    let passenger = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 9_000);
    let _crab = s.lab_spawn_at(Seat::A, UnitKind::CargoCrab, 0, 0, 9_050);
    // A B unit at 3 950 from the crab: far out of the passenger's own old
    // range, in range from the crab.
    let _enemy = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 13_000);
    let mut acted = false;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::AttackStart {
                from: Target::Unit(a),
                ..
            } = e
            {
                if a == passenger {
                    acted = true;
                }
            }
        }
        if acted {
            break;
        }
    }
    assert!(acted, "the carried archer must attack from the crab's x");
}
