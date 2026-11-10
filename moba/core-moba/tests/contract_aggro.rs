//! M-07: committed targets remain stable under competing nearby targets.
use tbg_moba_core::{geom::Pos, state::Brain, types::Side, Champion, MobaMatch, Team, UnitKind};

#[test]
fn a_closer_enemy_does_not_steal_aggro_until_the_target_leaves_pursuit() {
    let mut game = MobaMatch::new(7);
    game.units.clear();
    let spawn = |game: &mut MobaMatch, team, x| {
        game.spawn(
            UnitKind::Troop(Champion::Mantis),
            Side::Team(team),
            None,
            Pos::units(x, 60),
            Brain::Static,
        )
    };
    let attacker = spawn(&mut game, Team::Blue, 60);
    let first = spawn(&mut game, Team::Red, 67);
    let challenger = spawn(&mut game, Team::Red, 68);
    for id in [first, challenger] {
        let slot = game.slot(id).unwrap();
        game.units[slot].stats.damage = 0;
    }
    game.step();
    assert_eq!(game.unit(attacker).unwrap().target, first);

    let slot = game.slot(challenger).unwrap();
    game.units[slot].pos = Pos::units(54, 60);
    for _ in 0..30 {
        game.step();
        assert_eq!(game.unit(attacker).unwrap().target, first);
    }

    let slot = game.slot(first).unwrap();
    game.units[slot].pos = Pos::units(100, 60);
    // The current windup may finish before the next targeting decision.
    for _ in 0..=tbg_core::content::WINDUP {
        game.step();
    }
    assert_eq!(game.unit(attacker).unwrap().target, challenger);
}
