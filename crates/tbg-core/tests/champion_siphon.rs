//! Proposed Siphon contracts (§C.14, BR-X06/X11): resolve real combat hits.
#![cfg(feature = "lab")]
use tbg_core::{ArcanaPreset, Event, MatchState, Ruleset, Seat, Target, UnitKind};

fn setup(tier: u8, branch: u8) -> (MatchState, u32, u32) {
    let mut s = MatchState::lab(31, Ruleset::canon(), [ArcanaPreset::NEUTRAL; 2]);
    let a = s.lab_spawn_at(Seat::A, UnitKind::Bloodweaver, tier, branch, 6_000);
    let b = s.lab_spawn_at(Seat::B, UnitKind::Rampart, 0, 0, 8_000);
    (s, a, b)
}
fn wounded(s: &mut MatchState, kind: UnitKind, x: i32, hp: i32) -> u32 {
    let id = s.lab_spawn_at(Seat::A, kind, 0, 0, x);
    let i = s.unit_index(id).unwrap();
    s.units[i].hp = hp;
    id
}
fn hit(s: &mut MatchState, a: u32, target: Target) -> Vec<Event> {
    let i = s.unit_index(a).unwrap();
    s.units[i].windup = Some((target, 1));
    s.step()
}
fn transfers(events: &[Event]) -> Vec<(u32, u32, i32)> {
    events
        .iter()
        .filter_map(|e| {
            if let Event::Transfer { from, to, amount } = e {
                Some((*from, *to, *amount))
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn heals_lowest_percent_outside_range_and_moves_without_a_target() {
    let (mut s, a, b) = setup(0, 0);
    let low = wounded(&mut s, UnitKind::BoneBrute, 1_000, 400);
    wounded(&mut s, UnitKind::Archer, 1_500, 100);
    assert_eq!(
        transfers(&hit(&mut s, a, Target::Unit(b))),
        vec![(b, low, 30)]
    );
    s.units.retain(|u| u.seat == Seat::A);
    let i = s.unit_index(a).unwrap();
    s.units[i].windup = None;
    let x = s.units[i].x;
    s.step();
    assert!(s.units[s.unit_index(a).unwrap()].x > x);
}
#[test]
fn healing_equals_actual_hp_removed_without_overkill() {
    let (mut s, a, b) = setup(0, 0);
    let ally = wounded(&mut s, UnitKind::BoneBrute, 1_000, 100);
    let i = s.unit_index(b).unwrap();
    s.units[i].hp = 7;
    assert_eq!(
        transfers(&hit(&mut s, a, Target::Unit(b))),
        vec![(b, ally, 7)]
    );
}
#[test]
fn stronghold_hits_heal_nobody() {
    let (mut s, a, _) = setup(0, 0);
    wounded(&mut s, UnitKind::BoneBrute, 1_000, 100);
    assert!(transfers(&hit(&mut s, a, Target::Stronghold(Seat::B))).is_empty());
}
#[test]
fn never_heals_summons_or_full_allies() {
    let (mut s, a, b) = setup(0, 0);
    wounded(&mut s, UnitKind::Beetling, 1_000, 1);
    assert!(transfers(&hit(&mut s, a, Target::Unit(b))).is_empty());
}
#[test]
fn heal_cut_and_lifesteal_cap_are_independent() {
    let (mut s, a, b) = setup(0, 0);
    let ally = wounded(&mut s, UnitKind::BoneBrute, 1_000, 100);
    let i = s.unit_index(ally).unwrap();
    s.units[i].heal_cut_bp = 5_000;
    s.units[i].heal_cut_until = u32::MAX;
    assert_eq!(
        transfers(&hit(&mut s, a, Target::Unit(b))),
        vec![(b, ally, 15)]
    );
    // 15 is 50% of damage, deliberately above the separate 12% lifesteal cap.
}
#[test]
fn last_offering_doubles_below_twenty_percent() {
    let (mut s, a, b) = setup(2, 0);
    let ally = wounded(&mut s, UnitKind::BoneBrute, 1_000, 100);
    assert_eq!(
        transfers(&hit(&mut s, a, Target::Unit(b))),
        vec![(b, ally, 78)]
    );
}
#[test]
fn shared_blood_splits_between_two_lowest() {
    let (mut s, a, b) = setup(2, 1);
    let first = wounded(&mut s, UnitKind::BoneBrute, 1_000, 100);
    let second = wounded(&mut s, UnitKind::Archer, 2_000, 100);
    assert_eq!(
        transfers(&hit(&mut s, a, Target::Unit(b))),
        vec![(b, first, 18), (b, second, 18)]
    );
}
#[test]
fn ties_use_nearest_then_lowest_id() {
    let (mut s, a, b) = setup(0, 0);
    wounded(&mut s, UnitKind::Archer, 1_000, 100);
    let near = wounded(&mut s, UnitKind::Archer, 4_000, 100);
    wounded(&mut s, UnitKind::Archer, 4_000, 100);
    assert_eq!(
        transfers(&hit(&mut s, a, Target::Unit(b))),
        vec![(b, near, 30)]
    );
}
#[test]
fn plates_do_not_feed_siphon() {
    let (mut s, a, _) = setup(0, 0);
    let b = s.lab_spawn_at(Seat::B, UnitKind::Shatterback, 0, 0, 8_000);
    wounded(&mut s, UnitKind::BoneBrute, 1_000, 100);
    assert!(transfers(&hit(&mut s, a, Target::Unit(b))).is_empty());
}

#[test]
fn unit_shields_do_not_feed_siphon() {
    // D-63 + D-68: only enemy HP removed feeds Siphon, never shield damage.
    let (mut s, a, b) = setup(0, 0);
    wounded(&mut s, UnitKind::BoneBrute, 1_000, 100);
    let i = s.unit_index(b).unwrap();
    s.units[i].fx.shield = 100;
    s.units[i].fx.shield_until = u32::MAX;
    assert!(transfers(&hit(&mut s, a, Target::Unit(b))).is_empty());
}
