//! Guard tests: pin the canon values of docs/GAME-DESIGN.md §5 and the
//! ranked rules (BR-R03, BR-R05) so no number changes by accident. A failure
//! here means a rule changed: update GAME-DESIGN (marked **Proposed**) in the
//! same commit, then this file (AGENTS.md non-negotiable 1). Values are
//! written in the document's units (seconds, tiles, whole `Energy`) and
//! converted with the simulation's own scales.

use tbg_core::account::{encode_secret, format_key, MAX_KEY_LEN, MAX_NAME_LEN, RESERVED_NAMES};
use tbg_core::content::*;
use tbg_core::protocol::{LEADERBOARD_PAGE, MAX_CLIENT_MSG_BYTES, ROOM_ALPHABET};
use tbg_core::rating::{self, apply, delta, Score};

/// Milli-tiles per tile (ARCHITECTURE §4 fixed point).
const TILE: i32 = 1_000;

fn secs(s: u32) -> u32 {
    s * TICK_HZ
}

#[test]
fn open_ended_match_and_lane() {
    // Owner revision 2026-10-29: countdown 3 s, no regulation/overtime.
    assert_eq!(TICK_HZ, 20, "simulation rate (ARCHITECTURE §5: 20 Hz)");
    assert_eq!(COUNTDOWN, secs(3));
    assert_eq!(RUNNING_START, COUNTDOWN);
    assert_eq!(LANE_LEN, 16 * TILE);
    assert_eq!(SPAWN_OFFSET, TILE / 2);
}

#[test]
fn uncapped_energy_and_interest() {
    // Owner revision: start 5 Energy, no gameplay cap, Interest every 10 s.
    assert_eq!(ENERGY_SCALE, 600);
    assert_eq!(ENERGY_START, 5 * ENERGY_SCALE);
    assert_eq!(REGEN_RUNNING, 17);
    assert_eq!(ROTATION_TICKS, secs(10));
    assert_eq!(
        (INTEREST_BAND, INTEREST_PER_BAND, INTEREST_MAX_BANDS),
        (5 * ENERGY_SCALE, ENERGY_SCALE / 4, 5)
    );
    assert_eq!(AUGMENT_ROUND_TICKS, secs(60));
    assert_eq!(AUGMENT_FIRST_ROUND, RUNNING_START + secs(60));
    assert_eq!(AUGMENT_COST, [3, 6, 10, 15].map(|e| e * ENERGY_SCALE));
    let secs_per_energy = ENERGY_SCALE as f64 / (REGEN_RUNNING * TICK_HZ) as f64;
    assert!((secs_per_energy - 1.76).abs() < 0.01, "{secs_per_energy}");
}

#[test]
fn shop_slots_stronghold_and_evolution() {
    // §5 (D-32): 5 distinct offers, full rotation every 10 s, no army cap,
    // Stronghold 4500 HP hitting 65 every 1.0 s at range 6, evolutions cost
    // 3 / 4 Energy. Hatch (BR-U08): every 4 s (Hive 3 s), 2 alive per Nester
    // (Brood 3), 6 per player, lifetime 10 s.
    assert_eq!(OFFERS, 5);
    assert_eq!(ROTATION_TICKS, secs(10));
    assert_eq!(
        (u32::from(HATCH_EVERY), u32::from(HATCH_EVERY_HIVE)),
        (secs(4), secs(3))
    );
    assert_eq!((BEETLES_PER_NESTER, BEETLES_PER_PLAYER), ([2, 3], 6));
    assert_eq!(BEETLE_LIFETIME, secs(10));
    assert_eq!(STRONGHOLD_HP, 4_500);
    assert_eq!(STRONGHOLD_DAMAGE, 53);
    assert_eq!(u32::from(STRONGHOLD_INTERVAL), secs(1));
    assert_eq!(STRONGHOLD_RANGE, 6 * TILE);
    assert_eq!(EVOLVE_COST, [2 * ENERGY_SCALE, 4 * ENERGY_SCALE]);
    // Combat timing shared by every unit (ARCHITECTURE §4).
    assert_eq!((WINDUP, MIN_INTERVAL, BP), (6, 5, 10_000));
}

