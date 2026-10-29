//! P16-D value-engine contracts: exact scenario placement, null controls,
//! intervention bounds, paired PVE, and interaction canaries. These are
//! intended to be append-only like the other lab contracts.

use tbg_balance::knobs::UnitField;
use tbg_balance::policy::Policy;
use tbg_balance::value::estimate::{self, ShadowPoint};
use tbg_balance::value::intervention::Intervention;
use tbg_balance::value::scenario::{Scenario, Spawn};
use tbg_balance::value::{self, validate};
use tbg_core::content::{self, RUNNING_START};
use tbg_core::{MatchState, Phase, Ruleset, Seat, UnitKind};

fn spawn(kind: UnitKind, seat: Seat, x: i32) -> Spawn {
    Spawn {
        tick: RUNNING_START,
        seat,
        kind,
        tier: 0,
        branch: 0,
        x,
        hp: None,
    }
}

fn scenario(name: &'static str, spawns: Vec<Spawn>, horizon: u32) -> Scenario {
    Scenario {
        name,
        spawns,
        horizon,
        stronghold_fire: false,
        regen: false,
    }
}

fn run_with(intervention: Intervention, scenario: &Scenario) -> i64 {
    let rules = intervention
        .rules()
        .expect("intervention is valid")
        .unwrap_or(Ruleset::canon());
    scenario.run(rules).margin
}

#[test]
fn lab_spawn_at_places_a_unit_exactly() {
    let mut state = MatchState::with_rules(0, Ruleset::canon());
    state.tick = RUNNING_START;
    state.phase = Phase::Running;
    let id = state.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 1_234);
    let unit = state.unit_index(id).map(|i| &state.units[i]).unwrap();
    assert_eq!(unit.x, 1_234);
    assert_eq!(unit.kind, UnitKind::Archer);
    assert_eq!(unit.max_hp, content::def(UnitKind::Archer).hp);
}

#[test]
fn scenario_is_deterministic() {
    let s = scenario(
        "archer_vs_shieldbearer",
        vec![
            spawn(UnitKind::Archer, Seat::A, 5_000),
            spawn(UnitKind::Shieldbearer, Seat::B, 10_000),
        ],
        600,
    );
    assert_eq!(s.run(Ruleset::canon()), s.run(Ruleset::canon()));
}

#[test]
fn null_scenario_has_no_side_bias() {
    let s = scenario(
        "mirror",
        vec![
            spawn(UnitKind::Shieldbearer, Seat::A, 5_000),
            spawn(UnitKind::Shieldbearer, Seat::B, 11_000),
        ],
        400,
    );
    let r = s.run(Ruleset::canon());
    assert_eq!(r.margin, 0, "mirrored scenario must be symmetric");
    assert_eq!(r.metrics[0], r.metrics[1]);
}

#[test]
fn zero_intervention_is_the_canon_ruleset() {
    let rules = Intervention::knob(UnitKind::Archer, UnitField::Damage, 0)
        .rules()
        .unwrap()
        .unwrap();
    assert_eq!(rules, Ruleset::canon());
}

#[test]
fn interventions_reject_negative_hp_and_zero_cost() {
    assert!(Intervention::knob(UnitKind::Archer, UnitField::Hp, -10_000)
        .rules()
        .is_err());
    assert!(
        Intervention::knob(UnitKind::Shieldbearer, UnitField::Cost, -10_000)
            .rules()
            .is_err()
    );
}

#[test]
fn pve_of_a_noop_is_zero() {
    let cmp = estimate::compare(
        Intervention::knob(UnitKind::Archer, UnitField::Damage, 0),
        [Policy::Bot, Policy::Bot],
        5,
        0..4,
        2,
    )
    .unwrap();
    assert_eq!(cmp.delta.mean, 0.0);
    assert_eq!(cmp.delta.lo, 0.0);
    assert_eq!(cmp.delta.hi, 0.0);
}

