//! M-73/M-74 regressions: persistent missions, body-safe routes and camp contact.
use tbg_moba_core::{
    content, geom::Pos, map, state::Brain, types::Side, Champion, MobaMatch, Seat, Team, UnitKind,
};

#[test]
fn melee_reaches_damages_and_clears_every_camp() {
    let mut failures = Vec::new();
    for kind in [
        Champion::Shieldbearer,
        Champion::BoneBrute,
        Champion::CargoCrab,
        Champion::Bonehorn,
        Champion::Colossus,
    ] {
        for camp in 0..map::CAMP_COUNT {
            let mut game = MobaMatch::new(73);
            game.units.clear();
            game.tick = content::secs(20) - 1;
            game.step();
            game.units
                .retain(|u| u.brain == Brain::Neutral { camp: camp as u8 });
            for c in 0..map::CAMP_COUNT {
                if c != camp {
                    game.camps[c].respawn_at = Some(u32::MAX);
                }
            }
            let home = map::camp(camp).0;
            let direction = if camp < 4 { -1 } else { 1 };
            let id = game.spawn(
                UnitKind::Troop(kind),
                Side::Team(Team::Blue),
                Some(Seat(0)),
                Pos {
                    x: home.x + direction * 12_000,
                    y: home.y,
                },
                Brain::Jungle {
                    camp: Some(camp as u8),
                },
            );
            let slot = game.slot(id).unwrap();
            game.units[slot].max_hp *= 20;
            game.units[slot].hp = game.units[slot].max_hp;
            game.units[slot].stats.damage *= 4;
            let mut damaged = false;
            let mut cleared = false;
            for _ in 0..content::secs(120) {
                game.step();
                damaged |= game
                    .units
                    .iter()
                    .any(|u| u.brain == Brain::Neutral { camp: camp as u8 } && u.hp < u.max_hp);
                damaged |= game
                    .units
                    .iter()
                    .filter(|u| u.brain == Brain::Neutral { camp: camp as u8 })
                    .count()
                    < content::camp_content(map::camp_kind(camp)).0.len();
                cleared = game.camps[camp].respawn_at.is_some();
                game.events.clear();
                if cleared {
                    break;
                }
            }
            if !damaged || !cleared {
                failures.push(format!(
                    "{kind:?} camp {camp}: damage={damaged} clear={cleared} at={:?}",
                    game.unit(id).map(|u| (u.pos, u.target))
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn quiet(seed: u64) -> MobaMatch {
    let mut game = MobaMatch::new(seed);
    game.units.clear();
    for camp in &mut game.camps {
        camp.respawn_at = Some(u32::MAX);
    }
    // Avoid unrelated automatic waves in these bounded route scenarios.
    game.tick = 201;
    game
}

#[test]
fn jungle_mission_survives_waiting_for_the_first_camp_spawn() {
    let mut game = MobaMatch::new(73);
    let seat = Seat(0);
    game.player_mut(seat).offers[0] = Champion::Shieldbearer;
    game.apply(
        seat,
        tbg_moba_core::MobaIntention::BuyTroop {
            slot: 0,
            to: tbg_moba_core::Destination::Jungle,
        },
    )
    .unwrap();
    let id = game
        .units
        .iter()
        .find(|u| u.owner == Some(seat))
        .unwrap()
        .id;
    for _ in 0..450 {
        game.step();
        assert!(matches!(game.unit(id).unwrap().brain, Brain::Jungle { .. }));
        game.events.clear();
    }
    assert!(game.unit(id).unwrap().pos.dist(map::stronghold(Team::Blue)) > 10_000);
}

#[test]
fn water_and_playable_border_clearance_use_the_entire_body() {
    assert!(!map::surface::walkable(Pos::units(70, 130), 500));
    assert!(map::surface::walkable(Pos::units(100, 100), 6_750));
    let bridge = map::bridges().next().unwrap();
    assert!(map::surface::walkable(bridge, 900));
    assert!(!map::surface::walkable(bridge, 6_750));
    assert!(!map::surface::walkable(Pos::units(8, 50), 2_700));
    assert!(map::surface::walkable(Pos::units(9, 50), 2_700));
    assert!(!map::surface::clear_segment(
        Pos::units(60, 120),
        Pos::units(80, 140),
        900
    ));
}

#[test]
fn large_and_small_lane_walkers_cross_water_and_resume_the_same_mission() {
    use tbg_moba_core::Lane;
    for kind in [
        UnitKind::Troop(Champion::Colossus),
        UnitKind::Minion(tbg_moba_core::types::MinionKind::Melee),
    ] {
        let mut game = quiet(74);
        let id = game.spawn(
            kind,
            Side::Team(Team::Blue),
            None,
            Pos::units(20, 150),
            Brain::Lane {
                lane: Lane::Top,
                waypoint: 4,
            },
        );
        let mut crossed = false;
        for _ in 0..5000 {
            game.step();
            game.units.retain(|u| u.id == id);
            let u = game.unit(id).unwrap();
            assert!(
                map::surface::walkable(u.pos, u.stats.radius),
                "{kind:?} left dry ground: {:?}",
                u.pos
            );
            assert!(matches!(
                u.brain,
                Brain::Lane {
                    lane: Lane::Top,
                    ..
                }
            ));
            crossed |= u.pos.x + u.pos.y > 225_000;
            if u.pos.x > 60_000 && u.pos.y > 165_000 {
                break;
            }
            game.events.clear();
        }
        let at = game.unit(id).unwrap().pos;
        assert!(
            crossed && at.x > 60_000 && at.y > 165_000,
            "{kind:?} did not finish its crossing: {at:?}, {:?}",
            game.unit(id).map(|u| (u.target, u.brain, u.stats.speed))
        );
    }
}

#[test]
fn competing_targets_near_water_and_border_replay_every_decision() {
    for start in [Pos::units(70, 110), Pos::units(10, 50)] {
        let mut game = quiet(74);
        let a = game.spawn(
            UnitKind::Troop(Champion::Shieldbearer),
            Side::Team(Team::Blue),
            Some(Seat(0)),
            start,
            Brain::Lane {
                lane: tbg_moba_core::Lane::Mid,
                waypoint: 2,
            },
        );
        for dy in [8_000, 10_000, 12_000] {
            let id = game.spawn(
                UnitKind::Minion(tbg_moba_core::types::MinionKind::Melee),
                Side::Team(Team::Red),
                None,
                Pos {
                    x: start.x,
                    y: start.y - dy,
                },
                Brain::Static,
            );
            let i = game.slot(id).unwrap();
            game.units[i].stats.speed = 0;
            game.units[i].stats.damage = 0;
        }
        let mut replay = game.clone();
        for _ in 0..400 {
            game.step();
            replay.step();
            assert_eq!(game.state_hash(), replay.state_hash());
            assert_eq!(
                tbg_moba_core::protocol::encode(&tbg_moba_core::view::world(&game, Team::Blue)),
                tbg_moba_core::protocol::encode(&tbg_moba_core::view::world(&replay, Team::Blue))
            );
            let u = game.unit(a).unwrap();
            assert!(map::surface::walkable(u.pos, u.stats.radius));
            game.events.clear();
            replay.events.clear();
        }
    }
}

#[test]
fn unreachable_target_is_released_and_the_lane_mission_continues() {
    use tbg_moba_core::{
        types::{MinionKind, TowerTier},
        Lane,
    };
    let mut game = quiet(74);
    let enemy = game.spawn(
        UnitKind::Minion(MinionKind::Melee),
        Side::Team(Team::Red),
        None,
        Pos::units(60, 50),
        Brain::Static,
    );
    let i = game.slot(enemy).unwrap();
    game.units[i].stats.speed = 0;
    game.units[i].stats.damage = 0;
    for (x, y) in [
        (57, 47),
        (60, 47),
        (63, 47),
        (57, 50),
        (63, 50),
        (57, 53),
        (60, 53),
        (63, 53),
    ] {
        let wall = game.spawn(
            UnitKind::Tower(TowerTier::Outer, Some(Lane::Mid)),
            Side::Team(Team::Blue),
            None,
            Pos::units(x, y),
            Brain::Static,
        );
        let i = game.slot(wall).unwrap();
        game.units[i].stats.damage = 0;
    }
    let a = game.spawn(
        UnitKind::Troop(Champion::Shieldbearer),
        Side::Team(Team::Blue),
        None,
        Pos::units(70, 50),
        Brain::Lane {
            lane: Lane::Mid,
            waypoint: 2,
        },
    );
    let i = game.slot(a).unwrap();
    game.units[i].target = enemy;
    let mut released = false;
    for _ in 0..150 {
        game.step();
        released |= game.unit(a).unwrap().target == 0;
        game.events.clear();
    }
    assert!(released);
    let u = game.unit(a).unwrap();
    assert!(u.pos.y > 55_000, "route must resume: {:?}", u.pos);
    assert!(matches!(
        u.brain,
        Brain::Lane {
            lane: Lane::Mid,
            ..
        }
    ));
}

#[test]
fn mixed_crowd_and_cargo_contents_keep_a_forward_mission() {
    use tbg_moba_core::{Destination, Lane, MobaIntention};
    let mut game = MobaMatch::new(11);
    game.units.retain(|u| u.team() == Some(Team::Blue));
    let kinds = [
        Champion::Rotcap,
        Champion::Rootcaller,
        Champion::Mantis,
        Champion::Leechroot,
        Champion::Colossus,
        Champion::Sawbot,
        Champion::Smuggler,
        Champion::Shieldbearer,
    ];
    for kind in kinds {
        let p = game.player_mut(Seat(0));
        p.gold = 10_000;
        p.offers[0] = kind;
        game.apply(
            Seat(0),
            MobaIntention::BuyTroop {
                slot: 0,
                to: Destination::Lane(Lane::Mid),
            },
        )
        .unwrap();
    }
    let ids = game
        .units
        .iter()
        .filter(|u| u.owner == Some(Seat(0)))
        .map(|u| u.id)
        .collect::<Vec<_>>();
    for _ in 0..900 {
        game.step();
        game.units.retain(|u| u.team() == Some(Team::Blue));
        game.events.clear();
    }
    for id in ids {
        let u = game.unit(id).unwrap();
        assert!(
            u.pos.dist(map::stronghold(Team::Blue)) > 30_000,
            "crowded unit never left the Nexus: {u:?}"
        );
    }
    assert!(
        game.units
            .iter()
            .filter(|u| matches!(u.kind, UnitKind::Troop(_)))
            .all(|u| matches!(
                u.brain,
                Brain::Lane {
                    lane: Lane::Mid,
                    ..
                }
            )),
        "cargo contents must inherit their route"
    );
}
