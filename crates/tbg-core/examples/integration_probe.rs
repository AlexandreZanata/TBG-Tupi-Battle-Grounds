//! Print reviewed integration fixtures after intentional roster activation.
fn main() {
    for seed in [1, 7, 42] {
        let mut state = tbg_core::MatchState::new(seed);
        let mut bots = [
            tbg_core::bot::Bot::new(seed * 2 + 1),
            tbg_core::bot::Bot::new(seed * 2 + 2),
        ];
        tbg_core::headless::play(
            &mut state,
            |seat, view| bots[seat.index()].decide(view),
            |_| {},
        );
        println!(
            "{seed}: {:016x} tick={} {:?}",
            state.hash(),
            state.tick,
            state.outcome
        );
    }
    println!(
        "ruleset {} {:016x}",
        tbg_core::ruleset_version::RULESET_VERSION,
        tbg_core::ruleset_version::ruleset_source_hash()
    );
}
