//! GD-04 Storm Judgment contract: counting, deterministic distinct links and form values.
use super::*;
use crate::{roster_slots, ArcanaPreset, Seat, Status};

fn arena(tier: u8, branch: u8, enemies: &[i32]) -> (MatchState, Vec<UnitId>) {
    let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
    s.lab_spawn_at(Seat::A, UnitKind::Volthar, tier, branch, 6_000);
    let ids = enemies
        .iter()
        .map(|&x| s.lab_spawn_at(Seat::B, UnitKind::Shieldbearer, 0, 0, x))
        .collect();
    (s, ids)
}

fn hit(s: &mut MatchState, t: Target) -> Vec<Event> {
    let a = s.units[0];
    let st = s.stats_of(&a);
    let mut ev = Vec::new();
    hits::resolve_hit(s, &a, t, &st, &mut ev);
    ev
}

fn hp(s: &MatchState, id: UnitId) -> i32 {
    s.units[s.unit_index(id).unwrap()].hp
}

fn arcs(ev: &[Event]) -> Vec<(UnitId, UnitId, u8)> {
    ev.iter()
        .filter_map(|e| match *e {
            Event::StormArc { from, to, jump, .. } => Some((from, to, jump)),
            _ => None,
        })
        .collect()
}

#[test]
fn gd04_fourth_completed_unit_hit_chains_three_decreasing_distinct_links() {
    let (mut s, e) = arena(0, 0, &[8_000, 9_000, 10_000, 11_000, 12_000]);
    let before: Vec<i32> = e.iter().map(|&id| hp(&s, id)).collect();
    for _ in 0..3 {
        assert!(arcs(&hit(&mut s, Target::Unit(e[0]))).is_empty());
    }
    assert_eq!(counter(&s.units[0]), 3);
    let ev = hit(&mut s, Target::Unit(e[0]));
    // Nearest-to-previous link, never revisiting; the fifth enemy is past three jumps.
    assert_eq!(
        arcs(&ev),
        vec![(e[0], e[1], 1), (e[1], e[2], 2), (e[2], e[3], 3)]
    );
    let lost: Vec<i32> = e
        .iter()
        .zip(&before)
        .map(|(&id, b)| b - hp(&s, id))
        .collect();
    assert_eq!(lost, vec![4 * 130, 91, 65, 45, 0]);
    assert_eq!(counter(&s.units[0]), 0);
}

#[test]
fn gd04_strongholds_never_count_or_link_and_allies_are_never_struck() {
    let (mut s, e) = arena(0, 0, &[8_000]);
    let ally = s.lab_spawn_at(Seat::A, UnitKind::Shieldbearer, 0, 0, 8_100);
    let ally_hp = hp(&s, ally);
    hit(&mut s, Target::Stronghold(Seat::B));
    assert_eq!(counter(&s.units[0]), 0);
    for _ in 0..3 {
        hit(&mut s, Target::Unit(e[0]));
    }
    hit(&mut s, Target::Stronghold(Seat::B));
    assert_eq!(counter(&s.units[0]), 3);
    // A lone enemy ends the chain at once; the ally beside it is untouched.
    let ev = hit(&mut s, Target::Unit(e[0]));
    assert!(arcs(&ev).is_empty());
    assert_eq!(hp(&s, ally), ally_hp);
    assert_eq!(counter(&s.units[0]), 0);
}

#[test]
fn gd04_jump_range_ties_and_shields_follow_the_authoritative_rules() {
    // Equal distances break by lower id; a link beyond 2.2 tiles is never chosen.
    let (mut s, e) = arena(0, 0, &[8_000, 9_000, 7_000, 12_000]);
    let (k, now) = (s.unit_index(e[2]).unwrap(), s.tick);
    roster_slots::apply_shield(&mut s.units[k].fx, 50, 200, now);
    let (shielded, far) = (hp(&s, e[2]), hp(&s, e[3]));
    for _ in 0..3 {
        hit(&mut s, Target::Unit(e[0]));
    }
    let ev = hit(&mut s, Target::Unit(e[0]));
    assert_eq!(arcs(&ev), vec![(e[0], e[1], 1), (e[1], e[2], 2)]);
    // 65 from the second jump, 50 absorbed by the shield first.
    assert_eq!(shielded - hp(&s, e[2]), 15);
    assert_eq!(hp(&s, e[3]), far);
}