#[test]
fn reference_lambda_is_the_median_and_must_be_positive() {
    let points = [
        ShadowPoint {
            tick: 0,
            lambda: 0.01,
            lo: 0.0,
            hi: 0.02,
        },
        ShadowPoint {
            tick: 600,
            lambda: 0.03,
            lo: 0.02,
            hi: 0.04,
        },
        ShadowPoint {
            tick: 1_200,
            lambda: 0.02,
            lo: 0.01,
            hi: 0.03,
        },
    ];
    assert!((estimate::reference_lambda(&points).unwrap() - 0.02).abs() < f64::EPSILON);
    let bad = [ShadowPoint {
        tick: 0,
        lambda: 0.0,
        lo: 0.0,
        hi: 0.0,
    }];
    assert!(estimate::reference_lambda(&bad).is_err());
}

#[test]
fn damage_plus_attack_speed_has_a_nonzero_interaction() {
    let pair = value::pairs()
        .into_iter()
        .find(|p| p.name == "damage_plus_attack_speed")
        .unwrap();
    let base = pair.scenario.run(Ruleset::canon()).margin;
    let a = run_with(pair.a, &pair.scenario);
    let b = run_with(pair.b, &pair.scenario);
    let both = {
        let a_world = pair.a.rules().unwrap().unwrap();
        let both_world = pair.b.rules_from(a_world).unwrap().unwrap();
        pair.scenario.run(both_world).margin
    };
    let interaction = (both - base) - (a - base) - (b - base);
    assert_ne!(interaction, 0, "damage and attack speed must interact");
}

#[test]
fn unrelated_intervention_has_zero_interaction() {
    let pair = value::pairs()
        .into_iter()
        .find(|p| p.name == "null_hp_plus_unrelated_range")
        .unwrap();
    let base = pair.scenario.run(Ruleset::canon()).margin;
    let a = run_with(pair.a, &pair.scenario);
    let b = run_with(pair.b, &pair.scenario);
    let both = {
        let a_world = pair.a.rules().unwrap().unwrap();
        let both_world = pair.b.rules_from(a_world).unwrap().unwrap();
        pair.scenario.run(both_world).margin
    };
    let interaction = (both - base) - (a - base) - (b - base);
    assert_eq!(
        interaction, 0,
        "an unrelated range change must not interact"
    );
}

#[test]
fn doubled_damage_beats_a_twenty_percent_increase() {
    let s = scenario(
        "archer_vs_rampart",
        vec![
            spawn(UnitKind::Archer, Seat::A, 5_000),
            spawn(UnitKind::Rampart, Seat::B, 10_000),
        ],
        200,
    );
    let plus = scenario_damage(
        &s,
        Intervention::knob(UnitKind::Archer, UnitField::Damage, 9),
    );
    let doubled = scenario_damage(
        &s,
        Intervention::knob(UnitKind::Archer, UnitField::Damage, 45),
    );
    assert!(
        doubled > plus,
        "the x2 canary must out-damage the +20 % change: {doubled} vs {plus}"
    );
}

fn scenario_damage(s: &Scenario, intervention: Intervention) -> i64 {
    let rules = intervention.rules().unwrap().unwrap();
    s.run(rules).metrics[0].unit_damage
}

#[test]
fn static_baseline_is_cost_aware() {
    let catalog = value::catalog();
    let cheap = catalog
        .iter()
        .find(|c| c.name == "canary_shieldbearer_cost_plus_1")
        .unwrap();
    let damage = catalog
        .iter()
        .find(|c| c.name == "canary_archer_damage_x2")
        .unwrap();
    assert!(validate::static_power(damage) > validate::static_power(cheap));
}

#[test]
fn bot_avoid_parses_and_never_buys_the_avoided_unit() {
    use tbg_balance::runner::{run_pairs, Matchup, Setup};
    let policy = Policy::parse("avoid-archer").expect("avoid-archer parses");
    assert_eq!(policy, Policy::BotAvoid(UnitKind::Archer));
    let m = Matchup {
        rules: Ruleset::canon(),
        policies: [policy, Policy::Bot],
        reaction: [5, 5],
        setup: Setup::None,
    };
    let games = run_pairs(&[m], 0..4, 2).remove(0);
    for pair in games {
        for game in pair {
            let archers = game.sides[0].purchases[UnitKind::Archer as usize];
            assert_eq!(archers, 0, "BotAvoid bought the avoided unit");
        }
    }
}
