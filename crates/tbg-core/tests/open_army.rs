//! Contract tests for the open army (D-32): no army cap (BR-U07), summons
//! (BR-U08), Guard (BR-U09), five distinct offers (BR-S01), and the
//! determinism and wire size of large battles. Each test names its rule.

use tbg_core::content::{
    def, BEETLES_PER_PLAYER, BEETLE_LIFETIME, HATCH_EVERY, OFFERS, STRONGHOLD_RANGE,
};
use tbg_core::protocol::{encode, ServerMsg};
use tbg_core::*;

fn running(seed: u64) -> MatchState {
    let mut s = MatchState::new(seed);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

/// Buy `kind` through a real `Purchase` intention with plenty of Energy.
fn buy(s: &mut MatchState, seat: Seat, kind: UnitKind) -> Result<(), Rejection> {
    s.players[seat.index()].offers[0] = kind;
    s.players[seat.index()].energy = 6_000;
    s.apply_intention(seat, 1, PlayerIntention::Purchase { offer: 0 })
}

fn living(s: &MatchState, seat: Seat, kind: UnitKind) -> usize {
    s.units
        .iter()
        .filter(|u| u.seat == seat && u.kind == kind && u.hp > 0)
        .count()
}

#[test]
fn br_u07_army_grows_past_eight_while_energy_lasts() {
    // Tests 1–3: more than eight living units; every purchase with enough
    // Energy succeeds at any army size; the only rejection is Energy.
    let mut s = running(1);
    for (n, kind) in UNIT_KINDS.iter().cycle().take(40).enumerate() {
        assert_eq!(buy(&mut s, Seat::A, *kind), Ok(()), "purchase {n}");
    }
    assert_eq!(s.army_size(Seat::A), 40);
    s.players[0].energy = def(s.players[0].offers[1]).cost - 1;
    let r = s.apply_intention(Seat::A, 2, PlayerIntention::Purchase { offer: 1 });
    assert_eq!(r, Err(Rejection::NotEnoughEnergy));
}

#[test]
fn br_u07_former_two_slot_units_have_no_capacity_condition() {
    // Test 4: Bone Brute, Siege Walker, and Rampart buy freely in numbers.
    let mut s = running(2);
    for kind in [
        UnitKind::BoneBrute,
        UnitKind::SiegeWalker,
        UnitKind::Rampart,
    ] {
        for _ in 0..12 {
            assert_eq!(buy(&mut s, Seat::A, kind), Ok(()), "{kind:?}");
        }
        assert_eq!(living(&s, Seat::A, kind), 12);
    }
}

#[test]
fn br_y01_synergy_with_large_armies_skips_summons() {
    // Test 6: counts past 255 do not overflow; summons neither count toward
    // nor receive Synergy (BR-U08).
    let mut s = running(3);
    for _ in 0..300 {
        buy(&mut s, Seat::A, UnitKind::Shieldbearer).unwrap();
    }
    s.step();
    assert_eq!(s.player(Seat::A).synergy[Faction::Humans.index()], 2);
    // Clear the horde so it cannot reach the Beetlings, then check Plants.
    s.units.clear();
    buy(&mut s, Seat::B, UnitKind::Nester).unwrap();
    buy(&mut s, Seat::B, UnitKind::Bloomkeeper).unwrap();
    for _ in 0..2 * u32::from(HATCH_EVERY) {
        s.step();
    }
    assert_eq!(living(&s, Seat::B, UnitKind::Beetling), 2);
    // Nester + Bloomkeeper = 2 Plants: level 1, Beetlings do not reach 4.
    assert_eq!(s.player(Seat::B).synergy[Faction::Plants.index()], 1);
    let beetle = s
        .units
        .iter()
        .find(|u| u.kind == UnitKind::Beetling)
        .unwrap();
    assert_eq!(
        beetle.max_hp,
        def(UnitKind::Beetling).hp,
        "no Synergy bonus"
    );
}

#[test]
fn br_u08_hatch_respects_its_own_limits() {
    // Test 7: every 4 s, at most 2 per Nester and 6 per player, lifetime
    // 10 s, summons never hatch, never evolve, and inherit the Nester's form.
    let mut s = running(4);
    buy(&mut s, Seat::A, UnitKind::Nester).unwrap();
    let nester = s.units[0].id;
    for _ in 0..HATCH_EVERY {
        s.step();
    }
    assert_eq!(
        living(&s, Seat::A, UnitKind::Beetling),
        1,
        "first hatch at 4 s"
    );
    for _ in 0..3 * HATCH_EVERY {
        s.step();
        assert!(
            living(&s, Seat::A, UnitKind::Beetling) <= 2,
            "per-Nester limit"
        );
    }
    let beetle = s
        .units
        .iter()
        .find(|u| u.kind == UnitKind::Beetling)
        .unwrap();
    assert_eq!(beetle.summoner, Some(nester));
    assert!(beetle.expires_at.unwrap() <= s.tick + BEETLE_LIFETIME);
    let id = beetle.id;
    s.players[0].energy = 6_000;
    let r = s.apply_intention(
        Seat::A,
        9,
        PlayerIntention::Evolve {
            unit: id,
            branch: 0,
        },
    );
    assert_eq!(r, Err(Rejection::NotEvolvable));

    for _ in 0..9 {
        buy(&mut s, Seat::A, UnitKind::Nester).unwrap();
    }
    for _ in 0..600 {
        s.step();
        assert!(living(&s, Seat::A, UnitKind::Beetling) <= BEETLES_PER_PLAYER);
        assert!(s
            .units
            .iter()
            .filter(|u| u.kind.is_summon())
            .all(|u| u.summoner.is_some() && u.hatch_cd == 0 && u.expires_at.is_some()));
    }
}

#[test]
fn br_u08_beetlings_take_the_nesters_form() {
    let mut s = running(5);
    buy(&mut s, Seat::A, UnitKind::Nester).unwrap();
    let id = s.units[0].id;
    for (branch, cost) in [(1, 1_800), (1, 2_400)] {
        s.players[0].energy = cost;
        s.apply_intention(Seat::A, 3, PlayerIntention::Evolve { unit: id, branch })
            .unwrap();
    }
    for _ in 0..HATCH_EVERY {
        s.step();
    }
    let beetle = s
        .units
        .iter()
        .find(|u| u.kind == UnitKind::Beetling)
        .unwrap();
    assert_eq!((beetle.tier, beetle.branch), (2, 1));
    assert_eq!(
        beetle.max_hp,
        def(UnitKind::Beetling).hp * 3 / 2,
        "Blight +50 % HP"
    );
}

/// `count` units of `kind` for `seat`, all placed at `x`.
fn place(s: &mut MatchState, seat: Seat, kind: UnitKind, count: usize, x: i32) {
    for _ in 0..count {
        buy(s, seat, kind).unwrap();
        s.units.last_mut().unwrap().x = x;
    }
}

fn attacks_on(events: &[Event], from_seat: Seat, s: &MatchState) -> Vec<Target> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::AttackStart { from, to } => Some((*from, *to)),
            _ => None,
        })
        .filter(|(from, _)| match from {
            Target::Unit(id) => s.units.iter().any(|u| u.id == *id && u.seat == from_seat),
            Target::Stronghold(seat) => *seat == from_seat,
        })
        .map(|(_, to)| to)
        .collect()
}

