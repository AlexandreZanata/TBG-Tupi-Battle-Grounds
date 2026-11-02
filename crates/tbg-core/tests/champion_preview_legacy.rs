//! CC-ID03: empty completion facts must preserve PR44's established match outcomes and ticks.
use tbg_core::{bot::Bot, headless, EndReason, MatchState, Seat};

#[test]
fn cc_id03_keeps_pr44_legacy_outcomes_and_ticks() {
    let legacy = [(1, 4044, Seat::B), (7, 9982, Seat::B), (42, 10382, Seat::A)];
    // INT-01: TW-01 deliberately changes tower combat; retain the preceding baseline.
    let expected = match (
        tbg_core::ruleset_version::RULESET_VERSION,
        tbg_core::ruleset_version::ruleset_source_hash(),
    ) {
        // GD-04 (R14) keeps R13's canonical outcomes exactly.
        (13, 0xb66993d9e87a9cfc) | (14, 0xf354f651c056d615) | (15, 0xcc459351581b7763)
        | (16, 0xf2e253b98ccb313a) => [
            (1, 16625, Seat::B),
            (7, 14265, Seat::B),
            (42, 23274, Seat::A),
        ],
        _ => legacy,
    };
    for (seed, tick, winner) in expected {
        let mut state = MatchState::new(seed);
        let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
        headless::play(
            &mut state,
            |seat, view| bots[seat.index()].decide(view),
            |_| {},
        );
        let outcome = state.outcome.as_ref().expect("canonical match finishes");
        assert_eq!(state.tick, tick, "documented revision tick for seed {seed}");
        assert_eq!(
            outcome.winner,
            Some(winner),
            "documented revision winner for seed {seed}"
        );
        assert_eq!(outcome.reason, EndReason::StrongholdDestroyed);
        println!(
            "CC-ID03 seed={seed} hash={:016x} tick={tick} winner={winner:?}",
            state.hash()
        );
    }
}
