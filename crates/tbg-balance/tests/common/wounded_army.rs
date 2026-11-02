//! C7 healing sensitivity with equal-budget wounded armies and no Shop-policy advantage.

use tbg_balance::{duels, runner::parallel_map};
use tbg_core::{content, MatchState, Ruleset, Seat, UnitKind, UNIT_KINDS};

/// Paired physical seats retain the same escort, budget and initial wound fraction.
/// The opposing army never fields the mutated healer. Every original damaging
/// kind, all three existing duel budgets and 25/50/75% initial HP are represented.
pub fn score(rules: &Ruleset, healer: UnitKind, pairs: u64) -> f64 {
    assert!(pairs > 0);
    let arena = duels::arena(rules);
    let escorts: Vec<_> = UNIT_KINDS
        .into_iter()
        .filter(|&kind| kind != healer && rules.def(kind).damage > 0)
        .collect();
    assert!(!escorts.is_empty());
    let seeds: Vec<_> = (0..pairs).collect();
    let rows = parallel_map(&seeds, 2, |&seed| {
        let escort = escorts[seed as usize % escorts.len()];
        let budget = duels::BUDGETS[(seed as usize / escorts.len()) % duels::BUDGETS.len()];
        let quarters = 1 + (seed / (escorts.len() * duels::BUDGETS.len()) as u64) % 3;
        Seat::BOTH
            .map(|seat| fight(arena, healer, escort, budget, quarters as i32, seed, seat))
            .iter()
            .sum::<f64>()
            / 2.0
    });
    rows.iter().sum::<f64>() / pairs as f64
}

fn fight(
    rules: &'static Ruleset,
    healer: UnitKind,
    escort: UnitKind,
    budget: u32,
    quarters: i32,
    seed: u64,
    row: Seat,
) -> f64 {
    let mut state = MatchState::with_rules(seed, rules);
    while state.tick < content::RUNNING_START {
        state.step();
    }
    let energy = budget * content::ENERGY_SCALE;
    let cost = rules.def(escort).cost;
    assert!(cost > 0 && energy >= rules.def(healer).cost + cost);
    for seat in Seat::BOTH {
        let spent = if seat == row {
            rules.def(healer).cost
        } else {
            0
        };
        for index in 0..(energy - spent) / cost {
            // Start in contact so the initial wounds cannot heal away on a long march.
            let x = 8_000 - seat.dir() * (400 + index as i32 * 50);
            let id = state.lab_spawn_at(seat, escort, 0, 0, x);
            let index = state.unit_index(id).expect("lab-spawned escort");
            state.units[index].hp = (state.units[index].hp * quarters / 4).max(1);
        }
    }
    state.lab_spawn_at(row, healer, 0, 0, 8_000 - row.dir() * 2_000);
    let start = state.tick;
    loop {
        state.step();
        let hp = fighting_hp(&state, healer);
        if let Some(outcome) = state.outcome {
            return outcome
                .winner
                .map_or(0.5, |seat| if seat == row { 1.0 } else { 0.0 });
        }
        // As in the duel contract, extinction or the existing 90 s limit decides.
        // A surviving pure healer cannot win after every combat escort has died.
        if hp.contains(&0) || state.tick - start >= 90 * content::TICK_HZ {
            return match hp[row.index()].cmp(&hp[row.other().index()]) {
                std::cmp::Ordering::Greater => 1.0,
                std::cmp::Ordering::Equal => 0.5,
                std::cmp::Ordering::Less => 0.0,
            };
        }
    }
}

fn fighting_hp(state: &MatchState, healer: UnitKind) -> [i64; 2] {
    Seat::BOTH.map(|seat| {
        state
            .units
            .iter()
            .chain(&state.deployments)
            .filter(|unit| unit.seat == seat && unit.kind != healer && unit.hp > 0)
            .map(|unit| i64::from(unit.hp))
            .sum()
    })
}
