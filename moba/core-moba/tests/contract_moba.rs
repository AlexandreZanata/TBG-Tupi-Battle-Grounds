//! MOBA contract tests (append-only, `docs/moba/CONTRACTS.md`). Each test
//! names the rule it guards.

use tbg_moba_core::content::{self, TICK_HZ, TROOP_CAP};
use tbg_moba_core::geom::Pos;
use tbg_moba_core::host;
use tbg_moba_core::protocol::{self, ClientMsg, ServerMsg, WireError, MAX_CLIENT_FRAME};
use tbg_moba_core::state::{Brain, Controller, EndReason};
use tbg_moba_core::types::{Side, TowerTier, UnitKind};
use tbg_moba_core::{
    view, Destination, Lane, MobaIntention, MobaMatch, Reject, Role, Seat, Team, TroopKind,
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
    assert!(Seat::all().all(|s| m.unit(m.player(s).commander).is_some()));
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
                    troop: TroopKind::Healer,
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
        UnitKind::Troop(TroopKind::Colossus),
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
fn m10_buying_spends_the_exact_card_price() {
    let mut m = MobaMatch::new(1);
    let seat = Seat(3);
    let before = m.player(seat).gold;
    m.apply(
        seat,
        MobaIntention::BuyTroop {
            troop: TroopKind::Guardian,
            to: Destination::Lane(Lane::Bot),
        },
    )
    .unwrap();
    assert_eq!(
        m.player(seat).gold,
        before - content::troop_cost(TroopKind::Guardian)
    );
    assert_eq!(m.troops_alive(seat), 1);
    m.player_mut(seat).gold = 5;
    assert_eq!(
        m.apply(
            seat,
            MobaIntention::BuyTroop {
                troop: TroopKind::Healer,
                to: Destination::Jungle
            }
        ),
        Err(Reject::NotEnoughGold)
    );
    assert_eq!(m.player(seat).gold, 5);
}

#[test]
fn m11_troop_cap_limits_alive_troops_per_player() {
    let mut m = MobaMatch::new(1);
    let seat = Seat(0);
    m.player_mut(seat).gold = 10_000;
    let buy = MobaIntention::BuyTroop {
        troop: TroopKind::Healer,
        to: Destination::Lane(Lane::Top),
    };
    for _ in 0..TROOP_CAP {
        m.apply(seat, buy).unwrap();
    }
    assert_eq!(m.apply(seat, buy), Err(Reject::TroopCap));
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
    let world = ServerMsg::World(view::world(&m));
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
