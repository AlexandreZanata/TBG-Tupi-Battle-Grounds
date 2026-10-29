//! Golden determinism guard (ARCHITECTURE §4, BR-M01–BR-M05). Three
//! bot-vs-bot matches with fixed seeds must end with exactly these state
//! hashes, outcomes, and ticks (the simulation is integer-only, so the
//! goldens must not depend on the platform). Any change to combat, targeting, synergies,
//! evolutions, the shop, or the bot moves a hash: if that change is
//! intended, record it in GAME-DESIGN (**Proposed**) and update the goldens
//! in the same commit; if not, it is a bug.

use tbg_core::bot::Bot;
use tbg_core::{headless, EndReason, MatchState, Phase, Seat};

fn bot_match(seed: u64) -> MatchState {
    let mut s = MatchState::new(seed);
    let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
    headless::play(&mut s, |seat, view| bots[seat.index()].decide(view), |_| {});
    s
}

#[test]
fn bot_matches_are_frozen() {
    // Owner revision 2026-10-29: uncapped Energy, Interest, the Augment
    // pause, and a Shop that changes only on purchase (BR-S04, no timed
    // rotation). The Shop change alters what the bots can buy, so outcomes
    // and end combat ticks moved too.
    use EndReason::StrongholdDestroyed;
    let goldens = [
        (
            1,
            0x7242_2aae_5147_d2e7_u64,
            3_664,
            Seat::B,
            StrongholdDestroyed,
        ),
        (
            7,
            0x7d5a_9954_45c7_a322,
            2_146,
            Seat::A,
            StrongholdDestroyed,
        ),
        (
            42,
            0xf3a6_1b6c_8581_3910,
            2_342,
            Seat::B,
            StrongholdDestroyed,
        ),
    ];
    for (seed, hash, tick, winner, reason) in goldens {
        let s = bot_match(seed);
        let o = s.outcome.expect("bot match must end");
        assert_eq!(
            (o.winner, o.reason, s.tick),
            (Some(winner), reason, tick),
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
