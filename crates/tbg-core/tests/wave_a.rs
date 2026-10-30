//! Wave A (P44): Farwatch never targets Strongholds, Bombard's friendly
//! blast, Longaxe's second-rank reach, and the splash-share math.
//! Micro-scenarios cite the §C rules they freeze.

#![cfg(feature = "lab")]

use tbg_core::{
    headless, ArcanaPreset, Event, MatchState, Ruleset, Seat, Target, UnitKind,
};

fn rules() -> &'static Ruleset {
    Ruleset::canon()
}

fn state(seed: u64) -> MatchState {
    MatchState::lab(seed, rules(), [ArcanaPreset::NEUTRAL; 2])
}

/// Step `n` ticks, collecting every event.
fn step_n(s: &mut MatchState, n: u32) -> Vec<Event> {
    let mut out = Vec::new();
    for _ in 0..n {
        out.extend(s.step());
    }
    out
}

fn unit_at(s: &MatchState, id: u32) -> tbg_core::Unit {
    let i = s.unit_index(id).expect("unit must exist");
    s.units[i]
}

// Farwatch (§C.2): the enemy Stronghold alone in range is held, never shot.
#[test]
fn farwatch_holds_instead_of_hitting_a_stronghold() {
    for seat in Seat::BOTH {
        let mut s = state(11);
        let (start, home) = match seat {
            Seat::A => (500, 4_000),
            Seat::B => (15_500, 12_000),
        };
        let fw = s.lab_spawn_at(seat, UnitKind::Farwatch, 0, 0, start);
        let evs = step_n(&mut s, 400);
        assert!(
            evs.iter().all(|e| !matches!(e, Event::Hit { .. })),
            "{seat:?}: Farwatch hit something with no enemy unit around"
        );
        assert!(
            (unit_at(&s, fw).x - home).abs() <= 45,
            "{seat:?}: Farwatch did not hold at the Stronghold's edge"
        );
        assert_eq!(
            s.player(seat.other()).stronghold_hp,
            4_500,
            "{seat:?}: the enemy Stronghold took damage"
        );
    }
}

// Guard (BR-U09) still forces Farwatch onto the Rampart (§C.2).
#[test]
fn guard_forces_farwatch_onto_the_rampart() {
    for seat in Seat::BOTH {
        let mut s = state(21);
        let fw = s.lab_spawn_at(seat, UnitKind::Farwatch, 0, 0, 8_000);
        let rampart = s.lab_spawn_at(seat.other(), UnitKind::Rampart, 0, 0, 9_000);
        let evs = step_n(&mut s, 120);
        assert!(
            evs.iter().any(|e| matches!(
                e,
                Event::AttackStart { from: Target::Unit(a), to: Target::Unit(b) }
                if *a == fw && *b == rampart
            )),
            "{seat:?}: Farwatch did not open fire on the Guard Rampart"
        );
    }
}

// Bombard (§C.4): the blast hits itself, at full damage, with no reduction.
#[test]
fn bombard_blast_hits_itself() {
    let mut s = state(31);
    let bombard = s.lab_spawn_at(Seat::A, UnitKind::Bombard, 0, 0, 5_000);
    let _enemy = s.lab_spawn_at(Seat::B, UnitKind::Sawbot, 0, 0, 5_800);
    let evs = step_n(&mut s, 120);
    assert!(
        evs.iter().any(|e| matches!(
            e,
            Event::Hit { to: Target::Unit(id), amount: 24 }
            if *id == bombard
        )),
        "no full-damage blast hit on the Bombard itself"
    );
}

// Bombard (§C.4): allies in the blast take the same full damage.
#[test]
fn bombard_blast_hits_allies_fully() {
    let mut s = state(41);
    let _bombard = s.lab_spawn_at(Seat::A, UnitKind::Bombard, 0, 0, 5_700);
    let ally = s.lab_spawn_at(Seat::A, UnitKind::BoneBrute, 0, 0, 5_600);
    // The enemy stands its ground on the Bombard (nearest), so every blast
    // lands on the same spot with the ally inside.
    let _enemy = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 6_500);
    let evs = step_n(&mut s, 200);
    let ally_hits: Vec<i32> = evs
        .iter()
        .filter_map(|e| match e {
            Event::Hit { to: Target::Unit(id), amount } if *id == ally => Some(*amount),
            _ => None,
        })
        .collect();
    assert!(!ally_hits.is_empty(), "the blast never reached the ally");
    assert!(
        ally_hits.iter().all(|&a| a == 24),
        "ally blast damage is not the full 24: {ally_hits:?}"
    );
}

