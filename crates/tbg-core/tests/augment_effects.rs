//! Effect-hook contracts for the 40 Augments (owner revision 2026-10-29).

use tbg_core::augment;
use tbg_core::content::{def, Stats, BP};
use tbg_core::{Event, MatchState, Phase, PlayerIntention, Ruleset, Seat, Status, UnitKind};

fn running(seed: u64) -> MatchState {
    let mut s = MatchState::new(seed);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

fn arena(seed: u64) -> MatchState {
    let mut rules = Ruleset::canon().clone();
    rules.stronghold_damage = 0;
    rules.regen_running = 0;
    rules.interest.max_bands = 0;
    rules.augment.round_ticks = 0;
    let rules: &'static Ruleset = Box::leak(Box::new(rules));
    let mut s = MatchState::with_rules(seed, rules);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

fn spawn(s: &mut MatchState, seat: Seat, kind: UnitKind, x: i32) -> u32 {
    s.players[seat.index()].offers[0] = kind;
    s.players[seat.index()].energy = 100 * tbg_core::content::ENERGY_SCALE;
    s.apply_intention(seat, 1, PlayerIntention::Purchase { offer: 0 })
        .unwrap();
    let unit = s.units.last_mut().unwrap();
    unit.x = x;
    unit.id
}

fn unit(s: &MatchState, id: u32) -> &tbg_core::Unit {
    s.units.iter().find(|u| u.id == id).unwrap()
}

fn stats(s: &MatchState, seat: Seat, kind: UnitKind, tier: u8) -> Stats {
    s.stats_for(seat, kind, tier, 0)
}

#[test]
fn stat_augments_add_in_bp_and_truncate_once() {
    let mut s = running(1);
    s.players[0].augments.add(1); // Field Kit +4 % HP
    s.players[0].augments.add(4); // Reinforced Shields +8 % Tank HP
    let st = stats(&s, Seat::A, UnitKind::Shieldbearer, 0);
    assert_eq!(st.max_hp, def(UnitKind::Shieldbearer).hp * 11_200 / 10_000);
    s.players[0].augments.add(1);
    let st = stats(&s, Seat::A, UnitKind::Shieldbearer, 0);
    assert_eq!(st.max_hp, def(UnitKind::Shieldbearer).hp * 11_600 / 10_000);
}

#[test]
fn faction_and_evolved_filters_only_reach_their_units() {
    let mut s = running(2);
    for id in [6, 7, 8, 9, 13] {
        s.players[0].augments.add(id);
    }
    assert_eq!(
        stats(&s, Seat::A, UnitKind::Shieldbearer, 0).damage,
        def(UnitKind::Shieldbearer).damage * 10_800 / 10_000
    );
    assert_eq!(
        stats(&s, Seat::A, UnitKind::BoneBrute, 0).interval,
        (i64::from(def(UnitKind::BoneBrute).interval) * BP / 10_800) as u16
    );
    assert_eq!(
        stats(&s, Seat::A, UnitKind::Sawbot, 0).interval,
        (i64::from(def(UnitKind::Sawbot).interval) * BP / 10_800) as u16
    );
    assert_eq!(
        stats(&s, Seat::A, UnitKind::Bloomkeeper, 0).max_hp,
        def(UnitKind::Bloomkeeper).hp * 10_800 / 10_000
    );
    assert_eq!(
        stats(&s, Seat::A, UnitKind::Shieldbearer, 1).max_hp,
        def(UnitKind::Shieldbearer).hp * 15_800 / 10_000,
        "Bulwark +50 % HP plus Veterans +8 %"
    );
}

#[test]
fn attack_speed_and_move_caps_hold_under_many_copies() {
    let mut s = running(3);
    for _ in 0..20 {
        s.players[0].augments.add(27); // Hunter's Instinct
        s.players[0].augments.add(19); // Shield Step
    }
    let st = stats(&s, Seat::A, UnitKind::Sawbot, 0);
    assert!(st.interval >= tbg_core::content::MIN_INTERVAL);
    assert_eq!(
        st.interval,
        (i64::from(def(UnitKind::Sawbot).interval) / 2) as u16,
        "attack speed is capped at +100 %"
    );
    assert!(st.speed <= def(UnitKind::Sawbot).speed * 3 / 2);
}

#[test]
fn forgo_discount_and_convergence_are_dynamic() {
    let mut s = running(4);
    s.players[0].augments.add(12); // Field Promotion
    s.players[0].augments.add(34); // Convergence
    s.players[0].synergy = [1, 1, 0, 0];
    let with = stats(&s, Seat::A, UnitKind::Shieldbearer, 0);
    s.players[0].synergy = [1, 0, 0, 0];
    let without = stats(&s, Seat::A, UnitKind::Shieldbearer, 0);
    assert!(with.max_hp > without.max_hp);
    assert!(with.damage > without.damage);
    assert_eq!(
        s.rules().evolve_cost[0] - augment::evolve_discount(&s.players[0].augments),
        600
    );
}

#[test]
fn lifesteal_heals_the_primary_attacker_only() {
    let mut s = arena(5);
    s.players[0].augments.add(20); // War Leech
    let archer = spawn(&mut s, Seat::A, UnitKind::Archer, 1_000);
    spawn(&mut s, Seat::B, UnitKind::Shieldbearer, 2_000);
    s.units[0].hp = 10;
    for _ in 0..30 {
        s.step();
        if unit(&s, archer).hp > 10 {
            break;
        }
    }
    assert!(unit(&s, archer).hp > 10, "lifesteal must heal the attacker");
}

#[test]
fn second_wind_survives_one_lethal_hit_per_copy() {
    let mut s = arena(6);
    s.players[0].augments.add(33);
    let target = spawn(&mut s, Seat::A, UnitKind::Shieldbearer, 1_000);
    let attacker = spawn(&mut s, Seat::B, UnitKind::Sawbot, 1_500);
    s.units.iter_mut().find(|u| u.id == attacker).unwrap().hp = 100_000;
    s.units.iter_mut().find(|u| u.id == target).unwrap().hp = 1;
    for _ in 0..20 {
        s.step();
        if unit(&s, target).second_wind_used > 0 {
            break;
        }
    }
    assert_eq!(unit(&s, target).hp, 1);
    assert_eq!(unit(&s, target).second_wind_used, 1);
}

#[test]
fn last_bastion_raises_one_shield_and_expires() {
    let mut s = arena(7);
    s.players[0].augments.add(30);
    s.players[0].stronghold_hp = 1_000;
    let attacker = spawn(&mut s, Seat::B, UnitKind::Sawbot, 500);
    s.units.iter_mut().find(|u| u.id == attacker).unwrap().hp = 100_000;
    for _ in 0..20 {
        s.step();
        if s.players[0].shield > 0 {
            break;
        }
    }
    assert!(s.players[0].shield > 0);
    let shield = s.players[0].shield;
    assert!(s.players[0].shield_until > s.tick);
    for _ in 0..301 {
        s.step();
    }
    assert_eq!(s.players[0].shield, 0);
    assert!(s.players[0].stronghold_hp <= 1_000);
    let _ = shield;
}

#[test]
fn bounty_and_stone_mend_credit_the_killer() {
    let mut s = arena(8);
    s.players[0].augments.add(10);
    s.players[0].augments.add(11);
    s.players[0].stronghold_hp = 4_000;
    let before = s.players[0].energy;
    let dead = spawn(&mut s, Seat::B, UnitKind::Archer, 8_000);
    s.units.iter_mut().find(|u| u.id == dead).unwrap().hp = 0;
    s.step();
    assert_eq!(s.players[0].energy.saturating_sub(before), 90);
    assert_eq!(s.players[0].stronghold_hp, 4_050);
}

#[test]
fn tower_fury_counts_only_stronghold_kills() {
    let mut s = arena(9);
    s.players[0].augments.add(22);
    let dead = spawn(&mut s, Seat::B, UnitKind::Archer, 500);
    s.units.iter_mut().find(|u| u.id == dead).unwrap().hp = 1;
    s.step();
    assert_eq!(s.players[0].fury_stacks, 1);
    assert!(s.players[0].fury_until > s.tick);
}

#[test]
fn damage_reduction_aura_respects_position_condition_and_cap() {
    let mut s = arena(10);
    let target = spawn(&mut s, Seat::A, UnitKind::Shieldbearer, 1_000);
    spawn(&mut s, Seat::A, UnitKind::Rampart, 1_200);
    s.players[0].stronghold_hp = 1_000;
    s.players[0].augments.add(14);
    s.players[0].augments.add(23);
    let dr = augment::damage_reduction_bp(&s, unit(&s, target));
    assert_eq!(dr, 1_800);
    for _ in 0..5 {
        s.players[0].augments.add(14);
        s.players[0].augments.add(23);
    }
    let capped = augment::damage_reduction_bp(&s, unit(&s, target));
    assert_eq!(capped, 2_000);
}

#[test]
fn great_wounds_cuts_direct_healing() {
    let mut s = arena(11);
    s.players[0].augments.add(5);
    s.players[0].augments.add(17);
    let healer = spawn(&mut s, Seat::A, UnitKind::Bloomkeeper, 1_000);
    let target = spawn(&mut s, Seat::A, UnitKind::Shieldbearer, 1_100);
    s.units.iter_mut().find(|u| u.id == target).unwrap().hp = 100;
    let heal = augment::heal_amount(
        &s,
        Seat::A,
        unit(&s, target),
        def(UnitKind::Bloomkeeper).heal,
        true,
    );
    assert_eq!(heal, 50, "Triage +25 %");
    s.units
        .iter_mut()
        .find(|u| u.id == target)
        .unwrap()
        .heal_cut_until = s.tick + 10;
    s.units
        .iter_mut()
        .find(|u| u.id == target)
        .unwrap()
        .heal_cut_bp = 4_000;
    let cut = augment::heal_amount(
        &s,
        Seat::A,
        unit(&s, target),
        def(UnitKind::Bloomkeeper).heal,
        true,
    );
    assert_eq!(cut, 30);
    let _ = healer;
    assert_eq!(
        stats(&s, Seat::A, UnitKind::Bloomkeeper, 0).heal,
        44,
        "First Aid applies to the healer's stats"
    );
}

#[test]
fn situational_damage_uses_hit_time_conditions() {
    let mut s = arena(12);
    s.players[0].augments.add(25); // Giant Slayer
    s.players[0].augments.add(26); // Executioner
    let attacker = spawn(&mut s, Seat::A, UnitKind::Archer, 1_000);
    let tank = spawn(&mut s, Seat::B, UnitKind::BoneBrute, 1_500);
    s.units.iter_mut().find(|u| u.id == tank).unwrap().hp = 10;
    let bp = augment::situational_bp(&s, unit(&s, attacker), tbg_core::Target::Unit(tank));
    assert_eq!(bp, 3_500);
}

#[test]
fn lingering_hex_lengthens_root_without_changing_strength() {
    let mut s = arena(13);
    for _ in 0..5 {
        s.players[0].augments.add(18); // cap +30 %
    }
    let rooter = spawn(&mut s, Seat::A, UnitKind::Rootcaller, 1_000);
    let target = spawn(&mut s, Seat::B, UnitKind::Shieldbearer, 2_000);
    let mut rooted = None;
    for _ in 0..30 {
        for ev in s.step() {
            if let Event::StatusApplied { unit: id } = ev {
                if id == target {
                    rooted = unit(&s, target).status.into();
                }
            }
        }
        if rooted.is_some() {
            break;
        }
    }
    let ticks = match unit(&s, target).status {
        Status::Rooted { ticks_left } => ticks_left,
        _ => panic!("target must be rooted"),
    };
    assert_eq!(ticks, 20, "16 ticks × 1.30, truncated");
    let _ = rooter;
}

#[test]
fn forced_march_stamps_only_new_units() {
    let mut s = running(14);
    let old = spawn(&mut s, Seat::A, UnitKind::Archer, 1_000);
    s.players[0].augments.add(0);
    s.players[0].energy = 100 * tbg_core::content::ENERGY_SCALE;
    let new = spawn(&mut s, Seat::A, UnitKind::Archer, 1_000);
    assert_eq!(unit(&s, old).haste_until, 0);
    assert_eq!(unit(&s, new).haste_until, s.tick + 100);
}
