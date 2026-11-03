//! M-70/M-71: deterministic collision, autonomous abilities and visibility-safe combat cues.
use tbg_moba_core::{
    geom::Pos,
    state::{Brain, Event},
    types::Side,
    view, Champion, Lane, MobaMatch, Seat, Team, UnitKind,
};

fn arena() -> MobaMatch {
    let mut m = MobaMatch::new(71);
    m.units.clear();
    m
}
fn spawn(m: &mut MobaMatch, kind: Champion, side: Team, pos: Pos) -> u32 {
    m.spawn(
        UnitKind::Troop(kind),
        Side::Team(side),
        Some(Seat(if side == Team::Blue { 0 } else { 5 })),
        pos,
        Brain::Lane {
            lane: Lane::Mid,
            waypoint: 4,
        },
    )
}
fn idle(m: &mut MobaMatch, id: u32) {
    let i = m.slot(id).unwrap();
    m.units[i].stats.speed = 0;
    m.units[i].stats.damage = 0;
}

#[test]
fn m70_champion_contact_stops_movement_and_allows_melee() {
    let mut m = arena();
    let a = spawn(&mut m, Champion::Mantis, Team::Blue, Pos::units(70, 70));
    let b = spawn(&mut m, Champion::Archer, Team::Red, Pos::units(76, 70));
    idle(&mut m, b);
    let radius = m.unit(a).unwrap().stats.radius + m.unit(b).unwrap().stats.radius;
    let hp = m.unit(b).unwrap().hp;
    for _ in 0..80 {
        m.step();
        let (a, b) = (m.unit(a).unwrap(), m.unit(b).unwrap());
        assert!(
            a.pos.dist2(b.pos) >= i64::from(radius).pow(2),
            "a moving body entered a reserved footprint"
        );
    }
    assert!(
        m.unit(b).unwrap().hp < hp,
        "body contact must not put melee out of range"
    );
}

#[test]
fn m71_weaver_has_a_locked_automatic_root_after_its_windup() {
    let mut m = arena();
    let a = spawn(&mut m, Champion::Weaver, Team::Blue, Pos::units(72, 72));
    let b = spawn(&mut m, Champion::Archer, Team::Red, Pos::units(76, 72));
    idle(&mut m, a);
    idle(&mut m, b);
    m.step();
    assert!(m.events.iter().any(
        |e| matches!(e,Event::ChampionAbility{unit,target,started:true,..} if *unit==a&&*target==b)
    ));
    let hp = m.unit(b).unwrap().hp;
    for _ in 1..tbg_core::content::WINDUP {
        m.step();
    }
    assert_eq!(m.unit(b).unwrap().hp, hp);
    m.step();
    let target = m.unit(b).unwrap();
    assert!(target.hp < hp && target.fx.root_until > m.tick);
    assert!(view::world(&m, Team::Blue)
        .units
        .iter()
        .any(|u| u.id == b && u.champion.rooted));
}

#[test]
fn m71_mantis_bonus_applies_to_the_fourth_completed_hit() {
    let mut m = arena();
    let a = spawn(&mut m, Champion::Mantis, Team::Blue, Pos::units(70, 70));
    let b = spawn(
        &mut m,
        Champion::Archer,
        Team::Red,
        Pos {
            x: 75_450,
            y: 70_000,
        },
    );
    idle(&mut m, b);
    let i = m.slot(b).unwrap();
    m.units[i].hp = 1000;
    m.units[i].max_hp = 1000;
    let hp = m.units[i].hp;
    let damage = tbg_core::content::def(Champion::Mantis).damage;
    let t = tbg_core::content::def(Champion::Mantis).interval;
    for _ in 0..(3 * t + tbg_core::content::WINDUP + 1) {
        m.step();
    }
    let special = tbg_core::content::def(Champion::Mantis)
        .ability(0, 0)
        .into_iter()
        .find_map(|a| match a {
            tbg_core::content::Ability::PreciseStrike { damage, .. } => Some(i32::from(damage)),
            _ => None,
        })
        .unwrap();
    assert_eq!(m.unit(b).unwrap().hp, hp - 4 * damage - special);
    assert_eq!(m.unit(a).unwrap().fx.attacks, 0);
}

#[test]
fn m71_poison_keeps_seat_credit_after_the_caster_dies() {
    let mut m = arena();
    let a = spawn(&mut m, Champion::Rotcap, Team::Blue, Pos::units(72, 72));
    let b = spawn(&mut m, Champion::Archer, Team::Red, Pos::units(77, 72));
    idle(&mut m, b);
    let source = m.slot(a).unwrap();
    m.units[source].stats.range = 7000;
    for _ in 0..=tbg_core::content::WINDUP {
        m.step();
    }
    let source = m.slot(a).unwrap();
    m.units[source].hp = 0;
    let victim = m.slot(b).unwrap();
    m.units[victim].hp = 1;
    for _ in 0..=tbg_core::content::REGEN_PERIOD {
        m.step();
    }
    assert!(m.unit(b).is_none());
    assert_eq!(m.player(Seat(0)).kills, 1);
    assert!(view::world(&m, Team::Blue)
        .events
        .iter()
        .any(|e| matches!(e,Event::UnitDied{unit,..} if *unit==b)));
}
