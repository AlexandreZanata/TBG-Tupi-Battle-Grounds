//! Stronghold kinds and Fortification (GAME-DESIGN §10.4, BR-SK01–SK09,
//! D-46). Append-only: change a test here only in the same commit as the
//! rule it guards.

use tbg_core::content::{def, ENERGY_SCALE, FORTIFY_COST, STRONGHOLD_HP, STRONGHOLD_INTERVAL};
use tbg_core::stronghold::{self, rescale_hp};
use tbg_core::{
    Event, MatchState, Phase, PlayerIntention, Rejection, Ruleset, Seat, StrongholdKind, Target,
    UnitKind,
};

use StrongholdKind::{Artillery, Bastion, Classic, Command, Treasury};

/// Running, with only the effects under test: no regen, no Interest, no
/// Augment rounds.
fn arena(kinds: [StrongholdKind; 2]) -> MatchState {
    let mut rules = Ruleset::canon().clone();
    rules.regen_running = 0;
    rules.interest.max_bands = 0;
    rules.augment.round_ticks = 0;
    let rules: &'static Ruleset = Box::leak(Box::new(rules));
    let mut s = MatchState::with_rules(7, rules).with_strongholds(kinds);
    while s.phase != Phase::Running {
        s.step();
    }
    s
}

fn energy(s: &mut MatchState, seat: Seat, energy: u32) {
    s.players[seat.index()].energy = energy * ENERGY_SCALE;
}

fn fortify(s: &mut MatchState, seat: Seat, branch: u8) -> Result<(), Rejection> {
    let seq = s.players[seat.index()].last_seq + 1;
    s.apply_intention(seat, seq, PlayerIntention::Fortify { branch })
}

fn spawn(s: &mut MatchState, seat: Seat, kind: UnitKind, x: i32) -> u32 {
    s.players[seat.index()].offers[0] = kind;
    let before = s.players[seat.index()].energy;
    s.players[seat.index()].energy = 100 * ENERGY_SCALE;
    let seq = s.players[seat.index()].last_seq + 1;
    s.apply_intention(seat, seq, PlayerIntention::Purchase { offer: 0 })
        .unwrap();
    s.players[seat.index()].energy = before;
    let unit = s.units.last_mut().unwrap();
    unit.x = x;
    unit.id
}

#[test]
fn br_sk01_four_free_kinds_default_bastion_classic_only_for_the_lab() {
    assert_eq!(
        StrongholdKind::SELECTABLE,
        [Treasury, Bastion, Artillery, Command]
    );
    assert!(StrongholdKind::SELECTABLE.iter().all(|k| k.selectable()));
    assert!(!Classic.selectable());
    assert_eq!(StrongholdKind::DEFAULT, Bastion);
    // A plain match is the Classic reference tower (tests, lab).
    let s = MatchState::new(1);
    assert_eq!(s.players[0].stronghold, Classic);
    assert_eq!(s.players[0].stronghold_hp, STRONGHOLD_HP);
}

#[test]
fn br_sk03_kind_bases_keep_their_roles() {
    let s = arena([Treasury, Bastion]);
    let t = s.stronghold(Seat::A);
    let b = s.stronghold(Seat::B);
    let rules = Ruleset::canon();
    let a = stronghold::stats(rules, Artillery, 0, 0);
    let c = stronghold::stats(rules, Command, 0, 0);
    // Bastion has the most HP, Artillery the most damage and the least HP.
    for other in [t, a, c] {
        assert!(b.max_hp > other.max_hp);
    }
    for other in [t, b, c] {
        assert!(a.damage > other.damage);
        assert!(a.max_hp <= other.max_hp);
    }
    // Command is in between: under Bastion's HP and Artillery's damage.
    assert!(c.max_hp < b.max_hp && c.damage < a.damage);
    // Each tower starts at its own full HP.
    assert_eq!(s.players[0].stronghold_hp, t.max_hp);
    assert_eq!(s.players[1].stronghold_hp, b.max_hp);
    assert_eq!(b.max_hp, STRONGHOLD_HP * 11_500 / 10_000);
}

