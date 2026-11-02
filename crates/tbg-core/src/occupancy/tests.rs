//! PK-01 capacity, bypass, mirrored blocking, queue conservation and replay regressions.
use super::*;
use crate::{Seat, UnitKind::*};
#[test]
fn pk01_lab_placement_keeps_canonical_movement_unless_explicitly_enabled() {
    let mut s = MatchState::new(1);
    s.lab_spawn(Seat::A, Archer, 0, 0);
    assert!(
        !s.physical_lanes,
        "PK-01: a placement hook cannot silently change the duel rules"
    );
    let preview = MatchState::with_preview(1, [crate::ArcanaPreset::NEUTRAL; 2]);
    assert!(!preview.physical_lanes);
    assert!(!preview.view_for(Seat::A, &[]).physical_lanes);
}
#[test]
fn tw01_bot_saves_for_the_repriced_fortification() {
    let mut s = MatchState::new(7).with_strongholds([
        crate::StrongholdKind::Bastion,
        crate::StrongholdKind::Classic,
    ]);
    s.physical_lanes = true;
    s.phase = crate::Phase::Running;
    for _ in 0..6 {
        s.lab_spawn_at(Seat::A, Archer, 0, 0, 8000);
    }
    arrange(&mut s);
    assert_eq!(s.army_size(Seat::A), 6);
    s.players[0].energy = content::FORTIFY_COST[0] - 1;
    assert_eq!(
        crate::bot::Bot::new(4).decide(&s.view_for(Seat::A, &[])),
        None
    );
    s.players[0].energy += 1;
    assert!(matches!(
        crate::bot::Bot::new(4).decide(&s.view_for(Seat::A, &[])),
        Some(crate::PlayerIntention::Fortify { .. })
    ));
}
#[test]
fn pk01_stationary_backline_does_not_permanently_prevent_a_large_birth() {
    for seat in Seat::BOTH {
        let mut s = MatchState::new(9);
        s.physical_lanes = true;
        s.lab_spawn(seat, Farwatch, 0, 0);
        arrange(&mut s);
        s.lab_spawn(seat, BoneBrute, 0, 0);
        arrange(&mut s);
        assert_eq!(s.units.len(), 2);
        assert!(s.deployments.is_empty());
        let behind = s.units[0];
        let ahead = s.units[1];
        assert!((ahead.x - behind.x) * seat.dir() >= spacing(&behind, &ahead));
        let progress = if seat == Seat::A {
            ahead.x
        } else {
            content::LANE_LEN - ahead.x
        };
        assert!(progress <= content::occupancy::DEPLOY_END);
    }
}
#[test]
fn pk01_backfield_birth_never_jumps_a_hostile_front() {
    let mut s = MatchState::new(9);
    s.physical_lanes = true;
    s.lab_spawn(Seat::A, Farwatch, 0, 0);
    s.lab_spawn_at(Seat::B, Colossus, 0, 0, 3000);
    arrange(&mut s);
    s.lab_spawn(Seat::A, BoneBrute, 0, 0);
    arrange(&mut s);
    assert!(s.deployments.iter().any(|u| u.kind == BoneBrute));
    assert_eq!(s.units.len() + s.deployments.len(), 3);
}
#[test]
fn pk01_contact_floor_does_not_extend_ranged_reach() {
    let mut s = MatchState::new(2);
    s.physical_lanes = true;
    s.lab_spawn_at(Seat::A, Farwatch, 0, 0, 500);
    s.lab_spawn_at(Seat::B, Archer, 0, 0, 13500);
    assert!(crate::targeting::enemy_in_range(&s, &s.units[0], 12000).is_none());
    let a = s.lab_spawn_at(Seat::A, Shieldbearer, 0, 0, 6000);
    let b = s.lab_spawn_at(Seat::B, Shieldbearer, 0, 0, 7900);
    let u = s.units[s.unit_index(a).unwrap()];
    assert_eq!(
        crate::targeting::enemy_in_range(&s, &u, 800),
        Some(crate::Target::Unit(b))
    );
}
#[test]
fn pk01_bot_prefers_deployed_power_when_entrance_is_blocked() {
    let mut s = MatchState::new(1).with_strongholds([
        crate::StrongholdKind::Bastion,
        crate::StrongholdKind::Classic,
    ]);
    s.physical_lanes = true;
    s.phase = crate::Phase::Running;
    s.players[0].energy = 30000;
    for _ in 0..8 {
        s.lab_spawn(Seat::A, Colossus, 0, 0);
    }
    arrange(&mut s);
    let mut bot = crate::bot::Bot::new(4);
    assert!(matches!(
        bot.decide(&s.view_for(Seat::A, &[])),
        Some(crate::PlayerIntention::Fortify { .. })
    ));
}
#[test]
fn pk01_ranged_rows_are_bounded_and_stable() {
    let mut s = MatchState::new(1);
    s.physical_lanes = true;
    for _ in 0..12 {
        s.lab_spawn_at(Seat::A, Archer, 0, 0, 8000);
    }
    arrange(&mut s);
    assert!(
        s.units.iter().filter(|u| u.x == 8000).count() <= (COLUMNS / cells(&s.units[0])) as usize
    );
    let before = s.units.clone();
    arrange(&mut s);
    assert_eq!(before, s.units);
    for (i, a) in s.units.iter().enumerate() {
        for b in &s.units[i + 1..] {
            if overlaps(a, i32::from(a.fx.lane_slot) - 1, b) {
                assert!((a.x - b.x).abs() >= spacing(a, b));
            }
        }
    }
}
#[test]
fn pk01_queue_conserves_paid_units_and_releases_when_space_opens() {
    let mut s = MatchState::new(1);
    s.physical_lanes = true;
    for _ in 0..8 {
        s.lab_spawn(Seat::A, Colossus, 0, 0);
    }
    arrange(&mut s);
    assert_eq!(s.units.len(), 1);
    assert_eq!(s.deployments.len(), 7);
    assert_eq!(s.view_for(Seat::A, &[]).queued, 7);
    assert_eq!(s.view_for(Seat::B, &[]).queued, 0);
    s.units.clear();
    arrange(&mut s);
    assert_eq!(s.units.len(), 1);
    assert_eq!(s.deployments.len(), 6);
    let bytes = postcard::to_allocvec(&s).unwrap();
    let mut replay: MatchState = postcard::from_bytes(&bytes).unwrap();
    arrange(&mut replay);
    assert_eq!(replay.hash(), s.hash());
}
#[test]
fn pk01_colossus_blocks_its_whole_width_and_has_exact_ogre_range_and_double_damage() {
    let a = content::def(Colossus);
    let b = content::def(SiegeWalker);
    assert_eq!(
        (a.cost, a.hp, a.damage, a.range),
        (10800, 2400, b.damage * 2, b.range)
    );
    for seat in Seat::BOTH {
        let mut s = MatchState::new(3);
        s.physical_lanes = true;
        let dir = seat.dir();
        let mid = 8000;
        s.lab_spawn_at(seat, Colossus, 0, 0, mid);
        s.lab_spawn_at(seat, Archer, 0, 0, mid - dir * 2600);
        arrange(&mut s);
        let old = s.units[1].x;
        move_unit(&mut s, 1, old + dir * 3000);
        let required = footprint(Archer, 0).front + footprint(Colossus, 0).back + GAP;
        assert!((s.units[1].x - mid).abs() >= required);
    }
}

