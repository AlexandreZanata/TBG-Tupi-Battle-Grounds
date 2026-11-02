//! GD-04 Verdict Lance contract: same-enemy sequence, capped max-HP pierce and forms.
use super::*;
use crate::{hits, roster_slots, types::UnitId, ArcanaPreset, Seat};

fn arena(tier: u8, branch: u8) -> (MatchState, UnitId, UnitId) {
    let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
    s.lab_spawn_at(Seat::A, UnitKind::Astreon, tier, branch, 6_000);
    let big = s.lab_spawn_at(Seat::B, UnitKind::Colossus, 0, 0, 8_000);
    let small = s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 8_200);
    (s, big, small)
}

fn hit(s: &mut MatchState, t: Target) {
    let a = s.units[0];
    let st = s.stats_of(&a);
    hits::resolve_hit(s, &a, t, &st, &mut Vec::new());
}

fn hp(s: &MatchState, id: UnitId) -> i32 {
    s.units[s.unit_index(id).unwrap()].hp
}

fn set_max(s: &mut MatchState, id: UnitId, max: i32, now: i32) {
    let i = s.unit_index(id).unwrap();
    (s.units[i].max_hp, s.units[i].hp) = (max, now);
}

#[test]
fn gd04_fourth_consecutive_hit_adds_five_percent_of_max_hp_once() {
    let (mut s, big, _) = arena(0, 0);
    let max = s.units[s.unit_index(big).unwrap()].max_hp;
    let start = hp(&s, big);
    for n in 1..=3 {
        hit(&mut s, Target::Unit(big));
        assert_eq!(counter(&s.units[0]), n);
    }
    hit(&mut s, Target::Unit(big));
    assert_eq!(start - hp(&s, big), 4 * 190 + max * 500 / 10_000);
    assert_eq!(counter(&s.units[0]), 0);
    // The next sequence starts again from one.
    hit(&mut s, Target::Unit(big));
    assert_eq!(counter(&s.units[0]), 1);
}

#[test]
fn gd04_switching_targets_or_hitting_a_stronghold_restarts_the_sequence() {
    let (mut s, big, small) = arena(0, 0);
    set_max(&mut s, small, 5_000, 5_000);
    for _ in 0..3 {
        hit(&mut s, Target::Unit(big));
    }
    let before = hp(&s, small);
    hit(&mut s, Target::Unit(small));
    assert_eq!(before - hp(&s, small), 190);
    assert_eq!(counter(&s.units[0]), 1);
    for _ in 0..2 {
        hit(&mut s, Target::Unit(small));
    }
    hit(&mut s, Target::Stronghold(Seat::B));
    assert_eq!(counter(&s.units[0]), 0);
    // A windup on a different enemy clears the mark before its first hit lands.
    hit(&mut s, Target::Unit(big));
    retarget(&mut s, 0, Target::Unit(small));
    assert_eq!(counter(&s.units[0]), 0);
    retarget(&mut s, 0, Target::Unit(small));
    hit(&mut s, Target::Unit(small));
    assert_eq!(counter(&s.units[0]), 1);
}

#[test]
fn gd04_the_pierce_reads_max_hp_and_its_cap_applies_before_shields() {
    let (mut s, big, _) = arena(0, 0);
    set_max(&mut s, big, 20_000, 9_000);
    for _ in 0..3 {
        hit(&mut s, Target::Unit(big));
    }
    let i = s.unit_index(big).unwrap();
    roster_slots::apply_shield(&mut s.units[i].fx, 100, 200, s.tick);
    let before = hp(&s, big);
    hit(&mut s, Target::Unit(big));
    // 5 % of 20 000 = 1 000, capped at 240 before the 100-point shield absorbs.
    assert_eq!(before - hp(&s, big), 190 + 240 - 100);
}

#[test]
fn gd04_branch_a_pierces_every_third_hit_and_a2_raises_share_and_cap() {
    let (mut a1, big, _) = arena(1, 0);
    set_max(&mut a1, big, 4_000, 4_000);
    for _ in 0..3 {
        hit(&mut a1, Target::Unit(big));
    }
    assert_eq!(4_000 - hp(&a1, big), 3 * 190 + 200);

    let (mut a2, big, _) = arena(2, 0);
    set_max(&mut a2, big, 20_000, 20_000);
    for _ in 0..3 {
        hit(&mut a2, Target::Unit(big));
    }
    assert_eq!(20_000 - hp(&a2, big), 3 * 190 + 300);
    set_max(&mut a2, big, 4_000, 4_000);
    for _ in 0..3 {
        hit(&mut a2, Target::Unit(big));
    }
    assert_eq!(4_000 - hp(&a2, big), 3 * 190 + 280);
}

#[test]
fn gd04_branch_b_executes_only_below_forty_percent_on_the_fourth_hit() {
    for (branch_tier, execute) in [(1u8, 66), (2u8, 114)] {
        let (mut s, big, _) = arena(branch_tier, 1);
        set_max(&mut s, big, 4_000, 1_700);
        for _ in 0..3 {
            hit(&mut s, Target::Unit(big));
        }
        // 1 700 − 570 = 1 130 HP left: below 40 % of 4 000 when the fourth lands.
        let before = hp(&s, big);
        hit(&mut s, Target::Unit(big));
        assert_eq!(before - hp(&s, big), 190 + execute + 200);

        let (mut s, big, _) = arena(branch_tier, 1);
        set_max(&mut s, big, 10_000, 10_000);
        for _ in 0..4 {
            hit(&mut s, Target::Unit(big));
        }
        assert_eq!(10_000 - hp(&s, big), 4 * 190 + 240);
    }
}

#[test]
fn gd04_a_completed_kill_ends_the_sequence_instead_of_carrying_it_over() {
    let (mut s, big, small) = arena(0, 0);
    set_max(&mut s, small, 300, 300);
    for _ in 0..2 {
        hit(&mut s, Target::Unit(small));
    }
    assert_eq!(hp(&s, small), 0);
    assert_eq!(counter(&s.units[0]), 0);
    hit(&mut s, Target::Unit(big));
    assert_eq!(counter(&s.units[0]), 1);
}
