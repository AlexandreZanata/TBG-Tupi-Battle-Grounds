//! Wave E2 (P48-B): Puppetmaster confusion/betrayal and Smuggler crates.
//! Micro-scenarios cite §C.10 and §C.11.

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

// Puppetmaster (§C.10): the confused unit's next attack hits its own ally.
// The betraying windup already names the ally (AttackStart shows it).
#[test]
fn betrayal_hits_the_ally_not_the_enemy() {
    let mut s = state(11);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 0, 0, 5_000);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    let ally = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 7_200);
    let _victim = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 7_800);
    let mut betrayed = false;
    let mut healed = false;
    for _ in 0..400 {
        for e in s.step() {
            match e {
                Event::Betrayal { unit, ally: a } if unit == brute && a == ally => {
                    betrayed = true;
                }
                Event::Heal { .. } => {
                    healed = true;
                }
                _ => {}
            }
        }
        if betrayed {
            break;
        }
    }
    assert!(betrayed, "the confused Brute must betray its ally");
    assert!(!healed, "a betrayal is damage, never a heal");
    // Confusion is consumed; immunity starts.
    let i = s.unit_index(brute).expect("brute must live");
    assert_eq!(s.units[i].fx.confused_until, 0);
    assert!(s.units[i].fx.immune_until > s.tick);
}

// Puppetmaster (§C.10): a windup already in progress resolves normally;
// only the *next* attack is redirected.
#[test]
fn windup_in_progress_resolves_normally() {
    let mut s = state(12);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 0, 0, 5_000);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    let _ally = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 7_200);
    let victim = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 7_800);
    // Wait for the Brute's windup to start, then confuse it mid-windup.
    let mut wound = false;
    for _ in 0..200 {
        for e in s.step() {
            if let Event::AttackStart {
                from: Target::Unit(a),
                ..
            } = e
            {
                if a == brute {
                    wound = true;
                }
            }
        }
        if wound {
            break;
        }
    }
    assert!(wound, "the Brute must wind up (scenario broken)");
    let i = s.unit_index(brute).expect("brute must live");
    s.units[i].fx.confused_until = s.tick + 100;
    s.units[i].fx.betray_bp = 2_500;
    // The in-flight windup still hits the enemy; the *next* one betrays.
    let mut hit_enemy = false;
    let mut betrayed = false;
    for _ in 0..200 {
        for e in s.step() {
            match e {
                Event::Hit {
                    to: Target::Unit(id),
                    ..
                } if id == victim => {
                    hit_enemy = true;
                }
                Event::Betrayal { .. } => {
                    betrayed = true;
                }
                _ => {}
            }
        }
        if hit_enemy && betrayed {
            break;
        }
    }
    assert!(hit_enemy, "the in-flight windup must resolve normally");
    assert!(betrayed, "the next attack must be a betrayal");
}

// Puppetmaster (§C.10): with no ally in range the attack is normal and the
// confusion waits (not consumed).
#[test]
fn no_ally_means_no_betrayal() {
    let mut s = state(13);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 0, 0, 5_000);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    let _far_ally = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 12_000);
    let victim = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 7_800);
    let mut hit_enemy = false;
    let mut betrayed = false;
    // Confusion lasts 100 ticks from the first cast (~tick 12): watch
    // inside that window, where a betrayal would fire if an ally existed.
    for _ in 0..90 {
        for e in s.step() {
            match e {
                Event::Hit {
                    to: Target::Unit(id),
                    ..
                } if id == victim => {
                    hit_enemy = true;
                }
                Event::Betrayal { .. } => {
                    betrayed = true;
                }
                _ => {}
            }
        }
    }
    assert!(hit_enemy, "attacks land normally (scenario broken)");
    assert!(!betrayed, "no ally in range means no betrayal");
    let i = s.unit_index(brute).expect("brute must live");
    assert!(
        s.units[i].fx.confused_until > s.tick,
        "the confusion waits for an ally"
    );
}

