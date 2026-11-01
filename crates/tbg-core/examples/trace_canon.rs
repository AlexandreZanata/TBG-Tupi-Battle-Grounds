//! Print reproducible canonical per-tick trace fixtures after a documented rules revision.
use tbg_core::{
    bot::Bot,
    headless::{self, Moment},
    MatchState,
};
fn main() {
    for seed in [1u64, 7, 42, 1000, 65535] {
        let mut state = MatchState::new(seed);
        let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
        let mut trace = 0xcbf2_9ce4_8422_2325u64;
        assert!(headless::play(
            &mut state,
            |seat, view| bots[seat.index()].decide(view),
            |moment| {
                if let Moment::Stepped { state } = moment {
                    trace = (trace ^ state.hash()).wrapping_mul(0x0100_0000_01b3);
                }
            }
        ));
        let outcome = state.outcome.expect("completed match has an outcome");
        println!(
            "{} {:016x} {seed} {trace:016x} {:016x} {} {:?} {:?}",
            tbg_core::ruleset_version::RULESET_VERSION,
            tbg_core::ruleset_version::ruleset_source_hash(),
            state.hash(),
            state.tick,
            outcome.winner.expect("reference match has a winner"),
            outcome.reason
        );
    }
}
