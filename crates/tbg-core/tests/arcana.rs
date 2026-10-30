//! Arcana contracts (GAME-DESIGN §9, BR-AR01–BR-AR14, D-40/P40).
//! Append-only: a changed rule needs the GAME-DESIGN update in this commit.

use tbg_core::arcana::{self, ArcanaError, ArcanaPreset, BUDGET, NODES};
use tbg_core::content::{def, AugmentBonus, ARCANA_BP, BP};
use tbg_core::{
    Archetype, MatchState, Phase, PlayerIntention, Ruleset, Seat, Status, UnitKind, UNIT_KINDS,
};

const ALL_ORIGINALS: [u8; NODES] = [2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const WAR_FULL: [u8; NODES] = [2, 2, 2, 2, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0];
const PRECISION_FULL: [u8; NODES] = [2, 2, 2, 2, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0];
const VITALITY_FULL: [u8; NODES] = [2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 1, 1, 1];
const NO_WAR: [u8; NODES] = [0, 0, 2, 2, 2, 2, 0, 0, 0, 1, 1, 1, 0, 0, 0];

#[test]
fn br_ar02_ar03_exactly_1842_complete_presets_with_costs_and_prerequisites() {
    let radix: [u64; NODES] = std::array::from_fn(|i| u64::from(arcana::MAX[i]) + 1);
    let total: u64 = radix.iter().product();
    let mut count = 0;
    for n in 0..total {
        let mut rest = n;
        let ranks: [u8; NODES] = std::array::from_fn(|i| {
            let rank = (rest % radix[i]) as u8;
            rest /= radix[i];
            rank
        });
        count += u32::from(ArcanaPreset::complete(ranks).is_ok());
    }
    assert_eq!(count, 1_842);
    assert_eq!(ArcanaPreset::complete(ALL_ORIGINALS).unwrap().spent(), 12);
    assert_eq!(ArcanaPreset::complete(WAR_FULL).unwrap().spent(), 12);
    assert_eq!(ArcanaPreset::complete(PRECISION_FULL).unwrap().spent(), 12);
    assert_eq!(ArcanaPreset::complete(VITALITY_FULL).unwrap().spent(), 12);
    assert_eq!(
        ArcanaPreset::complete([255; NODES]),
        Err(ArcanaError::BadRanks)
    );
    assert_eq!(
        ArcanaPreset::complete(ArcanaPreset::NEUTRAL.ranks),
        Err(ArcanaError::BadRanks)
    );
    assert!(ArcanaPreset::complete(ArcanaPreset::DEFAULT.ranks).is_ok());
}

#[test]
fn br_ar03_children_and_supremes_require_their_parents_and_completion() {
    // Shieldwall without Guard, with Battle Rush valid and the rest of the
    // budget in the other original nodes.
    let mut no_parent = [0u8; NODES];
    no_parent[arcana::ASSAULT] = 2;
    no_parent[arcana::PIERCING] = 2;
    no_parent[arcana::ENDURANCE] = 2;
    no_parent[arcana::RESILIENCE] = 2;
    no_parent[arcana::RENEWAL] = 2;
    no_parent[arcana::SHIELDWALL] = 1;
    no_parent[arcana::BATTLE_RUSH] = 1;
    assert_eq!(
        ArcanaPreset::complete(no_parent),
        Err(ArcanaError::BadRanks)
    );
    // Combined Arms with both originals but no children.
    let mut no_children = [0u8; NODES];
    no_children[arcana::GUARD] = 2;
    no_children[arcana::ASSAULT] = 2;
    no_children[arcana::PIERCING] = 2;
    no_children[arcana::ENDURANCE] = 2;
    no_children[arcana::HUNTERS_FOCUS] = 1;
    no_children[arcana::QUICKDRAW] = 1;
    no_children[arcana::COMBINED_ARMS] = 1;
    assert_eq!(ArcanaPreset { ranks: no_children }.spent(), 12);
    assert_eq!(
        ArcanaPreset::complete(no_children),
        Err(ArcanaError::BadRanks)
    );
}

#[test]
fn br_ar13_legacy_six_node_presets_migrate_to_twelve_points() {
    for old in [
        [2, 0, 2, 0, 2, 0],
        [2, 2, 2, 0, 0, 0],
        [1, 1, 1, 1, 1, 1],
        [0, 0, 0, 2, 2, 2],
    ] {
        let migrated = ArcanaPreset::migrate_legacy(old);
        assert_eq!(migrated.ranks, ArcanaPreset::DEFAULT.ranks, "{old:?}");
        assert_eq!(migrated.spent(), u32::from(BUDGET));
        assert!(ArcanaPreset::complete(migrated.ranks).is_ok());
    }
    let clamped = ArcanaPreset::migrate_legacy([9, 9, 9, 9, 9, 9]);
    assert_eq!(clamped, ArcanaPreset::DEFAULT);
}

/// Expected base stats with the six original nodes, by hand.
fn expect(kind: UnitKind, ranks: [u8; NODES]) -> (i32, i32, i32) {
    let d = def(kind);
    let per = |node: usize| i64::from(ranks[node]) * i64::from(ARCANA_BP[node]);
    let (hp, dmg, heal) = match d.archetype {
        Archetype::Tank | Archetype::Dps => (per(arcana::GUARD), per(arcana::ASSAULT), 0),
        Archetype::Ranged => (per(arcana::ENDURANCE), per(arcana::PIERCING), 0),
        Archetype::Support | Archetype::Control => {
            let heal = if d.heal > 0 { per(arcana::RENEWAL) } else { 0 };
            (per(arcana::RESILIENCE), 0, heal)
        }
    };
    let f = |base: i32, bp: i64| (i64::from(base) * (BP + bp) / BP) as i32;
    (f(d.hp, hp), f(d.damage, dmg), f(d.heal, heal))
}

#[test]
fn br_ar04_ar05_bonus_by_base_archetype_without_conditional_double_counting() {
    let r = Ruleset::canon();
    for kind in UNIT_KINDS {
        for ranks in [ALL_ORIGINALS, WAR_FULL, PRECISION_FULL, VITALITY_FULL] {
            let p = ArcanaPreset::complete(ranks).unwrap();
            let s = r.stats_with(kind, 0, 0, 0, &p, &AugmentBonus::default());
            assert_eq!(
                (s.max_hp, s.damage, s.heal),
                expect(kind, ranks),
                "{kind:?} {ranks:?}"
            );
        }
    }
}

#[test]
fn br_ar10_deadeye_adds_range_only_to_conventional_ranged() {
    let r = Ruleset::canon();
    let p = ArcanaPreset::complete(PRECISION_FULL).unwrap();
    for kind in [UnitKind::Archer, UnitKind::SiegeWalker] {
        let base = r.effective_stats(kind, 0, 0, 0).range;
        let ranged = r
            .stats_with(kind, 0, 0, 0, &p, &AugmentBonus::default())
            .range;
        assert_eq!(ranged, base + r.arcana.deadeye_range, "{kind:?}");
    }
    for kind in [
        UnitKind::Shieldbearer,
        UnitKind::Rampart,
        UnitKind::Bloomkeeper,
        UnitKind::Rootcaller,
    ] {
        let base = r.effective_stats(kind, 0, 0, 0).range;
        let got = r
            .stats_with(kind, 0, 0, 0, &p, &AugmentBonus::default())
            .range;
        assert_eq!(got, base, "Deadeye reached {kind:?}");
    }
    let summon = tbg_core::Unit {
        id: 1,
        seat: Seat::A,
        kind: UnitKind::Beetling,
        tier: 0,
        branch: 0,
        x: 0,
        hp: 1,
        max_hp: 1,
        cooldown: 0,
        windup: None,
        status: Status::None,
        summoner: None,
        expires_at: None,
        action_cd: 0,
        haste_until: 0,
        heal_cut_until: 0,
        heal_cut_bp: 0,
        second_wind_used: 0,
    };
    let mut s = MatchState::with_arcanas(9, [p, ArcanaPreset::DEFAULT]);
    s.units.push(summon);
    assert_eq!(
        s.stats_of(&summon).range,
        r.effective_stats(UnitKind::Beetling, 0, 0, 0).range
    );
}

#[test]
fn br_ar05_ar06_ar07_ar08_ar09_conditions_are_exact() {
    let r = Ruleset::canon();
    let v = r.arcana;
    let war = ArcanaPreset::complete(WAR_FULL).unwrap();
    let precision = ArcanaPreset::complete(PRECISION_FULL).unwrap();
    let vitality = ArcanaPreset::complete(VITALITY_FULL).unwrap();

    assert_eq!(
        arcana::damage_reduction_bp(&war, &v, Archetype::Tank, true, true),
        1_100
    );
    assert_eq!(
        arcana::damage_reduction_bp(&war, &v, Archetype::Tank, false, true),
        600
    );
    assert_eq!(
        arcana::damage_reduction_bp(&war, &v, Archetype::Tank, true, false),
        0
    );
    assert_eq!(
        arcana::damage_reduction_bp(&war, &v, Archetype::Dps, true, true),
        0
    );
    assert_eq!(
        arcana::damage_bonus_bp(&war, &v, Archetype::Dps, Status::None, true),
        800
    );
    assert_eq!(
        arcana::damage_bonus_bp(&war, &v, Archetype::Dps, Status::None, false),
        0
    );
    assert_eq!(
        arcana::damage_bonus_bp(&precision, &v, Archetype::Ranged, Status::None, false),
        0
    );
    assert_eq!(
        arcana::damage_bonus_bp(
            &precision,
            &v,
            Archetype::Ranged,
            Status::Slowed {
                bp: 3_000,
                ticks_left: 5
            },
            false
        ),
        1_200
    );
    assert_eq!(
        arcana::damage_bonus_bp(
            &precision,
            &v,
            Archetype::Ranged,
            Status::Rooted { ticks_left: 5 },
            false
        ),
        1_200
    );
    assert_eq!(arcana::move_bonus_bp(&war, &v, Archetype::Dps, true), 1_200);
    assert_eq!(arcana::move_bonus_bp(&war, &v, Archetype::Dps, false), 0);
    assert_eq!(
        arcana::attack_speed_bp(&precision, &v, Archetype::Ranged, 2_000),
        1_500
    );
    assert_eq!(
        arcana::attack_speed_bp(&precision, &v, Archetype::Ranged, 2_001),
        0
    );
    assert_eq!(
        arcana::shorten_status(&vitality, &v, Archetype::Support, 16),
        12
    );
    assert_eq!(
        arcana::shorten_status(&vitality, &v, Archetype::Support, 1),
        1
    );
    assert_eq!(
        arcana::shorten_status(&vitality, &v, Archetype::Tank, 16),
        16
    );
    assert_eq!(arcana::emergency_heal_bp(&vitality, &v, 39, 100), 1_500);
    assert_eq!(arcana::emergency_heal_bp(&vitality, &v, 40, 100), 0);
    assert_eq!(
        arcana::purify_status(
            &vitality,
            &v,
            Status::Slowed {
                bp: 3_000,
                ticks_left: 9
            }
        ),
        Some(Status::None)
    );
    assert_eq!(
        arcana::purify_status(&vitality, &v, Status::Rooted { ticks_left: 9 }),
        Some(Status::Rooted { ticks_left: 4 })
    );
    assert_eq!(
        arcana::purify_status(&vitality, &v, Status::Rooted { ticks_left: 1 }),
        Some(Status::Rooted { ticks_left: 1 })
    );
}

#[test]
fn summons_keep_neutral_arcanas_and_presets_apply_per_seat() {
    let no_war = ArcanaPreset::complete(NO_WAR).unwrap();
    let mut s = MatchState::with_arcanas(5, [ArcanaPreset::complete(WAR_FULL).unwrap(), no_war]);
    while s.phase != Phase::Running {
        s.step();
    }
    for seat in Seat::BOTH {
        s.players[seat.index()].offers[0] = UnitKind::Shieldbearer;
        s.players[seat.index()].energy = 6_000;
        s.apply_intention(seat, 1, PlayerIntention::Purchase { offer: 0 })
            .unwrap();
    }
    assert_eq!(s.units[0].max_hp, 650 * 10_800 / 10_000);
    assert_eq!(s.units[1].max_hp, 650);
    let summon = tbg_core::Unit {
        kind: UnitKind::Beetling,
        ..s.units[0]
    };
    assert_eq!(
        s.stats_of(&summon).max_hp,
        s.rules()
            .effective_stats(UnitKind::Beetling, 0, 0, 0)
            .max_hp
    );
}
