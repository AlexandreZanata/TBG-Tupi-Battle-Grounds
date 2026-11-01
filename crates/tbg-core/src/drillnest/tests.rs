//! R21 contract scenarios: bounded production, snapshot/fusion semantics and ordinary summon combat.
use super::*;
use crate::{
    combat, roster_slots,
    state::{Bleed, Stack},
    ArcanaPreset,
};
fn arena() -> MatchState {
    let mut s = MatchState::with_arcanas(1, [ArcanaPreset::NEUTRAL; 2]);
    s.lab_spawn_at(Seat::A, UnitKind::Drillnest, 0, 0, 4000);
    s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7000);
    s
}
fn drill(s: &mut MatchState, source: &Unit, kind: UnitKind, x: i32) -> u32 {
    let mut u = spawn(s, source, kind, &mut Vec::new());
    u.x = x;
    let id = u.id;
    s.units.push(u);
    id
}
#[test]
fn r21_full_period_then_six_tick_production_has_no_basic_damage_or_tower_hit() {
    let mut s = arena();
    assert_eq!(s.units[0].action_cd, 44);
    let mut ev = Vec::new();
    for _ in 0..44 {
        combat::tick_timers(&mut s);
        produce(&mut s, &mut ev);
    }
    assert_eq!(count(&s, Seat::A), 0);
    assert_eq!(s.units[0].fx.special.unwrap().left, WINDUP);
    for _ in 0..6 {
        combat::tick_timers(&mut s);
        produce(&mut s, &mut ev);
    }
    assert_eq!(count(&s, Seat::A), 1);
    let d = s.units.last().unwrap();
    assert_eq!(
        (d.hp, d.kind, d.x, d.expires_at),
        (45, UnitKind::Microdrill, 4000, Some(240))
    );
    assert!(!ev.iter().any(|e| matches!(e, Event::Hit { .. })));
    s.units[0].action_cd = 0;
    s.units[0].x = 15000;
    s.units[1].hp = 0;
    produce(&mut s, &mut ev);
    assert_eq!(s.units[0].fx.special.unwrap().target, None);
}
#[test]
fn r21_cap_is_shared_across_factories_and_resolution_leaves_one_readiness() {
    let mut s = arena();
    let a = s.units[0];
    let second = s.lab_spawn_at(Seat::A, UnitKind::Drillnest, 0, 0, 4100);
    for n in 0..7 {
        drill(&mut s, &a, UnitKind::Megadrill, 4000 + n * 10);
    }
    for id in [a.id, second] {
        let i = s.unit_index(id).unwrap();
        s.units[i].fx.special = Some(PendingCast {
            target: Some(s.units[1].id),
            left: 1,
        });
    }
    produce(&mut s, &mut Vec::new());
    assert_eq!(count(&s, Seat::A), 8);
    let i = s.unit_index(second).unwrap();
    assert_eq!(s.units[i].action_cd, 0);
    assert!(s.units[i].fx.special.is_none());
    produce(&mut s, &mut Vec::new());
    assert_eq!(count(&s, Seat::A), 8);
    assert!(s.units[i].fx.special.is_none());
    s.units.last_mut().unwrap().hp = 0;
    produce(&mut s, &mut Vec::new());
    assert_eq!(s.units[i].fx.special.unwrap().left, 6);
}
#[test]
fn r21_fusion_preserves_weighted_hp_profile_midpoint_and_cooldown_without_death() {
    let mut s = arena();
    let mut a = s.units[0];
    a.tier = 2;
    a.branch = 1;
    let id = drill(&mut s, &a, UnitKind::Microdrill, 4000);
    let base = s.units[0];
    let other = drill(&mut s, &base, UnitKind::Microdrill, 4001);
    let i = s.unit_index(id).unwrap();
    s.units[i].hp = 20;
    s.units[i].cooldown = 13;
    let j = s.unit_index(other).unwrap();
    s.units[j].cooldown = 3;
    s.units[j].windup = Some((Target::Unit(s.units[1].id), 1));
    let energy = s.players[0].energy;
    let mut ev = Vec::new();
    fuse(&mut s, &mut ev);
    let out = s.units.last().unwrap();
    assert_eq!(
        (out.kind, out.hp, out.max_hp, out.x, out.cooldown),
        (UnitKind::Duodrill, 79, 110, 4000, 13)
    );
    assert_eq!((out.tier, out.branch, out.summoner), (2, 1, Some(a.id)));
    assert!(out.windup.is_none());
    assert_eq!(s.stats_of(out).damage, 46);
    assert_eq!(s.players[0].energy, energy);
    assert!(!ev
        .iter()
        .any(|e| matches!(e, Event::Death { .. } | Event::Hit { .. })));
}
#[test]
fn r21_pairs_are_id_ordered_and_outputs_never_merge_recursively_in_the_same_pass() {
    let mut s = arena();
    let a = s.units[0];
    let ids: Vec<_> = [4000, 4200, 4001, 4201]
        .into_iter()
        .map(|x| drill(&mut s, &a, UnitKind::Microdrill, x))
        .collect();
    let mut ev = Vec::new();
    fuse(&mut s, &mut ev);
    assert_eq!(count(&s, Seat::A), 2);
    assert!(s
        .units
        .iter()
        .filter(|u| rank(u.kind) > 0)
        .all(|u| u.kind == UnitKind::Duodrill));
    assert!(ev.iter().any(
        |e| matches!(e,Event::DrillFused{first,second,..} if *first==ids[0]&&*second==ids[1])
    ));
    fuse(&mut s, &mut Vec::new());
    assert_eq!(count(&s, Seat::A), 1);
    assert_eq!(s.units.last().unwrap().kind, UnitKind::Megadrill);
}
#[test]
fn r21_fusion_never_cleanses_control_dots_cuts_or_weakness_and_never_adds_shields() {
    let mut s = arena();
    let mut a = s.units[0];
    a.tier = 2;
    let id = drill(&mut s, &a, UnitKind::Microdrill, 4000);
    let other = drill(&mut s, &a, UnitKind::Microdrill, 4000);
    let i = s.unit_index(id).unwrap();
    let j = s.unit_index(other).unwrap();
    s.units[i].status = Status::Rooted { ticks_left: 8 };
    s.units[j].status = Status::Slowed {
        bp: 3000,
        ticks_left: 50,
    };
    s.units[i].fx.poison[0] = Stack { value: 3, left: 20 };
    s.units[j].fx.poison[0] = Stack { value: 5, left: 10 };
    s.units[i].fx.fish[0] = Stack {
        value: 2000,
        left: 40,
    };
    s.units[i].fx.bleed = Bleed {
        value: 5,
        left: 4,
        next: 20,
        source: Some(Seat::B),
    };
    s.units[j].fx.bleed = Bleed {
        value: 7,
        left: 2,
        next: 10,
        source: Some(Seat::A),
    };
    s.units[j].fx.venom = Bleed {
        value: 7,
        left: 4,
        next: 20,
        source: Some(Seat::B),
    };
    roster_slots::apply_weaken(&mut s.units[i].fx, 1000, 20, 0);
    roster_slots::apply_weaken(&mut s.units[j].fx, 2000, 10, 0);
    roster_slots::apply_ability_cut(&mut s.units[i].fx, 3500, 40, 0);
    roster_slots::apply_shield(&mut s.units[i].fx, 50, 10, 0);
    roster_slots::apply_shield(&mut s.units[j].fx, 40, 100, 0);
    s.units[i].haste_until = 100;
    s.units[i].fx.bond = Some(a.id);
    fuse(&mut s, &mut Vec::new());
    let out = s.units.last().unwrap();
    assert_eq!(out.status, Status::Rooted { ticks_left: 50 });
    assert_eq!(out.fx.poison[0], Stack { value: 5, left: 10 });
    assert_eq!(out.fx.poison[1], Stack { value: 3, left: 20 });
    assert_eq!(out.fx.fish[0].left, 40);
    assert_eq!(
        (out.fx.bleed.value, out.fx.bleed.left, out.fx.bleed.next),
        (7, 4, 10)
    );
    assert_eq!(out.fx.venom.left, 4);
    assert_eq!((out.fx.weaken_bp, out.fx.weaken_until), (2000, 20));
    assert_eq!(out.fx.ability_cut_bp, 3500);
    assert_eq!((out.fx.shield, out.fx.shield_until), (50, 10));
    assert_eq!(out.haste_until, 0);
    assert!(out.fx.bond.is_none());
}
#[test]
fn r21_profiles_survive_producer_death_and_higher_rank_bonus_uses_printed_hp_only() {
    let mut s = arena();
    let mut a = s.units[0];
    a.tier = 2;
    a.branch = 1;
    let id = drill(&mut s, &a, UnitKind::Megadrill, 4000);
    let i = s.unit_index(id).unwrap();
    s.units[0].hp = 0;
    s.units[0].tier = 0;
    let d = s.units[i];
    assert_eq!((d.tier, d.branch), (2, 1));
    assert_eq!(s.stats_of(&d).damage, 90);
    s.units[1].kind = UnitKind::Jawguard;
    assert_eq!(bonus(&s, &d, Target::Unit(s.units[1].id), 90), 112);
    s.units[1].kind = UnitKind::Shieldbearer;
    s.units[1].max_hp = 1500;
    assert_eq!(bonus(&s, &d, Target::Unit(s.units[1].id), 90), 90);
    assert_eq!(bonus(&s, &d, Target::Stronghold(Seat::B), 90), 90);
    s.tick = 320;
    crate::summon::expire(&mut s);
    assert_eq!(s.units[i].hp, 0);
    assert_eq!(count(&s, Seat::A), 0);
    assert_eq!(period(s.rules(), UnitKind::Drillnest, 2, 0), Some(38));
}
#[test]
fn r21_root_and_dead_primary_cancel_production_and_drills_use_ordinary_guard_targeting() {
    let mut s = arena();
    s.units[0].action_cd = 0;
    produce(&mut s, &mut Vec::new());
    s.units[0].status = Status::Rooted { ticks_left: 5 };
    produce(&mut s, &mut Vec::new());
    assert!(s.units[0].fx.special.is_none());
    assert_eq!(count(&s, Seat::A), 0);
    s.units[0].status = Status::None;
    s.units[0].action_cd = 0;
    produce(&mut s, &mut Vec::new());
    s.units[1].hp = 0;
    produce(&mut s, &mut Vec::new());
    assert!(s.units[0].fx.special.is_none());
    let a = s.units[0];
    let id = drill(&mut s, &a, UnitKind::Microdrill, 7000);
    let guard = s.lab_spawn_at(Seat::B, UnitKind::Rampart, 0, 0, 7550);
    s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 7100);
    let d = s.units[s.unit_index(id).unwrap()];
    assert_eq!(
        targeting::enemy_in_range(&s, &d, 600),
        Some(Target::Unit(guard))
    );
}

#[test]
fn r21_external_hostile_mark_survives_fusion_with_its_original_deadline() {
    let mut s = arena();
    let a = s.units[0];
    let first = drill(&mut s, &a, UnitKind::Microdrill, 4000);
    drill(&mut s, &a, UnitKind::Microdrill, 4100);
    let crow = s.lab_spawn_at(Seat::B, UnitKind::Crowcaller, 0, 0, 4500);
    let i = s.unit_index(crow).unwrap();
    s.units[i].fx.mark_target = Some(first);
    s.units[i].fx.mark_until = 80;
    fuse(&mut s, &mut Vec::new());
    let out = s.units.last().unwrap().id;
    crate::roster_slots::tick_marks(&mut s);
    let i = s.unit_index(crow).unwrap();
    assert_eq!(
        (s.units[i].fx.mark_target, s.units[i].fx.mark_until),
        (Some(out), 80)
    );
}
