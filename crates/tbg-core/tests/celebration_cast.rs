//! BR-VC02/06/07: lifetime counts, final forms, bounded decode and simulation isolation.

use tbg_core::{
    celebration::{CastEntry, CastPayload, CelebrationCast, CONTENT_VERSION, MAX_BUCKETS},
    *,
};

fn running(seed: u64) -> MatchState {
    let mut state = MatchState::new(seed);
    while state.phase != Phase::Running {
        state.step();
    }
    state.players[0].energy = 100_000;
    state
}
fn buy(state: &mut MatchState, kind: UnitKind) -> UnitId {
    state.players[0].offers[0] = kind;
    state
        .apply_intention(
            Seat::A,
            state.clock + 1,
            PlayerIntention::Purchase { offer: 0 },
        )
        .unwrap();
    state.units.last().unwrap().id
}
fn finish(state: &mut MatchState) -> CelebrationCast {
    state.forfeit(Seat::B);
    state.celebration_cast().unwrap()
}

#[test]
fn celebration_includes_early_dead_duplicates_and_only_the_last_evolution() {
    let mut state = running(63);
    let first = buy(&mut state, UnitKind::Shieldbearer);
    state.units.iter_mut().find(|u| u.id == first).unwrap().hp = 0;
    state.step();
    let second = buy(&mut state, UnitKind::Shieldbearer);
    state
        .apply_intention(
            Seat::A,
            2,
            PlayerIntention::Evolve {
                unit: second,
                branch: 1,
            },
        )
        .unwrap();
    state
        .apply_intention(
            Seat::A,
            3,
            PlayerIntention::Evolve {
                unit: second,
                branch: 1,
            },
        )
        .unwrap();
    state.units.iter_mut().find(|u| u.id == second).unwrap().hp = 0;
    state.step();
    buy(&mut state, UnitKind::Shieldbearer);
    let cast = finish(&mut state);
    assert_eq!(cast.total(Seat::A), Some(3));
    assert_eq!(cast.entries.len(), 2);
    assert_eq!((cast.entries[0].tier, cast.entries[0].count), (0, 2));
    assert_eq!(
        (
            cast.entries[1].tier,
            cast.entries[1].branch,
            cast.entries[1].count
        ),
        (2, 1, 1)
    );
    assert!(cast.at(3).is_none());
}

#[test]
fn celebration_keeps_expired_beetlings_and_animated_cargo_but_not_crates() {
    let mut cargo_seen = 0;
    for seed in 0..16 {
        let mut state = running(seed);
        buy(&mut state, UnitKind::Nester);
        buy(&mut state, UnitKind::Smuggler);
        for unit in &mut state.units {
            unit.action_cd = 1;
        }
        state.step();
        let beetles = state
            .units
            .iter()
            .filter(|u| u.kind == UnitKind::Beetling)
            .count();
        assert!(beetles > 0);
        for unit in &mut state.units {
            if unit.kind == UnitKind::Crate {
                unit.action_cd = 1;
            }
            if unit.kind == UnitKind::Beetling {
                unit.expires_at = Some(state.tick);
            }
        }
        state.step();
        let cargo = state
            .units
            .iter()
            .filter(|u| {
                u.summoner.is_some() && u.kind != UnitKind::Crate && u.kind != UnitKind::Beetling
            })
            .count();
        cargo_seen += cargo;
        for u in &mut state.units {
            u.hp = 0;
        }
        state.step();
        let cast = finish(&mut state);
        assert_eq!(cast.total(Seat::A), Some((2 + beetles + cargo) as u64));
        assert!(!cast.entries.iter().any(|e| e.kind == UnitKind::Crate));
        assert_eq!(
            cast.entries
                .iter()
                .filter(|e| e.summoned)
                .map(|e| e.count)
                .sum::<u64>(),
            (beetles + cargo) as u64
        );
    }
    assert!(
        cargo_seen > 0,
        "fixture must exercise animated cargo, not just empty crates"
    );
}

