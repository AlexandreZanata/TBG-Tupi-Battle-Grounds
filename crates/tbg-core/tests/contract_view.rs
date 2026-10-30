//! P30-A H8 contract: the max HP a client derives from the view alone
//! (`kind`, `tier`, `branch`, Synergy level, Arcanas) equals the engine's
//! `stats_of` for every live unit, on 100 seeded matches with Arcanas.
//! This is evidence that `UnitView.max_hp` is redundant, not a wire change.

use tbg_core::arcana::ArcanaPreset;
use tbg_core::augment;
use tbg_core::bot::Bot;
use tbg_core::content;
use tbg_core::headless::{self, Moment};
use tbg_core::ruleset::Ruleset;
use tbg_core::MatchState;

fn preset(seed: u64) -> ArcanaPreset {
    const RANKS: [[u8; 6]; 4] = [
        [2, 0, 2, 0, 2, 0],
        [2, 2, 2, 0, 0, 0],
        [0, 2, 0, 2, 2, 0],
        [1, 1, 1, 1, 1, 1],
    ];
    // Six-node ranks from before the fifteen-node tree (D-42), migrated
    // as saved presets are, so the seeds keep their variety.
    ArcanaPreset::migrate_legacy(RANKS[seed as usize % RANKS.len()])
}

#[test]
fn derived_max_hp_matches_every_live_unit_on_100_seeds() {
    let rules = Ruleset::canon();
    let mut samples = 0u64;
    for seed in 1..=100u64 {
        let arcanas = [preset(seed), preset(seed * 7 + 3)];
        let mut state = MatchState::with_arcanas(seed, arcanas);
        let mut bots = [Bot::new(seed), Bot::new(seed ^ 0x5eed)];
        let ended = headless::play(
            &mut state,
            |seat, view| bots[seat.index()].decide(view),
            |moment| {
                let Moment::Stepped { state } = moment else {
                    return;
                };
                for u in state.units.iter().filter(|u| u.hp > 0) {
                    assert_eq!(
                        state.stats_of(u).max_hp,
                        u.max_hp,
                        "seed {seed} tick {} unit {}",
                        state.tick,
                        u.id
                    );
                    let player = state.player(u.seat);
                    let level = player.synergy[content::def(u.kind).faction.index()];
                    let bonus =
                        augment::unit_bonus(&player.augments, player.synergy, u.kind, u.tier);
                    assert_eq!(
                        rules.unit_max_hp(u.kind, u.tier, u.branch, level, &player.arcanas, &bonus),
                        u.max_hp,
                        "derived seed {seed} tick {} unit {} seat {:?}",
                        state.tick,
                        u.id,
                        u.seat
                    );
                    samples += 1;
                }
            },
        );
        assert!(ended, "seed {seed} did not end");
    }
    assert!(samples > 10_000, "only {samples} live-unit samples");
}