// Puppetmaster (§C.10): A1 adds +25 % to the betrayal (75 → 93).
#[test]
fn betrayal_uses_the_caster_bonus() {
    let mut s = state(14);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 1, 0, 5_000);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    let ally = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 7_200);
    let _victim = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 7_800);
    let mut dealt = 0;
    let mut last_hit = 0;
    for _ in 0..400 {
        for e in s.step() {
            match e {
                Event::Hit {
                    to: Target::Unit(id),
                    amount,
                } if id == ally => {
                    last_hit = amount;
                }
                // The damage lands in the same tick as the betrayal, right
                // before it: take the hit that the betrayal reports.
                Event::Betrayal { .. } => {
                    dealt = last_hit;
                }
                _ => {}
            }
        }
        if dealt > 0 {
            break;
        }
    }
    assert_eq!(dealt, 93, "A1 betrayal deals 75 x 1.25");
    let _ = brute;
}

// Puppetmaster (§C.10): A2 adds +50 % and slows the betrayer.
#[test]
fn betrayal_a2_stumbles_the_betrayer() {
    let mut s = state(15);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 2, 0, 5_000);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    let ally = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 7_200);
    let _victim = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 7_800);
    let mut dealt = 0;
    let mut last_hit = 0;
    for _ in 0..400 {
        for e in s.step() {
            match e {
                Event::Hit {
                    to: Target::Unit(id),
                    amount,
                } if id == ally => {
                    last_hit = amount;
                }
                // The damage lands in the same tick as the betrayal, right
                // before it: take the hit that the betrayal reports.
                Event::Betrayal { .. } => {
                    dealt = last_hit;
                }
                _ => {}
            }
        }
        if dealt > 0 {
            break;
        }
    }
    assert_eq!(dealt, 112, "A2 betrayal deals 75 x 1.5 = 112 (truncated)");
    let i = s.unit_index(brute).expect("betrayer must live");
    assert!(
        matches!(s.units[i].status, tbg_core::Status::Slowed { .. }),
        "A2 stumbles the betrayer after the hit"
    );
}

// Puppetmaster (§C.10): betrayal never picks a carried unit, and a
// windup already aimed at it hits the crab instead. Everybody stands:
// the Brute and the victim Archer are in range, the crab is rooted after
// boarding so nobody walks out of betrayal range.
#[test]
fn betrayal_never_targets_the_carried() {
    let mut s = state(16);
    // Spawn the carried ally FIRST so it wins every tie-break by id.
    let carried = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 6_500);
    let crab = s.lab_spawn_at(Seat::B, UnitKind::CargoCrab, 0, 0, 6_550);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 0, 0, 4_000);
    let _victim = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 7_700);
    step_n(&mut s, 10);
    // The archer boards (nearest eligible ally).
    let i = s.unit_index(crab).expect("crab must live");
    assert_eq!(s.units[i].fx.passenger, Some(carried));
    // Root the crab where it stands: it would otherwise walk out of
    // betrayal range before the confused Brute acts again.
    s.units[i].status = tbg_core::Status::Rooted {
        ticks_left: u16::MAX,
    };
    // The confused Brute betrays the crab, never the passenger.
    let mut betrayed_crab = false;
    let mut betrayed_carried = false;
    for _ in 0..400 {
        for e in s.step() {
            if let Event::Betrayal { unit, ally } = e {
                if unit == brute && ally == crab {
                    betrayed_crab = true;
                }
                if ally == carried {
                    betrayed_carried = true;
                }
            }
        }
        if betrayed_crab {
            break;
        }
    }
    assert!(betrayed_crab, "betrayal must pick the crab");
    assert!(!betrayed_carried, "betrayal must never pick a carried unit");
}

// Puppetmaster (§C.10): a confused Farwatch betrays at its own range.
#[test]
fn confused_farwatch_betrays_at_range() {
    let mut s = state(17);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 0, 0, 3_500);
    let far = s.lab_spawn_at(Seat::B, UnitKind::Farwatch, 0, 0, 7_000);
    let ally = s.lab_spawn_at(Seat::B, UnitKind::Sawbot, 0, 0, 7_500);
    let _victim = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 7_800);
    let i = s.unit_index(ally).expect("ally must exist");
    s.units[i].status = tbg_core::Status::Rooted {
        ticks_left: u16::MAX,
    };
    let mut betrayed = false;
    for _ in 0..400 {
        for e in s.step() {
            if let Event::Betrayal { unit, .. } = e {
                if unit == far {
                    betrayed = true;
                }
            }
        }
        if betrayed {
            break;
        }
    }
    assert!(
        betrayed,
        "a confused Farwatch must betray in its 12-tile range"
    );
}

