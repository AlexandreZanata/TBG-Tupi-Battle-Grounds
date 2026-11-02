//! Contract scenarios for each completion champion and shared local-preview boundaries.

use super::*;
use crate::{
    content, hits, state::PendingCast, ArcanaPreset, Phase, PlayerIntention, Status, Target,
    UnitKind::*,
};

fn arena() -> MatchState {
    let mut s = MatchState::with_preview(7, [ArcanaPreset::NEUTRAL; 2]);
    s.phase = Phase::Running;
    s.tick = content::RUNNING_START;
    s
}
fn add(s: &mut MatchState, kind: crate::UnitKind, seat: Seat, x: i32) -> UnitId {
    let seq = s.player(seat).last_seq + 1;
    s.player_mut(seat).energy = 100_000;
    s.player_mut(seat).offers[0] = kind;
    s.apply_intention(seat, seq, PlayerIntention::Purchase { offer: 0 })
        .expect("trusted fixture purchase");
    let u = s.units.last_mut().expect("purchased unit");
    u.x = x;
    u.id
}
fn get(s: &MatchState, id: UnitId) -> Unit {
    s.units[s.unit_index(id).expect("fixture unit")]
}
fn wound(s: &mut MatchState, id: UnitId, hp: i32) {
    let i = s.unit_index(id).expect("fixture unit");
    s.units[i].hp = hp;
}
fn basic(s: &mut MatchState, id: UnitId, target: UnitId) {
    let u = get(s, id);
    let stats = s.stats_of(&u);
    hits::resolve_hit(s, &u, Target::Unit(target), &stats, &mut Vec::new());
}