#[test]
fn pk01_free_lateral_space_allows_a_bypass_and_locks_the_new_lane() {
    let mut s = MatchState::new(2);
    s.physical_lanes = true;
    s.lab_spawn_at(Seat::A, Beetling, 0, 0, 8000);
    s.lab_spawn_at(Seat::A, Beetling, 0, 0, 4500);
    arrange(&mut s);
    s.units[0].fx.lane_slot = 1;
    s.units[1].fx.lane_slot = 1;
    for _ in 0..COLUMNS {
        let before = lateral(&s.units[1]);
        move_unit(&mut s, 1, 8500);
        assert!((lateral(&s.units[1]) - before).abs() <= CELL);
        s.tick += 1;
    }
    assert_eq!(s.units[1].x, 8500);
    assert!(s.units[1].fx.lane_slot > 1);
}

#[test]
fn pk01_blocked_evolution_keeps_energy_and_deployed_units() {
    let mut s = MatchState::new(3);
    s.physical_lanes = true;
    s.phase = crate::Phase::Running;
    s.players[0].energy = 10000;
    let id = s.lab_spawn(Seat::A, AbyssHarpooner, 1, 0);
    s.lab_spawn(Seat::A, AbyssHarpooner, 1, 0);
    arrange(&mut s);
    assert_eq!(s.units.len(), 2);
    let result = s.apply_intention(
        Seat::A,
        1,
        crate::PlayerIntention::Evolve {
            unit: id,
            branch: 0,
        },
    );
    assert_eq!(result, Err(crate::Rejection::LaneFull));
    assert_eq!(s.players[0].energy, 10000);
    assert!(s.deployments.is_empty());
    assert!(s.units.iter().all(|u| u.tier == 1));
}

