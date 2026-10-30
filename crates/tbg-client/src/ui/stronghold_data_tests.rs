//! Contract: every number in `assets/stronghold/strongholds.json` (the
//! lobby picker and the desktop Fortification panel) equals `tbg-core`
//! (BR-SK01–SK07, D-46), so the web copy can never drift from the rules.

use super::json_lite::{parse, Json};
use tbg_core::content::{
    Fortification, BOUNTY_WINDOW_TICKS, ENERGY_SCALE, FORTIFY_COST, STRONGHOLD_KINDS, TICK_HZ,
};
use tbg_core::stronghold::{stats, StrongholdStats};
use tbg_core::{Ruleset, StrongholdKind};

fn data() -> Json {
    let text = include_str!("../../assets/stronghold/strongholds.json");
    parse(text).unwrap_or_else(|e| panic!("strongholds.json: {e}"))
}

fn int(value: &Json, what: &str) -> i64 {
    value
        .int()
        .unwrap_or_else(|| panic!("{what} is not an integer"))
}

fn assert_stats(json: &Json, expected: StrongholdStats, what: &str) {
    let fields = [
        ("maxHp", i64::from(expected.max_hp)),
        ("damage", i64::from(expected.damage)),
        ("interval", i64::from(expected.interval)),
        ("range", i64::from(expected.range)),
        ("spoils", i64::from(expected.spoils)),
        ("spoilsCap", i64::from(expected.spoils_cap)),
        ("auraBp", i64::from(expected.aura_bp)),
        ("auraRange", i64::from(expected.aura_range)),
    ];
    assert_eq!(json.keys().len(), fields.len(), "{what}: stats fields");
    for (key, value) in fields {
        assert_eq!(int(json.get(key), key), value, "{what}.{key}");
    }
}

/// Only the step's non-zero fields are written; a missing one means 0.
fn assert_delta(json: &Json, step: &Fortification, what: &str) {
    let fields = [
        ("hpBp", i64::from(step.hp_bp)),
        ("dmgBp", i64::from(step.dmg_bp)),
        ("rangeAdd", i64::from(step.range_add)),
        ("interval", i64::from(step.interval)),
        ("spoilsAdd", i64::from(step.spoils_add)),
        ("spoilsCapAdd", i64::from(step.spoils_cap_add)),
        ("auraBpAdd", i64::from(step.aura_bp_add)),
        ("auraRangeAdd", i64::from(step.aura_range_add)),
    ];
    for key in json.keys() {
        assert!(
            fields.iter().any(|(k, _)| *k == key),
            "{what}: unknown {key}"
        );
    }
    for (key, value) in fields {
        assert_eq!(json.get(key).int().unwrap_or(0), value, "{what}.{key}");
    }
}

#[test]
fn picker_data_matches_core_constants() {
    let d = data();
    assert_eq!(d.get("storageKey").str(), Some("tbg.stronghold"));
    let default = format!("{:?}", StrongholdKind::DEFAULT);
    assert_eq!(d.get("defaultKind").str(), Some(default.as_str()));
    assert_eq!(
        int(d.get("energyScale"), "energyScale"),
        i64::from(ENERGY_SCALE)
    );
    assert_eq!(int(d.get("tickHz"), "tickHz"), i64::from(TICK_HZ));
    assert_eq!(
        int(d.get("windowTicks"), "windowTicks"),
        i64::from(BOUNTY_WINDOW_TICKS)
    );
    let costs: Vec<i64> = d
        .get("fortifyCost")
        .list()
        .iter()
        .map(|c| int(c, "cost"))
        .collect();
    let canon: Vec<i64> = FORTIFY_COST.iter().map(|c| i64::from(*c)).collect();
    assert_eq!(costs, canon);
}

#[test]
fn every_kind_tier_and_branch_matches_core_stats() {
    // BR-SK01: exactly the four selectable kinds, in order; never Classic.
    let rules = Ruleset::canon();
    let d = data();
    let kinds = d.get("kinds").list();
    assert_eq!(kinds.len(), StrongholdKind::SELECTABLE.len());
    for (json, kind) in kinds.iter().zip(StrongholdKind::SELECTABLE) {
        let def = &STRONGHOLD_KINDS[kind.index()];
        let name = format!("{kind:?}");
        assert_eq!(json.get("kind").str(), Some(name.as_str()));
        assert_eq!(json.get("name").str(), Some(def.name));
        assert_stats(json.get("stats"), stats(rules, kind, 0, 0), &name);
        let branches = json.get("branches").list();
        assert_eq!(branches.len(), 2, "{name} branches");
        for (b, branch) in branches.iter().enumerate() {
            assert_eq!(branch.get("name").str(), Some(def.branch_names[b]));
            let steps = branch.get("steps").list();
            assert_eq!(steps.len(), 2, "{name} branch {b} steps");
            for (t, step) in steps.iter().enumerate() {
                let what = format!("{name} branch {b} tier {}", t + 1);
                let core = &def.branches[b][t];
                assert_eq!(step.get("name").str(), Some(core.name), "{what}");
                assert_delta(step.get("delta"), core, &what);
                // BR-SK06: tier 2 includes tier 1.
                assert_stats(
                    step.get("stats"),
                    stats(rules, kind, t as u8 + 1, b as u8),
                    &what,
                );
            }
        }
    }
}
