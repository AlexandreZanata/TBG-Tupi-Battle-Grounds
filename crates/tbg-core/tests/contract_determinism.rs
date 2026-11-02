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
    // Owner revision 2026-10-29: uncapped Energy, Interest as a regen boost
    // (every full 5 Energy held adds 25 % of the base regen), the Augment
    // pause, and a Shop that redraws all five offers on every purchase
    // (BR-S04). The faster Energy flow and the new draws change what the
    // bots can buy, so outcomes and end combat ticks moved with the hashes.
    // P43: the zeroed `UnitFx` block joined the serialized state, so hashes
    // moved again with tick, winner and reason identical (verified).
    // P47: `march_ignore_slow` grew `UnitFx`; hashes moved once more,
    // behavior identical (verified).
    // P48-A: `surge_until` grew `UnitFx`; hashes moved once more,
    // behavior identical (verified).
    // P48-B: `betray_bp`/`betray_stumble` grew `UnitFx` and `cargo_rng`
    // joined `PlayerState`; hashes moved once more, behavior identical
    // (verified).
    // D-46 added the Stronghold kind and Fortification to the state: the
    // Classic matches, winners and end ticks are unchanged, only the hash.
    // Merge of the roster waves with D-46: both grew the serialized state;
    // hashes refrozen once, tick, winner and reason identical (verified).
    // D-50: the Shop draws all 25 kinds, so the bots buy differently; end
    // ticks moved, every match still ends by a destroyed Stronghold.
    // D-55 (ARENAS-G): `MatchState::arena` and `arena_discount_used` joined
    // the serialized state; the Forest is neutral, so hashes moved once with
    // tick, winner and reason identical (verified on all three seeds).
    // W0 (P58 §3, D-62): the bleed, weakening, queued-Slow, mark, self-reduction
    // and ability heal-cut slots joined `UnitFx`; hashes moved once, with tick,
    // winner and reason identical on all three seeds (verified).
    // W1 (P58 §13.5, D-64): the Synergy array grew from four to seven Factions;
    // hashes moved once, with tick, winner and reason identical (verified).
    // D-71: serialized pending-special/venom slots are empty on legacy kinds.
    // The consolidated three-seed comparison confirmed all ticks/winners/reasons
    // unchanged before re-freezing these hashes (D-71).
    use EndReason::StrongholdDestroyed;
    // D-68: owner champion changes in GAME-DESIGN, collected with update_probe.
    // D-69: appended private Shop state; all D-68 outcomes and ticks identical.
    // D-68/D-69 integration: protocol-20 slots + Shop state; owner range correction.
    // Owner 2026-10-31: activating all 46 conventional kinds changes draws and outcomes.
    let goldens = [
        (
            1,
            0x7890_8734_d3dc_7ec1u64,
            3_844,
            Seat::B,
            StrongholdDestroyed,
        ),
        (
            7,
            0x7ab5_0dfd_d573_5803,
            15_586,
            Seat::A,
            StrongholdDestroyed,
        ),
        (
            42,
            0xc437_7231_f72a_0c57,
            5_049,
            Seat::B,
            StrongholdDestroyed,
        ),
    ];
    // D-101: keep the revision-2 baseline above; append the documented P70/P71
    // candidate instead of applying old outcomes to a different ruleset.
    let goldens = match (
        tbg_core::ruleset_version::RULESET_VERSION,
        tbg_core::ruleset_version::ruleset_source_hash(),
    ) {
        (2, 0x3d56_1560_49e6_26c5) => goldens,
        (6, 0x0973_85fe_e9a1_0b0f) => [
            (
                1,
                0xe318_aa87_b746_f36c,
                4_044,
                Seat::B,
                StrongholdDestroyed,
            ),
            (
                7,
                0xb796_fd4f_3eb1_2d9f,
                9_982,
                Seat::B,
                StrongholdDestroyed,
            ),
            (
                42,
                0x5259_4b93_cce3_8cf5,
                10_382,
                Seat::A,
                StrongholdDestroyed,
            ),
        ],
        // CC-ID03: schema-only completion slots preserve all three PR44 outcomes/ticks.
        (7, 0xa9fa_44b5_5cde_a10b) => [
            (
                1,
                0xe553_d0e8_59e5_b19c,
                4_044,
                Seat::B,
                StrongholdDestroyed,
            ),
            (
                7,
                0x50f9_1188_6d89_5e19,
                9_982,
                Seat::B,
                StrongholdDestroyed,
            ),
            (
                42,
                0x4f6f_6f12_eae1_933f,
                10_382,
                Seat::A,
                StrongholdDestroyed,
            ),
        ],
        // OG-01: Ogre range/faction and appended eighth Synergy slot.
        (8, 0x3d07d8f3e73671ef) => [
            (1, 0xdc6ddee0ab7abe7e, 4044, Seat::B, StrongholdDestroyed),
            (7, 0x981a89b67d681f3d, 9982, Seat::B, StrongholdDestroyed),
            (42, 0x750b2a389290829d, 10382, Seat::A, StrongholdDestroyed),
        ],
        // OG-01 candidate 9 changes staged kinds only; canonical traces remain frozen.
        (9, 0x89698dcd4bcb5ddd) => [
            (1, 0xdc6ddee0ab7abe7e, 4044, Seat::B, StrongholdDestroyed),
            (7, 0x981a89b67d681f3d, 9982, Seat::B, StrongholdDestroyed),
            (42, 0x750b2a389290829d, 10382, Seat::A, StrongholdDestroyed),
        ],
        // AB-01/OG-02: ninth Synergy slot and empty rolling-cask state; staged tuning is private.
        (10, 0x523044fc973bac39) => [
            (1, 0x4b854f968429a97c, 4044, Seat::B, StrongholdDestroyed),
            (7, 0x70c562418d530b55, 9982, Seat::B, StrongholdDestroyed),
            (42, 0xcea96d41f552661d, 10382, Seat::A, StrongholdDestroyed),
        ],
        // TW-01/PK-01/GD-01: stronger towers; physical-lanes candidate stays private.
        (11, 0xa0a1d508c59e5841) => [
            (1, 0xae47594e85732bbc, 16625, Seat::B, StrongholdDestroyed),
            (7, 0x34994c75252c5c39, 14265, Seat::B, StrongholdDestroyed),
            (42, 0xec212d8ee38450d7, 23274, Seat::A, StrongholdDestroyed),
        ],
        // RESTORE-01: canonical behavior unchanged; only private constructors restore legacy movement.
        (12, 0x8f5a8ebb3e4f88ec) => [
            (1, 0xae47594e85732bbc, 16625, Seat::B, StrongholdDestroyed),
            (7, 0x34994c75252c5c39, 14265, Seat::B, StrongholdDestroyed),
            (42, 0xec212d8ee38450d7, 23274, Seat::A, StrongholdDestroyed),
        ],
        revision => panic!("register an explicit canonical baseline for {revision:?}"),
    };
    let mut observed = Vec::with_capacity(goldens.len());
    let mut expected = Vec::with_capacity(goldens.len());
    for (seed, hash, tick, winner, reason) in goldens {
        let s = bot_match(seed);
        let o = s.outcome.expect("bot match must end");
        assert_eq!(
            (o.winner, o.reason, s.tick),
            (Some(winner), reason, tick),
            "seed {seed}"
        );
        observed.push((seed, s.hash()));
        expected.push((seed, hash));
    }
    // Compare all three only after their outcome/tick guards passed. A layout
    // migration reports every checksum without weakening any frozen behavior.
    assert_eq!(observed, expected, "simulation changed (see module docs)");
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
