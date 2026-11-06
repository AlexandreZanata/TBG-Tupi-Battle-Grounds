//! Owner-requested territory vision contracts: full home half, invading scouts and conquered Towers.

use tbg_moba_core::geom::{Pos, MM};
use tbg_moba_core::state::Brain;
use tbg_moba_core::types::{MinionKind, NeutralKind, Side, TowerTier};
use tbg_moba_core::{map, view, vision::Vision, Champion, Lane, MobaMatch, Team, UnitKind};

fn reflected(team: Team, p: Pos) -> Pos {
    match team {
        Team::Blue => p,
        Team::Red => Pos {
            x: map::SIZE * MM - p.y,
            y: map::SIZE * MM - p.x,
        },
    }
}

#[test]
fn m67_home_half_is_fully_visible_without_scouts_or_standing_home_towers() {
    // M-50/M-67: enemy intruders, neutrals and camp timers on the home half are always visible.
    for team in Team::ALL {
        let mut m = MobaMatch::new(67);
        let home = reflected(team, Pos::units(88, 88));
        let enemy = m.spawn(
            UnitKind::Troop(Champion::Longaxe),
            Side::Team(team.enemy()),
            None,
            home,
            Brain::Static,
        );
        let neutral = m.spawn(
            UnitKind::Neutral(NeutralKind::WolfPup),
            Side::Neutral,
            None,
            home,
            Brain::Static,
        );
        // This fixture has no allied mobile unit, and no allied structure is in sight of the point.
        assert!(m
            .units
            .iter()
            .filter(|u| u.team() == Some(team))
            .all(|u| u.kind.structure() && !u.pos.within(home, u.stats.sight)));
        for remove_towers in [false, true] {
            if remove_towers {
                m.units
                    .retain(|u| u.team() != Some(team) || !matches!(u.kind, UnitKind::Tower(..)));
            }
            let frame = view::world(&m, team);
            assert!(frame.units.iter().any(|u| u.id == enemy));
            assert!(frame.units.iter().any(|u| u.id == neutral));
            assert!(Vision::new(&m, team).point(home));
            assert_eq!(
                frame
                    .camps
                    .iter()
                    .filter(|c| map::camp(c.camp as usize).1 == team)
                    .count(),
                4
            );
        }
    }
    let at_river = Pos::units(100, 100);
    assert!(tbg_moba_core::vision::home_half(Team::Blue, at_river));
    assert!(tbg_moba_core::vision::home_half(Team::Red, at_river));
}

#[test]
fn m68_only_allied_troops_across_the_river_reveal_enemy_ground() {
    // M-50/M-68: sight from the home half never spills across; crossing grants it, retreat removes it.
    for team in Team::ALL {
        let mut m = MobaMatch::new(68);
        let own = reflected(team, Pos::units(99, 100));
        let enemy_pos = reflected(team, Pos::units(101, 100));
        let enemy = m.spawn(
            UnitKind::Troop(Champion::Longaxe),
            Side::Team(team.enemy()),
            None,
            enemy_pos,
            Brain::Static,
        );
        let ally = m.spawn(
            UnitKind::Minion(MinionKind::Melee),
            Side::Team(team),
            None,
            own,
            Brain::Static,
        );
        let contains = |m: &MobaMatch| view::world(m, team).units.iter().any(|u| u.id == enemy);
        assert!(!contains(&m), "a scout has not crossed the river yet");
        let i = m.slot(ally).unwrap();
        m.units[i].pos = enemy_pos;
        assert!(contains(&m));
        assert!(Vision::new(&m, team).point(enemy_pos));
        let i = m.slot(ally).unwrap();
        m.units[i].pos = own;
        assert!(
            !contains(&m),
            "ordinary scouting leaves no permanent vision"
        );
        // Purchased champions grant the same vision as Minions after crossing.
        let i = m.slot(ally).unwrap();
        m.units[i].kind = UnitKind::Troop(Champion::Longaxe);
        m.units[i].pos = enemy_pos;
        assert!(contains(&m));
        m.units.retain(|u| u.id != ally);
        assert!(!contains(&m), "dead or removed scouts cannot keep vision");
    }
}

#[test]
fn m69_destroyed_enemy_tower_reveals_only_its_sight_circle_until_match_end() {
    // M-51/M-69: only a full destruction captures a permanent, inclusive 9-unit sight circle.
    for team in Team::ALL {
        let mut m = MobaMatch::new(69);
        let i = m
            .units
            .iter()
            .position(|u| {
                u.team() == Some(team.enemy())
                    && u.kind == UnitKind::Tower(TowerTier::Outer, Some(Lane::Mid))
            })
            .unwrap();
        let (id, centre) = (m.units[i].id, m.units[i].pos);
        m.units[i].hp = 1;
        assert!(
            !Vision::new(&m, team).point(centre),
            "a damaged standing Tower is not conquered"
        );
        m.units[i].hp = 0;
        m.step();
        assert!(m.unit(id).is_none());
        let edge = reflected(team, Pos::units(130, 127));
        // Red's mid Outer is (121,127), and Blue's is its reflection (73,79).
        assert_eq!(centre.dist2(edge), i64::from(9 * MM).pow(2));
        let outside = match team {
            Team::Blue => Pos {
                x: edge.x + 1,
                ..edge
            },
            Team::Red => Pos {
                y: edge.y - 1,
                ..edge
            },
        };
        let seen = m.spawn(
            UnitKind::Neutral(NeutralKind::WolfPup),
            Side::Neutral,
            None,
            edge,
            Brain::Static,
        );
        let hidden = m.spawn(
            UnitKind::Neutral(NeutralKind::WolfPup),
            Side::Neutral,
            None,
            outside,
            Brain::Static,
        );
        for _ in 0..3 {
            let vision = Vision::new(&m, team);
            assert!(vision.point(centre));
            assert!(vision.point(edge));
            assert!(!vision.point(outside));
            let frame = view::world(&m, team);
            assert!(frame.units.iter().any(|u| u.id == seen));
            assert!(!frame.units.iter().any(|u| u.id == hidden));
            m.step();
        }
        assert!(
            !Vision::new(&MobaMatch::new(69), team).point(centre),
            "a new match resets conquered vision"
        );
    }
}
