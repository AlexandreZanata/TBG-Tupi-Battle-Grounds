//! M-77 contract: readable, tremor-free crowd motion. Planted attackers hold
//! their ground, attackers surround a target on distinct places, idle walkers
//! assist a fighting ally, and no body walks back and forth or into water.
use std::collections::HashMap;

use tbg_moba_core::{
    geom::Pos,
    host, map,
    state::{Activity, Brain},
    types::{MinionKind, Side},
    Champion, Lane, MobaMatch, Team, UnitKind,
};

fn quiet() -> MobaMatch {
    let mut g = MobaMatch::new(77);
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

/// A tough Red target that neither moves nor hits back.
fn dummy(g: &mut MobaMatch, p: Pos) -> u32 {
    let id = walker(g, UnitKind::Minion(MinionKind::Melee), Team::Red, p);
    let i = g.slot(id).unwrap();
    g.units[i].stats.speed = 0;
    g.units[i].stats.damage = 0;
    g.units[i].hp = 1_000_000;
    g.units[i].max_hp = 1_000_000;
    id
}

#[test]
fn a_bot_match_shows_no_back_and_forth_tremor_and_no_body_in_water() {
    let mut g = MobaMatch::new(11);
    let mut last: HashMap<u32, (Pos, Option<Pos>)> = HashMap::new();
    let (mut moving, mut tremor) = (0u32, 0u32);
    while g.tick < 20 * 150 {
        host::tick(&mut g, &mut Vec::new());
        for u in g.units.iter().filter(|u| {
            u.mobile()
                && u.fx.carrier == 0
                && matches!(u.kind, UnitKind::Troop(_) | UnitKind::Minion(_))
        }) {
            assert!(
                map::surface::walkable(u.pos, u.stats.radius),
                "tick {}: {:?} body overlaps water or the rim at {:?}",
                g.tick,
                u.kind,
                u.pos
            );
            if let Some(&(previous, before)) = last.get(&u.id) {
                if previous.dist(u.pos) > 60 {
                    moving += 1;
                    // A → B → A: the body came back to where it stood two ticks ago.
                    tremor += u32::from(before.is_some_and(|b| b.dist(u.pos) < 40));
                }
                last.insert(u.id, (u.pos, Some(previous)));
            } else {
                last.insert(u.id, (u.pos, None));
            }
        }
    }
    assert!(moving > 10_000, "the match must actually move ({moving})");
    // One A-B-A per thousand moving ticks at most (the original engine had ~23).
    assert!(
        tremor * 1_000 <= moving,
        "{tremor} back-and-forth steps in {moving} moving ticks"
    );
}

#[test]
fn planted_attackers_hold_their_ground_while_allies_walk_past() {
    let mut g = quiet();
    let target = dummy(&mut g, Pos::units(60, 60));
    let attackers: Vec<u32> = [Pos::units(54, 54), Pos::units(56, 52)]
        .into_iter()
        .map(|p| {
            walker(
                &mut g,
                UnitKind::Troop(Champion::Shieldbearer),
                Team::Blue,
                p,
            )
        })
        .collect();
    for &a in &attackers {
        let i = g.slot(a).unwrap();
        g.units[i].target = target;
    }
    // Let them plant, then send a column of walkers through the same spot.
    for _ in 0..60 {
        g.step();
        g.events.clear();
    }
    for k in 0..6 {
        walker(
            &mut g,
            UnitKind::Troop(Champion::Colossus),
            Team::Blue,
            Pos::units(44 - k * 3, 44),
        );
    }
    // Where each attacker planted; a fresh plant (after a pursuit) re-anchors.
    let mut anchor: HashMap<u32, Pos> = HashMap::new();
    let mut planted_ticks = 0;
    for _ in 0..200 {
        g.step();
        g.events.clear();
        for &a in &attackers {
            let u = g.unit(a).unwrap();
            if u.activity != Activity::Attack {
                anchor.remove(&a);
                continue;
            }
            planted_ticks += 1;
            let at = *anchor.entry(a).or_insert(u.pos);
            assert!(
                u.pos.within(at, 300),
                "a planted attacker was shoved {} mm",
                u.pos.dist(at)
            );
        }
    }
    assert!(planted_ticks > 200, "the attackers must actually fight");
}

#[test]
fn melee_attackers_surround_one_target_on_distinct_places() {
    let mut g = quiet();
    let target = dummy(&mut g, Pos::units(60, 60));
    let ids: Vec<u32> = [Pos::units(50, 60), Pos::units(50, 64), Pos::units(50, 56)]
        .into_iter()
        .map(|p| walker(&mut g, UnitKind::Troop(Champion::Mantis), Team::Blue, p))
        .collect();
    for &a in &ids {
        let i = g.slot(a).unwrap();
        g.units[i].target = target;
    }
    let mut hit = vec![false; ids.len()];
    for _ in 0..200 {
        g.step();
        for (k, &a) in ids.iter().enumerate() {
            hit[k] |= g.unit(a).unwrap().last_attack == g.tick;
        }
        g.events.clear();
    }
    assert!(
        hit.iter().all(|&h| h),
        "every attacker must reach and strike: {hit:?}"
    );
    for (k, &a) in ids.iter().enumerate() {
        for &b in &ids[k + 1..] {
            let (u, v) = (g.unit(a).unwrap(), g.unit(b).unwrap());
            assert!(
                u.pos.dist(v.pos) >= i64::from(u.stats.radius + v.stats.radius) - 100,
                "two attackers share one place"
            );
        }
    }
}

#[test]
fn a_walker_with_no_enemy_in_sight_assists_a_fighting_ally() {
    let mut g = quiet();
    let target = dummy(&mut g, Pos::units(60, 60));
    let fighter = walker(
        &mut g,
        UnitKind::Troop(Champion::Archer),
        Team::Blue,
        Pos::units(50, 67),
    );
    let i = g.slot(fighter).unwrap();
    g.units[i].target = target;
    for _ in 0..40 {
        g.step();
        g.events.clear();
    }
    assert_eq!(g.unit(fighter).unwrap().activity, Activity::Attack);
    // Out of its own sight of the target, but within sight of the fighter.
    let helper = walker(
        &mut g,
        UnitKind::Troop(Champion::Shieldbearer),
        Team::Blue,
        Pos::units(49, 60),
    );
    let h = g.slot(helper).unwrap();
    assert!(!g.units[h]
        .pos
        .within(Pos::units(60, 60), g.units[h].stats.sight));
    g.step();
    assert_eq!(
        g.unit(helper).unwrap().target,
        target,
        "the walker joins the fight"
    );
}
