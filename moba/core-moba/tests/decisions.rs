//! M-76: coherent commitments, moving endpoints and explicit recovery decisions.
use tbg_moba_core::{
    geom::Pos,
    state::Brain,
    types::{MinionKind, Side},
    Champion, Lane, MobaMatch, Team, UnitKind,
};

fn quiet() -> MobaMatch {
    let mut g = MobaMatch::new(76);
    g.units.clear();
    g.tick = 201;
    for c in &mut g.camps {
        c.respawn_at = Some(u32::MAX);
    }
    g
}
fn walker(g: &mut MobaMatch, kind: UnitKind, team: Team, p: Pos) -> u32 {
    g.spawn(
        kind,
        Side::Team(team),
        None,
        p,
        Brain::Lane {
            lane: Lane::Mid,
            waypoint: 2,
        },
    )
}
fn dummy(g: &mut MobaMatch, p: Pos) -> u32 {
    let id = walker(g, UnitKind::Minion(MinionKind::Melee), Team::Red, p);
    let i = g.slot(id).unwrap();
    g.units[i].stats.speed = 0;
    g.units[i].stats.damage = 0;
    g.units[i].hp = 100_000;
    g.units[i].max_hp = 100_000;
    id
}

#[test]
fn a_moving_contact_point_does_not_release_a_reachable_target() {
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::Shieldbearer),
        Team::Blue,
        Pos::units(60, 60),
    );
    let b = dummy(&mut g, Pos::units(70, 60));
    let alternate = dummy(&mut g, Pos::units(69, 53));
    let i = g.slot(a).unwrap();
    g.units[i].target = b;
    let mut hits = 0;
    for tick in 0..180 {
        let j = g.slot(b).unwrap();
        g.units[j].pos.y += 30;
        g.step();
        let u = g.unit(a).unwrap();
        assert_eq!(
            u.target, b,
            "tick {tick}: moving endpoint lost, alternate={alternate}, unit={u:?}"
        );
        hits += usize::from(u.last_attack == g.tick);
        g.events.clear();
    }
    assert!(hits > 0, "the committed pursuit must result in an attack");
}

fn still(g: &mut MobaMatch, id: u32) {
    let i = g.slot(id).unwrap();
    g.units[i].stats.speed = 0;
    g.units[i].stats.damage = 0;
}

#[test]
fn windup_and_recovery_keep_the_target_despite_pressure_at_range_edge() {
    use tbg_moba_core::{champions, sim::decision::Phase};
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::Shieldbearer),
        Team::Blue,
        Pos::units(60, 60),
    );
    let b = dummy(&mut g, Pos::units(64, 60));
    let c = dummy(&mut g, Pos::units(60, 67));
    let reach = champions::reach(g.unit(a).unwrap(), g.unit(b).unwrap());
    let j = g.slot(b).unwrap();
    g.units[j].pos.x = 60_000 + reach - 10;
    g.step();
    assert!(g.unit(a).unwrap().fx.basic.is_some());
    for _ in 0..tbg_core::content::WINDUP {
        g.step();
        assert_eq!(g.unit(a).unwrap().target, b);
    }
    let hp = g.unit(b).unwrap().hp;
    assert!(hp < 100_000);
    for tick in 0..8 {
        let j = g.slot(b).unwrap();
        g.units[j].pos.x = 60_000
            + reach
            + if tick % 2 == 0 {
                tbg_core::content::occupancy::GAP / 2
            } else {
                -10
            };
        let k = g.slot(c).unwrap();
        g.units[k].pos.y = 63_500;
        g.step();
        let u = g.unit(a).unwrap();
        assert_eq!(u.target, b);
        assert_eq!(
            u.decision.phase,
            Phase::Recover,
            "small range drift must not restart gait"
        );
        assert_eq!(
            g.unit(b).unwrap().hp,
            hp,
            "recovery does not extend legal damage range"
        );
    }
}