#[test]
fn completion_cc01_finisher_has_a_wounded_unit_window_and_no_tower_bonus() {
    let mut s = arena();
    let a = add(&mut s, Gravecutter, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    let u = get(&s, a);
    let st = s.stats_of(&u);
    assert_eq!(basic_bonus(&mut s, &u, Target::Unit(b), &st), 0);
    wound(&mut s, b, 100);
    assert!(basic_bonus(&mut s, &u, Target::Unit(b), &st) > 0);
    assert_eq!(basic_bonus(&mut s, &u, Target::Stronghold(Seat::B), &st), 0);
}

#[test]
fn completion_cc02_echo_is_consumed_once_and_never_resurrects() {
    let mut s = arena();
    let a = add(&mut s, LastlightCantor, Seat::A, 4000);
    let b = add(&mut s, Shieldbearer, Seat::A, 4500);
    wound(&mut s, b, 300);
    basic(&mut s, a, b);
    basic(&mut s, a, b);
    let echo = get(&s, b).fx.completion.echo.amount;
    assert!(echo > 0);
    let before = get(&s, b).hp;
    hits::damage(&mut s, Target::Unit(b), 10, &mut Vec::new());
    assert_eq!(get(&s, b).hp, before - 10 + echo);
    assert_eq!(get(&s, b).fx.completion.echo.amount, 0);
    basic(&mut s, a, b);
    wound(&mut s, b, 1);
    hits::damage(&mut s, Target::Unit(b), 10_000, &mut Vec::new());
    assert_eq!(get(&s, b).hp, 0);
}

#[test]
fn completion_cc03_sweep_caps_secondary_victims_and_excludes_allies() {
    let mut s = arena();
    let a = add(&mut s, Cragclaw, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    let enemy: Vec<_> = (0..4)
        .map(|n| add(&mut s, BoneBrute, Seat::B, 4750 + n * 100))
        .collect();
    let friend = add(&mut s, Archer, Seat::A, 4800);
    let before = get(&s, friend).hp;
    for _ in 0..3 {
        basic(&mut s, a, b);
    }
    assert_eq!(
        enemy
            .into_iter()
            .filter(|&id| get(&s, id).hp < get(&s, id).max_hp)
            .count(),
        2
    );
    assert_eq!(get(&s, friend).hp, before);
}

#[test]
fn completion_cc04_charge_requires_movement_is_single_use_and_control_cancels_it() {
    let mut s = arena();
    let a = add(&mut s, Hushfang, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 6000);
    let i = s.unit_index(a).unwrap();
    for _ in 0..20 {
        let before = s.units[i].x;
        s.units[i].x += 20;
        movement(&mut s, i, before);
    }
    assert!(get(&s, a).fx.completion.charged);
    let u = get(&s, a);
    let st = s.stats_of(&u);
    let x = u.x;
    assert!(basic_bonus(&mut s, &u, Target::Unit(b), &st) > 0);
    assert!(!get(&s, a).fx.completion.charged);
    assert_eq!(get(&s, a).x, x);
    s.units[i].fx.completion.charged = true;
    hits::apply_on_hit(
        &mut s,
        a,
        crate::OnHit::Slow {
            bp: 1000,
            ticks: 20,
        },
        Seat::B,
        &mut Vec::new(),
    );
    assert!(!get(&s, a).fx.completion.charged);
}

#[test]
fn completion_cc05_piercing_hits_only_one_enemy_behind_the_primary() {
    let mut s = arena();
    let a = add(&mut s, Spinehurler, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    let behind = add(&mut s, Archer, Seat::B, 5000);
    let farther = add(&mut s, Archer, Seat::B, 5300);
    let front = add(&mut s, Archer, Seat::B, 4400);
    for _ in 0..3 {
        basic(&mut s, a, b);
    }
    assert!(get(&s, behind).hp < get(&s, behind).max_hp);
    assert_eq!(get(&s, farther).hp, get(&s, farther).max_hp);
    assert_eq!(get(&s, front).hp, get(&s, front).max_hp);
}

#[test]
fn completion_cc06_regrowth_survives_absorbed_damage_but_stops_on_hp_damage() {
    let mut s = arena();
    let a = add(&mut s, MireTender, Seat::A, 4000);
    let b = add(&mut s, Shieldbearer, Seat::A, 4500);
    wound(&mut s, b, 300);
    basic(&mut s, a, b);
    let i = s.unit_index(b).unwrap();
    s.units[i].fx.shield = 50;
    s.units[i].fx.shield_until = s.tick + 100;
    hits::damage(&mut s, Target::Unit(b), 10, &mut Vec::new());
    assert!(get(&s, b).fx.completion.regrowth.left > 0);
    hits::damage(&mut s, Target::Unit(b), 100, &mut Vec::new());
    assert_eq!(get(&s, b).fx.completion.regrowth.left, 0);
    let hp = get(&s, b).hp;
    s.tick += 20;
    tick(&mut s, &mut Vec::new());
    assert_eq!(get(&s, b).hp, hp);
}

#[test]
fn completion_cc07_interrupt_preserves_basic_attack_and_respects_ward_and_shared_lockout() {
    let mut s = arena();
    let a = add(&mut s, Gorgehowl, Seat::A, 4000);
    let b = add(&mut s, Whisperer, Seat::B, 4700);
    let i = s.unit_index(b).unwrap();
    s.units[i].fx.special = Some(PendingCast {
        target: Some(a),
        left: 6,
    });
    s.units[i].windup = Some((Target::Unit(a), 5));
    let u = get(&s, a);
    s.units[i].fx.ward_until = s.tick + 10;
    special_resolve(&mut s, &u, b, &mut Vec::new());
    assert!(get(&s, b).fx.special.is_some());
    s.units[i].fx.ward_until = 0;
    special_resolve(&mut s, &u, b, &mut Vec::new());
    assert!(get(&s, b).fx.special.is_none());
    assert_eq!(get(&s, b).windup, Some((Target::Unit(a), 5)));
    s.units[i].fx.special = Some(PendingCast {
        target: Some(a),
        left: 6,
    });
    special_resolve(&mut s, &u, b, &mut Vec::new());
    assert!(get(&s, b).fx.special.is_some());
    assert!(special_windup(s.rules(), &u) < content::WINDUP);
}

#[test]
fn completion_cc08_spark_needs_another_conventional_melee_and_is_consumed_once() {
    let mut s = arena();
    let a = add(&mut s, Sunlancer, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    let melee = add(&mut s, Sawbot, Seat::A, 4200);
    let ranged = add(&mut s, Archer, Seat::A, 3900);
    for _ in 0..3 {
        basic(&mut s, a, b);
    }
    assert!(get(&s, b).fx.completion.spark.amount > 0);
    for id in [a, ranged] {
        let u = get(&s, id);
        let st = s.stats_of(&u);
        assert_eq!(basic_bonus(&mut s, &u, Target::Unit(b), &st), 0);
    }
    let u = get(&s, melee);
    let st = s.stats_of(&u);
    assert!(basic_bonus(&mut s, &u, Target::Unit(b), &st) > 0);
    assert_eq!(basic_bonus(&mut s, &u, Target::Unit(b), &st), 0);
}

#[test]
fn completion_cc09_lunar_healing_needs_actual_enemy_hp_damage() {
    let mut s = arena();
    let a = add(&mut s, EclipseDancer, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    let i = s.unit_index(a).unwrap();
    s.units[i].fx.attacks = 1;
    wound(&mut s, a, 100);
    wound(&mut s, b, 200);
    let j = s.unit_index(b).unwrap();
    s.units[j].fx.shield = 1000;
    s.units[j].fx.shield_until = s.tick + 100;
    basic(&mut s, a, b);
    assert_eq!(get(&s, a).hp, 100);
    s.units[i].fx.attacks = 1;
    s.units[j].fx.shield = 0;
    basic(&mut s, a, b);
    assert!(get(&s, a).hp > 100);
    assert_eq!(lunar_bp(&s, &get(&s, a), Target::Stronghold(Seat::B)), 0);
}

#[test]
fn completion_cc10_crusher_reads_printed_hp_not_buffs_and_never_bypasses_shields() {
    let mut s = arena();
    let a = add(&mut s, Starbreaker, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    let u = get(&s, a);
    let st = s.stats_of(&u);
    let bonus = basic_bonus(&mut s, &u, Target::Unit(b), &st);
    assert!(bonus > 0);
    let j = s.unit_index(b).unwrap();
    s.units[j].max_hp *= 3;
    s.units[j].hp *= 3;
    assert_eq!(basic_bonus(&mut s, &u, Target::Unit(b), &st), bonus);
    s.units[j].fx.shield = 1000;
    s.units[j].fx.shield_until = s.tick + 100;
    let hp = get(&s, b).hp;
    basic(&mut s, a, b);
    assert_eq!(get(&s, b).hp, hp);
    assert_eq!(basic_bonus(&mut s, &u, Target::Stronghold(Seat::B), &st), 0);
}

#[test]
fn completion_cc11_front_pressure_is_snapshotted_at_heal_preparation() {
    let mut s = arena();
    let a = add(&mut s, Dawnmender, Seat::A, 4000);
    let b = add(&mut s, Shieldbearer, Seat::A, 4500);
    let enemy = add(&mut s, Archer, Seat::B, 5000);
    wound(&mut s, b, 100);
    let i = s.unit_index(a).unwrap();
    let u = get(&s, a);
    let st = s.stats_of(&u);
    prepare_heal(&mut s, i, Target::Unit(b), &st);
    let prepared = get(&s, a).fx.completion.prepared_heal;
    assert!(prepared > 0);
    let j = s.unit_index(enemy).unwrap();
    s.units[j].x = 12000;
    basic(&mut s, a, b);
    assert!(get(&s, b).hp > 100 + st.heal);
    prepare_heal(&mut s, i, Target::Unit(b), &st);
    assert_eq!(get(&s, a).fx.completion.prepared_heal, 0);
}

#[test]
fn completion_cc12_beam_keeps_its_original_segment_after_primary_death() {
    let mut s = arena();
    let a = add(&mut s, Prismancer, Seat::A, 4000);
    let b = add(&mut s, Archer, Seat::B, 6000);
    let in_line = add(&mut s, BoneBrute, Seat::B, 6100);
    let outside = add(&mut s, Archer, Seat::B, 9000);
    let i = s.unit_index(a).unwrap();
    special_start(&mut s, i, 6000);
    s.units[i].fx.special = Some(PendingCast {
        target: Some(b),
        left: 1,
    });
    wound(&mut s, b, 0);
    crate::specials::cancel_gone(&mut s);
    assert!(get(&s, a).fx.special.is_some());
    let u = get(&s, a);
    special_resolve(&mut s, &u, b, &mut Vec::new());
    assert!(get(&s, in_line).hp < get(&s, in_line).max_hp);
    assert_eq!(get(&s, outside).hp, get(&s, outside).max_hp);
    assert_eq!(state::view(&u, s.tick).unwrap().beam_center, Some(6000));
}

#[test]
fn completion_cc13_bleed_refreshes_one_family_and_keeps_producer_attribution() {
    let mut s = arena();
    let a = add(&mut s, Leafcutter, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    for _ in 0..5 {
        basic(&mut s, a, b);
    }
    let victim = get(&s, b);
    assert_eq!(victim.fx.venom.left, 0);
    assert!(victim.fx.bleed.left > 0);
    assert_eq!(victim.fx.bleed.producer, Some(Leafcutter));
    let i = s.unit_index(b).unwrap();
    assert!(crate::roster_slots::cleanse_bleed(&mut s.units[i].fx));
    assert_eq!(get(&s, b).fx.bleed.left, 0);
}

#[test]
fn completion_cc14_death_pop_is_single_combat_only_and_never_recursively_detonates() {
    let mut s = arena();
    let a = add(&mut s, Blastbeetle, Seat::A, 4000);
    let b = add(&mut s, Blastbeetle, Seat::B, 4500);
    let ally = add(&mut s, Archer, Seat::A, 4100);
    wound(&mut s, b, 1);
    let before = get(&s, ally).hp;
    hits::damage(&mut s, Target::Unit(a), 10000, &mut Vec::new());
    assert!(death_bursts(&mut s, &mut Vec::new()));
    assert_eq!(get(&s, b).hp, 0);
    assert!(!death_bursts(&mut s, &mut Vec::new()));
    assert_eq!(get(&s, ally).hp, before);
    let c = add(&mut s, Blastbeetle, Seat::A, 4100);
    wound(&mut s, c, 0);
    assert!(!death_bursts(&mut s, &mut Vec::new()));
}

#[test]
fn completion_cc15_opportunity_is_a_timing_bonus_not_an_interrupt() {
    let mut s = arena();
    let a = add(&mut s, Needlewing, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    let i = s.unit_index(a).unwrap();
    s.units[i].fx.attacks = 2;
    let u = get(&s, a);
    let st = s.stats_of(&u);
    assert_eq!(basic_bonus(&mut s, &u, Target::Unit(b), &st), 0);
    let j = s.unit_index(b).unwrap();
    s.units[j].windup = Some((Target::Unit(a), 4));
    let before = get(&s, b).windup;
    assert!(basic_bonus(&mut s, &u, Target::Unit(b), &st) > 0);
    assert_eq!(get(&s, b).windup, before);
}

#[test]
fn completion_cc16_channel_locks_patient_and_root_stops_remaining_pulses() {
    let mut s = arena();
    let a = add(&mut s, NectarNurse, Seat::A, 4000);
    let b = add(&mut s, Shieldbearer, Seat::A, 4500);
    let other = add(&mut s, Shieldbearer, Seat::A, 4300);
    wound(&mut s, b, 100);
    wound(&mut s, other, 50);
    basic(&mut s, a, b);
    assert_eq!(get(&s, a).fx.completion.channel.unwrap().target, b);
    let before = get(&s, b).hp;
    s.tick += 5;
    tick(&mut s, &mut Vec::new());
    assert!(get(&s, b).hp > before);
    assert_eq!(get(&s, other).hp, 50);
    crate::hits::apply_on_hit(
        &mut s,
        a,
        crate::OnHit::Root { ticks: 30 },
        Seat::B,
        &mut Vec::new(),
    );
    assert!(
        !channeling(&get(&s, a)),
        "Root must atomically clear the public channel"
    );
    s.tick += 5;
    tick(&mut s, &mut Vec::new());
    assert!(!channeling(&get(&s, a)));
    let before = get(&s, b).hp;
    s.tick += 50;
    tick(&mut s, &mut Vec::new());
    assert_eq!(get(&s, b).hp, before);
}

#[test]
fn completion_cc17_rhythm_keeps_standard_root_priority_and_two_victim_cap() {
    let mut s = arena();
    let a = add(&mut s, Stridulator, Seat::A, 4000);
    let victims: Vec<_> = (0..4)
        .map(|n| add(&mut s, BoneBrute, Seat::B, 4700 + n * 100))
        .collect();
    let i = s.unit_index(victims[0]).unwrap();
    s.units[i].status = Status::Rooted { ticks_left: 50 };
    let u = get(&s, a);
    special_resolve(&mut s, &u, victims[0], &mut Vec::new());
    assert!(matches!(
        get(&s, victims[0]).status,
        Status::Rooted { ticks_left: 50 }
    ));
    assert_eq!(
        victims
            .iter()
            .filter(|&&id| matches!(get(&s, id).status, Status::Slowed { .. }))
            .count(),
        1
    );
}

#[test]
fn completion_pool_keeps_nine_full_factions_and_one_god() {
    assert_eq!(crate::FULL_POOL.len(), 46);
    assert_eq!(crate::COMPLETION_POOL.len(), 82);
    for faction in crate::FACTIONS {
        assert_eq!(
            crate::COMPLETION_POOL
                .iter()
                .filter(|&&kind| content::def(kind).faction == faction)
                .count(),
            if faction == crate::Faction::Gods {
                1
            } else {
                9
            }
        );
    }
    assert!(crate::COMPLETION_POOL.iter().all(|kind| !kind.is_summon()));
}

#[test]
fn completion_fusion_preserves_harmful_spark_and_view_roundtrips() {
    let mut a = state::Fx::default();
    a.spark = state::Spark {
        amount: 20,
        until: 100,
        source: 5,
    };
    let b = state::Fx::default();
    let result = state::inherit(a, b, 60);
    assert_eq!(result.spark, a.spark);
    let mut s = arena();
    let id = add(&mut s, Archer, Seat::A, 4000);
    let i = s.unit_index(id).unwrap();
    s.units[i].fx.completion = result;
    let view = state::view(&get(&s, id), 60).unwrap();
    let bytes = postcard::to_allocvec(&view).unwrap();
    assert_eq!(postcard::from_bytes::<state::View>(&bytes).unwrap(), view);
}

#[test]
fn completion_seeded_mixed_preview_remains_deterministic_with_both_evolution_branches() {
    fn run() -> (u64, Vec<crate::UnitView>) {
        let mut s = arena();
        for (n, kind) in crate::COMPLETION_POOL.iter().skip(46).copied().enumerate() {
            for seat in Seat::BOTH {
                let id = add(
                    &mut s,
                    kind,
                    seat,
                    if seat == Seat::A { 5000 } else { 5800 },
                );
                let i = s.unit_index(id).unwrap();
                s.units[i].tier = 2;
                s.units[i].branch = (n % 2) as u8;
            }
        }
        for _ in 0..100 {
            s.step();
        }
        let view = s.view_for(Seat::A, &[]);
        (s.hash(), view.units)
    }
    assert_eq!(run(), run());
}

#[cfg(feature = "training")]
#[test]
fn completion_all_180_forms_start_and_advance_in_private_training() {
    use crate::training::{
        Opponent, SessionId, TrainingCommand, TrainingScenario, TrainingSession, TrainingUnitSpec,
        SCHEMA,
    };
    let mut covered = 0;
    for kind in crate::COMPLETION_POOL.into_iter().skip(46) {
        for (tier, branch) in [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)] {
            let scenario = TrainingScenario {
                schema: SCHEMA,
                seed: 1717,
                arena: crate::Arena::Forest,
                allies: vec![
                    TrainingUnitSpec {
                        kind,
                        tier,
                        branch,
                        x: Some(4000),
                    },
                    TrainingUnitSpec {
                        kind: Shieldbearer,
                        tier: 0,
                        branch: 0,
                        x: Some(4500),
                    },
                ],
                opponent: Opponent::Champions(vec![TrainingUnitSpec {
                    kind: BoneBrute,
                    tier: 0,
                    branch: 0,
                    x: Some(5100),
                }]),
            };
            let mut session = TrainingSession::new(scenario, SessionId(covered + 1))
                .expect("CC-ID01 local roster must be trainable");
            assert!(session
                .state()
                .units
                .iter()
                .any(|u| u.kind == kind && u.tier == tier && u.branch == branch));
            let energies = session.state().players.each_ref().map(|p| p.energy);
            session
                .command(session.id(), TrainingCommand::Start)
                .expect("start exact draft");
            for _ in 0..200 {
                session.tick();
            }
            assert!(session.active_ticks() > 0, "{kind:?} {tier}/{branch}");
            assert_eq!(
                session.state().players.each_ref().map(|p| p.energy),
                energies,
                "training must not mutate economy"
            );
            assert!(session.state().outcome.is_none());
            covered += 1;
        }
    }
    assert_eq!(covered, 180);
}

#[test]
fn completion_ogre_sweep_and_stitches_keep_allied_damage_and_pulse_contracts() {
    // OG-01: reuse the authoritative sweep/regrowth families; visual equipment adds no rules.
    let mut s = arena();
    let a = add(&mut s, CinderMaul, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 4700);
    let extra = add(&mut s, BoneBrute, Seat::B, 4800);
    let ally = add(&mut s, Archer, Seat::A, 4800);
    let before = get(&s, ally).hp;
    for _ in 0..2 {
        basic(&mut s, a, b);
    }
    assert_eq!(get(&s, extra).hp, get(&s, extra).max_hp);
    basic(&mut s, a, b);
    assert!(get(&s, extra).hp < get(&s, extra).max_hp);
    assert_eq!(get(&s, ally).hp, before);
    let healer = add(&mut s, HideMender, Seat::A, 4000);
    wound(&mut s, ally, 100);
    basic(&mut s, healer, ally);
    let healed = get(&s, ally).hp;
    s.tick += 20;
    tick(&mut s, &mut Vec::new());
    assert!(get(&s, ally).hp > healed);
    hits::damage(&mut s, Target::Unit(ally), 1, &mut Vec::new());
    assert_eq!(get(&s, ally).fx.completion.regrowth.left, 0);
}
#[test]
fn completion_shared_interrupt_preserves_basic_windup_after_og02_replacement() {
    // OG-02 removes the drum interrupt; Gorgehowl retains this shared correctness contract.
    let mut s = arena();
    let a = add(&mut s, Gorgehowl, Seat::A, 4000);
    let b = add(&mut s, Whisperer, Seat::B, 4700);
    let i = s.unit_index(b).unwrap();
    s.units[i].fx.special = Some(PendingCast {
        target: Some(a),
        left: 6,
    });
    s.units[i].windup = Some((Target::Unit(a), 5));
    let u = get(&s, a);
    s.units[i].fx.ward_until = s.tick + 10;
    special_resolve(&mut s, &u, b, &mut Vec::new());
    assert!(get(&s, b).fx.special.is_some());
    s.units[i].fx.ward_until = 0;
    special_resolve(&mut s, &u, b, &mut Vec::new());
    assert!(get(&s, b).fx.special.is_none());
    assert_eq!(get(&s, b).windup, Some((Target::Unit(a), 5)));
}

#[test]
fn completion_ogre_earth_telegraph_is_snapshot_authoritative() {
    let mut s = arena();
    let a = add(&mut s, ClayBinder, Seat::A, 4000);
    let b = add(&mut s, BoneBrute, Seat::B, 6000);
    let i = s.unit_index(a).unwrap();
    special_start(&mut s, i, 6000);
    s.units[i].fx.special = Some(PendingCast {
        target: Some(b),
        left: 6,
    });
    let u = get(&s, a);
    assert_eq!(state::view(&u, s.tick).unwrap().beam_center, Some(6000));
    s.units[i].fx.special = None;
    assert_eq!(
        state::view(&get(&s, a), s.tick).and_then(|v| v.beam_center),
        None
    );
}
