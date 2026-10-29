//! Arcana contracts (GAME-DESIGN §9, BR-AR01–BR-AR10, D-40). Append-only.

use tbg_core::arcana::{self, ArcanaError};
use tbg_core::content::{def, AugmentBonus, ARCANA_BP, BP};
use tbg_core::{
    ArcanaPreset, Archetype, MatchState, Phase, PlayerIntention, Ruleset, Seat, UnitKind,
    UNIT_KINDS,
};

#[test]
fn br_ar02_exactly_141_complete_presets_and_no_panic() {
    let mut complete = 0;
    for n in 0..4u32.pow(6) {
        let ranks: [u8; 6] = std::array::from_fn(|i| (n / 4u32.pow(i as u32) % 4) as u8);
        let ok =
            ranks.iter().all(|&r| r <= 2) && ranks.iter().map(|&r| u32::from(r)).sum::<u32>() == 6;
        let got = ArcanaPreset::complete(ranks);
        assert_eq!(got.is_ok(), ok, "{ranks:?}");
        complete += u32::from(ok);
    }
    assert_eq!(complete, 141);
    assert_eq!(ArcanaPreset::complete([255; 6]), Err(ArcanaError::BadRanks));
    assert!(ArcanaPreset::complete(ArcanaPreset::DEFAULT.ranks).is_ok());
    assert!(ArcanaPreset::complete(ArcanaPreset::NEUTRAL.ranks).is_err());
}

/// Expected base stats with a preset, by hand from GAME-DESIGN §9.
fn expect(kind: UnitKind, ranks: [u8; 6]) -> (i32, i32, i32) {
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
fn br_ar03_ar04_bonus_by_base_archetype_on_every_unit() {
    let r = Ruleset::canon();
    for kind in UNIT_KINDS {
        for ranks in [
            [2, 2, 2, 0, 0, 0],
            [0, 0, 0, 2, 2, 2],
            [1, 1, 1, 1, 1, 1],
            [2, 0, 2, 0, 2, 0],
        ] {
            let p = ArcanaPreset::complete(ranks).unwrap();
            let s = r.stats_with(kind, 0, 0, 0, &p, &AugmentBonus::default());
            assert_eq!(
                (s.max_hp, s.damage, s.heal),
                expect(kind, ranks),
                "{kind:?} {ranks:?}"
            );
            // Nothing outside HP, damage, healing (BR-AR10).
            let n = r.effective_stats(kind, 0, 0, 0);
            assert_eq!(
                (s.interval, s.range, s.speed, s.splash, s.on_hit),
                (n.interval, n.range, n.speed, n.splash, n.on_hit)
            );
        }
    }
}

#[test]
fn br_ar04_bp_add_to_evolution_and_synergy_then_truncate_once() {
    // Shieldbearer Bulwark (+50 % HP), Humans level 1 (+20 % HP), Guard 2 (+8 %).
    let r = Ruleset::canon();
    let p = ArcanaPreset::complete([2, 0, 2, 0, 2, 0]).unwrap();
    let s = r.stats_with(
        UnitKind::Shieldbearer,
        1,
        0,
        1,
        &p,
        &AugmentBonus::default(),
    );
    assert_eq!(s.max_hp, 650 * (10_000 + 5_000 + 2_000 + 800) / 10_000);
}

#[test]
fn summons_get_no_arcana_and_presets_apply_per_seat() {
    let no_war = ArcanaPreset::complete([0, 0, 2, 2, 2, 0]).unwrap();
    let mut s = MatchState::with_arcanas(
        5,
        [ArcanaPreset::complete([2, 2, 0, 0, 2, 0]).unwrap(), no_war],
    );
    while s.phase != Phase::Running {
        s.step();
    }
    for seat in Seat::BOTH {
        s.players[seat.index()].offers[0] = UnitKind::Shieldbearer;
        s.players[seat.index()].energy = 6_000;
        s.apply_intention(seat, 1, PlayerIntention::Purchase { offer: 0 })
            .unwrap();
    }
    // Seat A: Guard 2 (+8 % HP); seat B: no War points.
    assert_eq!(s.units[0].max_hp, 650 * 10_800 / 10_000);
    assert_eq!(s.units[1].max_hp, 650);
    let beetle = s
        .rules()
        .effective_stats(UnitKind::Beetling, 0, 0, 0)
        .max_hp;
    let u = tbg_core::Unit {
        kind: UnitKind::Beetling,
        ..s.units[0]
    };
    assert_eq!(
        s.stats_of(&u).max_hp,
        beetle,
        "a summon took the seat's Arcanas"
    );
}
