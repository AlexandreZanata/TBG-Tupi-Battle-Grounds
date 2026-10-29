//! Owner revision 2026-10-29: uncapped Energy and Interest as a regen boost
//! (every full 5 Energy held adds 25 % of the base regen, at most 25 Energy
//! counted), plus open-ended matches (BR-E02–E08).

use tbg_core::content::{
    ENERGY_SCALE, INTEREST_BAND, INTEREST_MAX_BANDS, INTEREST_PER_BAND, REGEN_RUNNING,
    RUNNING_START,
};
use tbg_core::{MatchState, Phase, PlayerState, Rejection, Ruleset, Seat};

fn live(seed: u64) -> MatchState {
    let mut s = MatchState::new(seed);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

/// Energy a seat gains over `ticks` steps from `energy` (no spending).
fn gain(energy: u32, ticks: u32) -> u32 {
    let mut s = live(1);
    s.players[0].energy = energy;
    s.players[0].regen_frac = 0;
    for _ in 0..ticks {
        s.step();
    }
    s.players[0].energy - energy
}

#[test]
fn br_e05_every_five_energy_adds_a_quarter_of_the_base_regen() {
    let rule = Ruleset::canon().interest;
    let cases = [
        (0, 10_000),
        (INTEREST_BAND - 1, 10_000),
        (INTEREST_BAND, 12_500),
        (2 * INTEREST_BAND - 1, 12_500),
        (2 * INTEREST_BAND, 15_000),
        (3 * INTEREST_BAND, 17_500),
        (4 * INTEREST_BAND, 20_000),
        (5 * INTEREST_BAND, 22_500),
        (50 * INTEREST_BAND, 22_500),
        (u32::MAX, 22_500),
    ];
    for (energy, want) in cases {
        assert_eq!(rule.regen_bp(energy), want, "{energy} sub-energy");
    }
    assert_eq!(INTEREST_PER_BAND, 2_500);
    assert_eq!(INTEREST_MAX_BANDS, 5);
}

#[test]
fn br_e05_the_rate_is_exact_over_time() {
    // 20 ticks of a 1.25x rate are 425 sub-energy; the fraction never leaks.
    let base = u64::from(REGEN_RUNNING) * 20;
    assert_eq!(u64::from(gain(INTEREST_BAND, 20)), base * 125 / 100);
    assert_eq!(u64::from(gain(2 * INTEREST_BAND, 20)), base * 150 / 100);
    assert_eq!(u64::from(gain(4 * INTEREST_BAND, 20)), base * 200 / 100);
    // 17 * 2.25 per tick is 38.25: 4 ticks give exactly 153.
    assert_eq!(gain(30 * INTEREST_BAND, 4), 153);
    assert_eq!(gain(0, 20), u32::try_from(base).unwrap());
}

#[test]
fn br_e05_the_boost_follows_the_balance_at_once() {
    let mut s = live(2);
    s.players[0].energy = 25 * ENERGY_SCALE;
    assert_eq!(s.view_for(Seat::A, &[]).regen_bp, 22_500);
    assert!(s.players[0].spend(6 * ENERGY_SCALE));
    // 19 Energy holds three full steps of five.
    assert_eq!(s.view_for(Seat::A, &[]).regen_bp, 17_500);
    assert!(s.players[0].spend(19 * ENERGY_SCALE));
    assert_eq!(s.view_for(Seat::A, &[]).regen_bp, 10_000);
}

#[test]
fn br_e05_the_cap_limits_the_boost_not_the_energy() {
    let mut s = live(3);
    s.players[0].energy = 100 * ENERGY_SCALE;
    s.players[0].regen_frac = 0;
    let start = s.players[0].energy;
    for _ in 0..1_000 {
        s.step();
    }
    // 1 000 ticks at the maximum 2.25x of 17 sub-energy per tick, pauses
    // excluded (the Augment round at 1:00 is not reached in 1 000 ticks).
    assert_eq!(s.players[0].energy - start, 38_250);
    assert!(s.outcome.is_none());
}

#[test]
fn br_e06_banking_compounds_and_spending_resets_it() {
    let mut banker = live(7);
    let mut spender = live(7);
    for _ in 0..1_000 {
        banker.step();
        spender.step();
        let p = &mut spender.players[0];
        let all = p.energy;
        p.spend(all);
    }
    let held = banker.players[0].energy;
    let flat = u64::from(REGEN_RUNNING) * 1_000;
    assert!(u64::from(held) > flat + u64::from(tbg_core::content::ENERGY_START));
    assert!(spender.players[0].energy < held);
}

#[test]
fn br_e06_the_remainder_survives_a_serialize_roundtrip() {
    let mut s = live(8);
    s.players[0].energy = 3 * INTEREST_BAND;
    for _ in 0..7 {
        s.step();
    }
    assert!(s.players[0].regen_frac > 0, "17 x 1.5 leaves a remainder");
    let bytes = postcard::to_allocvec(&s).unwrap();
    let mut restored: MatchState = postcard::from_bytes(&bytes).unwrap();
    for _ in 0..50 {
        s.step();
        restored.step();
    }
    assert_eq!(s.players[0].energy, restored.players[0].energy);
    assert_eq!(s.hash(), restored.hash());
}

#[test]
fn br_e07_energy_saturates_without_overflow() {
    let mut s = live(9);
    s.players[0].energy = u32::MAX - 2;
    s.players[1].energy = u32::MAX - 2;
    s.step();
    assert_eq!(s.players[0].energy, u32::MAX);
    assert_eq!(s.players[1].energy, u32::MAX);
}

#[test]
fn br_e03_no_regen_before_running_or_after_ended() {
    let mut s = MatchState::new(10);
    let start = s.players[0].energy;
    for _ in 0..RUNNING_START {
        s.step();
    }
    assert_eq!(s.players[0].energy, start, "the countdown pays nothing");
    s.forfeit(Seat::B);
    let ended = s.players[0].energy;
    for _ in 0..1_000 {
        s.step();
    }
    assert_eq!(s.players[0].energy, ended);
}

#[test]
fn br_e08_the_bounty_window_resets_every_ten_seconds_of_combat() {
    let mut s = live(11);
    s.players[0].bounty_window = 123;
    while s.tick < RUNNING_START + 199 {
        s.step();
    }
    assert_eq!(s.players[0].bounty_window, 123);
    s.step();
    s.step();
    assert_eq!(s.players[0].bounty_window, 0);
}

#[test]
fn br_e03_overspend_is_rejected_and_spend_is_the_only_path() {
    let mut p = PlayerState {
        energy: 100,
        ..MatchState::new(1).players[0].clone()
    };
    assert!(!p.spend(101));
    assert_eq!(p.energy, 100);
    assert!(p.spend(100));
    assert_eq!(p.energy, 0);
}

#[test]
fn br_e06_a_lab_forgo_reward_is_paid_once_per_round() {
    let mut rules = Ruleset::canon().clone();
    rules.augment.round_ticks = 200;
    rules.augment.window = 200;
    rules.augment.forgo_energy = 2 * ENERGY_SCALE;
    rules.regen_running = 0;
    let rules: &'static Ruleset = Box::leak(Box::new(rules));
    let mut s = MatchState::with_rules(12, rules);
    while s.augments.round == 0 {
        s.step();
    }
    let before = s.players[0].energy;
    let round = s.augments.round;
    s.apply_intention(Seat::A, 1, tbg_core::PlayerIntention::Forgo { round })
        .unwrap();
    assert_eq!(
        s.apply_intention(Seat::A, 2, tbg_core::PlayerIntention::Forgo { round }),
        Err(Rejection::RoundClosed)
    );
    // Paid once when the paused round resolves; Energy is frozen until then.
    assert_eq!(s.players[0].energy, before);
    while s.augments.selecting() {
        s.step();
    }
    assert_eq!(s.players[0].energy, before + 2 * ENERGY_SCALE);
}