// Bombard (§C.4): the own Stronghold is never touched by the blast.
#[test]
fn bombard_blast_spares_the_own_stronghold() {
    let mut s = state(51);
    let _bombard = s.lab_spawn_at(Seat::A, UnitKind::Bombard, 0, 0, 2_000);
    // A Farwatch holds its ground, so the blast stays on the same spot.
    let _enemy = s.lab_spawn_at(Seat::B, UnitKind::Farwatch, 0, 0, 800);
    step_n(&mut s, 400);
    assert_eq!(
        s.player(Seat::A).stronghold_hp,
        4_500,
        "the own Stronghold took blast damage"
    );
}

// Bombard (§C.4): an ally killed by the blast is the opponent's eligible
// kill — here observed as Plunder bounty on the enemy seat.
#[test]
fn friendly_fire_kill_credits_the_enemy() {
    let mut s = state(61);
    s.players[1].augments.add(10); // Plunder on seat B.
    let _bombard = s.lab_spawn_at(Seat::A, UnitKind::Bombard, 0, 0, 6_300);
    let ally = s.lab_spawn_at(Seat::A, UnitKind::Archer, 0, 0, 6_100);
    let _enemy = s.lab_spawn_at(Seat::B, UnitKind::Rampart, 0, 0, 7_000);
    for _ in 0..1_200 {
        s.step();
        if s.unit_index(ally).is_none() {
            break;
        }
    }
    assert!(
        s.unit_index(ally).is_none(),
        "the ally survived every blast"
    );
    assert!(
        s.players[1].bounty_window > 0,
        "the enemy seat got no bounty for the blasted ally"
    );
}

// Longaxe (§C.7): range 1.6 stops it behind its own tank, so it hits over
// the front line while the enemy's melee cannot reach it.
#[test]
fn longaxe_strikes_over_its_own_tank() {
    let mut s = state(71);
    let tank = s.lab_spawn_at(Seat::A, UnitKind::Shieldbearer, 0, 0, 7_000);
    let axe = s.lab_spawn_at(Seat::A, UnitKind::Longaxe, 0, 0, 5_400);
    let brute = s.lab_spawn_at(Seat::B, UnitKind::BoneBrute, 0, 0, 8_600);
    let mut saw_attack = false;
    let mut tank_engaged = false;
    for _ in 0..300 {
        for e in s.step() {
            match e {
                Event::AttackStart { from: Target::Unit(a), .. } if a == axe => {
                    saw_attack = true;
                }
                Event::AttackStart { from: Target::Unit(a), .. } if a == brute => {
                    tank_engaged = true;
                }
                _ => {}
            }
        }
    }
    assert!(saw_attack, "Longaxe never attacked over its tank");
    assert!(tank_engaged, "the enemy never engaged (scenario broken)");
    let axe_unit = unit_at(&s, axe);
    assert_eq!(
        axe_unit.hp, axe_unit.max_hp,
        "Longaxe was hit: it is not fighting over its tank"
    );
    assert!(
        unit_at(&s, brute).hp < 1_300,
        "the enemy took no damage from the second rank"
    );
    let _ = tank;
}

// Splash share (§F.1, P44): an upgrade replaces the share (highest active
// form wins); the radius still adds up.
#[test]
fn splash_share_follows_the_highest_form() {
    let r = rules();
    let b1 = r.effective_stats(UnitKind::Longaxe, 1, 1, 0);
    assert_eq!((b1.splash, b1.splash_bp), (800, 4_000));
    let b2 = r.effective_stats(UnitKind::Longaxe, 2, 1, 0);
    assert_eq!((b2.splash, b2.splash_bp), (1_200, 6_000));
    let base = r.effective_stats(UnitKind::Bombard, 0, 0, 0);
    assert_eq!((base.splash, base.splash_bp), (1_000, 10_000));
    let a2 = r.effective_stats(UnitKind::Bombard, 2, 0, 0);
    assert_eq!((a2.splash, a2.splash_bp), (2_000, 10_000));
}
