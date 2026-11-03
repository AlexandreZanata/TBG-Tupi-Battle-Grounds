//! MOBA contract tests (append-only, `docs/moba/CONTRACTS.md`). Each test
//! names the rule it guards.

use tbg_moba_core::content::{self, TICK_HZ, TROOP_CAP};
use tbg_moba_core::geom::Pos;
use tbg_moba_core::host;
use tbg_moba_core::protocol::{self, ClientMsg, ServerMsg, WireError, MAX_CLIENT_FRAME};
use tbg_moba_core::state::{Brain, Controller, EndReason};
use tbg_moba_core::types::{Side, TowerTier, UnitKind};
use tbg_moba_core::{
    view, Champion, Destination, Lane, MobaIntention, MobaMatch, Reject, Role, Seat, Team,
};

fn run(m: &mut MobaMatch, ticks: u32) {
    let mut none = Vec::new();
    for _ in 0..ticks {
        host::tick(m, &mut none);
    }
}

#[test]
fn m01_ten_seats_two_teams_five_roles() {
    let seats: Vec<_> = Seat::all().collect();
    assert_eq!(seats.len(), 10);
    assert_eq!(Seat(0).team(), Team::Blue);
    assert_eq!(Seat(5).team(), Team::Red);
    assert_eq!(Seat(4).role(), Role::Support);
    assert_eq!(Seat::of(Team::Red, Role::Mid), Seat(7));
    let m = MobaMatch::new(1);
    assert_eq!(m.towers_left(Team::Blue), 11);
    assert_eq!(m.towers_left(Team::Red), 11);
    assert!(
        !m.units
            .iter()
            .any(|u| matches!(u.kind, UnitKind::Commander(_))),
        "M-01/M-65: Seats have no avatar"
    );
}

#[test]
fn m03_same_seed_and_inputs_replay_identically() {
    let (mut a, mut b) = (MobaMatch::new(42), MobaMatch::new(42));
    for m in [&mut a, &mut b] {
        m.set_controller(Seat(4), Controller::Human);
    }
    for tick in 0..2400u32 {
        let mut inputs = Vec::new();
        if tick % 97 == 0 {
            inputs.push((
                Seat(4),
                MobaIntention::BuyTroop {
                    slot: 0,
                    to: Destination::Lane(Lane::Bot),
                },
            ));
        }
        let mut copy = inputs.clone();
        host::tick(&mut a, &mut inputs);
        host::tick(&mut b, &mut copy);
        assert_eq!(a.state_hash(), b.state_hash(), "diverged at tick {tick}");
    }
}

#[test]
fn m04_bots_never_act_for_human_seats() {
    let mut m = MobaMatch::new(7);
    m.set_controller(Seat(2), Controller::Human);
    run(&mut m, 150);
    assert_eq!(m.troops_alive(Seat(2)), 0);
    assert_eq!(m.player(Seat(2)).gold, content::START_GOLD + 150 / TICK_HZ);
}

#[test]
fn m05_seat_outside_the_match_is_rejected() {
    let mut m = MobaMatch::new(1);
    assert_eq!(
        m.apply(Seat(10), MobaIntention::Upgrade),
        Err(Reject::InvalidSeat)
    );
}

#[test]
fn m08_inner_tower_is_invulnerable_until_the_outer_falls() {
    let mut m = MobaMatch::new(3);
    let inner = m
        .units
        .iter()
        .find(|u| {
            u.side == Side::Team(Team::Blue)
                && u.kind == UnitKind::Tower(TowerTier::Inner, Some(Lane::Top))
        })
        .map(|u| (u.id, u.pos, u.hp))
        .expect("blue top inner tower");
    let near = Pos {
        x: inner.1.x + 2000,
        y: inner.1.y,
    };
    m.spawn(
        UnitKind::Troop(Champion::BoneBrute),
        Side::Team(Team::Red),
        None,
        near,
        Brain::Lane {
            lane: Lane::Top,
            waypoint: 7,
        },
    );
    m.step();
    m.step();
    assert_eq!(
        m.unit(inner.0).map(|u| u.hp),
        Some(inner.2),
        "inner tower took damage behind a standing outer"
    );
    m.units.retain(|u| {
        !(u.side == Side::Team(Team::Blue)
            && u.kind == UnitKind::Tower(TowerTier::Outer, Some(Lane::Top)))
    });
    for _ in 0..40 {
        m.step();
    }
    assert!(
        m.unit(inner.0).is_some_and(|u| u.hp < inner.2),
        "inner tower must be attackable once the outer fell"
    );
}