#[test]
fn death_and_invalid_carried_targets_release_then_resume_the_lane() {
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::Mantis),
        Team::Blue,
        Pos::units(60, 60),
    );
    let b = dummy(&mut g, Pos::units(65, 60));
    let c = dummy(&mut g, Pos::units(60, 66));
    g.step();
    assert_eq!(g.unit(a).unwrap().target, b);
    let j = g.slot(b).unwrap();
    g.units[j].hp = 0;
    for _ in 0..=tbg_core::content::WINDUP {
        g.step();
    }
    assert_eq!(g.unit(a).unwrap().target, c);
    let j = g.slot(c).unwrap();
    g.units[j].fx.carrier = b;
    for _ in 0..=tbg_core::content::WINDUP {
        g.step();
    }
    assert_eq!(g.unit(a).unwrap().target, 0);
    let before = g.unit(a).unwrap().pos;
    for _ in 0..20 {
        g.step();
    }
    assert!(g.unit(a).unwrap().pos.dist(before) > 500);
    assert!(matches!(
        g.unit(a).unwrap().brain,
        Brain::Lane {
            lane: Lane::Mid,
            ..
        }
    ));
}

#[test]
fn root_cancels_the_hit_and_releases_movement_after_expiry() {
    use tbg_moba_core::sim::decision::Phase;
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::Shieldbearer),
        Team::Blue,
        Pos::units(60, 60),
    );
    let b = dummy(&mut g, Pos::units(63, 60));
    g.step();
    assert!(g.unit(a).unwrap().fx.basic.is_some());
    tbg_moba_core::champions::root(&mut g, a, 12);
    for _ in 0..10 {
        g.step();
        assert_eq!(g.unit(a).unwrap().decision.phase, Phase::Disabled);
        assert!(g.unit(a).unwrap().fx.basic.is_none());
        assert_eq!(g.unit(b).unwrap().hp, 100_000);
    }
    for _ in 0..40 {
        g.step();
    }
    assert!(g.unit(b).unwrap().hp < 100_000);
}

#[test]
fn support_keeps_the_healing_recipient_through_recovery() {
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::GustDrummer),
        Team::Blue,
        Pos::units(60, 60),
    );
    let b = walker(
        &mut g,
        UnitKind::Troop(Champion::Shieldbearer),
        Team::Blue,
        Pos::units(65, 60),
    );
    let c = walker(
        &mut g,
        UnitKind::Troop(Champion::Shieldbearer),
        Team::Blue,
        Pos::units(60, 66),
    );
    for id in [b, c] {
        still(&mut g, id);
        let j = g.slot(id).unwrap();
        g.units[j].stats.speed = 1;
    }
    let j = g.slot(b).unwrap();
    g.units[j].hp -= 200;
    let j = g.slot(c).unwrap();
    g.units[j].hp -= 100;
    g.step();
    assert_eq!(g.unit(a).unwrap().target, b);
    for _ in 0..80 {
        let j = g.slot(c).unwrap();
        g.units[j].hp = 10;
        g.step();
        assert_eq!(
            g.unit(a).unwrap().target,
            b,
            "new injury must not change a recovery aim"
        );
    }
}

#[test]
fn idle_support_holds_at_the_enemy_front_without_losing_its_mission() {
    use tbg_moba_core::sim::decision::Phase;
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::NectarNurse),
        Team::Blue,
        Pos::units(60, 60),
    );
    dummy(&mut g, Pos::units(66, 60));
    let from = g.unit(a).unwrap().pos;
    for _ in 0..40 {
        g.step();
        let u = g.unit(a).unwrap();
        assert_eq!(u.target, 0);
        assert_eq!(u.decision.phase, Phase::Wait);
        assert_eq!(u.pos, from);
        assert!(matches!(
            u.brain,
            Brain::Lane {
                lane: Lane::Mid,
                ..
            }
        ));
    }
}

