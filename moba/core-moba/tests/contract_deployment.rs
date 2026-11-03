//! M-72: stable lane exits and immediate deployment under crowding.
use tbg_moba_core::{
    geom::Pos, map, state::Brain, types::Side, Champion, Destination, Lane, MobaIntention,
    MobaMatch, Role, Seat, Team, UnitKind,
};

#[test]
fn crowded_base_drains_purchased_troops_through_their_own_gate() {
    for team in Team::ALL {
        let mut game = MobaMatch::new(72);
        let seat = Seat::of(team, Role::Top);
        game.units.retain(|u| u.team() == Some(team));
        let lanes = [(Lane::Top, 4), (Lane::Mid, 2), (Lane::Bot, 2)];
        for (lane, count) in lanes {
            for _ in 0..count {
                let player = game.player_mut(seat);
                player.gold = 10_000;
                player.offers[0] = Champion::Colossus;
                game.apply(
                    seat,
                    MobaIntention::BuyTroop {
                        slot: 0,
                        to: Destination::Lane(lane),
                    },
                )
                .unwrap();
            }
        }
        assert_eq!(
            game.units.iter().filter(|u| u.owner == Some(seat)).count(),
            8,
            "every purchase must deploy immediately"
        );
        let starts: Vec<_> = game
            .units
            .iter()
            .filter(|u| u.owner == Some(seat))
            .map(|u| (u.id, u.pos))
            .collect();
        for _ in 0..500 {
            game.step();
        }
        for (id, start) in starts {
            let base = map::stronghold(team);
            let at = game.unit(id).unwrap().pos;
            assert!(
                base.dist(at) > base.dist(start) + 8_000,
                "a crowded troop must make forward progress: {start:?} -> {at:?}"
            );
        }
        for (lane, count) in lanes {
            let base = map::stronghold(team);
            let troops: Vec<_> = game
                .units
                .iter()
                .filter(|u| {
                    u.owner == Some(seat)
                        && matches!(u.brain, Brain::Lane { lane: actual, .. } if actual == lane)
                })
                .collect();
            assert_eq!(
                troops.len(),
                count,
                "all Colossi must deploy through {team:?} {lane:?}"
            );
            assert!(
                troops.iter().all(|u| !u.pos.within(base, 15_000)),
                "troops must clear the base"
            );
        }
    }
}

#[test]
fn a_troop_passes_a_static_blocker_without_turning_back() {
    let mut game = MobaMatch::new(72);
    game.units.clear();
    let start = Pos::units(63, 63);
    let center = Pos::units(67, 67);
    let side = Side::Team(Team::Blue);
    let structure = game.spawn(
        UnitKind::Tower(tbg_moba_core::types::TowerTier::Outer, Some(Lane::Mid)),
        side,
        None,
        center,
        Brain::Static,
    );
    let troop = game.spawn(
        UnitKind::Troop(Champion::Shieldbearer),
        side,
        None,
        start,
        Brain::Lane {
            lane: Lane::Mid,
            waypoint: 2,
        },
    );
    for _ in 0..200 {
        game.step();
    }
    assert_eq!(game.unit(structure).unwrap().pos, center);
    let at = game.unit(troop).unwrap().pos;
    assert!(
        at.x + at.y > center.x + center.y + 5_000,
        "troop must finish the bypass: {at:?}"
    );
}