#[test]
fn br_sk05_tier_one_picks_the_branch_tier_two_repeats_it() {
    let mut s = arena([Bastion, Classic]);
    energy(&mut s, Seat::A, 20);
    assert_eq!(fortify(&mut s, Seat::A, 2), Err(Rejection::WrongBranch));
    assert_eq!(fortify(&mut s, Seat::A, 1), Ok(()));
    let p = &s.players[0];
    assert_eq!((p.fort_tier, p.fort_branch), (1, 1));
    assert_eq!(p.energy, 20 * ENERGY_SCALE - FORTIFY_COST[0]);
    // The other branch is closed for the rest of the match.
    assert_eq!(fortify(&mut s, Seat::A, 0), Err(Rejection::WrongBranch));
    assert_eq!(fortify(&mut s, Seat::A, 1), Ok(()));
    assert_eq!(s.players[0].fort_tier, 2);
    // No tier 3, and no second purchase of a tier.
    let before = s.players[0].energy;
    assert_eq!(fortify(&mut s, Seat::A, 1), Err(Rejection::MaxTier));
    assert_eq!(s.players[0].energy, before);
    // The other seat's tower never changed: the seat comes from the
    // connection, there is no target to name (BR-A01).
    assert_eq!(s.players[1].fort_tier, 0);
    assert!(s.step().contains(&Event::Fortified {
        seat: Seat::A,
        tier: 2,
        branch: 1
    }));
}

#[test]
fn br_sk05_costs_are_three_and_four_energy_and_need_the_energy() {
    assert_eq!(FORTIFY_COST, [3 * ENERGY_SCALE, 4 * ENERGY_SCALE]);
    let mut s = arena([Artillery, Classic]);
    s.players[0].energy = FORTIFY_COST[0] - 1;
    assert_eq!(fortify(&mut s, Seat::A, 0), Err(Rejection::NotEnoughEnergy));
    assert_eq!(s.players[0].fort_tier, 0);
    s.players[0].energy = FORTIFY_COST[0];
    assert_eq!(fortify(&mut s, Seat::A, 0), Ok(()));
    assert_eq!(s.players[0].energy, 0);
    s.players[0].energy = FORTIFY_COST[1] - 1;
    assert_eq!(fortify(&mut s, Seat::A, 0), Err(Rejection::NotEnoughEnergy));
    s.players[0].energy = FORTIFY_COST[1];
    assert_eq!(fortify(&mut s, Seat::A, 0), Ok(()));
}

#[test]
fn br_sk05_no_fortification_after_the_end() {
    let mut s = arena([Bastion, Classic]);
    energy(&mut s, Seat::A, 20);
    s.forfeit(Seat::B);
    assert_eq!(fortify(&mut s, Seat::A, 0), Err(Rejection::NotRunning));
    assert_eq!(s.players[0].fort_tier, 0);
}

#[test]
fn br_sk07_more_max_hp_keeps_the_ratio_never_heals() {
    let mut s = arena([Bastion, Classic]);
    let max0 = s.stronghold(Seat::A).max_hp;
    s.players[0].stronghold_hp = max0 / 2;
    energy(&mut s, Seat::A, 20);
    fortify(&mut s, Seat::A, 0).unwrap();
    let max1 = s.stronghold(Seat::A).max_hp;
    assert_eq!(max1, STRONGHOLD_HP * 11_500 / 10_000 * 11_500 / 10_000);
    assert_eq!(s.players[0].stronghold_hp, rescale_hp(max0 / 2, max0, max1));
    // Same ratio, rounded down: never a free heal of the HP already lost.
    let (hp0, hp) = (i64::from(max0 / 2), i64::from(s.players[0].stronghold_hp));
    let (max0, max1) = (i64::from(max0), i64::from(max1));
    assert!(hp * max0 <= hp0 * max1);
    assert!((hp + 1) * max0 > hp0 * max1);
    assert_eq!(rescale_hp(1, 4_500, 9_000), 2);
    assert_eq!(rescale_hp(1, 9_000, 4_500), 1, "alive stays alive");
    assert_eq!(rescale_hp(0, 4_500, 9_000), 0);
}

#[test]
fn br_sk06_every_branch_changes_the_real_numbers() {
    let rules = Ruleset::canon();
    let at = |kind, tier, branch| stronghold::stats(rules, kind, tier, branch);
    // Treasury: Armored Vault +12 % / +12 % HP; Mint 0.20 / 0.25 Energy.
    let t = at(Treasury, 0, 0);
    assert_eq!((t.spoils, t.spoils_cap), (90, 360));
    assert_eq!(at(Treasury, 2, 0).max_hp, t.max_hp * 12_400 / 10_000);
    assert_eq!(at(Treasury, 1, 1).spoils, 120);
    assert_eq!(
        (at(Treasury, 2, 1).spoils, at(Treasury, 2, 1).spoils_cap),
        (150, 540)
    );
    // Bastion: Fortress +15 % / +15 % HP; Sentinel +1 range then +15 % damage.
    let b = at(Bastion, 0, 0);
    assert_eq!(at(Bastion, 2, 0).max_hp, b.max_hp * 13_000 / 10_000);
    assert_eq!(at(Bastion, 1, 1).range, b.range + 1_000);
    assert_eq!(at(Bastion, 2, 1).damage, b.damage * 11_500 / 10_000);
    // Artillery: Heavy Cannon +20 % / +20 % damage; Rapid Fire 17 then 14.
    let a = at(Artillery, 0, 0);
    assert_eq!(a.interval, STRONGHOLD_INTERVAL);
    assert_eq!(at(Artillery, 2, 0).damage, a.damage * 14_000 / 10_000);
    assert_eq!(at(Artillery, 1, 1).interval, 17);
    assert_eq!(at(Artillery, 2, 1).interval, 14);
    // Command: War Standard 8 % then 11 %; Forward Banner radius 5 then 6.
    let c = at(Command, 0, 0);
    assert_eq!((c.aura_bp, c.aura_range), (500, 4_000));
    assert_eq!(at(Command, 2, 0).aura_bp, 1_100);
    assert_eq!(at(Command, 2, 0).aura_range, 4_000);
    assert_eq!(at(Command, 1, 1).aura_range, 5_000);
    assert_eq!(
        (at(Command, 2, 1).aura_bp, at(Command, 2, 1).aura_range),
        (500, 6_000)
    );
}