#[test]
fn celebration_runtime_bookkeeping_does_not_change_hashes_events_or_outcomes() {
    let mut tracked = MatchState::new(63);
    let mut untracked: MatchState =
        postcard::from_bytes(&postcard::to_allocvec(&tracked).unwrap()).unwrap();
    for tick in 0..300 {
        if tick == 100 {
            for s in [&mut tracked, &mut untracked] {
                s.players[0].energy = 100_000;
                buy(s, UnitKind::Nester);
                buy(s, UnitKind::Shieldbearer);
            }
        }
        assert_eq!(tracked.step(), untracked.step());
        assert_eq!(tracked.hash(), untracked.hash());
    }
    tracked.forfeit(Seat::B);
    untracked.forfeit(Seat::B);
    assert_eq!(tracked.outcome, untracked.outcome);
    assert_eq!(tracked.hash(), untracked.hash());
    assert!(tracked.celebration_cast().is_some());
    assert!(
        untracked.celebration_cast().is_none(),
        "a restored snapshot has no proven history"
    );
}

#[test]
fn celebration_validates_duplicates_forms_overflow_and_expands_huge_counts_lazily() {
    let entry = CastEntry {
        kind: UnitKind::Shieldbearer,
        tier: 0,
        branch: 0,
        summoned: false,
        count: u64::MAX,
    };
    let mut cast = CelebrationCast {
        content_version: CONTENT_VERSION,
        winner: Seat::B,
        entries: vec![entry.clone()],
    };
    assert_eq!(cast.total(Seat::B), Some(u64::MAX));
    assert_eq!(cast.at(u64::MAX - 1), Some(&entry));
    assert!(cast.total(Seat::A).is_none());
    let payload = CastPayload::new(&cast).unwrap();
    assert_eq!(payload.decode(Seat::B), Some(cast.clone()));
    cast.entries.push(entry.clone());
    assert!(cast.total(Seat::B).is_none());
    cast.entries = vec![
        entry.clone(),
        CastEntry {
            kind: UnitKind::Hexer,
            count: 1,
            ..entry.clone()
        },
    ];
    assert!(cast.total(Seat::B).is_none());
    cast.entries = vec![CastEntry {
        tier: 0,
        branch: 1,
        ..entry.clone()
    }];
    assert!(cast.total(Seat::B).is_none());
    cast.entries = vec![CastEntry {
        kind: UnitKind::Crate,
        ..entry
    }];
    assert!(cast.total(Seat::B).is_none());
}

#[test]
fn celebration_decoder_bounds_buckets_before_allocating_from_a_length_hint() {
    let entry = CastEntry {
        kind: UnitKind::Shieldbearer,
        tier: 0,
        branch: 0,
        summoned: false,
        count: 1,
    };
    let cast = CelebrationCast {
        content_version: CONTENT_VERSION,
        winner: Seat::A,
        entries: vec![entry; MAX_BUCKETS + 1],
    };
    let bytes = postcard::to_allocvec(&cast).unwrap();
    assert!(postcard::from_bytes::<CelebrationCast>(&bytes).is_err());
    let mut empty = running(0);
    assert_eq!(finish(&mut empty).total(Seat::A), Some(0));
}

#[test]
fn celebration_bad_cosmetic_bytes_preserve_a_well_framed_match_end() {
    use tbg_core::protocol::{decode_server, encode, ServerMsg};
    let bad: CastPayload = postcard::from_bytes(&encode(&vec![255u8; 32])).unwrap();
    let outcome = Outcome {
        winner: Some(Seat::A),
        reason: EndReason::Forfeit,
    };
    let msg = ServerMsg::MatchEnd {
        result_id: [63; 16],
        last_tick: 321,
        cast: Some(bad),
        outcome,
    };
    let ServerMsg::MatchEnd {
        outcome: received,
        cast,
        last_tick,
        ..
    } = decode_server(&encode(&msg)).unwrap()
    else {
        panic!("expected terminal result")
    };
    assert_eq!(received, outcome);
    assert_eq!(last_tick, 321);
    assert!(cast.unwrap().decode(Seat::A).is_none());
}
