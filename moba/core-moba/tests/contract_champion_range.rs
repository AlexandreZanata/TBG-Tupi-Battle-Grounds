//! M-18: champion reach scales with its figure and can acquire at full range.
use tbg_moba_core::{
    content, geom::Pos, state::Brain, types::Side, Champion, Lane, MobaMatch, Team, UnitKind,
};

#[test]
fn every_shop_champion_keeps_its_relative_battleline_range() {
    for &kind in tbg_moba_core::types::POOL {
        assert_eq!(
            content::base_stats(UnitKind::Troop(kind)).range,
            tbg_core::content::def(kind).range * content::CHAMPION_FIGURE_SCALE,
            "{kind:?}"
        );
    }
}

#[test]
fn farwatch_starts_its_attack_at_full_range_without_walking_in() {
    let mut game = MobaMatch::new(18);
    game.units.clear();
    let start = Pos::units(40, 40);
    let sniper = game.spawn(
        UnitKind::Troop(Champion::Farwatch),
        Side::Team(Team::Blue),
        None,
        start,
        Brain::Lane {
            lane: Lane::Mid,
            waypoint: 1,
        },
    );
    let reach = game.unit(sniper).unwrap().stats.range;
    let target = game.spawn(
        UnitKind::Troop(Champion::Shieldbearer),
        Side::Team(Team::Red),
        None,
        Pos {
            x: start.x + reach,
            y: start.y,
        },
        Brain::Static,
    );
    let slot = game.slot(target).unwrap();
    game.units[slot].stats.speed = 0;
    let hp = game.units[slot].hp;

    game.step();
    assert_eq!(game.unit(sniper).unwrap().target, target);
    assert_eq!(game.unit(sniper).unwrap().last_attack, game.tick);
    for _ in 0..tbg_core::content::WINDUP {
        game.step();
    }
    assert_eq!(game.unit(sniper).unwrap().pos, start);
    assert!(game.unit(target).unwrap().hp < hp);
}