#[test]
fn pk01_colossus_frontal_area_damage_reaches_multiple_enemies() {
    let mut s = MatchState::with_arcanas(2, [crate::ArcanaPreset::NEUTRAL; 2]);
    s.physical_lanes = true;
    s.phase = crate::Phase::Running;
    s.tick = content::RUNNING_START;
    s.lab_spawn_at(Seat::A, Colossus, 0, 0, 6000);
    let a = s.lab_spawn_at(Seat::B, FoamWeaver, 0, 0, 8000);
    let b = s.lab_spawn_at(Seat::B, FoamWeaver, 0, 0, 8100);
    let mut hit = std::collections::BTreeSet::new();
    for _ in 0..12 {
        for e in s.step() {
            if let crate::Event::Hit {
                to: crate::Target::Unit(id),
                amount: 170,
            } = e
            {
                if id == a || id == b {
                    hit.insert(id);
                }
            }
        }
    }
    assert_eq!(
        hit.len(),
        2,
        "GD-01: a real completed hit damages both units in the frontal area"
    );
    for (t, b) in [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)] {
        assert_eq!(
            content::effective_stats(Colossus, t, b, 0).range,
            content::effective_stats(SiegeWalker, t, b, 0).range
        );
    }
}

#[test]
fn pk01_mixed_form_envelopes_pack_without_losing_units() {
    let mut s = MatchState::new(11);
    s.physical_lanes = true;
    let mut total = 0;
    for kind in crate::COMPLETION_POOL {
        for (t, b) in [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)] {
            s.lab_spawn_at(Seat::A, kind, t, b, 8000);
            total += 1;
        }
    }
    arrange(&mut s);
    assert_eq!(s.units.len() + s.deployments.len(), total);
    for (i, a) in s.units.iter().enumerate() {
        for b in &s.units[i + 1..] {
            if overlaps(a, i32::from(a.fx.lane_slot) - 1, b) {
                let (front, back) = if a.x >= b.x { (a, b) } else { (b, a) };
                let required = footprint(back.kind, back.tier).front
                    + footprint(front.kind, front.tier).back
                    + GAP;
                assert!(
                    (a.x - b.x).abs() >= required,
                    "PK-01 overlapping {:?}/{:?}",
                    a.kind,
                    b.kind
                );
            }
        }
    }
    let before = s.units.clone();
    arrange(&mut s);
    assert_eq!(
        before, s.units,
        "PK-01: stationary layouts do not flicker on recomputation"
    );
}

#[test]
fn gd01_slam_splash_excludes_the_rear_half_plane() {
    let mut s = MatchState::with_arcanas(1, [crate::ArcanaPreset::NEUTRAL; 2]);
    s.physical_lanes = true;
    let source = s.lab_spawn_at(Seat::A, Colossus, 0, 0, 6000);
    let primary = s.lab_spawn_at(Seat::B, FoamWeaver, 0, 0, 7000);
    let forward = s.lab_spawn_at(Seat::B, FoamWeaver, 0, 0, 7200);
    let rear = s.lab_spawn_at(Seat::B, FoamWeaver, 0, 0, 5500);
    let u = s.units[s.unit_index(source).unwrap()];
    let stats = s.stats_of(&u);
    let rear_hp = s.units[s.unit_index(rear).unwrap()].hp;
    let before = s.units[s.unit_index(forward).unwrap()].hp;
    crate::hits::resolve_hit(
        &mut s,
        &u,
        crate::Target::Unit(primary),
        &stats,
        &mut vec![],
    );
    assert_eq!(s.units[s.unit_index(forward).unwrap()].hp, before - 170);
    assert_eq!(s.units[s.unit_index(rear).unwrap()].hp, rear_hp);
}
