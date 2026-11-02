//! TW-01: exact base tower boost, effective Fortifications and distinct passives.
use tbg_core::{content, Ruleset, StrongholdKind};
#[test]
fn tw01_every_base_tower_has_triple_damage_and_fifty_percent_more_printed_hp() {
    for (kind, hp, damage) in [
        (StrongholdKind::Classic, 4500, 53),
        (StrongholdKind::Treasury, 4185, 48),
        (StrongholdKind::Bastion, 5175, 47),
        (StrongholdKind::Artillery, 3825, 58),
        (StrongholdKind::Command, 4320, 48),
    ] {
        let s = tbg_core::stronghold::stats(Ruleset::canon(), kind, 0, 0);
        assert_eq!(s.max_hp, hp * 3 / 2);
        assert_eq!(s.damage, damage * 3);
    }
    assert_eq!(content::FORTIFY_COST, [3600, 6000]);
}
#[test]
fn tw01_owned_branches_raise_power_without_erasing_kind_identity() {
    let r = Ruleset::canon();
    for kind in StrongholdKind::SELECTABLE {
        for branch in 0..2 {
            let a = tbg_core::stronghold::stats(r, kind, 0, branch);
            let b = tbg_core::stronghold::stats(r, kind, 2, branch);
            assert!(b.max_hp >= a.max_hp && b.damage >= a.damage && b.interval <= a.interval);
            assert!(
                b.max_hp > a.max_hp
                    || b.damage > a.damage
                    || b.interval < a.interval
                    || b.range > a.range
                    || b.spoils > a.spoils
                    || b.aura_bp > a.aura_bp
                    || b.aura_range > a.aura_range
            );
        }
    }
    assert_eq!(
        tbg_core::stronghold::stats(r, StrongholdKind::Bastion, 2, 0).armor_bp,
        2500
    );
    assert_eq!(
        tbg_core::stronghold::stats(r, StrongholdKind::Artillery, 2, 0).splash_bp,
        8000
    );
    assert_eq!(
        tbg_core::stronghold::stats(r, StrongholdKind::Treasury, 2, 1).spoils,
        450
    );
    assert_eq!(
        tbg_core::stronghold::stats(r, StrongholdKind::Command, 2, 0).aura_bp,
        3000
    );
}
