//! Owner revision 2026-10-29: four options every 60 s forever, one purchase
//! per round, unlimited copies and stacking.

use tbg_core::augment::AugmentCounts;
use tbg_core::content::{
    augment_def, AUGMENT_CATALOG, AUGMENT_COST, AUGMENT_OPTIONS, AUGMENT_ROUND_TICKS, ENERGY_SCALE,
    RUNNING_START,
};
use tbg_core::{MatchState, Phase, PlayerIntention, Rejection, Ruleset, Seat, UnitKind};

fn live(seed: u64) -> MatchState {
    let mut s = MatchState::new(seed);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

fn open(s: &mut MatchState, round: u32) {
    while s.augments.round < round {
        s.step();
    }
}

/// Step through the pause until the open round resolves (BR-AG13).
fn resolve(s: &mut MatchState) {
    while s.augments.selecting() {
        s.step();
    }
}

fn rich(s: &mut MatchState) {
    for p in &mut s.players {
        p.energy = 1_000 * ENERGY_SCALE;
        p.low = p.energy;
    }
}

fn choose(s: &mut MatchState, seat: Seat, option: u8) -> Result<(), Rejection> {
    let round = s.augments.round;
    s.apply_intention(
        seat,
        s.tick.saturating_add(1),
        PlayerIntention::ChooseAugment { round, option },
    )
}

#[test]
fn br_ag01_first_round_is_at_one_minute_and_repeats_forever() {
    let mut s = live(1);
    open(&mut s, 1);
    assert_eq!(s.augments.opened_at, RUNNING_START + AUGMENT_ROUND_TICKS);
    assert_eq!(s.augments.options.len(), AUGMENT_OPTIONS);
    for round in 2..=25 {
        let previous = s.augments.round;
        open(&mut s, round);
        assert_eq!(s.augments.round, previous + 1);
        assert_eq!(
            s.augments.opened_at,
            RUNNING_START + AUGMENT_ROUND_TICKS * round
        );
    }
}

#[test]
fn br_ag02_four_distinct_options_and_both_seats_see_them() {
    for seed in 0..100 {
        let mut s = live(seed);
        open(&mut s, 1);
        let mut options = s.augments.options;
        options.sort_unstable();
        assert!(options.windows(2).all(|w| w[0] != w[1]), "{options:?}");
        let a = s.view_for(Seat::A, &[]);
        let b = s.view_for(Seat::B, &[]);
        assert_eq!(a.augment_round, b.augment_round);
        assert!(options.iter().all(|id| augment_def(*id).is_some()));
    }
}

#[test]
fn br_ag03_each_cycle_offers_all_forty_and_reshuffles() {
    let mut s = live(2);
    let mut seen = [false; AUGMENT_CATALOG];
    for _ in 0..10 {
        let next = s.augments.round + 1;
        open(&mut s, next);
        for id in s.augments.options {
            seen[usize::from(id)] = true;
        }
    }
    assert!(seen.iter().all(|v| *v), "first cycle missed an Augment");
    let position = s.augments.position;
    assert_eq!(position, 40);
    open(&mut s, 11);
    assert_eq!(s.augments.round, 11);
    assert_eq!(s.augments.position, 4, "second cycle must reshuffle");
    let mut options = s.augments.options;
    options.sort_unstable();
    assert!(options.windows(2).all(|w| w[0] != w[1]));
}

#[test]
fn br_ag05_one_purchase_per_round_and_repeats_are_allowed_later() {
    let mut s = live(3);
    open(&mut s, 1);
    rich(&mut s);
    choose(&mut s, Seat::A, 0).unwrap();
    assert_eq!(choose(&mut s, Seat::A, 1), Err(Rejection::RoundClosed));
    let first = s.players[0].augment_resolved;
    assert_eq!(first, 1);
    open(&mut s, 2);
    choose(&mut s, Seat::A, 0).unwrap();
    assert_eq!(s.players[0].augment_resolved, 2);
}

#[test]
fn br_ag08_every_catalog_id_can_be_bought_and_stacked() {
    for id in 0..AUGMENT_CATALOG as u8 {
        let mut s = live(u64::from(id) + 10);
        open(&mut s, 1);
        rich(&mut s);
        s.augments.options = [id, id, id, id];
        choose(&mut s, Seat::A, 0).unwrap();
        resolve(&mut s);
        assert_eq!(s.players[0].copies(id), 1, "id {id}");
        assert_eq!(
            s.players[0].energy,
            1_000 * ENERGY_SCALE - AUGMENT_COST[augment_def(id).unwrap().rarity.index()]
        );
        // Every effect must keep the simulation stable for a full sentence.
        for _ in 0..100 {
            s.step();
        }
    }
}

#[test]
fn field_kit_copies_stack_on_existing_units() {
    let mut s = live(4);
    // Buy on the last combat tick before the draw, so the unit is alive
    // when the pause starts.
    while s.tick + 1 < s.augments.next_open_at {
        s.step();
    }
    rich(&mut s);
    s.players[0].offers[0] = UnitKind::Shieldbearer;
    s.apply_intention(Seat::A, 1, PlayerIntention::Purchase { offer: 0 })
        .unwrap();
    let id = s.units[0].id;
    let base = s.units[0].max_hp;
    open(&mut s, 1);
    s.augments.options = [1, 0, 2, 3];
    choose(&mut s, Seat::A, 0).unwrap();
    resolve(&mut s);
    let once = s.units.iter().find(|u| u.id == id).unwrap().max_hp;
    assert_eq!(once, base * 10_400 / 10_000);
    open(&mut s, 2);
    s.augments.options = [1, 0, 2, 3];
    choose(&mut s, Seat::A, 0).unwrap();
    resolve(&mut s);
    s.players[0].offers[0] = UnitKind::Shieldbearer;
    s.players[0].energy = 10 * ENERGY_SCALE;
    s.apply_intention(Seat::A, 2, PlayerIntention::Purchase { offer: 0 })
        .unwrap();
    let twice = s.units.last().unwrap().max_hp;
    assert_eq!(twice, base * 10_800 / 10_000);
    assert_eq!(s.players[0].copies(1), 2);
}

#[test]
fn br_ag06_forgo_marks_the_round_and_pays_at_most_once() {
    let mut rules = Ruleset::canon().clone();
    rules.augment.forgo_energy = 2 * ENERGY_SCALE;
    let rules: &'static Ruleset = Box::leak(Box::new(rules));
    let mut s = MatchState::with_rules(5, rules);
    open(&mut s, 1);
    let round = s.augments.round;
    let before = s.players[1].energy;
    s.apply_intention(Seat::B, 1, PlayerIntention::Forgo { round })
        .unwrap();
    assert_eq!(
        s.apply_intention(Seat::B, 2, PlayerIntention::Forgo { round }),
        Err(Rejection::RoundClosed)
    );
    assert_eq!(s.players[0].augment_resolved, 0);
    // The reward is paid once, when the round resolves (BR-AG14).
    assert_eq!(s.players[1].energy, before);
    resolve(&mut s);
    assert_eq!(s.players[1].energy, before + 2 * ENERGY_SCALE);
}

#[test]
fn br_ag04_expiry_grants_forgo_once_when_configured() {
    let mut rules = Ruleset::canon().clone();
    rules.augment.round_ticks = 200;
    rules.augment.window = 200;
    rules.augment.forgo_energy = 2 * ENERGY_SCALE;
    rules.regen_running = 0;
    rules.interest.max_bands = 0;
    let rules: &'static Ruleset = Box::leak(Box::new(rules));
    let mut s = MatchState::with_rules(6, rules);
    open(&mut s, 1);
    let before = s.players[0].energy;
    open(&mut s, 2);
    assert_eq!(
        s.players[0].energy,
        before + 2 * ENERGY_SCALE,
        "round {} resolved {} opened {} deadline {} next {} tick {}",
        s.augments.round,
        s.players[0].augment_resolved,
        s.augments.opened_at,
        s.augments.deadline,
        s.augments.next_open_at,
        s.tick
    );
    let after = s.players[0].energy;
    open(&mut s, 3);
    assert_eq!(s.players[0].energy, after + 2 * ENERGY_SCALE);
}

#[test]
fn br_ag07_a_short_player_may_wait_and_buy_later() {
    let mut s = live(7);
    open(&mut s, 1);
    let cost = AUGMENT_COST[augment_def(s.augments.options[0]).unwrap().rarity.index()];
    s.players[0].energy = cost - 1;
    s.players[0].low = s.players[0].energy;
    assert_eq!(choose(&mut s, Seat::A, 0), Err(Rejection::NotEnoughEnergy));
    s.players[0].energy = cost;
    choose(&mut s, Seat::A, 0).unwrap();
    assert_eq!(s.players[0].energy, 0);
}

#[test]
fn br_ag08_counts_are_bounded_and_compact() {
    let counts = AugmentCounts::default();
    assert_eq!(counts.get(39), 0);
    let mut counts = counts;
    counts.add(39);
    counts.add(39);
    assert_eq!(counts.get(39), 2);
}
