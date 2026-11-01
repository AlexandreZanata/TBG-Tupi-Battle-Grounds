//! Print deterministic match traces for explicitly authorized game-rule updates.
use tbg_core::{
    bot::Bot,
    headless::{self, Moment},
    MatchState,
};
fn main() {
    for seed in [1, 7, 42, 1000, 65535] {
        let mut state = MatchState::new(seed);
        let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
        let mut trace = 0xcbf2_9ce4_8422_2325_u64;
        headless::play(
            &mut state,
            |seat, view| bots[seat.index()].decide(view),
            |moment| {
                if let Moment::Stepped { state } = moment {
                    trace = (trace ^ state.hash()).wrapping_mul(0x0100_0000_01b3);
                }
            },
        );
        println!(
            "{seed} {trace:#018x} {:#018x} {} {:?}",
            state.hash(),
            state.tick,
            state.outcome
        );
    }
}
