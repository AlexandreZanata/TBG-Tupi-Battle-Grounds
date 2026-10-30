//! Lab combat counters (P42): every HP a unit loses comes from an enemy
//! unit's hit or the enemy Stronghold, and the stall measure stays within
//! the match.

#![cfg(feature = "lab")]

use tbg_core::bot::Bot;
use tbg_core::headless;
use tbg_core::{ArcanaPreset, MatchState, Ruleset, Seat, UNIT_KINDS};

#[test]
fn lab_counters_balance_on_bot_vs_bot() {
    // The ten original kinds: poison, self-burn, blood transfer and betrayal
    // move HP outside this invariant, so it holds only on that pool (D-50).
    let rules: &'static Ruleset = Box::leak(Box::new(Ruleset {
        pool: &UNIT_KINDS,
        ..Ruleset::canon().clone()
    }));
    for seed in 1..=20u64 {
        let mut state = MatchState::lab(seed, rules, [ArcanaPreset::NEUTRAL; 2]);
        let mut bots = [
            Bot::with_rules(seed.wrapping_mul(4) + 1, rules),
            Bot::with_rules(seed.wrapping_mul(4) + 2, rules),
        ];
        let ended = headless::play(
            &mut state,
            |seat, view| bots[seat.index()].decide(view),
            |_| {},
        );
        assert!(ended, "seed {seed}: match did not end");
        let end = state.tick;
        for seat in Seat::BOTH {
            let other = seat.other();
            let taken: u64 = state.lab.taken[seat.index()].iter().sum();
            let dealt: u64 = state.lab.dealt[other.index()].iter().sum();
            assert_eq!(
                taken,
                dealt + state.lab.stronghold_dealt[other.index()],
                "seed {seed} seat {seat:?}: taken != dealt + stronghold",
            );
        }
        assert!(
            state.lab.stall_at_end(end) <= end,
            "seed {seed}: stall exceeds the match",
        );
    }
}
