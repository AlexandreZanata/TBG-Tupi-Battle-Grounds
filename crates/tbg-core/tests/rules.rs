//! Contract tests for the MVP rules (docs/GAME-DESIGN.md §3–§5).
//! Each test names the rule it protects.

use tbg_core::bot::Bot;
use tbg_core::content::{def, ENERGY_START, MATCH_END, OVERTIME_START};
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
fn empty_match_ends_in_draw_at_time_up() {
    // BR-M05: tie at regulation → overtime; tie after overtime → draw.
    let mut s = MatchState::new(1);
    let mut calls = 0;
    while s.outcome.is_none() {
        s.step();
        calls += 1;
        if s.tick == OVERTIME_START + 1 {
            assert_eq!(s.phase, Phase::Overtime);
        }
    }
    assert_eq!(calls, MATCH_END + 1);
    let o = s.outcome.unwrap();
    assert_eq!((o.winner, o.reason), (None, EndReason::TimeUp));
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
fn energy_regenerates_and_caps() {
    // BR-E02
    let mut s = running(3);
    assert_eq!(
        s.players[0].energy,
        ENERGY_START + tbg_core::content::REGEN_RUNNING
    );
    for _ in 0..1_000 {
        s.step();
    }
    assert_eq!(s.players[0].energy, 6_000);
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
fn purchased_offer_refills_and_rotation_is_shared() {
    // BR-S02, BR-S04
    let mut s = running(7);
    let before = s.players[0].rng.clone();
    buy(&mut s, Seat::A, UnitKind::Archer).unwrap();
    assert_ne!(s.players[0].rng, before, "purchase redraws the offer");
    let (va, vb) = (s.view_for(Seat::A, &[]), s.view_for(Seat::B, &[]));
    assert_eq!(va.next_rotation_in, vb.next_rotation_in);
    for _ in 0..va.next_rotation_in {
        s.step();
    }
    let rngs = [s.players[0].rng.clone(), s.players[1].rng.clone()];
    s.step();
    assert_ne!(s.players[0].rng, rngs[0]);
    assert_ne!(s.players[1].rng, rngs[1]);
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
fn time_up_higher_stronghold_wins() {
    // BR-M05
    let mut s = running(12);
    s.players[0].stronghold_hp -= 1;
    while s.outcome.is_none() {
        s.step();
    }
    assert_eq!(s.tick, OVERTIME_START + 1);
    let o = s.outcome.unwrap();
    assert_eq!((o.winner, o.reason), (Some(Seat::B), EndReason::TimeUp));
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