/// Ids of the living crates.
fn crates(s: &MatchState) -> Vec<tbg_core::UnitId> {
    s.units
        .iter()
        .filter(|u| u.hp > 0 && u.kind == UnitKind::Crate)
        .map(|u| u.id)
        .collect()
}

// Smuggler (§C.11): the first crate drops after a full 20 s interval and
// opens 2 s later; its cargo is a permanent summon of the Smuggler's line.
#[test]
fn crate_drops_after_a_full_interval_and_opens() {
    let mut s = state(21);
    let smuggler = s.lab_spawn_at(Seat::A, UnitKind::Smuggler, 0, 0, 1_000);
    step_n(&mut s, 395);
    assert!(crates(&s).is_empty(), "no crate before a full interval");
    step_n(&mut s, 10);
    let dropped = crates(&s);
    assert_eq!(dropped.len(), 1, "one crate after 20 s");
    let mut opened = None;
    for _ in 0..50 {
        for e in s.step() {
            if let Event::CrateOpened { unit, opened: o } = e {
                assert_eq!(unit, dropped[0]);
                opened = Some(o);
            }
        }
        if opened.is_some() {
            break;
        }
    }
    let opened = opened.expect("the crate opens about 2 s after the drop");
    assert!(crates(&s).is_empty(), "an opened crate is spent");
    if let Some(cargo) = opened {
        let i = s.unit_index(cargo).expect("cargo must live");
        assert_eq!(s.units[i].summoner, Some(smuggler));
        assert_eq!(s.units[i].expires_at, None, "cargo is permanent");
        assert!(!matches!(
            s.units[i].kind,
            UnitKind::Smuggler | UnitKind::Nester
        ));
    }
}

// Smuggler (§C.11): a destroyed crate releases nothing.
#[test]
fn destroyed_crate_releases_nothing() {
    let mut s = state(22);
    let _smuggler = s.lab_spawn_at(Seat::A, UnitKind::Smuggler, 0, 0, 1_000);
    step_n(&mut s, 405);
    let krate = crates(&s)[0];
    let i = s.unit_index(krate).expect("crate must exist");
    s.units[i].hp = 0;
    let ev = step_n(&mut s, 80);
    assert!(
        !ev.iter().any(|e| matches!(e, Event::CrateOpened { .. })),
        "a destroyed crate never opens"
    );
}

// Smuggler (§C.11): crates already placed still open if the Smuggler dies.
#[test]
fn crate_opens_after_the_smuggler_dies() {
    let mut s = state(23);
    let smuggler = s.lab_spawn_at(Seat::A, UnitKind::Smuggler, 0, 0, 1_000);
    step_n(&mut s, 405);
    let krate = crates(&s)[0];
    let i = s.unit_index(smuggler).expect("smuggler must exist");
    s.units[i].hp = 0;
    let ev = step_n(&mut s, 80);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::CrateOpened { unit, .. } if *unit == krate)),
        "the orphaned crate still opens"
    );
}

// Smuggler (§C.11, BR-S05): a replay of a match with crates gives the
// same hash and the same events — the cargo stream is seeded.
#[test]
fn replay_with_crates_gives_the_same_hash() {
    let run = || {
        let mut s = state(24);
        s.lab_spawn_at(Seat::A, UnitKind::Smuggler, 2, 1, 1_500);
        s.lab_spawn_at(Seat::A, UnitKind::Shieldbearer, 0, 0, 3_000);
        s.lab_spawn_at(Seat::B, UnitKind::Smuggler, 2, 0, 14_000);
        s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 12_000);
        let ev = step_n(&mut s, 2_400);
        (s.hash(), ev)
    };
    let (h1, e1) = run();
    let (h2, e2) = run();
    let opened = e1
        .iter()
        .filter(|e| matches!(e, Event::CrateOpened { .. }))
        .count();
    assert!(opened >= 4, "crates must open in the replay ({opened})");
    assert_eq!(h1, h2);
    assert_eq!(e1, e2);
}