#[test]
fn m10_buying_spends_the_exact_card_price_of_a_1v1_champion() {
    let mut m = MobaMatch::new(1);
    let seat = Seat(3);
    m.player_mut(seat).gold = 1_000;
    let champion = m.player(seat).offers[2];
    assert!(tbg_moba_core::types::POOL.contains(&champion));
    let to = Destination::Lane(Lane::Bot);
    m.apply(seat, MobaIntention::BuyTroop { slot: 2, to })
        .unwrap();
    assert_eq!(m.player(seat).gold, 1_000 - content::troop_cost(champion));
    assert!(m
        .units
        .iter()
        .any(|u| u.owner == Some(seat) && u.kind == UnitKind::Troop(champion)));
    m.player_mut(seat).gold = 0;
    assert_eq!(
        m.apply(seat, MobaIntention::BuyTroop { slot: 0, to }),
        Err(Reject::NotEnoughGold)
    );
    assert_eq!(
        m.apply(seat, MobaIntention::BuyTroop { slot: 3, to }),
        Err(Reject::InvalidSlot)
    );
}

#[test]
fn m11_troop_cap_limits_alive_troops_per_player() {
    let mut m = MobaMatch::new(1);
    let seat = Seat(0);
    m.player_mut(seat).gold = 10_000;
    let buy = MobaIntention::BuyTroop {
        slot: 0,
        to: Destination::Lane(Lane::Top),
    };
    for _ in 0..TROOP_CAP {
        m.apply(seat, buy).unwrap();
    }
    assert_eq!(m.apply(seat, buy), Err(Reject::TroopCap));
}

#[test]
fn m19_shop_offers_follow_the_1v1_rules_with_locks_and_reroll() {
    let mut m = MobaMatch::new(5);
    let seat = Seat(7);
    let distinct = |o: &[Champion; content::OFFERS]| {
        (0..content::OFFERS).all(|i| (0..i).all(|j| o[i] != o[j]))
    };
    assert!(distinct(&m.player(seat).offers));
    assert_ne!(
        m.player(Seat(0)).offers,
        m.player(Seat(1)).offers,
        "each seat draws its own stream"
    );
    m.player_mut(seat).gold = 1_000;
    m.apply(seat, MobaIntention::ToggleLock { slot: 1 })
        .unwrap();
    let kept = m.player(seat).offers[1];
    m.apply(seat, MobaIntention::Reroll).unwrap();
    assert_eq!(m.player(seat).gold, 1_000 - content::REROLL_COST);
    assert_eq!(
        m.player(seat).offers[1],
        kept,
        "a locked slot survives a reroll"
    );
    assert!(distinct(&m.player(seat).offers));
    let to = Destination::Jungle;
    m.apply(seat, MobaIntention::BuyTroop { slot: 1, to })
        .unwrap();
    assert_eq!(
        m.player(seat).locked & 0b10,
        0,
        "buying a locked slot frees it"
    );
    m.player_mut(seat).gold = 0;
    assert_eq!(
        m.apply(seat, MobaIntention::Reroll),
        Err(Reject::NotEnoughGold)
    );
}

