//! AB-01: the transferred Trident and Plant replacement preserve nine champions per Faction.
use tbg_core::{
    content::def, Archetype, Faction, UnitKind::*, COMPLETION_POOL, FACTIONS, FULL_POOL,
};
#[test]
fn ab01_trident_moves_to_abyssals_and_pod_snapper_replaces_its_plant_slot() {
    assert_eq!(def(Trident).faction, Faction::Abyssals);
    assert_eq!(def(PodSnapper).faction, Faction::Plants);
    assert_eq!(def(PodSnapper).archetype, Archetype::Ranged);
    assert_eq!(FULL_POOL.len(), 46);
    assert!(!FULL_POOL.contains(&PodSnapper));
    for f in FACTIONS {
        assert_eq!(
            COMPLETION_POOL
                .iter()
                .filter(|k| def(**k).faction == f)
                .count(),
            // GD-04 grows Gods from Colossus alone to Colossus, Volthar and Astreon.
            if f == Faction::Gods { 3 } else { 9 },
            "{f:?}"
        );
    }
    let counts = [
        Archetype::Tank,
        Archetype::Dps,
        Archetype::Ranged,
        Archetype::Control,
        Archetype::Support,
    ]
    .map(|a| {
        COMPLETION_POOL
            .iter()
            .filter(|k| {
                let d = def(**k);
                d.faction == Faction::Abyssals && d.archetype == a
            })
            .count()
    });
    assert_eq!(counts, [1, 3, 2, 2, 1]);
    for k in [
        NacreDuelist,
        ReefLancer,
        TrenchFist,
        PearlSlinger,
        AbyssHarpooner,
        UndertowHerald,
        TrenchLamplighter,
        FoamWeaver,
    ] {
        assert!(COMPLETION_POOL.contains(&k));
        assert!(!FULL_POOL.contains(&k));
    }
}
