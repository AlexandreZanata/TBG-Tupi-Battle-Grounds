//! OG-01 Proposed: range equality, appended identities and the private 72-champion roster.
use tbg_core::{
    content::{def, effective_stats},
    Archetype, Faction,
    UnitKind::*,
    COMPLETION_POOL, FACTIONS, FULL_POOL,
};
#[test]
fn ogre_range_matches_archer_in_every_evolution() {
    for (t, b) in [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)] {
        assert_eq!(
            effective_stats(SiegeWalker, t, b, 0).range,
            effective_stats(Archer, t, b, 0).range
        );
    }
    assert_eq!(def(SiegeWalker).faction, Faction::Ogres);
}
#[test]
fn ogre_roster_has_nine_per_faction_and_balanced_roles() {
    assert_eq!(COMPLETION_POOL.len(), 72);
    assert_eq!(FACTIONS.len(), 8);
    assert_eq!(FULL_POOL.len(), 46);
    assert_eq!(Faction::Ogres.index(), 7);
    assert_eq!(SiegeWalker as usize, 5);
    for faction in FACTIONS {
        assert_eq!(
            COMPLETION_POOL
                .iter()
                .filter(|&&k| def(k).faction == faction)
                .count(),
            9,
            "{faction:?}"
        );
    }
    let counts = [
        Archetype::Tank,
        Archetype::Dps,
        Archetype::Ranged,
        Archetype::Control,
        Archetype::Support,
    ]
    .map(|role| {
        COMPLETION_POOL
            .iter()
            .filter(|&&k| def(k).faction == Faction::Ogres && def(k).archetype == role)
            .count()
    });
    assert_eq!(counts, [1, 3, 2, 2, 1]);
    for k in [
        BasaltBulwark,
        CinderMaul,
        HookBrawler,
        FlintRunner,
        ClayBinder,
        GustDrummer,
        HideMender,
        PebbleSlinger,
        RivetBallista,
    ] {
        assert!(!FULL_POOL.contains(&k));
        assert!(COMPLETION_POOL.contains(&k));
    }
    assert_eq!(def(RivetBallista).faction, Faction::Machines);
}