/// The tower's real shot: damage on the `Hit` and the cooldown it sets.
fn first_shot(kinds: [StrongholdKind; 2], tier: u8, branch: u8) -> (i32, u16) {
    let mut s = arena(kinds);
    s.players[0].fort_tier = tier;
    s.players[0].fort_branch = branch;
    let id = spawn(&mut s, Seat::B, UnitKind::BoneBrute, 2_000);
    for _ in 0..40 {
        let ev = s.step();
        let shot = ev.iter().any(|e| {
            *e == Event::AttackStart {
                from: Target::Stronghold(Seat::A),
                to: Target::Unit(id),
            }
        });
        if shot {
            let amount = ev
                .iter()
                .find_map(|e| match *e {
                    Event::Hit {
                        to: Target::Unit(t),
                        amount,
                    } if t == id => Some(amount),
                    _ => None,
                })
                .unwrap();
            return (amount, s.players[0].stronghold_cooldown);
        }
    }
    panic!("the tower never fired");
}

#[test]
fn br_sk06_artillery_branches_change_the_real_shot() {
    let base = stronghold::stats(Ruleset::canon(), Artillery, 0, 0);
    assert_eq!(
        first_shot([Artillery, Classic], 0, 0),
        (base.damage, base.interval)
    );
    let heavy = stronghold::stats(Ruleset::canon(), Artillery, 2, 0);
    assert_eq!(
        first_shot([Artillery, Classic], 2, 0),
        (heavy.damage, heavy.interval)
    );
    assert_eq!(first_shot([Artillery, Classic], 2, 1).1, 14);
}

fn kill(s: &mut MatchState, id: u32) {
    s.units.iter_mut().find(|u| u.id == id).unwrap().hp = 0;
    s.step();
}

#[test]
fn br_sk02_spoils_pay_eligible_enemy_deaths_within_the_window_cap() {
    let mut s = arena([Treasury, Classic]);
    s.players[0].energy = 0;
    s.players[1].energy = 0;
    // A friendly death never pays the Treasury.
    let own = spawn(&mut s, Seat::A, UnitKind::Shieldbearer, 1_000);
    kill(&mut s, own);
    assert_eq!(s.players[0].energy, 0);
    // A summon's death never pays (BR-U08).
    let beetle = spawn(&mut s, Seat::B, UnitKind::Shieldbearer, 15_000);
    s.units.last_mut().unwrap().kind = UnitKind::Beetling;
    kill(&mut s, beetle);
    assert_eq!(s.players[0].energy, 0);
    // Each conventional enemy death pays 90 until the 360 window ceiling.
    for i in 0..6 {
        let id = spawn(&mut s, Seat::B, UnitKind::Shieldbearer, 15_000);
        kill(&mut s, id);
        assert_eq!(s.players[0].energy, (90 * (i + 1)).min(360));
    }
    // The opponent (Classic) earns nothing from the deaths of A's units.
    assert_eq!(s.players[1].energy, 0);
    // Any 200 ticks cross a window boundary: the next window pays again.
    for _ in 0..200 {
        s.step();
    }
    let id = spawn(&mut s, Seat::B, UnitKind::Shieldbearer, 15_000);
    kill(&mut s, id);
    assert_eq!(s.players[0].energy, 450);
}

#[test]
fn br_sk02_royal_mint_pays_more_with_a_higher_ceiling() {
    let mut s = arena([Treasury, Classic]);
    s.players[0].energy = 0;
    s.players[0].fort_tier = 2;
    s.players[0].fort_branch = 1;
    for _ in 0..6 {
        let id = spawn(&mut s, Seat::B, UnitKind::Shieldbearer, 15_000);
        kill(&mut s, id);
    }
    assert_eq!(s.players[0].energy, 540);
}