#[test]
fn temporary_obstruction_replans_without_target_churn() {
    use tbg_moba_core::types::TowerTier;
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::Shieldbearer),
        Team::Blue,
        Pos::units(70, 50),
    );
    let b = dummy(&mut g, Pos::units(60, 50));
    let i = g.slot(a).unwrap();
    g.units[i].target = b;
    let mut walls = Vec::new();
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
        let id = g.spawn(
            UnitKind::Tower(TowerTier::Outer, Some(Lane::Mid)),
            Side::Team(Team::Blue),
            None,
            Pos::units(x, y),
            Brain::Static,
        );
        still(&mut g, id);
        walls.push(id);
    }
    for _ in 0..15 {
        g.step();
        assert_eq!(g.unit(a).unwrap().target, b);
    }
    g.units.retain(|u| !walls.contains(&u.id));
    for _ in 0..100 {
        g.step();
        assert_eq!(g.unit(a).unwrap().target, b);
    }
    assert!(g.unit(b).unwrap().hp < 100_000);
}

#[test]
fn every_shop_champion_leaves_the_nexus_with_its_purchased_mission() {
    use tbg_moba_core::{map, Destination, MobaIntention, Seat};
    let mut failures = Vec::new();
    for kind in tbg_core::types::FULL_POOL {
        let mut g = MobaMatch::new(76);
        g.units.retain(|u| u.team() == Some(Team::Blue));
        let p = g.player_mut(Seat(0));
        p.gold = 10_000;
        p.offers[0] = kind;
        g.apply(
            Seat(0),
            MobaIntention::BuyTroop {
                slot: 0,
                to: Destination::Lane(Lane::Mid),
            },
        )
        .unwrap();
        let id = g
            .units
            .iter()
            .find(|u| u.owner == Some(Seat(0)))
            .unwrap()
            .id;
        let from = g.unit(id).unwrap().pos;
        for _ in 0..160 {
            g.step();
            g.units.retain(|u| u.team() == Some(Team::Blue));
            g.events.clear();
        }
        let Some(u) = g.unit(id) else { continue };
        if u.stats.speed > 0
            && (u.pos.dist(from) < 1000 || !map::surface::walkable(u.pos, u.stats.radius))
        {
            failures.push(format!("{kind:?}: {from:?} -> {:?}", u.pos));
        }
        assert!(matches!(
            u.brain,
            Brain::Lane {
                lane: Lane::Mid,
                ..
            }
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_crowded_five_role_engagement_replays_decisions_and_team_frames() {
    use tbg_moba_core::{host, protocol, view};
    let mut g = MobaMatch::new(11);
    let mut replay = g.clone();
    let mut input = Vec::new();
    for _ in 0..800 {
        host::tick(&mut g, &mut input);
        host::tick(&mut replay, &mut input);
        assert_eq!(g.state_hash(), replay.state_hash());
        assert_eq!(
            protocol::encode(&view::world(&g, Team::Blue)),
            protocol::encode(&view::world(&replay, Team::Blue))
        );
        for u in &g.units {
            if let Some(cast) = u.fx.basic {
                assert_eq!(
                    tbg_moba_core::sim::decision::presentation_target(u),
                    cast.target
                );
            }
        }
    }
}

#[test]
fn moving_pursuits_at_the_bank_and_rim_keep_body_clearance_and_commitment() {
    use tbg_moba_core::map;
    for kind in [Champion::Shieldbearer, Champion::Colossus] {
        for from in [Pos::units(60, 100), Pos::units(13, 60)] {
            let mut g = quiet();
            let a = walker(&mut g, UnitKind::Troop(kind), Team::Blue, from);
            let b = dummy(
                &mut g,
                Pos {
                    x: from.x + 10_000,
                    y: from.y,
                },
            );
            let i = g.slot(a).unwrap();
            g.units[i].target = b;
            for _ in 0..180 {
                let j = g.slot(b).unwrap();
                g.units[j].pos.y += 20;
                g.step();
                let u = g.unit(a).unwrap();
                assert_eq!(
                    u.target, b,
                    "{kind:?} must retain a reachable bank/rim target"
                );
                assert!(map::surface::walkable(u.pos, u.stats.radius));
            }
            assert!(g.unit(b).unwrap().hp < 100_000);
        }
    }
}

#[test]
fn moving_patients_are_followed_without_repeated_release_at_range_boundary() {
    for kind in [Champion::Bloomkeeper, Champion::AntidoteBrewer] {
        let mut g = quiet();
        let a = walker(
            &mut g,
            UnitKind::Troop(kind),
            Team::Blue,
            Pos::units(60, 60),
        );
        let b = walker(
            &mut g,
            UnitKind::Minion(MinionKind::Melee),
            Team::Blue,
            Pos::units(68, 60),
        );
        let j = g.slot(b).unwrap();
        g.units[j].stats.damage = 0;
        g.units[j].stats.speed = 1;
        g.units[j].hp = 100;
        g.units[j].max_hp = 100_000;
        for _ in 0..120 {
            let j = g.slot(b).unwrap();
            g.units[j].pos.x += 50;
            g.units[j].fx.slow_until = g.tick + 40;
            g.units[j].fx.slow = 100;
            g.step();
            assert_eq!(
                g.unit(a).unwrap().target,
                b,
                "{kind:?} flickered at its patient's range edge"
            );
        }
        assert!(
            g.unit(a).unwrap().pos.x > 60_500,
            "the support must follow the patient"
        );
    }
}

#[test]
fn narrow_bridge_centres_remain_connected_for_medium_bodies() {
    use tbg_moba_core::map;
    for kind in [Champion::Bulwark, Champion::Sawbot] {
        let mut g = quiet();
        let a = g.spawn(
            UnitKind::Troop(kind),
            Side::Team(Team::Blue),
            None,
            Pos {
                x: 19_492,
                y: 164_508,
            },
            Brain::Lane {
                lane: Lane::Top,
                waypoint: 4,
            },
        );
        for _ in 0..500 {
            g.step();
            g.units.retain(|u| u.id == a);
            let u = g.unit(a).unwrap();
            assert!(map::surface::walkable(u.pos, u.stats.radius));
        }
        let u = g.unit(a).unwrap();
        assert!(
            u.pos.x + u.pos.y > 220_000,
            "{kind:?} remained at the bank: {u:?}"
        );
    }
}

#[test]
fn cargo_opened_on_a_bridge_places_larger_contents_on_valid_ground() {
    use tbg_moba_core::{map, Seat};
    let mut g = quiet();
    let at = map::bridges().next().unwrap();
    let parent = walker(&mut g, UnitKind::Troop(Champion::Smuggler), Team::Blue, at);
    still(&mut g, parent);
    for _ in 0..32 {
        let id = g.spawn(
            UnitKind::Troop(Champion::Crate),
            Side::Team(Team::Blue),
            Some(Seat(0)),
            at,
            Brain::Lane {
                lane: Lane::Top,
                waypoint: 4,
            },
        );
        let i = g.slot(id).unwrap();
        g.units[i].fx.summon = parent;
        g.units[i].fx.summon_cd = 1;
    }
    g.step();
    let contents = g
        .units
        .iter()
        .filter(|u| u.fx.summon == parent && u.kind != UnitKind::Troop(Champion::Crate))
        .collect::<Vec<_>>();
    assert!(contents
        .iter()
        .any(|u| u.stats.radius > map::BRIDGE_HALF_WIDTH));
    for u in contents {
        assert!(
            map::surface::walkable(u.pos, u.stats.radius),
            "large cargo spawned in water: {u:?}"
        );
        assert!(matches!(
            u.brain,
            Brain::Lane {
                lane: Lane::Top,
                ..
            }
        ));
    }
}

#[test]
fn mission_approaches_a_distant_occupied_front_instead_of_waiting_for_it_to_clear() {
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::Shieldbearer),
        Team::Blue,
        Pos::units(75, 75),
    );
    let i = g.slot(a).unwrap();
    g.units[i].hp = 100_000;
    g.units[i].max_hp = 100_000;
    let mut defenders = Vec::new();
    for (x, y) in [
        (94, 94),
        (100, 94),
        (106, 94),
        (94, 100),
        (106, 100),
        (94, 106),
        (100, 106),
        (106, 106),
    ] {
        let id = dummy(&mut g, Pos::units(x, y));
        let j = g.slot(id).unwrap();
        g.units[j].stats.radius = 5_000;
        defenders.push(id);
    }
    let mut engaged = false;
    for _ in 0..500 {
        g.step();
        engaged |= defenders.contains(&g.unit(a).unwrap().target);
        if defenders
            .iter()
            .any(|id| g.unit(*id).is_some_and(|u| u.hp < 100_000))
        {
            break;
        }
    }
    assert!(
        engaged,
        "a distant crowd must not immobilise the mission at its origin"
    );
    assert!(defenders
        .iter()
        .any(|id| g.unit(*id).is_some_and(|u| u.hp < 100_000)));
}

#[test]
fn lane_detours_around_passive_camps_that_it_must_not_attack() {
    use tbg_moba_core::content;
    for (kind, team, lane, camp, from) in [
        (
            Champion::Ravager,
            Team::Red,
            Lane::Top,
            5,
            Pos {
                x: 84_801,
                y: 133_424,
            },
        ),
        (
            Champion::Leechroot,
            Team::Blue,
            Lane::Bot,
            3,
            Pos {
                x: 113_906,
                y: 67_447,
            },
        ),
    ] {
        let mut g = MobaMatch::new(11);
        g.units.clear();
        g.tick = content::secs(20) - 1;
        g.step();
        g.units.retain(|u| u.brain == Brain::Neutral { camp });
        let a = g.spawn(
            UnitKind::Troop(kind),
            Side::Team(team),
            None,
            from,
            Brain::Lane { lane, waypoint: 4 },
        );
        for _ in 0..500 {
            g.step();
            g.units
                .retain(|u| u.id == a || u.brain == Brain::Neutral { camp });
            g.events.clear();
            assert_eq!(
                g.unit(a).unwrap().target,
                0,
                "a Lane mission must not attack the passive camp"
            );
        }
        let u = g.unit(a).unwrap();
        assert!(
            u.pos.dist(from) > 10_000,
            "passive camp pinned {kind:?}: {u:?}"
        );
        assert!(g
            .units
            .iter()
            .filter(|u| u.brain == Brain::Neutral { camp })
            .all(|u| u.hp == u.max_hp));
    }
}

#[test]
fn patient_returning_to_range_clears_the_previous_unreachable_hint() {
    use tbg_moba_core::types::TowerTier;
    let mut g = quiet();
    let a = walker(
        &mut g,
        UnitKind::Troop(Champion::Bloomkeeper),
        Team::Blue,
        Pos::units(62, 60),
    );
    let b = walker(
        &mut g,
        UnitKind::Minion(MinionKind::Melee),
        Team::Blue,
        Pos::units(75, 60),
    );
    let j = g.slot(b).unwrap();
    g.units[j].hp = 10;
    g.units[j].max_hp = 100_000;
    g.units[j].stats.speed = 1;
    g.units[j].stats.damage = 0;
    g.units[j].fx.root_until = 10_000;
    let wall = g.spawn(
        UnitKind::Tower(TowerTier::Outer, Some(Lane::Mid)),
        Side::Team(Team::Blue),
        None,
        Pos::units(75, 60),
        Brain::Static,
    );
    let j = g.slot(wall).unwrap();
    g.units[j].stats.radius = 10_000;
    g.units[j].stats.damage = 0;
    let i = g.slot(a).unwrap();
    g.units[i].target = b;
    let mut released = false;
    for _ in 0..80 {
        g.step();
        if g.unit(a).unwrap().target == 0 {
            released = true;
            break;
        }
    }
    assert!(released);
    assert_eq!(
        g.unit(a).unwrap().decision.changed_because,
        tbg_moba_core::sim::decision::Reason::Unreachable
    );
    g.units.retain(|u| u.id != wall);
    let from = g.unit(a).unwrap().pos;
    let j = g.slot(b).unwrap();
    g.units[j].pos = Pos {
        x: from.x + 8_900,
        y: from.y,
    };
    g.step();
    assert_eq!(g.unit(a).unwrap().target, b);
    for _ in 0..30 {
        let j = g.slot(b).unwrap();
        g.units[j].pos.x += 80;
        g.step();
        assert_eq!(
            g.unit(a).unwrap().target,
            b,
            "old rejection must not undo successful reacquisition"
        );
    }
}
