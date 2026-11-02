//! TW-01 real shot collateral targeting, Fury attribution and Bastion mitigation.
use super::*;
use crate::{ArcanaPreset, UnitKind};
#[test]
fn tw01_artillery_hits_three_additional_enemies_and_fury_counts_each_eligible_kill() {
    let mut s = MatchState::with_arcanas(8, [ArcanaPreset::NEUTRAL; 2])
        .with_strongholds([StrongholdKind::Artillery, StrongholdKind::Classic]);
    s.players[0].augments.add(22);
    for i in 0..5 {
        s.lab_spawn_at(Seat::B, UnitKind::Archer, 0, 0, 1000 + i * 100);
        s.units.last_mut().unwrap().hp = 50;
    }
    let ally = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 1200);
    let hp = s.units.last().unwrap().hp;
    let mut events = Vec::new();
    crate::combat::stronghold_fire(&mut s, &mut events);
    assert_eq!(
        s.units
            .iter()
            .filter(|u| u.seat == Seat::B && u.hp <= 0)
            .count(),
        4
    );
    assert_eq!(s.units[s.unit_index(ally).unwrap()].hp, hp);
    assert_eq!(s.players[0].fury_stacks, 3);
}
#[test]
fn tw01_bastion_armor_mitigates_before_the_shield() {
    let mut s =
        MatchState::new(8).with_strongholds([StrongholdKind::Bastion, StrongholdKind::Classic]);
    let hp = s.players[0].stronghold_hp;
    s.players[0].shield = 20;
    s.players[0].shield_until = 100;
    assert_eq!(
        crate::hits::damage(&mut s, Target::Stronghold(Seat::A), 100, &mut Vec::new()),
        65
    );
    assert_eq!(s.players[0].stronghold_hp, hp - 65);
}
