//! D-44: a favored offered unit must be affordable before the policy spends on a redraw.
use tbg_balance::policy::{Controller, Policy};
use tbg_core::{
    content::ENERGY_SCALE, MatchState, Phase, PlayerIntention, Ruleset, Seat, UnitKind,
};

#[test]
fn offered_favored_unit_banks_through_reaction_ticks_then_buys_it() {
    let mut rules = Ruleset::canon().clone();
    rules.units[UnitKind::SiegeWalker as usize].cost *= 3;
    let rules = Box::leak(Box::new(rules));
    let mut state = MatchState::with_rules(1, rules);
    state.phase = Phase::Running;
    state.players[0].offers = [
        UnitKind::SiegeWalker,
        UnitKind::Archer,
        UnitKind::Shieldbearer,
        UnitKind::Hexer,
        UnitKind::Bloomkeeper,
    ];
    state.players[0].energy = 5 * ENERGY_SCALE;
    let mut controller = Controller::new(Policy::Favor(UnitKind::SiegeWalker), rules, 1, 5);
    let mut view = state.view_for(Seat::A, &[]);
    for _ in 0..12 {
        assert_eq!(
            controller.decide(&view),
            None,
            "redrawing an already offered price canary masks its cost, including fallback ticks"
        );
    }
    view.energy = rules.def(UnitKind::SiegeWalker).cost;
    let action = (0..5).find_map(|_| controller.decide(&view));
    assert_eq!(action, Some(PlayerIntention::Purchase { offer: 0 }));
}

#[test]
fn missing_favored_unit_still_buys_the_cheapest_redraw() {
    let rules = Ruleset::canon();
    let mut state = MatchState::with_rules(2, rules);
    state.phase = Phase::Running;
    state.players[0].offers = [
        UnitKind::Archer,
        UnitKind::Shieldbearer,
        UnitKind::Hexer,
        UnitKind::BoneBrute,
        UnitKind::Rampart,
    ];
    state.players[0].energy = 5 * ENERGY_SCALE;
    let mut controller = Controller::new(Policy::Favor(UnitKind::SiegeWalker), rules, 2, 1);
    assert_eq!(
        controller.decide(&state.view_for(Seat::A, &[])),
        Some(PlayerIntention::Purchase { offer: 0 })
    );
}
