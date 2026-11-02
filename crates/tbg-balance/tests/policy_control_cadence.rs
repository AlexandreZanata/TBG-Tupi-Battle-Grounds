//! A filtered counterfactual control retains the shipped bot's timing when its filter cannot apply.
use tbg_balance::policy::{Controller, Policy};
use tbg_core::{MatchState, Ruleset, Seat, UnitKind};

#[test]
fn an_absent_avoided_kind_preserves_bot_actions_and_cadence() {
    static POOL: [UnitKind; 6] = [
        UnitKind::Shieldbearer,
        UnitKind::Archer,
        UnitKind::Hexer,
        UnitKind::Rootcaller,
        UnitKind::Nester,
        UnitKind::Rampart,
    ];
    let mut rules = Ruleset::canon().clone();
    rules.pool = &POOL;
    let rules = Box::leak(Box::new(rules));
    let mut normal = MatchState::with_rules(29, rules);
    let mut filtered = normal.clone();
    let mut bot = Controller::new(Policy::Bot, rules, 1, 5);
    let mut control = Controller::new(Policy::BotAvoid(UnitKind::Bloomkeeper), rules, 1, 5);
    for _ in 0..360 {
        let a = bot.decide(&normal.view_for(Seat::A, &[]));
        let b = control.decide(&filtered.view_for(Seat::A, &[]));
        assert_eq!(
            a, b,
            "an inactive avoidance filter must not slow the control at clock {}",
            normal.clock
        );
        if let Some(intent) = a {
            let _ = normal.apply_intention(Seat::A, normal.clock.max(1), intent);
        }
        if let Some(intent) = b {
            let _ = filtered.apply_intention(Seat::A, filtered.clock.max(1), intent);
        }
        normal.step();
        filtered.step();
    }
    assert_eq!(normal.hash(), filtered.hash());
}
