//! MOBA content tables. Every number is **Proposed** and unbalanced
//! (`docs/moba/GAME-DESIGN-MOBA.md` §6); balance waits for an explicit owner request.

use crate::geom::MM;
use crate::types::{Champion, MinionKind, NeutralKind, Role, TowerTier, UnitKind};
use tbg_core::types::Archetype;

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
    /// HP mended per attack on the most injured ally in range (healers).
    pub heal: i32,
    /// Radius around the target that also takes splash damage.
    pub splash: i32,
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
        heal: 0,
        splash: 0,
    }
}

pub const COMMANDER_LEVEL_HP: i32 = 80;
pub const COMMANDER_LEVEL_DAMAGE: i32 = 4;
pub const MAX_LEVEL: u8 = 12;

pub fn base_stats(kind: UnitKind) -> Stats {
    match kind {
        UnitKind::Commander(role) => {
            // Reach and rhythm of the role's champion; bulk of a Commander.
            let champion = tbg_core::content::def(commander_champion(role));
            Stats {
                armor: 10,
                radius: u(8),
                sight: u(90),
                range: champion.range.max(u(18)),
                attack_ticks: champion.interval.max(1),
                ..stats(900, 40, 0, 0, 42)
            }
        }
        UnitKind::Minion(MinionKind::Melee) => stats(300, 14, 16, 20, 32),
        UnitKind::Minion(MinionKind::Ranged) => stats(200, 18, 60, 24, 32),
        UnitKind::Troop(kind) => troop_stats(kind),
        UnitKind::Neutral(kind) => neutral_stats(kind),
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

/// A 1v1 champion on the MOBA map: its own HP, damage, attack interval and
/// range (both modes tick at 20 Hz and measure in millimetres), walking
/// faster because lanes here are ~12x longer than the 1v1 BattleLine (M-18).
fn troop_stats(kind: Champion) -> Stats {
    let def = tbg_core::content::def(kind);
    let melee = def.range < 1_500;
    Stats {
        hp: def.hp,
        damage: def.damage,
        range: def.range.max(u(10)),
        attack_ticks: def.interval.max(1),
        speed: def.speed * CHAMPION_SPEED_FACTOR,
        armor: if def.archetype == Archetype::Tank {
            20
        } else {
            0
        },
        sight: u(80),
        radius: if melee { u(7) } else { u(6) },
        heal: def.heal,
        splash: def.splash,
    }
}

/// Champion walking speed multiplier over its 1v1 value (Proposed).
pub const CHAMPION_SPEED_FACTOR: i32 = 4;
/// Gold per 1v1 Energy point of a champion's price (Proposed).
pub const GOLD_PER_ENERGY: u32 = 6;

/// Gold price on the Shop card: the 1v1 Energy cost in Gold (M-10).
pub fn troop_cost(kind: Champion) -> u32 {
    tbg_core::content::def(kind).cost / tbg_core::content::ENERGY_SCALE * GOLD_PER_ENERGY
}

/// Redrawing the unlocked offers costs one 1v1 Energy in Gold (M-19).
pub const REROLL_COST: u32 = GOLD_PER_ENERGY;
/// Splash victims around a champion's target take this percent of the hit (M-07).
pub const SPLASH_PERCENT: i32 = 50;
/// Three role-locked Shop slots (Proposed, M-19/M-20).
pub const OFFERS: usize = 3;

/// Per-slot categories, derived separately for future player-selected rules (M-20).
pub fn slot_rules(role: Role) -> [&'static [Archetype]; OFFERS] {
    let allowed: &'static [Archetype] = match role {
        Role::Top => &[Archetype::Dps, Archetype::Tank],
        Role::Mid => &[Archetype::Control, Archetype::Dps],
        Role::Bot => &[Archetype::Ranged, Archetype::Dps],
        Role::Support => &[Archetype::Support, Archetype::Control],
        Role::Jungle => &[Archetype::Dps, Archetype::Tank, Archetype::Ranged],
    };
    [allowed; OFFERS]
}

/// Bits follow the shared Archetype order: Tank, Dps, Support, Ranged, Control.
pub fn archetype_mask(rules: &[Archetype]) -> u8 {
    rules.iter().fold(0, |mask, kind| mask | (1 << *kind as u8))
}

/// The champion each role's Commander is drawn as and fights like (Proposed).
pub fn commander_champion(role: Role) -> Champion {
    match role {
        Role::Top => Champion::Longaxe,
        Role::Jungle => Champion::Mantis,
        Role::Mid => Champion::Hexer,
        Role::Bot => Champion::CelestialArcher,
        Role::Support => Champion::LuminousOracle,
    }
}

/// Alive Troops one player may own at once (M-11).
pub const TROOP_CAP: u8 = 8;

// Economy (M-12..M-16).
pub const START_GOLD: u32 = 60;
pub const GOLD_PER_SECOND: u32 = 1;
pub const ESSENCE_EVERY_TICKS: u32 = secs(10);
pub const MINION_BOUNTY: u32 = 4;
pub const COMMANDER_BOUNTY: u32 = 40;
pub const ASSIST_BOUNTY: u32 = 15;
pub const TOWER_TEAM_BOUNTY: u32 = 25;
pub const TOWER_TEAM_ESSENCE: u32 = 2;
pub const COMMANDER_KILL_ESSENCE: u32 = 3;
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
        UnitKind::Neutral(kind) => neutral_rewards(kind).2,
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
pub const CAMP_LEASH: i32 = 9 * MM;
/// Manual Hold leash around the hold point.
pub const HOLD_LEASH: i32 = 7 * MM;
/// Lane units advance to the next waypoint inside this radius.
pub const WAYPOINT_RADIUS: i32 = 2 * MM;

/// Monster tables are Proposed placeholders, not a balance pass (M-60/M-64).
fn neutral_stats(kind: NeutralKind) -> Stats {
    use NeutralKind::*;
    let (hp, damage, range, interval, pace, armor, radius) = match kind {
        Small => (350, 16, 16, 20, 30, 0, 6),
        Big => (900, 35, 20, 24, 28, 0, 12),
        WolfAlpha => (600, 30, 18, 16, 42, 0, 10),
        WolfPup => (180, 12, 14, 16, 45, 0, 5),
        Golem => (1200, 32, 20, 30, 20, 25, 12),
        SmallGolem => (450, 14, 16, 30, 22, 20, 7),
        Jaguar => (1000, 70, 20, 20, 40, 5, 12),
        Frog => (750, 28, 60, 24, 28, 0, 10),
        SmallFrog => (240, 12, 50, 24, 30, 0, 6),
    };
    Stats {
        armor,
        radius: u(radius),
        ..stats(hp, damage, range, interval, pace)
    }
}

/// Last-hit Gold, Essence and nearby Commander XP (M-64).
pub fn neutral_rewards(kind: NeutralKind) -> (u32, u32, u32) {
    use NeutralKind::*;
    match kind {
        Small => (6, 0, 30),
        Big => (15, 2, 60),
        WolfAlpha => (18, 1, 60),
        WolfPup => (5, 0, 20),
        Golem => (20, 2, 70),
        SmallGolem => (6, 0, 30),
        Jaguar => (32, 3, 120),
        Frog => (18, 2, 60),
        SmallFrog => (6, 0, 25),
    }
}

/// Composition and seconds after a full clear (M-60/M-63).
pub fn camp_content(kind: crate::types::CampKind) -> (&'static [NeutralKind], u32) {
    use crate::types::CampKind;
    use NeutralKind::*;
    match kind {
        CampKind::Wolves => (&[WolfAlpha, WolfPup, WolfPup, WolfPup], secs(45)),
        CampKind::Golems => (&[Golem, SmallGolem, SmallGolem], secs(75)),
        CampKind::Jaguar => (&[Jaguar], secs(90)),
        CampKind::Frogs => (&[Frog, SmallFrog, SmallFrog], secs(60)),
    }
}
