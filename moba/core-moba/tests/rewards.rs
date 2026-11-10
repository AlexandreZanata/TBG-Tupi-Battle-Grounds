//! M-13/M-15/M-64/M-75: one authoritative receipt per victim, owner and Team.
use tbg_moba_core::{
    champions::{
        self,
        state::{Pulse, PulseKind},
    },
    content,
    geom::Pos,
    state::{Brain, Credit, Event},
    types::{MinionKind, NeutralKind, Side, TowerTier},
    view, Champion, Lane, MobaMatch, Seat, Team, UnitKind,
};

fn arena() -> MobaMatch {
    let mut g = MobaMatch::new(75);
    g.units.clear();
    for c in &mut g.camps {
        c.respawn_at = Some(u32::MAX);
    }
    g
}
fn victim(g: &mut MobaMatch, kind: UnitKind, at: Pos) -> u32 {
    let side = if matches!(kind, UnitKind::Neutral(_)) {
        Side::Neutral
    } else {
        Side::Team(Team::Red)
    };
    let id = g.spawn(kind, side, None, at, Brain::Static);
    let i = g.slot(id).unwrap();
    g.units[i].stats.speed = 0;
    g.units[i].stats.damage = 0;
    g.units[i].hp = 1;
    id
}
fn attacker(g: &mut MobaMatch, owner: Option<Seat>) -> u32 {
    g.spawn(
        UnitKind::Troop(Champion::Shieldbearer),
        Side::Team(Team::Blue),
        owner,
        Pos::units(40, 40),
        Brain::Jungle { camp: None },
    )
}
fn gold_events(g: &MobaMatch) -> usize {
    g.events
        .iter()
        .filter(|e| matches!(e, Event::GoldGranted { .. }))
        .count()
}

#[test]
fn direct_owned_summon_and_ownerless_last_hits_pay_only_the_correct_owner() {
    for summon in [false, true] {
        for owner in [None, Some(Seat(3))] {
            let mut g = arena();
            let a = attacker(&mut g, owner);
            if summon {
                let i = g.slot(a).unwrap();
                g.units[i].fx.summon = 9999;
            }
            let b = victim(
                &mut g,
                UnitKind::Minion(MinionKind::Melee),
                Pos::units(43, 40),
            );
            let source = g.unit(a).unwrap().clone();
            champions::hurt(&mut g, &source, b, 1000);
            champions::hurt(&mut g, &source, b, 1000);
            g.step();
            for seat in Seat::all() {
                assert_eq!(
                    g.player(seat).gold,
                    content::START_GOLD + if Some(seat) == owner { 4 } else { 0 }
                );
            }
            assert_eq!(gold_events(&g), usize::from(owner.is_some()));
            let earned = g.players.iter().map(|p| p.gold).collect::<Vec<_>>();
            g.events.clear();
            g.step();
            assert_eq!(gold_events(&g), 0);
            assert_eq!(earned, g.players.iter().map(|p| p.gold).collect::<Vec<_>>());
        }
    }
}

#[test]
fn every_monster_pays_its_own_gold_and_essence_once() {
    for camp in 0..4 {
        let mut g = arena();
        let a = attacker(&mut g, Some(Seat(1)));
        let source = g.unit(a).unwrap().clone();
        let mut total = (0, 0);
        let kinds = content::camp_content(tbg_moba_core::map::camp_kind(camp)).0;
        for &kind in kinds {
            let b = victim(&mut g, UnitKind::Neutral(kind), Pos::units(43, 40));
            champions::hurt(&mut g, &source, b, 10_000);
            let reward = content::neutral_rewards(kind);
            total.0 += reward.0;
            total.1 += reward.1;
        }
        g.step();
        assert_eq!(g.player(Seat(1)).gold, content::START_GOLD + total.0);
        assert_eq!(g.player(Seat(1)).essence, total.1);
        assert_eq!(gold_events(&g), kinds.len());
    }
}

