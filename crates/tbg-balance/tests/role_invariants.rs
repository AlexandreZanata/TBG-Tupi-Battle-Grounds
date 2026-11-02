//! D-33 / P71 range-policy boundaries: accept the registered exception without widening other kinds.

use tbg_balance::knobs;
use tbg_core::{content::Ability, Ruleset, UnitKind, ALL_KINDS};

#[test]
fn range_policy_accepts_farwatch_only_through_its_registered_exception() {
    let mut rules = Ruleset::canon().clone();
    let farwatch = UnitKind::Farwatch as usize;
    let registered = rules.units[farwatch].range;
    assert!(registered > rules.stronghold_range);
    assert!(!rules.units[farwatch]
        .ability(0, 0)
        .contains(&Ability::NoStronghold));
    assert_eq!(knobs::valid(&rules), Ok(()), "P71's active CANON");

    rules.units[farwatch].range = registered - 1;
    assert_eq!(knobs::valid(&rules), Ok(()));
    rules.units[farwatch].range = registered + 1;
    let error = knobs::valid(&rules).unwrap_err();
    assert!(error.contains("Farwatch: base range"), "{error}");
    assert!(error.contains(&format!("allowed {registered}")), "{error}");
}

#[test]
fn range_policy_rejects_every_other_stronghold_attackers_excess_range() {
    for kind in ALL_KINDS.into_iter().filter(|kind| {
        kind.is_conventional()
            && *kind != UnitKind::Farwatch
            && !Ruleset::canon()
                .def(*kind)
                .ability(0, 0)
                .contains(&Ability::NoStronghold)
    }) {
        let mut rules = Ruleset::canon().clone();
        rules.units[kind as usize].range = rules.stronghold_range + 1;
        let error = knobs::valid(&rules).unwrap_err();
        assert!(
            error.starts_with(&format!("{kind:?}: base range")),
            "{kind:?}: {error}"
        );
    }
}

#[test]
fn range_policy_preserves_the_no_stronghold_exemption() {
    let mut rules = Ruleset::canon().clone();
    let archer = &mut rules.units[UnitKind::Archer as usize];
    archer.range = rules.stronghold_range + 1;
    archer.abilities[0][0] = Ability::NoStronghold;
    assert_eq!(knobs::valid(&rules), Ok(()), "ROSTER-EXPANSION §C.2");
}

#[test]
fn range_policy_keeps_experimental_stronghold_reach_tunable() {
    let mut rules = Ruleset::canon().clone();
    rules.stronghold_range = rules.def(UnitKind::Farwatch).range + 1;
    rules.units[UnitKind::Archer as usize].range = rules.stronghold_range;
    rules.units[UnitKind::Farwatch as usize].range = rules.stronghold_range;
    assert_eq!(
        knobs::valid(&rules),
        Ok(()),
        "ordinary Stronghold range cap"
    );
}
