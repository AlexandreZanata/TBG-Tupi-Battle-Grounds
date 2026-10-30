//! Arena modifiers (ARENAS-G, D-55): the Forest is neutral, every other
//! arena changes exactly one thing, the same for both seats, applied once
//! on the shared stats/Energy/Evolution paths.

use tbg_core::content::{AugmentBonus, ARENA_RULES, ENERGY_SCALE, EVOLVE_COST};
use tbg_core::{
    ArcanaPreset, Arena, MatchState, Phase, PlayerIntention, Rejection, Seat, UnitKind,
};

fn running(arena: Arena) -> MatchState {
    let mut s = MatchState::new(11).with_arena(arena);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

/// Buy `kind` for `seat` directly (offer 0, Energy topped up).
fn buy(s: &mut MatchState, seat: Seat, kind: UnitKind, seq: u32) -> tbg_core::UnitId {
    s.players[seat.index()].offers[0] = kind;
    s.players[seat.index()].energy = 60_000;
    s.apply_intention(seat, seq, PlayerIntention::Purchase { offer: 0 })
        .expect("purchase");
    s.units.last().expect("bought").id
}

fn unit(s: &MatchState, id: tbg_core::UnitId) -> tbg_core::Unit {
    *s.units.iter().find(|u| u.id == id).expect("unit")
}

#[test]
fn every_arena_has_an_identity_and_the_forest_is_neutral() {
    for (i, arena) in Arena::ALL.into_iter().enumerate() {
        assert_eq!(arena.index(), i);
        assert_eq!(Arena::from_index(i), Some(arena));
        assert_eq!(Arena::from_slug(arena.slug()), Some(arena));
    }
    assert_eq!(Arena::from_index(5), None);
    assert_eq!(Arena::default(), Arena::Forest);
    assert_eq!(ARENA_RULES[0], tbg_core::content::ArenaRule::NONE);
}

#[test]
fn the_forest_keeps_stats_energy_and_prices() {
    let mut forest = running(Arena::Forest);
    let mut plain = running(Arena::Forest);
    plain.arena = Arena::Forest;
    let a = buy(&mut forest, Seat::A, UnitKind::Shieldbearer, 1);
    let rules = tbg_core::Ruleset::canon();
    let expected = rules.stats_with(
        UnitKind::Shieldbearer,
        0,
        0,
        0,
        &ArcanaPreset::NEUTRAL,
        &AugmentBonus::default(),
    );
    assert_eq!(forest.stats_of(&unit(&forest, a)), expected);
    assert_eq!(forest.evolve_price(Seat::A, 0), EVOLVE_COST[0]);
    assert_eq!(forest.evolve_price(Seat::B, 1), EVOLVE_COST[1]);
    // Below the first Interest band the gain is exactly the base rate.
    plain.players[1].energy = 0;
    plain.step();
    assert_eq!(plain.players[1].energy, rules.regen_running);
}

#[test]
fn tundra_slows_every_troop_of_both_seats_once() {
    let mut s = running(Arena::Tundra);
    let rules = tbg_core::Ruleset::canon();
    for (seat, seq) in [(Seat::A, 1), (Seat::B, 1)] {
        let id = buy(&mut s, seat, UnitKind::Sawbot, seq);
        let base = rules.def(UnitKind::Sawbot).speed;
        // −10 % with the single truncation of BR-X01.
        assert_eq!(s.stats_of(&unit(&s, id)).speed, base * 9_000 / 10_000);
    }
    // Exactly one application: equal to a unit given −1 000 move bp once.
    let once = rules.stats_with(
        UnitKind::Sawbot,
        0,
        0,
        0,
        &ArcanaPreset::NEUTRAL,
        &AugmentBonus {
            move_bp: -1_000,
            ..AugmentBonus::default()
        },
    );
    let id = s.units[0].id;
    assert_eq!(s.stats_of(&unit(&s, id)), once);
}

#[test]
fn volcanic_speeds_attacks_through_the_interval_formula() {
    let mut s = running(Arena::Volcanic);
    let rules = tbg_core::Ruleset::canon();
    for kind in [UnitKind::Archer, UnitKind::Shieldbearer, UnitKind::Sawbot] {
        let seq = s.players[0].last_seq + 1;
        let id = buy(&mut s, Seat::A, kind, seq);
        let base = i64::from(rules.def(kind).interval);
        let expected = ((base * 10_000 / 10_800) as u16).max(tbg_core::content::MIN_INTERVAL);
        assert_eq!(s.stats_of(&unit(&s, id)).interval, expected, "{kind:?}");
    }
}

#[test]
fn summons_and_cargo_take_the_arena_too() {
    let mut s = running(Arena::Tundra);
    let nester = buy(&mut s, Seat::B, UnitKind::Nester, 1);
    let mut beetle = None;
    for _ in 0..400 {
        s.step();
        beetle = s
            .units
            .iter()
            .find(|u| u.kind == UnitKind::Beetling && u.hp > 0)
            .copied();
        if beetle.is_some() {
            break;
        }
    }
    let beetle = beetle.expect("the Nester hatched");
    assert!(s.units.iter().any(|u| u.id == nester));
    let base = tbg_core::Ruleset::canon().def(UnitKind::Beetling).speed;
    assert_eq!(s.stats_of(&beetle).speed, base * 9_000 / 10_000);
}

#[test]
fn ruins_discounts_each_players_first_successful_evolution_once() {
    let mut s = running(Arena::Ruins);
    let a = buy(&mut s, Seat::A, UnitKind::Shieldbearer, 1);
    let b = buy(&mut s, Seat::B, UnitKind::Shieldbearer, 1);
    let first = EVOLVE_COST[0] - ENERGY_SCALE;
    assert_eq!(s.evolve_price(Seat::A, 0), first);
    assert_eq!(s.evolve_price(Seat::B, 0), first);
    assert_eq!(
        s.view_for(Seat::A, &[]).evolve_costs,
        [first, EVOLVE_COST[1] - ENERGY_SCALE]
    );
    // Too poor: rejected, nothing spent, the discount stays.
    s.players[0].energy = first - 1;
    assert_eq!(
        s.apply_intention(Seat::A, 2, PlayerIntention::Evolve { unit: a, branch: 0 }),
        Err(Rejection::NotEnoughEnergy)
    );
    assert!(!s.players[0].arena_discount_used);
    // Exactly the discounted price is charged, once.
    s.players[0].energy = first;
    s.apply_intention(Seat::A, 3, PlayerIntention::Evolve { unit: a, branch: 0 })
        .expect("evolve");
    assert_eq!(s.players[0].energy, 0);
    assert!(s.players[0].arena_discount_used);
    assert_eq!(s.evolve_price(Seat::A, 1), EVOLVE_COST[1]);
    // The next Evolution pays the full price: the discount is spent.
    // (Retransmitted intentions are refused by the server before the core;
    // see tbg-server's arena tests.)
    s.players[0].energy = 50_000;
    s.apply_intention(Seat::A, 4, PlayerIntention::Evolve { unit: a, branch: 0 })
        .expect("evolve to tier 2");
    assert_eq!(s.players[0].energy, 50_000 - EVOLVE_COST[1]);
    // The other seat still has its own discount.
    assert!(!s.players[1].arena_discount_used);
    s.players[1].energy = first;
    s.apply_intention(Seat::B, 2, PlayerIntention::Evolve { unit: b, branch: 1 })
        .expect("evolve B");
    assert_eq!(s.players[1].energy, 0);
}

#[test]
fn ruins_stacks_with_augment_discounts_but_never_below_one_energy() {
    let mut s = running(Arena::Ruins);
    let discount = tbg_core::content::AUGMENTS
        .iter()
        .find_map(|d| match d.effect {
            tbg_core::content::AugmentEffect::EvolveDiscount { amount } => Some((d.id, amount)),
            _ => None,
        })
        .expect("an Evolution discount Augment");
    s.players[0].augments[usize::from(discount.0)] = 30;
    assert_eq!(s.evolve_price(Seat::A, 0), ENERGY_SCALE);
    s.players[0].augments[usize::from(discount.0)] = 1;
    assert_eq!(
        s.evolve_price(Seat::A, 0),
        EVOLVE_COST[0]
            .saturating_sub(discount.1 + ENERGY_SCALE)
            .max(ENERGY_SCALE)
    );
}

#[test]
fn storm_raises_only_the_base_regen_and_keeps_the_remainder() {
    let ticks = 1_000u32;
    let gain = |arena: Arena, energy: u32| {
        let mut s = running(arena);
        let mut total = 0u64;
        for _ in 0..ticks {
            // Hold Energy fixed so Interest stays on one step.
            s.players[0].energy = energy;
            s.step();
            total += u64::from(s.players[0].energy - energy);
        }
        total
    };
    let regen = u64::from(tbg_core::Ruleset::canon().regen_running);
    // No Interest: exactly +8 % of the base over many ticks.
    assert_eq!(gain(Arena::Forest, 0), regen * u64::from(ticks));
    assert_eq!(
        gain(Arena::Storm, 0),
        regen * u64::from(ticks) * 10_800 / 10_000
    );
    // With Interest: the arena adds 8 % of the base, Interest is unchanged.
    let band = tbg_core::Ruleset::canon().interest.band * 2;
    let interest_bp = u64::from(tbg_core::Ruleset::canon().interest.regen_bp(band));
    assert_eq!(
        gain(Arena::Forest, band),
        regen * u64::from(ticks) * interest_bp / 10_000
    );
    assert_eq!(
        gain(Arena::Storm, band),
        regen * u64::from(ticks) * (interest_bp + 800) / 10_000
    );
}

#[test]
fn an_arena_is_deterministic_and_fixed_for_the_match() {
    let play = |arena: Arena| {
        let mut s = MatchState::new(5).with_arena(arena);
        let mut bots = [tbg_core::bot::Bot::new(5), tbg_core::bot::Bot::new(6)];
        let mut seq = [0u32; 2];
        for _ in 0..3_000 {
            for seat in Seat::BOTH {
                let view = s.view_for(seat, &[]);
                if let Some(intent) = bots[seat.index()].decide(&view) {
                    seq[seat.index()] += 1;
                    let _ = s.apply_intention(seat, seq[seat.index()], intent);
                }
            }
            s.step();
            assert_eq!(s.arena, arena);
            if s.outcome.is_some() {
                break;
            }
        }
        s.hash()
    };
    for arena in Arena::ALL {
        assert_eq!(play(arena), play(arena), "{arena:?}");
    }
}