/// Damage of one Shieldbearer hit from A at `x` on a B unit right ahead.
fn hit_at(kinds: [StrongholdKind; 2], x: i32, tier: u8, branch: u8) -> i32 {
    let mut rules = Ruleset::canon().clone();
    rules.regen_running = 0;
    rules.interest.max_bands = 0;
    rules.augment.round_ticks = 0;
    // Towers out of the picture: only the unit's own hit is measured.
    rules.stronghold_range = -1;
    let rules: &'static Ruleset = Box::leak(Box::new(rules));
    let mut s = MatchState::with_rules(3, rules).with_strongholds(kinds);
    while s.phase != Phase::Running {
        s.step();
    }
    s.players[0].fort_tier = tier;
    s.players[0].fort_branch = branch;
    spawn(&mut s, Seat::A, UnitKind::Shieldbearer, x);
    let b = spawn(&mut s, Seat::B, UnitKind::BoneBrute, x + 300);
    for _ in 0..80 {
        let ev = s.step();
        let hit = ev.iter().find_map(|e| match *e {
            Event::Hit {
                to: Target::Unit(t),
                amount,
            } if t == b => Some(amount),
            _ => None,
        });
        if let Some(amount) = hit {
            return amount;
        }
    }
    panic!("no hit");
}

#[test]
fn br_sk04_war_banner_reaches_only_conventional_allies_in_its_radius() {
    let base = def(UnitKind::Shieldbearer).damage;
    // Inside the 4-tile radius: +5 %; outside: nothing.
    assert_eq!(
        hit_at([Command, Classic], 3_000, 0, 0),
        base * 10_500 / 10_000
    );
    assert_eq!(hit_at([Command, Classic], 5_500, 0, 0), base);
    // Other kinds have no aura.
    assert_eq!(hit_at([Bastion, Classic], 3_000, 0, 0), base);
    // War Standard tier 2: +11 %; Forward Banner tier 2 reaches 6 tiles.
    assert_eq!(
        hit_at([Command, Classic], 3_000, 2, 0),
        base * 11_100 / 10_000
    );
    assert_eq!(
        hit_at([Command, Classic], 5_500, 2, 1),
        base * 10_500 / 10_000
    );
}

#[test]
fn br_sk04_the_aura_is_read_at_hit_time_and_ends_outside() {
    // The same unit hits for the bonus inside, then for its base outside:
    // nothing is stored on the unit, so it cannot stack or linger.
    let base = def(UnitKind::Shieldbearer).damage;
    let inside = hit_at([Command, Classic], 3_900, 0, 0);
    let outside = hit_at([Command, Classic], 4_100, 0, 0);
    assert_eq!((inside, outside), (base * 10_500 / 10_000, base));
}

#[test]
fn br_sk08_both_seats_see_both_towers() {
    let mut s = arena([Treasury, Command]);
    energy(&mut s, Seat::B, 10);
    fortify(&mut s, Seat::B, 1).unwrap();
    for seat in Seat::BOTH {
        let v = s.view_for(seat, &[]);
        assert_eq!(v.strongholds[0].kind, Treasury);
        assert_eq!(v.strongholds[1].kind, Command);
        assert_eq!((v.strongholds[1].tier, v.strongholds[1].branch), (1, 1));
        assert_eq!(v.strongholds[0].max_hp, s.stronghold(Seat::A).max_hp);
    }
}

#[test]
fn br_sk09_kind_matches_still_end_by_destruction_and_replay_identically() {
    use tbg_core::bot::Bot;
    use tbg_core::replay::Replay;
    let kinds = [Artillery, Treasury];
    let mut s = MatchState::new(11).with_strongholds(kinds);
    let mut bots = [Bot::new(1), Bot::new(2)];
    let mut replay = Replay {
        seed: 11,
        strongholds: kinds,
        ..Replay::default()
    };
    let mut fortified = [false; 2];
    while s.phase != Phase::Ended && s.tick < 20 * 60 * 60 {
        for seat in Seat::BOTH {
            let view = s.view_for(seat, &[]);
            if let Some(intent) = bots[seat.index()].decide(&view) {
                let seq = s.players[seat.index()].last_seq + 1;
                replay.record(&s, seat, Some((seq, intent)));
                if s.apply_intention(seat, seq, intent).is_ok()
                    && matches!(intent, PlayerIntention::Fortify { .. })
                {
                    fortified[seat.index()] = true;
                }
            }
        }
        s.step();
    }
    assert_eq!(s.phase, Phase::Ended);
    assert!(
        fortified.iter().any(|f| *f),
        "the bot fortifies a chosen kind"
    );
    assert_eq!(replay.run().hash(), s.hash());
}