// Smuggler B2 (§C.11): Double Shipment keeps B1's 18 s deliveries — the
// "+48 % per minute" model needs both — and every 3rd crate draws twice.
#[test]
fn double_shipment_keeps_faster_deliveries() {
    let mut s = state(25);
    let smuggler = s.lab_spawn_at(Seat::A, UnitKind::Smuggler, 2, 1, 1_000);
    // A rooted enemy far away: the Smuggler holds 5 tiles short of it
    // instead of walking into the enemy Stronghold's fire.
    let wall = s.lab_spawn_at(Seat::B, UnitKind::Rampart, 0, 0, 9_000);
    let w = s.unit_index(wall).expect("wall exists");
    s.units[w].status = tbg_core::Status::Rooted {
        ticks_left: u16::MAX,
    };
    let mut drops = Vec::new();
    let mut seen = Vec::new();
    for _ in 0..1_600 {
        s.step();
        for id in crates(&s) {
            if !seen.contains(&id) {
                seen.push(id);
                let i = s.unit_index(id).expect("crate exists");
                drops.push((s.tick, s.units[i].fx.attacks));
            }
        }
    }
    assert!(s.unit_index(smuggler).is_some(), "the Smuggler must live");
    assert!(drops.len() >= 3, "drops: {drops:?}");
    for w in drops.windows(2) {
        assert_eq!(w[1].0 - w[0].0, 360, "B2 delivers every 18 s: {drops:?}");
    }
    let draws: Vec<u8> = drops.iter().map(|d| d.1).collect();
    assert_eq!(&draws[..3], &[1, 1, 2], "every 3rd crate holds two draws");
}

// Antidote ward (§C.9, §D): a warded unit cannot be confused; the cast
// picks the next eligible enemy instead.
#[test]
fn ward_blocks_the_confusion() {
    let mut s = state(26);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 0, 0, 5_000);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7_000);
    let archer = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 7_200);
    let i = s.unit_index(brute).expect("brute exists");
    s.units[i].fx.ward_until = u32::MAX;
    let mut confused = Vec::new();
    for _ in 0..200 {
        for e in s.step() {
            if let Event::StatusApplied { unit } = e {
                if s.unit_index(unit)
                    .is_some_and(|k| s.units[k].fx.confused_until > s.tick)
                {
                    confused.push(unit);
                }
            }
        }
    }
    assert!(!confused.contains(&brute), "the ward blocks the cast");
    assert!(
        confused.contains(&archer),
        "the cast goes to the next enemy"
    );
}

// Puppetmaster (§C.10) + Cargo Crab: a betrayal already winding up on an
// ally that boards hits the crab instead, never the carried unit.
#[test]
fn betrayal_windup_on_a_boarding_ally_hits_the_crab() {
    let mut s = state(27);
    let archer = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 6_500);
    let crab = s.lab_spawn_at(Seat::B, UnitKind::CargoCrab, 0, 0, 6_550);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_600);
    let i = s.unit_index(brute).expect("brute exists");
    s.units[i].windup = Some((Target::Unit(archer), 5));
    s.units[i].fx.betray = true;
    let mut hit = None;
    for _ in 0..10 {
        for e in s.step() {
            if let Event::Betrayal { unit, ally } = e {
                if unit == brute {
                    hit = Some(ally);
                }
            }
        }
    }
    let c = s.unit_index(crab).expect("crab exists");
    assert_eq!(s.units[c].fx.passenger, Some(archer), "the archer boarded");
    assert_eq!(hit, Some(crab), "the betrayal lands on the crab");
}

// Oilpress A2 (§C.8) is immune to control, confusion included.
#[test]
fn immovable_oilpress_is_never_confused() {
    let mut s = state(28);
    let _puppet = s.lab_spawn_at(Seat::A, UnitKind::Puppetmaster, 0, 0, 5_000);
    let press = s.lab_spawn_at(Seat::B, UnitKind::Oilpress, 2, 0, 7_000);
    // A Drumguard has no attack: the press is the only eligible victim.
    let _ally = s.lab_spawn_at(Seat::B, UnitKind::Drumguard, 0, 0, 7_200);
    for _ in 0..300 {
        s.step();
        let i = s.unit_index(press).expect("press lives");
        assert!(s.units[i].fx.confused_until <= s.tick, "never confused");
    }
}