#[test]
fn synergies() {
    // §5 synergies at 2 / 4 living units: Humans +20/40% max HP, Undead
    // +20/40% attack speed, Machines +5/20% damage, Plants regen 0.75/2% max
    // HP per second (basis points, REGEN_PERIOD = 1 s).
    assert_eq!(SYNERGY_THRESHOLDS, [2, 4]);
    assert_eq!(REGEN_PERIOD, secs(1));
    assert_eq!(
        SYNERGIES,
        [
            (SynergyEffect::MaxHp, [2_000, 4_000]),
            (SynergyEffect::AttackSpeed, [2_000, 4_000]),
            (SynergyEffect::Damage, [500, 2_000]),
            (SynergyEffect::Regen, [75, 200]),
        ]
    );
}

#[test]
fn ranked_rating_constants_and_table() {
    // BR-R03/BR-R05 and §5 "Ranked rating": start 1000, floor 100, K 40 for
    // the first 20 ranked games, then 24; each player uses their own K.
    assert_eq!(rating::START_RATING, 1_000);
    assert_eq!(rating::RATING_FLOOR, 100);
    assert_eq!(rating::PLACEMENT_GAMES, 20);
    assert_eq!((rating::K_PLACEMENT, rating::K_STANDARD), (40.0, 24.0));
    // (own, opponent, own games before, score) -> delta. Frozen outputs of
    // round(K × (S − E)) with the decisive-result ±1 rule.
    let table = [
        (1000, 1000, 0, Score::Win, 20),
        (1000, 1000, 0, Score::Loss, -20),
        (1000, 1000, 0, Score::Draw, 0),
        (1000, 1000, 20, Score::Win, 12),
        (1000, 1000, 19, Score::Loss, -20),
        (1400, 1000, 20, Score::Win, 2),
        (1000, 1400, 20, Score::Win, 22),
        (1000, 1400, 20, Score::Loss, -2),
        (2000, 1000, 20, Score::Win, 1),
        (1000, 2000, 20, Score::Loss, -1),
        (1200, 1000, 20, Score::Draw, -6),
        (1000, 1200, 20, Score::Draw, 6),
    ];
    for (own, opp, games, score, want) in table {
        assert_eq!(
            delta(own, opp, games, score),
            want,
            "delta({own}, {opp}, {games}, {score:?})"
        );
    }
    assert_eq!(apply(110, -20), rating::RATING_FLOOR);
    assert_eq!(apply(1000, 20), 1020);
    assert_eq!(apply(u16::MAX, 5), u16::MAX);
}

#[test]
fn account_and_protocol_limits() {
    // BR-P01/BR-P02: names 3–16 chars, keys `<Name>-XXXX-XXXX-XXXX-XXXX`
    // (16 secret chars, 80 bits), at most 40 bytes; 18 reserved names.
    assert_eq!((MAX_NAME_LEN, MAX_KEY_LEN), (16, 40));
    assert_eq!(RESERVED_NAMES.len(), 18);
    let key = format_key("Abc", &encode_secret(&[0; 10]));
    assert_eq!(key, "Abc-0000-0000-0000-0000");
    let longest = format_key(&"N".repeat(MAX_NAME_LEN), &encode_secret(&[0xff; 10]));
    assert!(longest.len() <= MAX_KEY_LEN, "{longest}");
    // ARCHITECTURE §5 limits and the P13 in-game page size.
    assert_eq!(MAX_CLIENT_MSG_BYTES, 1_024);
    assert_eq!(ROOM_ALPHABET, b"ABCDEFGHJKMNPQRSTUVWXYZ23456789");
    assert_eq!(LEADERBOARD_PAGE, 20);
}