#[test]
fn gd04_branch_a_adds_a_fourth_link_and_a2_widens_and_raises_each_share() {
    let line = [8_000, 9_000, 10_000, 11_000, 12_000];
    let (mut a1, e) = arena(1, 0, &line);
    let before: Vec<i32> = e.iter().map(|&id| hp(&a1, id)).collect();
    for _ in 0..4 {
        hit(&mut a1, Target::Unit(e[0]));
    }
    let lost: Vec<i32> = e
        .iter()
        .zip(&before)
        .map(|(&id, b)| b - hp(&a1, id))
        .collect();
    assert_eq!(lost, vec![520, 91, 65, 45, 32]);

    let wide = [8_000, 10_500, 13_000, 15_500, 15_900];
    let (mut a2, e) = arena(2, 0, &wide);
    let before: Vec<i32> = e.iter().map(|&id| hp(&a2, id)).collect();
    for _ in 0..4 {
        hit(&mut a2, Target::Unit(e[0]));
    }
    let lost: Vec<i32> = e
        .iter()
        .zip(&before)
        .map(|(&id, b)| b - hp(&a2, id))
        .collect();
    assert_eq!(lost, vec![520, 104, 78, 58, 45]);
}

#[test]
fn gd04_branch_b_powers_only_the_trigger_and_b2_roots_and_discharges_once() {
    let (mut b1, e) = arena(1, 1, &[8_000, 9_000]);
    let first = hp(&b1, e[0]);
    for _ in 0..3 {
        hit(&mut b1, Target::Unit(e[0]));
    }
    assert_eq!(first - hp(&b1, e[0]), 390);
    hit(&mut b1, Target::Unit(e[0]));
    assert_eq!(first - hp(&b1, e[0]), 390 + 130 + 45);

    // Chain runs east; the B2 extra discharge reaches the two unchained enemies west.
    let (mut b2, e) = arena(2, 1, &[8_000, 9_000, 10_000, 11_000, 7_000, 6_900, 6_800]);
    let before: Vec<i32> = e.iter().map(|&id| hp(&b2, id)).collect();
    for _ in 0..3 {
        hit(&mut b2, Target::Unit(e[0]));
    }
    let ev = hit(&mut b2, Target::Unit(e[0]));
    let links = arcs(&ev);
    assert_eq!(links.len(), 5);
    assert_eq!(links.iter().filter(|l| l.2 == 0).count(), 2);
    let lost: Vec<i32> = e
        .iter()
        .zip(&before)
        .map(|(&id, b)| b - hp(&b2, id))
        .collect();
    assert_eq!(lost[0], 3 * 130 + 130 + 71);
    assert_eq!(&lost[1..4], &[91, 65, 45]);
    assert_eq!(lost[4..].iter().filter(|&&l| l == 39).count(), 2);
    assert_eq!(lost[4..].iter().filter(|&&l| l == 0).count(), 1);
    let primary = &b2.units[b2.unit_index(e[0]).unwrap()];
    assert!(matches!(primary.status, Status::Rooted { ticks_left: 10 }));
}

#[test]
fn gd04_a_ward_blocks_the_root_but_not_the_damage_and_nothing_recurses() {
    let (mut s, e) = arena(2, 1, &[8_000, 9_000]);
    let i = s.unit_index(e[0]).unwrap();
    s.units[i].fx.ward_until = 1_000;
    for _ in 0..4 {
        hit(&mut s, Target::Unit(e[0]));
    }
    assert_eq!(s.units[i].status, Status::None);
    // The chain links are auxiliary damage: they never advance the counter again.
    assert_eq!(counter(&s.units[0]), 0);
    assert_eq!(arcs(&hit(&mut s, Target::Unit(e[0]))).len(), 0);
}
