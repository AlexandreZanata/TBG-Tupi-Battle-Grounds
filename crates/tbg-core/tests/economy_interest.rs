//! Owner revision 2026-10-29: uncapped Energy, ten-second Interest on the
//! low-water mark, compounding, and open-ended matches.

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

fn at(s: &mut MatchState, tick: u32) {
    while s.tick < tick {
        s.step();
    }
}

fn payment(s: &mut MatchState) -> (u32, u32) {
    let before = [0, 1].map(|i| s.players[i].interest_paid);
    s.step();
    (
        (s.players[0].interest_paid - before[0]) as u32,
        (s.players[1].interest_paid - before[1]) as u32,
    )
}

#[test]
fn br_e05_first_payment_is_ten_seconds_after_running() {
    let mut s = live(1);
    at(&mut s, RUNNING_START + 200);
    assert_eq!(s.players[0].interest_paid, 0);
    let paid = payment(&mut s);
    assert_eq!(paid, (150, 150), "5 Energy low pays 0.25");
    assert_eq!(s.players[0].low, s.players[0].energy);
}

#[test]
fn br_e05_both_seats_are_paid_on_the_same_tick() {
    let mut s = live(2);
    at(&mut s, RUNNING_START + 200);
    s.players[0].low = 9_000;
    s.players[1].low = 9_000;
    let paid = payment(&mut s);
    assert_eq!(paid, (450, 450));
}

#[test]
fn br_e05_formula_covers_every_band_and_caps_per_payment() {
    let cases = [
        (0, 0),
        (INTEREST_BAND - 1, 0),
        (INTEREST_BAND, INTEREST_PER_BAND),
        (2 * INTEREST_BAND - 1, INTEREST_PER_BAND),
        (2 * INTEREST_BAND, 2 * INTEREST_PER_BAND),
        (5 * INTEREST_BAND, 5 * INTEREST_PER_BAND),
        (50 * INTEREST_BAND, INTEREST_MAX_BANDS * INTEREST_PER_BAND),
    ];
    for (low, want) in cases {
        let mut s = live(3);
        at(&mut s, RUNNING_START + 200);
        s.players[0].low = low;
        s.players[1].low = low;
        let paid = payment(&mut s);
        assert_eq!(paid, (want, want), "low {low}");
    }
}

#[test]
fn br_e05_regen_never_raises_the_low_water_mark() {
    let mut s = live(4);
    at(&mut s, RUNNING_START + 200);
    s.players[0].energy = 2 * INTEREST_BAND;
    s.players[0].low = 2 * INTEREST_BAND;
    // Regen in the payment tick must not move the mark.
    s.players[0].energy = s.players[0].energy.saturating_add(REGEN_RUNNING);
    let paid = payment(&mut s);
    assert_eq!(paid.0, 2 * INTEREST_PER_BAND);
    assert_eq!(
        s.players[0].low, s.players[0].energy,
        "the next interval starts from the post-payment balance"
    );
}

#[test]
fn br_e05_a_spend_lowers_the_mark_even_before_payment() {
    let mut s = live(5);
    at(&mut s, RUNNING_START + 200);
    s.players[0].energy = 5 * INTEREST_BAND;
    s.players[0].low = 5 * INTEREST_BAND;
    assert!(s.players[0].spend(6 * ENERGY_SCALE));
    assert_eq!(s.players[0].low, 19 * ENERGY_SCALE);
    let paid = payment(&mut s);
    assert_eq!(paid.0, 3 * INTEREST_PER_BAND, "19 Energy is 0.75");
}

#[test]
fn br_e05_a_purchase_on_the_payment_tick_counts_first() {
    let mut s = live(6);
    at(&mut s, RUNNING_START + 200);
    s.players[0].energy = 25 * ENERGY_SCALE;
    s.players[0].low = 25 * ENERGY_SCALE;
    s.players[0].offers[0] = tbg_core::UnitKind::Archer;
    s.apply_intention(Seat::A, 1, tbg_core::PlayerIntention::Purchase { offer: 0 })
        .unwrap();
    assert_eq!(s.players[0].low, 22 * ENERGY_SCALE);
    let paid = payment(&mut s);
    assert_eq!(paid.0, 4 * INTEREST_PER_BAND, "22 Energy is 1.00");
}

#[test]
fn br_e06_interest_is_credited_and_compounds() {
    let mut idle = live(7);
    let mut spender = live(7);
    let mut idle_total = 0u64;
    for i in 1..=20 {
        let tick = RUNNING_START + 200 * i;
        at(&mut idle, tick);
        let paid = payment(&mut idle);
        idle_total += u64::from(paid.0);
        at(&mut spender, tick);
        let p = &mut spender.players[0];
        let cost = p.energy;
        if cost > 0 {
            p.spend(cost);
        }
        payment(&mut spender);
    }
    assert!(idle_total > 0);
    assert!(
        idle.players[0].interest_paid > spender.players[0].interest_paid,
        "banking must earn more Interest than immediate spending"
    );
    assert!(idle.players[0].interest_paid > u64::from(INTEREST_PER_BAND));
}

#[test]
fn br_e06_payment_never_duplicates_on_serialize_roundtrip() {
    let mut s = live(8);
    at(&mut s, RUNNING_START + 200);
    let bytes = postcard::to_allocvec(&s).unwrap();
    let mut restored: MatchState = postcard::from_bytes(&bytes).unwrap();
    let a = payment(&mut s);
    let b = payment(&mut restored);
    assert_eq!(a, b);
    assert_eq!(s.hash(), restored.hash());
}

#[test]
fn br_e07_energy_saturates_without_overflow() {
    let mut s = live(9);
    at(&mut s, RUNNING_START + 200);
    s.players[0].energy = u32::MAX - 2;
    s.players[0].low = u32::MAX - 2;
    s.players[1].energy = u32::MAX - 2;
    s.players[1].low = u32::MAX - 2;
    s.step();
    assert_eq!(s.players[0].energy, u32::MAX);
    assert_eq!(s.players[0].low, u32::MAX);
    assert_eq!(s.players[1].energy, u32::MAX);
}

#[test]
fn br_e03_no_interest_before_running_or_after_ended() {
    let mut s = MatchState::new(10);
    for _ in 0..RUNNING_START {
        s.step();
    }
    assert_eq!(s.players[0].interest_paid, 0);
    s.forfeit(Seat::B);
    let paid = s.players[0].interest_paid;
    for _ in 0..1_000 {
        s.step();
    }
    assert_eq!(s.players[0].interest_paid, paid);
}

#[test]
fn br_e02_energy_has_no_gameplay_cap() {
    let mut s = live(11);
    s.players[0].energy = 100 * ENERGY_SCALE;
    s.players[0].low = s.players[0].energy;
    for _ in 0..1_000 {
        s.step();
    }
    assert!(s.players[0].energy > 100 * ENERGY_SCALE);
    assert!(s.outcome.is_none());
}

#[test]
fn br_e03_overspend_is_rejected_and_spend_is_the_only_path() {
    let mut p = PlayerState {
        energy: 100,
        low: 200,
        ..MatchState::new(1).players[0].clone()
    };
    assert!(!p.spend(101));
    assert_eq!((p.energy, p.low), (100, 200));
    assert!(p.spend(100));
    assert_eq!((p.energy, p.low), (0, 0));
}

#[test]
fn br_e06_a_lab_forgo_reward_is_paid_once_per_round() {
    let mut rules = Ruleset::canon().clone();
    rules.augment.round_ticks = 200;
    rules.augment.window = 200;
    rules.augment.forgo_energy = 2 * ENERGY_SCALE;
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