#[test]
fn delayed_lethal_pulse_keeps_credit_after_its_source_dies_and_redacts_hidden_origin() {
    let mut g = arena();
    let a = attacker(&mut g, Some(Seat(2)));
    let b = victim(
        &mut g,
        UnitKind::Neutral(NeutralKind::Jaguar),
        Pos::units(150, 150),
    );
    let i = g.slot(b).unwrap();
    g.units[i].fx.pulses.push(Pulse {
        kind: PulseKind::Poison,
        amount: 1000,
        left: 1,
        next: 3,
        gap: 20,
        credit: Credit {
            side: Side::Team(Team::Blue),
            seat: Some(Seat(2)),
        },
    });
    let i = g.slot(a).unwrap();
    g.units[i].hp = 0;
    g.step();
    assert!(g.unit(a).is_none());
    g.events.clear();
    g.step();
    g.step();
    assert_eq!(g.player(Seat(2)).gold, content::START_GOLD + 32);
    assert_eq!(g.player(Seat(2)).essence, 3);
    let blue = view::world(&g, Team::Blue);
    let red = view::world(&g, Team::Red);
    assert!(blue.events.iter().any(|e| matches!(
        e,
        Event::GoldGranted {
            seat: Seat(2),
            origin: None,
            ..
        }
    )));
    assert!(!red
        .events
        .iter()
        .any(|e| matches!(e, Event::GoldGranted { .. })));
}

#[test]
fn tower_last_hit_by_ownerless_minion_pays_all_five_teammates_once() {
    let mut g = arena();
    let a = g.spawn(
        UnitKind::Minion(MinionKind::Melee),
        Side::Team(Team::Blue),
        None,
        Pos::units(40, 40),
        Brain::Static,
    );
    let b = victim(
        &mut g,
        UnitKind::Tower(TowerTier::Outer, Some(Lane::Mid)),
        Pos::units(45, 40),
    );
    let source = g.unit(a).unwrap().clone();
    champions::hurt(&mut g, &source, b, 10_000);
    g.step();
    for seat in Seat::all() {
        let won = seat.team() == Team::Blue;
        assert_eq!(
            g.player(seat).gold,
            content::START_GOLD + if won { 25 } else { 0 }
        );
        assert_eq!(g.player(seat).essence, if won { 2 } else { 0 });
    }
    assert_eq!(gold_events(&g), 5);
    assert!(g
        .events
        .iter()
        .filter(|e| matches!(e, Event::GoldGranted { .. }))
        .all(|e| matches!(
            e,
            Event::GoldGranted {
                team_reward: true,
                ..
            }
        )));
    g.events.clear();
    g.step();
    assert_eq!(gold_events(&g), 0);
}

#[test]
fn splash_and_simultaneous_source_death_preserve_the_attacking_seat() {
    let mut g = arena();
    let a = attacker(&mut g, Some(Seat(4)));
    let i = g.slot(a).unwrap();
    g.units[i].stats.splash = 5_000;
    let b = victim(
        &mut g,
        UnitKind::Minion(MinionKind::Melee),
        Pos::units(43, 40),
    );
    let c = victim(
        &mut g,
        UnitKind::Minion(MinionKind::Ranged),
        Pos::units(44, 40),
    );
    g.step();
    for _ in 0..tbg_core::content::WINDUP {
        g.step();
    }
    assert!(g.unit(b).is_none() && g.unit(c).is_none());
    assert_eq!(g.player(Seat(4)).gold, content::START_GOLD + 8);
    assert_eq!(gold_events(&g), 2);
    // Both pending windups land in the same batch, including the dying source.
    let mut g = arena();
    let a = attacker(&mut g, Some(Seat(4)));
    let b = victim(
        &mut g,
        UnitKind::Minion(MinionKind::Melee),
        Pos::units(43, 40),
    );
    let i = g.slot(a).unwrap();
    g.units[i].hp = 1;
    let i = g.slot(b).unwrap();
    g.units[i].stats.damage = 10_000;
    g.units[i].fx.basic = Some(champions::state::Cast {
        slot: 0,
        target: a,
        centre: Pos::units(40, 40),
        left: 1,
        windup: 1,
    });
    let i = g.slot(a).unwrap();
    g.units[i].fx.basic = Some(champions::state::Cast {
        slot: 0,
        target: b,
        centre: Pos::units(43, 40),
        left: 1,
        windup: 1,
    });
    g.step();
    assert!(g.unit(a).is_none() && g.unit(b).is_none());
    assert_eq!(g.player(Seat(4)).gold, content::START_GOLD + 4);
}
