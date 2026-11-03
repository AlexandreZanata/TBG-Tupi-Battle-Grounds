//! MOBA content tables. Every number is **Proposed** and unbalanced
//! (`docs/moba/GAME-DESIGN-MOBA.md` §6); balance waits for an explicit owner request.

use crate::geom::MM;
use crate::types::{MinionKind, NeutralKind, TowerTier, TroopKind, UnitKind};

/// Simulation ticks per second (M-01).
pub const TICK_HZ: u32 = 20;
/// Hard match limit: 25 minutes (M-30).
pub const MATCH_TICKS: u32 = 25 * 60 * TICK_HZ;

pub const fn secs(s: u32) -> u32 {
    s * TICK_HZ
}

/// Per-unit combat and movement stats; distances in millimetres.
#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub hp: i32,
    pub damage: i32,
    pub range: i32,
    pub attack_ticks: u16,
    /// Millimetres per tick; 0 = immobile.
    pub speed: i32,
    /// Percent of incoming damage ignored.
    pub armor: i32,
    /// Distance at which the unit notices enemies.
    pub sight: i32,
    pub radius: i32,
}

const fn u(units_x10: i32) -> i32 {
    units_x10 * MM / 10
}

/// Units/s × 10 → mm/tick.
const fn speed(units_x10: i32) -> i32 {
    units_x10 * MM / 10 / TICK_HZ as i32
}

const fn stats(hp: i32, damage: i32, range_x10: i32, attack_ticks: u16, speed_x10: i32) -> Stats {
    Stats {
        hp,
        damage,
        range: u(range_x10),
        attack_ticks,
        speed: speed(speed_x10),
        armor: 0,
        sight: u(80),
        radius: u(6),
    }
}

pub const COMMANDER_LEVEL_HP: i32 = 80;
pub const COMMANDER_LEVEL_DAMAGE: i32 = 4;
pub const MAX_LEVEL: u8 = 12;

pub fn base_stats(kind: UnitKind) -> Stats {
    match kind {
        UnitKind::Commander(role) => Stats {
            armor: 10,
            radius: u(8),
            sight: u(90),
            ..stats(900, 40, if role.ranged() { 65 } else { 18 }, 18, 42)
        },
        UnitKind::Minion(MinionKind::Melee) => stats(300, 14, 16, 20, 32),
        UnitKind::Minion(MinionKind::Ranged) => stats(200, 18, 60, 24, 32),
        UnitKind::Troop(kind) => troop_stats(kind),
        UnitKind::Neutral(NeutralKind::Small) => stats(350, 16, 16, 20, 30),
        UnitKind::Neutral(NeutralKind::Big) => Stats {
            radius: u(12),
            ..stats(900, 35, 20, 24, 28)
        },
        UnitKind::Tower(tier, _) => Stats {
            hp: match tier {
                TowerTier::Outer => 2500,
                TowerTier::Inner => 2800,
                TowerTier::Base => 3000,
                TowerTier::Nexus => 3500,
            },
            sight: u(90),
            radius: u(16),
            ..stats(0, 120, 90, 20, 0)
        },
        UnitKind::Stronghold => Stats {
            sight: u(100),
            radius: u(40),
            ..stats(6000, 150, 100, 20, 0)
        },
    }
}

fn troop_stats(kind: TroopKind) -> Stats {
    match kind {
        TroopKind::Guardian => Stats {
            armor: 30,
            radius: u(8),
            ..stats(700, 20, 16, 20, 30)
        },
        TroopKind::Healer => stats(320, 8, 50, 24, 32),
        TroopKind::Bard => stats(340, 14, 50, 22, 32),
        TroopKind::Colossus => Stats {
            armor: 20,
            radius: u(11),
            ..stats(1300, 34, 20, 28, 24)
        },
        TroopKind::Summoner => stats(360, 26, 70, 26, 30),
    }
}

