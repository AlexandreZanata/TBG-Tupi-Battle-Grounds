//! Golden determinism guard (ARCHITECTURE §4, BR-M01–BR-M05). Three
//! bot-vs-bot matches with fixed seeds must end with exactly these state
//! hashes, outcomes, and ticks (the simulation is integer-only, so the
//! goldens must not depend on the platform). Any change to combat, targeting, synergies,
//! evolutions, the shop, or the bot moves a hash: if that change is
//! intended, record it in GAME-DESIGN (**Proposed**) and update the goldens
//! in the same commit; if not, it is a bug.

use tbg_core::bot::Bot;
use tbg_core::content::MATCH_END;
use tbg_core::{EndReason, MatchState, Phase, Seat};

fn bot_match(seed: u64) -> MatchState {
    let mut s = MatchState::new(seed);
    let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
    let mut events = Vec::new();
    while s.phase != Phase::Ended && s.tick < MATCH_END + 5 {
        if s.tick.is_multiple_of(2) {
            for seat in Seat::BOTH {
                let view = s.view_for(seat, &events);
                if let Some(intent) = bots[seat.index()].decide(&view) {
                    let _ = s.apply_intention(seat, s.tick, intent);
                }
            }
            events.clear();
        }
        events.extend(s.step());
    }
    s
}

#[test]
fn bot_matches_are_frozen() {
    let goldens = [
        (1, 0x7b59_d4cc_3576_df97_u64, 2_414, Seat::A),
        (7, 0xcf7b_951e_9dd2_9935, 1_832, Seat::B),
        (42, 0x4ad6_363d_2357_8a89, 2_572, Seat::A),
    ];
    for (seed, hash, tick, winner) in goldens {
        let s = bot_match(seed);
        let o = s.outcome.expect("bot match must end");
        assert_eq!(
            (o.winner, o.reason, s.tick),
            (Some(winner), EndReason::StrongholdDestroyed, tick),
            "seed {seed}"
        );
        assert_eq!(
            s.hash(),
            hash,
            "seed {seed}: simulation changed (see module docs)"
        );
    }
}

#[test]
fn surrender_is_only_a_forfeit() {
    // BR-M06/BR-M03: surrender reuses `forfeit`; it ends the match for the
    // other seat, and a second call (or a call after the end) changes nothing.
    let mut s = MatchState::new(3);
    for _ in 0..100 {
        s.step();
    }
    s.forfeit(Seat::A);
    let o = s.outcome.expect("forfeit ends the match");
    assert_eq!((o.winner, o.reason), (Some(Seat::B), EndReason::Forfeit));
    s.forfeit(Seat::B);
    s.step();
    assert_eq!(
        s.outcome,
        Some(o),
        "a later forfeit must not flip the result"
    );
    assert_eq!(s.phase, Phase::Ended);
}
