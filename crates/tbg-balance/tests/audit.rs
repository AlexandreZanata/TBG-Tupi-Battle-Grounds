//! D-33 audit regressions: full coverage, lossless compaction and worker-invariant statistics.
use tbg_balance::{
    gate,
    knobs::{self, Knob, UnitField},
    policy::Policy,
    results::Results,
    runner::{self, Matchup, Setup},
};
use tbg_core::{ArcanaPreset, Ruleset};

#[test]
fn every_active_champion_cost_is_searchable() {
    let rules = Ruleset::canon();
    let knobs = knobs::all(rules);
    for &kind in rules.pool {
        assert!(
            knobs.contains(&Knob::Unit(kind as usize, UnitField::Cost)),
            "{kind:?}"
        );
    }
}

#[test]
fn compaction_preserves_observations_combat_totals_and_parallel_intervals() {
    let pair = Matchup {
        rules: Ruleset::canon(),
        policies: [Policy::Bot, Policy::Evolver],
        reaction: [5; 2],
        setup: Setup::None,
        arcanas: [ArcanaPreset::NEUTRAL; 2],
    };
    let raw = runner::run_pairs(&[pair], 40..43, 1).remove(0);
    assert_eq!(raw, runner::run_pairs(&[pair], 40..43, 4).remove(0));
    let total: u64 = raw
        .iter()
        .flat_map(|p| p.iter())
        .flat_map(|g| g.stats.as_ref().unwrap().dealt.iter().flatten())
        .sum();
    let winners: Vec<_> = raw.iter().map(|p| [p[0].winner, p[1].winner]).collect();
    let mut results = Results::default();
    results.insert((Policy::Bot, Policy::Evolver), raw);
    assert_eq!(results.combat.dealt.iter().flatten().sum::<u64>(), total);
    let stored = &results.pairs[&(Policy::Bot, Policy::Evolver)];
    assert_eq!(
        stored
            .iter()
            .map(|p| [p[0].winner, p[1].winner])
            .collect::<Vec<_>>(),
        winners
    );
    assert!(stored.iter().flatten().all(|g| g.stats.is_none()));
    let serial = tbg_balance::report::rows_csv(&gate::rows_with_threads(&results, None, 50, 1));
    let parallel = tbg_balance::report::rows_csv(&gate::rows_with_threads(&results, None, 50, 4));
    assert_eq!(serial, parallel);
    eprintln!(
        "Compact bytes/game: {}; removed per-game combat bytes: {}",
        std::mem::size_of::<tbg_balance::metrics::MatchResult>(),
        std::mem::size_of::<tbg_core::lab::LabStats>()
    );
}

#[test]
fn missing_evidence_cannot_improve_search_loss() {
    let row = gate::Row {
        name: "unobserved cohort".into(),
        value: f64::NAN,
        interval: None,
        band: (0.45, 0.55),
        unit: gate::Unit::Percent,
        searched: true,
    };
    assert!(!row.pass());
    assert!(gate::loss(&[row]).is_infinite());
}

#[test]
fn every_pair_composition_keeps_a_distinct_policy_identity() {
    let mut seen = std::collections::BTreeSet::new();
    let kinds = Ruleset::canon().pool;
    for (i, &a) in kinds.iter().enumerate() {
        for &b in &kinds[i + 1..] {
            assert!(
                seen.insert(Policy::Comp([Some(a), Some(b), None])),
                "{a:?}/{b:?} collided"
            );
        }
    }
    assert_eq!(seen.len(), kinds.len() * (kinds.len() - 1) / 2);
}

#[test]
fn roster_health_retains_the_owner_approved_normalized_floor() {
    assert_eq!(gate::purchase_share_floor(25), 0.025);
    assert!((gate::purchase_share_floor(46) * 46.0 - 0.625).abs() < 1e-12);
    assert!(gate::purchase_share_floor(46) < 1.0 / 46.0);
}

#[test]
fn partial_screening_cannot_claim_strict_gate_acceptance() {
    let args = tbg_balance::cli::parse(
        ["gate", "--scope", "suite", "--strict"]
            .into_iter()
            .map(str::to_owned),
    )
    .unwrap();
    let error = tbg_balance::commands::dispatch(&args).unwrap_err();
    assert!(error.contains("full pool"), "{error}");
}

#[test]
fn every_active_kind_receives_experimental_hp_in_the_real_simulation() {
    // D-33: expanded content must read the supplied Ruleset, not hidden CANON values.
    for &kind in Ruleset::canon().pool {
        let mut changed = Ruleset::canon().clone();
        let expected = changed.def(kind).hp * 3;
        knobs::set(
            &mut changed,
            Knob::Unit(kind as usize, UnitField::Hp),
            i64::from(expected),
        );
        let rules = Box::leak(Box::new(changed));
        let mut game = tbg_core::MatchState::with_rules(7, rules);
        let id = game.lab_spawn(tbg_core::Seat::A, kind, 0, 0);
        let spawned = game.units.iter().find(|unit| unit.id == id).unwrap();
        assert_eq!(spawned.hp, expected, "{kind:?}: ignored experimental HP");
    }
}
