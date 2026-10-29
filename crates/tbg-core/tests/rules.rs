//! Contract tests for the MVP rules (docs/GAME-DESIGN.md §3–§5).
//! Each test names the rule it protects.

use tbg_core::bot::Bot;
use tbg_core::content::{def, ENERGY_START};
use tbg_core::protocol::{decode_client, encode, ClientMsg};
use tbg_core::*;

/// A fresh match advanced to the first Running tick.
fn running(seed: u64) -> MatchState {
    let mut s = MatchState::new(seed);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

fn buy(s: &mut MatchState, seat: Seat, kind: UnitKind) -> Result<(), Rejection> {
    s.players[seat.index()].offers[0] = kind;
    s.players[seat.index()].energy = 6_000;
    s.apply_intention(seat, 1, PlayerIntention::Purchase { offer: 0 })
}

fn bot_match(seed: u64) -> MatchState {
    let mut s = MatchState::new(seed);
    let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
    let mut events = Vec::new();
    while s.phase != Phase::Ended && s.tick < 20 * 60 * 60 * 6 {
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
fn empty_match_never_ends_by_time() {
    // Owner revision: no clock, no Siege, no Collapse, no automatic damage.
    let mut s = MatchState::new(1);
    for _ in 0..20 * 60 * 20 {
        s.step();
    }
    assert_eq!(s.outcome, None);
    assert_eq!(s.phase, Phase::Running);
    assert!(s.players.iter().all(|p| p.stronghold_hp > 0));
}

#[test]
fn same_seed_and_script_same_hash() {
    // D-04: deterministic outcomes.
    let (a, b) = (bot_match(42), bot_match(42));
    assert_eq!(a.hash(), b.hash());
    assert_eq!(a.outcome, b.outcome);
    assert!(a.outcome.is_some());
}

#[test]
fn energy_regenerates_without_cap_and_pays_interest() {
    // BR-E04–E07: uncapped regen plus compounding interest.
    let mut s = running(3);
    assert_eq!(
        s.players[0].energy,
        ENERGY_START + tbg_core::content::REGEN_RUNNING
    );
    for _ in 0..1_000 {
        s.step();
    }
    assert!(s.players[0].energy > 6_000, "{}", s.players[0].energy);
    assert!(s.players[0].interest_paid > 0);
}

#[test]
fn overspend_is_rejected_and_energy_unchanged() {
    // BR-E03
    let mut s = running(4);
    s.players[0].energy = 100;
    let r = s.apply_intention(Seat::A, 5, PlayerIntention::Purchase { offer: 0 });
    assert_eq!(r, Err(Rejection::NotEnoughEnergy));
    assert_eq!(s.players[0].energy, 100);
    assert_eq!(s.players[0].last_seq, 5);
}

#[test]
fn purchase_rejected_during_countdown() {
    // BR-M02 / BR-B02: deployment only while the match is live.
    let mut s = MatchState::new(5);
    let r = s.apply_intention(Seat::A, 1, PlayerIntention::Purchase { offer: 0 });
    assert_eq!(r, Err(Rejection::NotRunning));
}

#[test]
fn br_s04_a_purchase_refills_with_an_unseen_unit_and_nothing_rotates() {
    // BR-S01, BR-S04 (owner revision 2026-10-29): offers change only on a
    // purchase; the refill is never one of the five units on screen, the
    // bought one included, and every unseen unit can come up.
    let mut s = running(7);
    let offers = s.players[0].offers;
    // Five former rotation periods, still before the first Augment pause.
    for _ in 0..1_000 {
        s.step();
    }
    assert_eq!(s.players[0].offers, offers, "no timed rotation");
    let mut seen = std::collections::BTreeSet::new();
    for seq in 1..=400 {
        let before = s.players[0].offers;
        s.players[0].energy = 10 * ENERGY_START;
        let slot = (seq % 5) as u8;
        s.apply_intention(Seat::A, seq, PlayerIntention::Purchase { offer: slot })
            .unwrap();
        let after = s.players[0].offers;
        let new = after[usize::from(slot)];
        assert!(!before.contains(&new), "{new:?} was already on screen");
        for i in (0..5).filter(|&i| i != usize::from(slot)) {
            assert_eq!(after[i], before[i], "other offers stay put");
        }
        let mut distinct = after.to_vec();
        distinct.sort_unstable_by_key(|k| *k as u8);
        distinct.dedup();
        assert_eq!(distinct.len(), 5, "five distinct offers");
        seen.insert(new as u8);
        s.units.clear();
    }
    assert_eq!(seen.len(), UNIT_KINDS.len(), "every unit can be drawn");
}

#[test]
fn evolution_branch_gating() {
    // BR-V01: tier 2 must follow the tier-1 branch; max tier is 2.
    let mut s = running(8);
    buy(&mut s, Seat::A, UnitKind::Shieldbearer).unwrap();
    let id = s.units[0].id;
    let evolve = |s: &mut MatchState, branch| {
        s.players[0].energy = 6_000;
        s.apply_intention(Seat::A, 2, PlayerIntention::Evolve { unit: id, branch })
    };
    assert_eq!(evolve(&mut s, 2), Err(Rejection::WrongBranch));
    evolve(&mut s, 1).unwrap();
    assert_eq!(evolve(&mut s, 0), Err(Rejection::WrongBranch));
    evolve(&mut s, 1).unwrap();
    assert_eq!(evolve(&mut s, 1), Err(Rejection::MaxTier));
    assert_eq!((s.units[0].tier, s.units[0].branch), (2, 1));
}

#[test]
fn evolve_other_seats_unit_rejected() {
    // BR-A02: a seat may only act on its own units.
    let mut s = running(9);
    buy(&mut s, Seat::B, UnitKind::Sawbot).unwrap();
    let id = s.units[0].id;
    let r = s.apply_intention(
        Seat::A,
        1,
        PlayerIntention::Evolve {
            unit: id,
            branch: 0,
        },
    );
    assert_eq!(r, Err(Rejection::NotYourUnit));
    assert_eq!(s.units[0].tier, 0);
}

#[test]
fn synergy_levels_at_two_and_four_and_drop_on_death() {
    // BR-Y01
    let mut s = running(10);
    let humans = Faction::Humans.index();
    for _ in 0..2 {
        buy(&mut s, Seat::A, UnitKind::Shieldbearer).unwrap();
    }
    s.step();
    assert_eq!(s.players[0].synergy[humans], 1);
    let level1 = tbg_core::content::effective_stats(UnitKind::Shieldbearer, 0, 0, 1).max_hp;
    assert!(
        level1 > def(UnitKind::Shieldbearer).hp,
        "Humans level 1 adds HP"
    );
    assert_eq!(s.units[0].max_hp, level1);
    for _ in 0..2 {
        buy(&mut s, Seat::A, UnitKind::Archer).unwrap();
    }
    s.step();
    assert_eq!(s.players[0].synergy[humans], 2);
    s.units[0].hp = 0;
    s.step();
    s.step();
    assert_eq!(s.players[0].synergy[humans], 1);
}

#[test]
fn stronghold_destroyed_wins() {
    // BR-M04
    let mut s = running(11);
    buy(&mut s, Seat::A, UnitKind::Sawbot).unwrap();
    s.units[0].x = 15_000;
    s.players[1].stronghold_hp = 50;
    for _ in 0..200 {
        s.step();
    }
    let o = s.outcome.expect("match ended");
    assert_eq!(
        (o.winner, o.reason),
        (Some(Seat::A), EndReason::StrongholdDestroyed)
    );
}

#[test]
fn unequal_strongholds_do_not_end_by_time() {
    // BR-M07: only destruction or forfeit decide the match.
    let mut s = running(12);
    s.players[0].stronghold_hp -= 1;
    for _ in 0..20 * 60 * 20 {
        s.step();
    }
    assert_eq!(s.outcome, None);
    assert_eq!(s.players[0].stronghold_hp, 4_499);
    assert_eq!(s.players[1].stronghold_hp, 4_500);
}

#[test]
fn forfeit_gives_win_to_other_seat() {
    // BR-M03
    let mut s = running(13);
    s.forfeit(Seat::B);
    let o = s.outcome.unwrap();
    assert_eq!((o.winner, o.reason), (Some(Seat::A), EndReason::Forfeit));
    assert!(s.step().contains(&Event::PhaseChanged {
        phase: Phase::Ended
    }));
}

#[test]
fn protocol_roundtrip_and_garbage_rejected() {
    // BR-A04: untrusted bytes never become messages by accident.
    let msg = ClientMsg::Intent {
        seq: 7,
        intent: PlayerIntention::Evolve { unit: 3, branch: 1 },
    };
    assert_eq!(decode_client(&encode(&msg)), Ok(msg));
    assert!(decode_client(&[0xff, 0x13, 0x07]).is_err());
    assert!(decode_client(&[0u8; 2_000]).is_err());
    let mut trailing = encode(&ClientMsg::QuickMatch);
    trailing.push(0);
    assert!(decode_client(&trailing).is_err());
}

#[test]
fn view_hides_opponent_energy_and_offers() {
    // BR-S05
    let mut s = running(14);
    s.players[0].energy = 1_234;
    s.players[1].energy = 4_321;
    s.players[0].offers = [UnitKind::Archer; 5];
    s.players[1].offers = [UnitKind::Hexer; 5];
    let v = s.view_for(Seat::A, &[]);
    assert_eq!(
        (v.energy, v.offers, v.you),
        (1_234, [UnitKind::Archer; 5], Seat::A)
    );
    assert_eq!(def(UnitKind::Archer).cost, 1_800);
}