#[test]
fn br_u09_guard_holds_with_one_hundred_units_per_side() {
    // Test 8: 100 Archers reach both a nearer Sawbot wall and a Rampart; every
    // one of them must shoot the Rampart.
    let mut s = running(6);
    place(&mut s, Seat::A, UnitKind::Archer, 100, 4_000);
    place(&mut s, Seat::B, UnitKind::Sawbot, 99, 7_000);
    place(&mut s, Seat::B, UnitKind::Rampart, 1, 8_000);
    let rampart = s.units.last().unwrap().id;
    let events = s.step();
    let shots = attacks_on(&events, Seat::A, &s);
    assert_eq!(shots.len(), 100);
    assert!(shots.iter().all(|t| *t == Target::Unit(rampart)));
}

#[test]
fn br_u09_guard_also_binds_the_stronghold() {
    let mut s = running(7);
    let near = 16_000 - STRONGHOLD_RANGE / 2;
    place(&mut s, Seat::A, UnitKind::Sawbot, 30, near);
    place(
        &mut s,
        Seat::A,
        UnitKind::Rampart,
        1,
        16_000 - STRONGHOLD_RANGE + 100,
    );
    let rampart = s.units.last().unwrap().id;
    let events = s.step();
    let shots = attacks_on(&events, Seat::B, &s);
    assert_eq!(shots, vec![Target::Unit(rampart)]);
}

