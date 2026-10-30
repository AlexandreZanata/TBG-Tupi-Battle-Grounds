//! The `comp-*` lab policy (P42): name/parse round-trip and rejections.

use tbg_balance::policy::Policy;
use tbg_core::UnitKind;

#[test]
fn comp_name_and_parse_round_trip() {
    let two = Policy::Comp([Some(UnitKind::Archer), Some(UnitKind::BoneBrute), None]);
    assert_eq!(two.name(), "comp-archer-bonebrute");
    assert_eq!(Policy::parse("comp-archer-bonebrute"), Some(two));
    let three = Policy::Comp([
        Some(UnitKind::SiegeWalker),
        Some(UnitKind::Archer),
        Some(UnitKind::Rampart),
    ]);
    assert_eq!(three.name(), "comp-siegewalker-archer-rampart");
    assert_eq!(
        Policy::parse("comp-siegewalker-archer-rampart"),
        Some(three)
    );
    // The §E.5 short name parses to the full kind.
    assert_eq!(
        Policy::parse("comp-archer-antidote"),
        Some(Policy::Comp([
            Some(UnitKind::Archer),
            Some(UnitKind::AntidoteBrewer),
            None
        ]))
    );
    // Order is kept: the reverse names a different policy.
    let reversed = Policy::parse("comp-bonebrute-archer").expect("reversed comp parses");
    assert_eq!(reversed.name(), "comp-bonebrute-archer");
    assert_ne!(reversed, two);
    // Every conventional pair parses (staged kinds included).
    let kinds: Vec<UnitKind> = tbg_core::ALL_KINDS
        .into_iter()
        .filter(|k| k.is_conventional())
        .collect();
    for &a in &kinds {
        for &b in &kinds {
            if a == b {
                continue;
            }
            let name = format!(
                "comp-{}-{}",
                format!("{a:?}").to_lowercase(),
                format!("{b:?}").to_lowercase()
            );
            assert_eq!(Policy::parse(&name).map(|p| p.name()), Some(name));
        }
    }
}

#[test]
fn comp_parse_rejects_bad_names() {
    // Unknown kind, wrong arity, duplicates, summons.
    for bad in [
        "comp-archer",
        "comp-archer-bonebrute-sawbot-rampart",
        "comp-",
        "comp-archer-archer",
        "comp-archer-beetling",
        "comp-beetling-nester",
        "comp-archer-foobar",
        "comp-foobar-bonebrute",
    ] {
        assert_eq!(Policy::parse(bad), None, "{bad} must not parse");
    }
}