#[test]
fn m17_upgrade_spends_essence_and_caps_at_max_tier() {
    let mut m = MobaMatch::new(1);
    let seat = Seat(6);
    m.player_mut(seat).essence = 1_000;
    for tier in 0..content::MAX_TIER {
        let before = m.player(seat).essence;
        m.apply(seat, MobaIntention::Upgrade).unwrap();
        assert_eq!(m.player(seat).essence, before - content::upgrade_cost(tier));
    }
    assert_eq!(m.apply(seat, MobaIntention::Upgrade), Err(Reject::MaxTier));
}

#[test]
fn m30_a_ten_bot_match_always_ends() {
    let mut m = MobaMatch::new(2026);
    let mut guard = 0;
    while m.outcome.is_none() {
        run(&mut m, 1);
        guard += 1;
        assert!(
            guard <= content::MATCH_TICKS + 1,
            "match exceeded its time limit"
        );
    }
    let outcome = m.outcome.unwrap();
    if outcome.reason == EndReason::StrongholdDestroyed {
        let loser = outcome.winner.unwrap().enemy();
        assert!(!m
            .units
            .iter()
            .any(|u| u.side == Side::Team(loser) && u.kind == UnitKind::Stronghold));
    }
    assert!(m.apply(Seat(0), MobaIntention::Upgrade) == Err(Reject::MatchOver));
}

#[test]
fn m40_frames_round_trip_and_hostile_frames_are_refused() {
    let mut m = MobaMatch::new(9);
    run(&mut m, 300);
    let world = ServerMsg::World(view::world(&m, Team::Blue));
    assert_eq!(
        protocol::decode_server(&protocol::encode(&world)),
        Ok(world)
    );
    let hello = ClientMsg::Hello {
        protocol: protocol::MOBA_PROTOCOL_VERSION,
        name: "Você".into(),
    };
    assert_eq!(
        protocol::decode_client(&protocol::encode(&hello)),
        Ok(hello)
    );
    assert_eq!(
        protocol::decode_client(&vec![0u8; MAX_CLIENT_FRAME + 1]),
        Err(WireError::TooLarge)
    );
    assert_eq!(
        protocol::decode_client(&[250, 255, 255, 255]),
        Err(WireError::Malformed)
    );
    let long = ClientMsg::Hello {
        protocol: 1,
        name: "x".repeat(40),
    };
    assert_eq!(
        protocol::decode_client(&protocol::encode(&long)),
        Err(WireError::BadName)
    );
}

#[test]
fn m20_role_slots_only_offer_the_role_archetypes() {
    // M-19/M-20: locked and redrawn offers remain distinct and inside each Role.
    let mut m = MobaMatch::new(77);
    for seat in Seat::all() {
        m.player_mut(seat).gold = 100_000;
        let rules = content::slot_rules(seat.role());
        for draw in 0..256 {
            let offers = m.player(seat).offers;
            for (i, kind) in offers.iter().enumerate() {
                assert!(rules[i].contains(&tbg_core::content::def(*kind).archetype));
                assert!(!offers[..i].contains(kind));
            }
            assert_eq!(
                view::private(&m, seat).slot_rules,
                rules.map(content::archetype_mask)
            );
            m.apply(
                seat,
                MobaIntention::ToggleLock {
                    slot: (draw % 3) as u8,
                },
            )
            .unwrap();
            m.apply(seat, MobaIntention::Reroll).unwrap();
        }
        assert_eq!(
            m.apply(seat, MobaIntention::ToggleLock { slot: 3 }),
            Err(Reject::InvalidSlot)
        );
        assert_eq!(
            m.apply(
                seat,
                MobaIntention::BuyTroop {
                    slot: 255,
                    to: Destination::Jungle
                }
            ),
            Err(Reject::InvalidSlot)
        );
    }
}

fn jungle_fixture() -> MobaMatch {
    let mut m = MobaMatch::new(60);
    m.units.clear();
    m.tick = content::secs(20) - 1;
    m.step();
    m
}

