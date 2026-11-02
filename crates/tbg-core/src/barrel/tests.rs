//! OG-02 collision, replay and bounded-travel scenarios for rolling area-heal barrels.
use super::*;
use crate::{content, ArcanaPreset, UnitKind::*};
fn arena() -> MatchState {
    MatchState::with_arcanas(7, [ArcanaPreset::NEUTRAL; 2])
}
fn add(s: &mut MatchState, seat: Seat, kind: UnitKind, x: i32) -> u32 {
    s.lab_spawn_at(seat, kind, 0, 0, x);
    s.units.last().unwrap().id
}
fn hp(s: &MatchState, id: u32) -> i32 {
    s.units[s.unit_index(id).unwrap()].hp
}
fn wound(s: &mut MatchState, id: u32, missing: i32) {
    let i = s.unit_index(id).unwrap();
    s.units[i].hp -= missing;
}
fn throw(s: &mut MatchState, source: u32, target: u32) {
    let u = s.units[s.unit_index(source).unwrap()];
    let st = s.stats_of(&u);
    assert!(launch(s, &u, target, &st));
}
fn finish(s: &mut MatchState) -> Vec<Event> {
    let mut e = Vec::new();
    for _ in 0..40 {
        if s.barrels.is_empty() {
            return e;
        }
        advance(s, &mut e);
        s.tick += 1;
    }
    panic!("OG-02: a barrel must expire within finite travel");
}

#[test]
fn og02_first_allied_contact_triggers_area_healing_after_real_travel() {
    let mut s = arena();
    let source = add(&mut s, Seat::A, GustDrummer, 4000);
    let guard = add(&mut s, Seat::A, Shieldbearer, 5200);
    let target = add(&mut s, Seat::A, Trident, 5800);
    wound(&mut s, target, 300);
    let other = add(&mut s, Seat::A, BoneBrute, 5400);
    wound(&mut s, other, 150);
    let other_before = hp(&s, other);
    let before = hp(&s, target);
    throw(&mut s, source, target);
    assert_eq!(
        hp(&s, target),
        before,
        "OG-02: launch is not an instant heal"
    );
    advance(&mut s, &mut Vec::new());
    assert_eq!(hp(&s, target), before);
    assert_eq!(s.barrels[0].x, 4220);
    let e = finish(&mut s);
    assert_eq!(hp(&s, target), before + 90);
    assert_eq!(
        hp(&s, other),
        other_before + 90,
        "OG-02: one impact heals multiple wounded allies"
    );
    assert!(e
        .iter()
        .any(|e| matches!(e,Event::BarrelBurst{contact:id,..} if *id==guard)));
}
#[test]
fn og02_enemy_interception_deals_little_damage_and_still_heals_nearby_allies() {
    let mut s = arena();
    let source = add(&mut s, Seat::A, GustDrummer, 4000);
    let enemy = add(&mut s, Seat::B, Shieldbearer, 4800);
    let target = add(&mut s, Seat::A, Trident, 5600);
    wound(&mut s, target, 300);
    let ti = s.unit_index(target).unwrap();
    s.units[ti].heal_cut_until = 100;
    s.units[ti].heal_cut_bp = 5000;
    let (a, b) = (hp(&s, enemy), hp(&s, target));
    throw(&mut s, source, target);
    finish(&mut s);
    assert_eq!(hp(&s, enemy), a - 8);
    assert_eq!(
        hp(&s, target),
        b + 45,
        "OG-02: area heals obey healing reduction"
    );
}
#[test]
fn og02_launch_snapshot_survives_source_death_and_roundtrips_identically() {
    let mut s = arena();
    let source = add(&mut s, Seat::A, GustDrummer, 4000);
    let target = add(&mut s, Seat::A, Trident, 6000);
    wound(&mut s, target, 200);
    throw(&mut s, source, target);
    s.units.retain(|u| u.id != source);
    let bytes = postcard::to_allocvec(&s).unwrap();
    let mut restored: MatchState = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(s.hash(), restored.hash());
    assert_eq!(
        s.view_for(Seat::A, &[]).barrels,
        s.public_view(&[], 0).barrels
    );
    assert_eq!(finish(&mut s), finish(&mut restored));
    assert_eq!(s.hash(), restored.hash());
    assert_eq!(hp(&s, target), s.units[0].max_hp - 110);
}
#[test]
fn og02_a_missed_barrel_expires_without_homing_or_free_healing() {
    let mut s = arena();
    let source = add(&mut s, Seat::A, GustDrummer, 4000);
    let target = add(&mut s, Seat::A, Trident, 6000);
    wound(&mut s, target, 200);
    throw(&mut s, source, target);
    let i = s.unit_index(target).unwrap();
    s.units[i].x = 9000;
    let before = hp(&s, target);
    let e = finish(&mut s);
    assert_eq!(hp(&s, target), before);
    assert!(!e.iter().any(|e| matches!(e, Event::Heal { .. })));
}
#[test]
fn og02_swept_contacts_and_heal_results_mirror_for_both_seats() {
    let mut results = Vec::new();
    for seat in Seat::BOTH {
        let mut s = arena();
        let mirror = |x| {
            if seat == Seat::A {
                x
            } else {
                content::LANE_LEN - x
            }
        };
        let source = add(&mut s, seat, GustDrummer, mirror(4000));
        let target = add(&mut s, seat, Trident, mirror(6500));
        let enemy = add(&mut s, seat.other(), Shieldbearer, mirror(5000));
        wound(&mut s, target, 200);
        throw(&mut s, source, target);
        s.barrels[0].speed = 2500;
        let e = finish(&mut s);
        let center = e
            .iter()
            .find_map(|e| match e {
                Event::BarrelBurst { center, .. } => Some(*center),
                _ => None,
            })
            .unwrap();
        results.push((hp(&s, target), hp(&s, enemy), mirror(center)));
    }
    assert_eq!(
        results[0], results[1],
        "OG-02: swept contact is independent of seat orientation"
    );
}
#[test]
fn og02_the_kit_uses_a_long_basic_heal_reload_and_no_interrupt() {
    let d = content::def(GustDrummer);
    assert_eq!(d.archetype, crate::Archetype::Support);
    assert_eq!((d.interval, d.heal, d.damage), (200, 90, 0));
    assert_eq!(
        crate::specials::period(&crate::ruleset::CANON, GustDrummer, 0, 0),
        None
    );
    let mut s = arena();
    let source = add(&mut s, Seat::A, GustDrummer, 4000);
    wound(&mut s, source, 100);
    let u = s.units[0];
    let st = s.stats_of(&u);
    assert!(!launch(&mut s, &u, source, &st) || s.barrels.is_empty());
}