/// Both seats buy every affordable offer each tick for `ticks` ticks.
fn spam_match(seed: u64, ticks: u32) -> MatchState {
    let mut s = running(seed);
    for t in 0..ticks {
        for seat in Seat::BOTH {
            s.players[seat.index()].energy = 6_000;
            let offer = (t as usize) % OFFERS;
            let _ = s.apply_intention(seat, t, PlayerIntention::Purchase { offer: offer as u8 });
        }
        s.step();
    }
    s
}

#[test]
fn determinism_with_large_armies() {
    // Test 9: identical seeds give identical states with 100+ units a side.
    let (a, b) = (spam_match(11, 250), spam_match(11, 250));
    assert!(a.army_size(Seat::A) > 100, "{}", a.army_size(Seat::A));
    assert_eq!(a.hash(), b.hash());
    assert_ne!(a.hash(), spam_match(12, 250).hash());
}

#[test]
fn br_s01_five_distinct_offers_after_draws_and_refills() {
    let mut s = running(8);
    // Purchases (refills) and rotations, while the match is still running.
    for t in 0..300u32 {
        let o = s.player(Seat::A).offers;
        for i in 0..OFFERS {
            assert!(!o[i + 1..].contains(&o[i]), "duplicate offer {o:?}");
            assert!(!o[i].is_summon());
        }
        s.players[0].energy = 6_000;
        let offer = (t % OFFERS as u32) as u8;
        if t % 3 == 0 {
            s.apply_intention(Seat::A, t, PlayerIntention::Purchase { offer })
                .unwrap();
        }
        s.step();
    }
}

#[test]
fn large_battle_views_fit_the_wire() {
    // Test 11 (wire part): a busy tick with 100 units a side plus the
    // Beetling limit stays far below the server's 64 KiB write cap.
    let mut s = running(9);
    place(&mut s, Seat::A, UnitKind::Archer, 100, 7_000);
    place(&mut s, Seat::B, UnitKind::Archer, 95, 9_000);
    place(&mut s, Seat::B, UnitKind::Nester, 5, 9_000);
    let mut largest = 0;
    let mut previous = Vec::new();
    for _ in 0..200 {
        let events = s.step();
        let mut both = std::mem::replace(&mut previous, events.clone());
        both.extend(events);
        let bytes = encode(&ServerMsg::View(Box::new(s.view_for(Seat::A, &both)))).len();
        largest = largest.max(bytes);
    }
    assert!(living(&s, Seat::B, UnitKind::Beetling) > 0);
    assert!(largest < 32 * 1024, "largest view {largest} bytes");
}

#[test]
fn no_slot_rule_survives_in_the_code() {
    // Test 12: the removed army-cap rule must not come back by accident.
    let banned = [
        ["SLOT", "_CAP"].concat(),
        ["Slot", "Count"].concat(),
        ["slots", "_used"].concat(),
        ["SlotCap", "Reached"].concat(),
    ];
    let crates = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let mut stack = vec![std::path::PathBuf::from(crates)];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() && !path.ends_with("target") && !path.ends_with("dist") {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                for word in &banned {
                    assert!(
                        !text.contains(word.as_str()),
                        "{word} in {}",
                        path.display()
                    );
                }
            }
        }
    }
}