#[test]
fn m60_each_camp_spawns_its_kind() {
    // M-60: identical camp composition on both sides, no generic placeholders.
    let m = jungle_fixture();
    for camp in 0..tbg_moba_core::map::CAMP_COUNT {
        let expected = content::camp_content(tbg_moba_core::map::camp_kind(camp)).0;
        let actual: Vec<_> = m
            .units
            .iter()
            .filter(|u| u.brain == Brain::Neutral { camp: camp as u8 })
            .map(|u| u.kind)
            .collect();
        assert_eq!(
            actual,
            expected
                .iter()
                .copied()
                .map(UnitKind::Neutral)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn m61_monsters_stay_passive_until_hit() {
    // M-61: a nearby opponent alone never starts a neutral attack.
    let mut m = jungle_fixture();
    let home = tbg_moba_core::map::camp(0).0;
    let id = m.spawn(
        UnitKind::Troop(Champion::Shieldbearer),
        Side::Team(Team::Blue),
        Some(Seat(1)),
        home,
        Brain::Static,
    );
    let i = m.slot(id).unwrap();
    m.units[i].stats.damage = 0;
    for _ in 0..30 {
        m.step();
    }
    let hp = m.unit(id).unwrap().hp;
    assert_eq!(hp, m.unit(id).unwrap().max_hp);
    assert!(m
        .units
        .iter()
        .filter(|u| u.side == Side::Neutral)
        .all(|u| u.target == 0));
    // A jungle attacker actually lands a hit; the victim retaliates.
    let i = m.slot(id).unwrap();
    m.units[i].brain = Brain::Jungle { camp: Some(0) };
    m.units[i].stats.damage = 1;
    m.units[i].stats.range = 6000;
    for _ in 0..100 {
        m.step();
    }
    assert!(m.unit(id).is_none_or(|u| u.hp < hp));
}

#[test]
fn m62_leashed_monsters_heal_home() {
    // M-62: an attacker outside the leash cannot drag a wounded monster away.
    let mut m = jungle_fixture();
    let home = tbg_moba_core::map::camp(0).0;
    let attacker = m.spawn(
        UnitKind::Troop(Champion::Shieldbearer),
        Side::Team(Team::Blue),
        None,
        Pos {
            x: home.x + 20_000,
            y: home.y,
        },
        Brain::Static,
    );
    m.units.last_mut().unwrap().stats.damage = 0;
    let i = m
        .units
        .iter()
        .position(|u| u.brain == Brain::Neutral { camp: 0 })
        .unwrap();
    let monster = m.units[i].id;
    m.units[i].pos.x += 10_000;
    m.units[i].hp = 10;
    m.units[i].target = attacker;
    for _ in 0..100 {
        m.step();
    }
    let u = m.unit(monster).unwrap();
    assert_eq!(u.target, 0);
    assert_eq!(u.hp, u.max_hp);
    assert!(u.pos.within(home, 4000));
}

#[test]
fn m63_cleared_camp_respawns_after_its_timer() {
    // M-63: partial kills do not start a timer; a full clear does, for each kind.
    for camp in 0..4 {
        let mut m = jungle_fixture();
        let ids: Vec<_> = m
            .units
            .iter()
            .filter(|u| u.brain == Brain::Neutral { camp })
            .map(|u| u.id)
            .collect();
        for &id in &ids[..ids.len() - 1] {
            let i = m.slot(id).unwrap();
            m.units[i].hp = 0;
        }
        m.step();
        assert!(m.camps[camp as usize].respawn_at.is_none());
        let i = m.slot(*ids.last().unwrap()).unwrap();
        m.units[i].hp = 0;
        m.step();
        let at = m.camps[camp as usize].respawn_at.unwrap();
        assert_eq!(
            at - m.tick,
            content::camp_content(tbg_moba_core::map::camp_kind(camp as usize)).1
        );
        while m.tick + 1 < at {
            m.step();
        }
        assert!(!m.units.iter().any(|u| u.brain == Brain::Neutral { camp }));
        m.step();
        assert!(m.camps[camp as usize].respawn_at.is_none());
        assert_eq!(
            m.units
                .iter()
                .filter(|u| u.brain == Brain::Neutral { camp })
                .count(),
            ids.len()
        );
    }
}

#[test]
fn m50_enemies_outside_allied_sight_are_absent_from_that_team_frame() {
    // M-50/M-54: inclusive exact sight; hidden ids, HP, targets, events and timers are absent.
    use tbg_moba_core::{state::Event, types::MinionKind};
    let mut m = MobaMatch::new(50);
    m.units.clear();
    let p = Pos {
        x: 100_000,
        y: 100_000,
    };
    let ally = m.spawn(
        UnitKind::Minion(MinionKind::Melee),
        Side::Team(Team::Blue),
        None,
        p,
        Brain::Static,
    );
    let near = m.spawn(
        UnitKind::Troop(Champion::Longaxe),
        Side::Team(Team::Red),
        Some(Seat(5)),
        Pos {
            x: p.x + 8000,
            y: p.y,
        },
        Brain::Static,
    );
    let hidden = m.spawn(
        UnitKind::Troop(Champion::Longaxe),
        Side::Team(Team::Red),
        Some(Seat(5)),
        Pos {
            x: p.x + 8001,
            y: p.y,
        },
        Brain::Static,
    );
    m.units[0].target = hidden;
    m.events.push(Event::TroopSent {
        seat: Seat(5),
        troop: Champion::Longaxe,
        to: Destination::Jungle,
    });
    let w = view::world(&m, Team::Blue);
    assert!(w.units.iter().any(|u| u.id == ally));
    assert!(w.units.iter().any(|u| u.id == near));
    assert!(!w.units.iter().any(|u| u.id == hidden));
    assert_eq!(w.units.iter().find(|u| u.id == ally).unwrap().target, 0);
    assert!(w.events.is_empty());
    assert!(w.camps.is_empty());
    assert_eq!(
        protocol::decode_server(&protocol::encode(&ServerMsg::World(w.clone()))),
        Ok(ServerMsg::World(w))
    );
}

#[test]
fn m51_structures_are_always_in_every_frame() {
    // M-51: map knowledge is public, unseen HP and attacks are not.
    let mut m = MobaMatch::new(51);
    let i = m
        .units
        .iter()
        .position(|u| u.team() == Some(Team::Red) && u.kind.structure())
        .unwrap();
    let id = m.units[i].id;
    m.units[i].hp = 1;
    m.units[i].last_attack = m.tick;
    for team in Team::ALL {
        let w = view::world(&m, team);
        assert_eq!(w.units.iter().filter(|u| u.kind.structure()).count(), 24);
    }
    let blue = view::world(&m, Team::Blue);
    let u = blue.units.iter().find(|u| u.id == id).unwrap();
    assert_eq!(u.hp_permille, 1000);
    assert!(!u.attacked);
    // Enter sight, remember damage; leave sight and subsequent damage stays hidden.
    let observer = m.spawn(
        UnitKind::Minion(tbg_moba_core::types::MinionKind::Melee),
        Side::Team(Team::Blue),
        None,
        m.units[i].pos,
        Brain::Static,
    );
    let a = m.slot(observer).unwrap();
    m.units[a].pos = m.units[i].pos;
    m.units[i].stats.damage = 0;
    m.units[a].stats.damage = 0;
    m.step();
    let observed = view::world(&m, Team::Blue)
        .units
        .iter()
        .find(|u| u.id == id)
        .unwrap()
        .hp_permille;
    let a = m.slot(observer).unwrap();
    m.units[a].pos = Pos { x: 0, y: 0 };
    let i = m.slot(id).unwrap();
    m.units[i].hp = m.units[i].max_hp;
    m.step();
    assert_eq!(
        view::world(&m, Team::Blue)
            .units
            .iter()
            .find(|u| u.id == id)
            .unwrap()
            .hp_permille,
        observed
    );
}

#[test]
fn m53_vision_is_deterministic() {
    // M-53: both team frames are byte-identical across a deterministic replay.
    let (mut a, mut b) = (MobaMatch::new(53), MobaMatch::new(53));
    for _ in 0..600 {
        run(&mut a, 1);
        run(&mut b, 1);
        for team in Team::ALL {
            assert_eq!(
                protocol::encode(&view::world(&a, team)),
                protocol::encode(&view::world(&b, team))
            );
        }
    }
}

#[test]
fn m65_seats_have_no_avatar_and_manual_control_is_rejected() {
    // Owner revision M-01/M-65: strategy only, including hostile legacy movement messages.
    let mut m = MobaMatch::new(65);
    assert!(m.units.iter().all(|u| u.kind.structure()));
    let before = m.state_hash();
    for seat in Seat::all() {
        assert_eq!(
            m.apply(
                seat,
                MobaIntention::MoveCommander {
                    to: Pos {
                        x: i32::MAX,
                        y: i32::MIN
                    }
                }
            ),
            Err(Reject::ManualControlDisabled)
        );
        assert_eq!(
            m.apply(seat, MobaIntention::CommanderAuto),
            Err(Reject::ManualControlDisabled)
        );
    }
    assert_eq!(
        m.state_hash(),
        before,
        "manual commands cannot affect the match"
    );
    run(&mut m, 600);
    assert!(!m
        .units
        .iter()
        .any(|u| matches!(u.kind, UnitKind::Commander(_))));
}

#[test]
fn m65_each_purchase_chooses_its_lane_then_walks_automatically() {
    // M-10/M-65: a Role filters Shop offers, never forces the purchased champion's lane.
    let mut m = MobaMatch::new(65);
    let seat = Seat::of(Team::Blue, Role::Support);
    m.set_controller(seat, Controller::Human);
    m.player_mut(seat).gold = 10_000;
    let mut bought = Vec::new();
    for lane in Lane::ALL {
        m.apply(
            seat,
            MobaIntention::BuyTroop {
                slot: 0,
                to: Destination::Lane(lane),
            },
        )
        .unwrap();
        let u = m.units.last().unwrap();
        assert!(matches!(u.brain,Brain::Lane{lane:l,..} if l==lane));
        bought.push((u.id, u.pos));
    }
    run(&mut m, 100);
    for (id, start) in bought {
        assert!(
            m.unit(id).is_some_and(|u| u.pos != start),
            "the champion must walk without movement inputs"
        );
    }
}

#[test]
fn m14_automatic_champion_kills_keep_card_bounty_and_update_score() {
    // Owner revision M-14/M-65: no avatar bounty/XP; automatic Troop kills drive the scoreboard.
    use tbg_moba_core::state::Credit;
    let mut m = MobaMatch::new(14);
    let victim = Seat(0);
    let killer = Seat(5);
    m.player_mut(victim).gold = 1000;
    m.apply(
        victim,
        MobaIntention::BuyTroop {
            slot: 0,
            to: Destination::Lane(Lane::Mid),
        },
    )
    .unwrap();
    let kind = match m.units.last().unwrap().kind {
        UnitKind::Troop(k) => k,
        _ => unreachable!(),
    };
    m.units.last_mut().unwrap().hp = 0;
    m.units.last_mut().unwrap().last_hit = Some(Credit {
        side: Side::Team(Team::Red),
        seat: Some(killer),
    });
    let gold = m.player(killer).gold;
    m.step();
    assert_eq!(m.player(killer).gold, gold + content::troop_cost(kind) / 3);
    assert_eq!(m.player(killer).kills, 1);
    assert_eq!(m.player(victim).deaths, 1);
    assert_eq!(m.score, [0, 1]);
}
