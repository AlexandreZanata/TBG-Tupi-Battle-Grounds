//! CC-ID03: empty completion facts must preserve PR44's established match outcomes and ticks.
use tbg_core::{bot::Bot, headless, EndReason, MatchState, Seat};

#[test]
fn cc_id03_keeps_pr44_legacy_outcomes_and_ticks() {
    for (seed, tick, winner) in [(1, 4044, Seat::B), (7, 9982, Seat::B), (42, 10382, Seat::A)] {
        let mut state = MatchState::new(seed);
        let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
        headless::play(
            &mut state,
            |seat, view| bots[seat.index()].decide(view),
            |_| {},
        );
        let outcome = state.outcome.as_ref().expect("canonical match finishes");
        assert_eq!(state.tick, tick, "PR44 tick for seed {seed}");
        assert_eq!(outcome.winner, Some(winner), "PR44 winner for seed {seed}");
        assert_eq!(outcome.reason, EndReason::StrongholdDestroyed);
        println!(
            "CC-ID03 seed={seed} hash={:016x} tick={tick} winner={winner:?}",
            state.hash()
        );
    }
}