/// Gold price shown on the Shop card (M-10).
pub fn troop_cost(kind: TroopKind) -> u32 {
    match kind {
        TroopKind::Guardian => 18,
        TroopKind::Healer => 12,
        TroopKind::Bard => 14,
        TroopKind::Colossus => 20,
        TroopKind::Summoner => 16,
    }
}

/// Distinct visible rule of each archetype (§6.3).
pub const HEAL_AMOUNT: i32 = 30;
pub const HEAL_TICKS: u16 = 30;
pub const HEAL_RANGE: i32 = 6 * MM;
pub const BARD_AURA_RANGE: i32 = 6 * MM;
pub const BARD_AURA_PERCENT: i32 = 25;
pub const SLOW_TICKS: u32 = 30;
pub const SLOW_PERCENT: i32 = 40;
pub const COLOSSUS_STRUCTURE_MULTIPLIER: i32 = 2;

/// Alive Troops one player may own at once (M-11).
pub const TROOP_CAP: u8 = 8;

// Economy (M-12..M-16).
pub const START_GOLD: u32 = 60;
pub const GOLD_PER_SECOND: u32 = 1;
pub const ESSENCE_EVERY_TICKS: u32 = secs(10);
pub const MINION_BOUNTY: u32 = 4;
pub const NEUTRAL_BOUNTY: [u32; 2] = [6, 15];
pub const COMMANDER_BOUNTY: u32 = 40;
pub const ASSIST_BOUNTY: u32 = 15;
pub const TOWER_TEAM_BOUNTY: u32 = 25;
pub const TOWER_TEAM_ESSENCE: u32 = 2;
pub const COMMANDER_KILL_ESSENCE: u32 = 3;
pub const BIG_NEUTRAL_ESSENCE: u32 = 2;
pub const SHARE_RANGE: i32 = 12 * MM;

/// Troop upgrade tiers bought with Essence (M-17).
pub const MAX_TIER: u8 = 5;
pub const TIER_PERCENT: i32 = 10;

pub fn upgrade_cost(tier: u8) -> u32 {
    15 + 10 * tier as u32
}

// Commander progression (M-20..M-22).
pub fn xp_to_next(level: u8) -> u32 {
    80 + 40 * level as u32
}

pub fn respawn_ticks(level: u8) -> u32 {
    secs(5) + level as u32 * TICK_HZ * 3 / 2
}

pub fn xp_reward(kind: UnitKind) -> u32 {
    match kind {
        UnitKind::Minion(_) => 20,
        UnitKind::Troop(_) => 25,
        UnitKind::Neutral(NeutralKind::Small) => 30,
        UnitKind::Neutral(NeutralKind::Big) => 60,
        UnitKind::Commander(_) => 120,
        UnitKind::Tower(..) | UnitKind::Stronghold => 0,
    }
}

/// Auto Commanders retreat below this HP percent and return above the second.
pub const RETREAT_PERCENT: i32 = 25;
pub const RETURN_PERCENT: i32 = 95;
pub const FOUNTAIN_RANGE: i32 = 14 * MM;
/// HP per tick at the fountain, as a divisor of max HP (≈5 %/s).
pub const FOUNTAIN_DIVISOR: i32 = 400;

// Waves and camps (M-23..M-25).
pub const FIRST_WAVE_TICK: u32 = secs(10);
pub const WAVE_EVERY_TICKS: u32 = secs(30);
pub const WAVE: [MinionKind; 5] = [
    MinionKind::Melee,
    MinionKind::Melee,
    MinionKind::Melee,
    MinionKind::Ranged,
    MinionKind::Ranged,
];
pub const CAMP_RESPAWN_TICKS: u32 = secs(60);
pub const CAMP_LEASH: i32 = 9 * MM;
/// Manual Hold leash around the hold point.
pub const HOLD_LEASH: i32 = 7 * MM;
/// Lane units advance to the next waypoint inside this radius.
pub const WAYPOINT_RADIUS: i32 = 2 * MM;
