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
    use EndReason::StrongholdDestroyed;
    // D-68: owner champion changes in GAME-DESIGN, collected with update_probe.
    // D-69: appended private Shop state; all D-68 outcomes and ticks identical.
    // D-68/D-69 integration: protocol-20 slots + Shop state; owner range correction.
    let goldens = [
        (1, 0x9615d6ececbe8b99u64, 3261, Seat::B, StrongholdDestroyed),
        (7, 0x008b5f628ee0b6dau64, 2266, Seat::B, StrongholdDestroyed),
        (
            42,
            0x8e40c6e671e82ee3u64,
            7977,
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
