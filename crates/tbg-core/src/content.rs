//! Every balance number of the MVP (docs/GAME-DESIGN.md §5), in integer
//! simulation units: ticks (20/s), milli-tiles, sub-energy (600 = 1 Energy),
//! basis points (10 000 = 100%). No other file may contain balance numbers;
//! the simulation reads them through `ruleset::CANON` (D-33).

use crate::ruleset::CANON;
use crate::types::{Archetype, Faction, OnHit, Tick, UnitKind};

pub const TICK_HZ: u32 = 20;
pub const LANE_LEN: i32 = 16_000;
pub const SPAWN_OFFSET: i32 = 500;

pub const ENERGY_SCALE: u32 = 600;
pub const ENERGY_START: u32 = 3_000;
pub const ENERGY_CAP: u32 = 6_000;
pub const REGEN_RUNNING: u32 = 17;
pub const REGEN_OVERTIME: u32 = 34;

pub const COUNTDOWN: Tick = 60;
pub const REGULATION: Tick = 3_600;
pub const OVERTIME: Tick = 1_200;
pub const RUNNING_START: Tick = COUNTDOWN;
pub const OVERTIME_START: Tick = COUNTDOWN + REGULATION;
pub const MATCH_END: Tick = OVERTIME_START + OVERTIME;

/// Five distinct offers (BR-S01, D-32 Proposed). There is no army cap (BR-U07).
pub const OFFERS: usize = 5;
pub const ROTATION_TICKS: Tick = 200;

/// Nester Hatch (BR-U08, Proposed): ticks between hatches, Beetlings alive
/// per Nester (base / Brood and Hive), per player, and their lifetime.
pub const HATCH_EVERY: u16 = 80;
pub const HATCH_EVERY_HIVE: u16 = 60;
pub const BEETLES_PER_NESTER: [usize; 2] = [2, 3];
pub const BEETLES_PER_PLAYER: usize = 6;
pub const BEETLE_LIFETIME: Tick = 200;

pub const STRONGHOLD_HP: i32 = 4_500;
pub const STRONGHOLD_DAMAGE: i32 = 65;
pub const STRONGHOLD_INTERVAL: u16 = 20;
pub const STRONGHOLD_RANGE: i32 = 6_000;

pub const WINDUP: u16 = 6;
pub const MIN_INTERVAL: u16 = 5;
/// Cost of evolving to tier 1 and to tier 2.
pub const EVOLVE_COST: [u32; 2] = [1_800, 2_400];
pub const BP: i64 = 10_000;

/// Synergy thresholds: living units of one faction for level 1 and 2.
pub const SYNERGY_THRESHOLDS: [u8; 2] = [2, 4];
/// Plants regeneration runs every this many ticks.
pub const REGEN_PERIOD: Tick = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SynergyEffect {
    MaxHp,
    AttackSpeed,
    Damage,
    /// Heal this many bp of max HP every `REGEN_PERIOD` ticks.
    Regen,
}

/// Per faction (index order): the effect and its bp at level 1 and 2.
pub const SYNERGIES: [(SynergyEffect, [i32; 2]); 4] = [
    (SynergyEffect::MaxHp, [2_000, 4_000]),
    (SynergyEffect::AttackSpeed, [2_000, 4_000]),
    (SynergyEffect::Damage, [500, 2_000]),
    (SynergyEffect::Regen, [75, 200]),
];

/// One evolution form: additive modifiers on the base unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormDef {
    pub name: &'static str,
    pub hp_bp: i32,
    pub dmg_bp: i32,
    pub atk_bp: i32,
    pub speed_bp: i32,
    pub range_add: i32,
    pub splash_add: i32,
    pub heal_bp: i32,
    pub dmg_add: i32,
    pub on_hit: Option<OnHit>,
}

const fn form(name: &'static str) -> FormDef {
    FormDef {
        name,
        hp_bp: 0,
        dmg_bp: 0,
        atk_bp: 0,
        speed_bp: 0,
        range_add: 0,
        splash_add: 0,
        heal_bp: 0,
        dmg_add: 0,
        on_hit: None,
    }
}

const fn slow(bp: u16, ticks: u16) -> Option<OnHit> {
    Some(OnHit::Slow { bp, ticks })
}

const fn root(ticks: u16) -> Option<OnHit> {
    Some(OnHit::Root { ticks })
}

/// Base stats of a unit kind plus its forms `[A1, A2, B1, B2]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitDef {
    pub kind: UnitKind,
    pub name: &'static str,
    pub faction: Faction,
    pub archetype: Archetype,
    pub cost: u32,
    pub hp: i32,
    pub damage: i32,
    pub interval: u16,
    pub range: i32,
    pub speed: i32,
    pub splash: i32,
    pub heal: i32,
    pub on_hit: Option<OnHit>,
    pub forms: [FormDef; 4],
}

#[allow(clippy::too_many_arguments)] // one call per roster row reads as a table
const fn unit(
    kind: UnitKind,
    name: &'static str,
    faction: Faction,
    archetype: Archetype,
    cost: u32,
    stats: [i32; 7], // hp, damage, interval, range, speed, splash, heal
    on_hit: Option<OnHit>,
    forms: [FormDef; 4],
) -> UnitDef {
    UnitDef {
        kind,
        name,
        faction,
        archetype,
        cost,
        hp: stats[0],
        damage: stats[1],
        interval: stats[2] as u16,
        range: stats[3],
        speed: stats[4],
        splash: stats[5],
        heal: stats[6],
        on_hit,
        forms,
    }
}

use Archetype::*;
use Faction::*;

/// Every kind, in `UnitKind` (`ALL_KINDS`) order.
#[rustfmt::skip]
pub const UNITS: [UnitDef; 11] = [
    unit(UnitKind::Shieldbearer, "Shieldbearer", Humans, Tank, 1_800,
        [650, 30, 24, 800, 40, 0, 0], None, [
        FormDef { hp_bp: 5_000, ..form("Bulwark") },
        FormDef { hp_bp: 5_000, on_hit: slow(3_000, 30), ..form("Fortress") },
        FormDef { dmg_bp: 4_000, speed_bp: 2_000, ..form("Vanguard") },
        FormDef { dmg_bp: 4_000, splash_add: 1_000, ..form("Champion") },
    ]),
    unit(UnitKind::Archer, "Archer", Humans, Ranged, 1_800,
        [220, 45, 20, 4_500, 50, 0, 0], None, [
        FormDef { range_add: 1_500, dmg_bp: 2_000, ..form("Longbow") },
        FormDef { dmg_bp: 6_000, ..form("Marksman") },
        FormDef { atk_bp: 4_000, ..form("Volley") },
        FormDef { splash_add: 1_000, ..form("Arrow Rain") },
    ]),
    unit(UnitKind::BoneBrute, "Bone Brute", Undead, Tank, 2_400,
        [1_300, 75, 30, 1_000, 30, 0, 0], None, [
        FormDef { hp_bp: 5_000, ..form("Ossified") },
        FormDef { hp_bp: 5_000, on_hit: slow(3_000, 30), ..form("Bone Wall") },
        FormDef { dmg_bp: 5_000, ..form("Butcher") },
        FormDef { dmg_bp: 3_000, splash_add: 1_200, ..form("Reaper") },
    ]),
    unit(UnitKind::Hexer, "Hexer", Undead, Control, 2_400,
        [300, 30, 28, 3_500, 50, 0, 0], slow(4_000, 40), [
        FormDef { on_hit: slow(6_000, 40), ..form("Withering") },
        FormDef { dmg_bp: 6_000, ..form("Doom") },
        FormDef { splash_add: 1_000, ..form("Plague") },
        FormDef { dmg_bp: 4_000, splash_add: 500, ..form("Pestilence") },
    ]),
    unit(UnitKind::Sawbot, "Sawbot", Machines, Dps, 1_800,
        [270, 55, 21, 800, 65, 0, 0], None, [
        FormDef { atk_bp: 4_000, ..form("Overclock") },
        FormDef { atk_bp: 4_000, speed_bp: 3_000, ..form("Shredder") },
        FormDef { hp_bp: 6_000, ..form("Plating") },
        FormDef { hp_bp: 6_000, splash_add: 800, ..form("Juggernaut") },
    ]),
    unit(UnitKind::SiegeWalker, "Siege Walker", Machines, Ranged, 3_000,
        [700, 85, 50, 6_000, 30, 1_500, 0], None, [
        FormDef { range_add: 2_000, ..form("Longshot") },
        FormDef { dmg_bp: 5_000, ..form("Artillery") },
        FormDef { splash_add: 1_000, ..form("Shrapnel") },
        FormDef { on_hit: root(16), ..form("Concussion") },
    ]),
    unit(UnitKind::Bloomkeeper, "Bloomkeeper", Plants, Support, 1_800,
        [280, 0, 30, 3_000, 45, 0, 40], None, [
        FormDef { heal_bp: 5_000, ..form("Blossom") },
        FormDef { splash_add: 1_500, ..form("Grove") },
        FormDef { hp_bp: 5_000, dmg_add: 25, ..form("Bramble") },
        FormDef { on_hit: root(12), ..form("Thornwarden") },
    ]),
    unit(UnitKind::Rootcaller, "Rootcaller", Plants, Control, 2_400,
        [300, 20, 32, 3_000, 45, 0, 0], root(16), [
        FormDef { on_hit: root(32), ..form("Deep Roots") },
        FormDef { dmg_bp: 8_000, ..form("Strangle") },
        FormDef { splash_add: 1_000, ..form("Spores") },
        FormDef { range_add: 1_000, on_hit: root(28), ..form("Overgrowth") },
    ]),
    // P16-B (D-32, Proposed). Nester forms only change Hatch (`hatch_rule`).
    unit(UnitKind::Nester, "Nester", Plants, Support, 2_400,
        [300, 15, 30, 2_500, 40, 0, 0], None, [
        form("Brood"), form("Hive"), form("Venom"), form("Blight"),
    ]),
    unit(UnitKind::Rampart, "Rampart", Machines, Tank, 3_000,
        [1_500, 35, 30, 800, 25, 0, 0], None, [
        FormDef { hp_bp: 5_000, ..form("Bastion") },
        FormDef { hp_bp: 5_000, on_hit: slow(3_000, 30), ..form("Citadel") },
        FormDef { dmg_bp: 6_000, ..form("Spiked") },
        FormDef { dmg_bp: 4_000, splash_add: 1_000, ..form("Siegebreaker") },
    ]),
    // Summon: hatched with its Nester's tier and branch, so these forms are
    // the Nester's Beetling upgrades (Venom slows, Blight +50 % HP and dmg).
    unit(UnitKind::Beetling, "Beetling", Plants, Dps, 0,
        [90, 18, 16, 600, 70, 0, 0], None, [
        form("Brood"), form("Hive"),
        FormDef { on_hit: slow(3_000, 20), ..form("Venom") },
        FormDef { hp_bp: 5_000, dmg_bp: 5_000, ..form("Blight") },
    ]),
];

/// Canonical Hatch rule (see `Ruleset::hatch_rule`).
pub fn hatch_rule(tier: u8, branch: u8) -> (u16, usize) {
    CANON.hatch_rule(tier, branch)
}

/// Canonical unit definition, for the client UI and the server.
pub fn def(kind: UnitKind) -> &'static UnitDef {
    CANON.def(kind)
}

/// Canonical form (see `Ruleset::form_of`).
pub fn form_of(kind: UnitKind, tier: u8, branch: u8) -> Option<&'static FormDef> {
    CANON.form_of(kind, tier, branch)
}

/// Effective stats after evolution forms and the seat's synergy level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub max_hp: i32,
    pub damage: i32,
    pub interval: u16,
    pub range: i32,
    pub speed: i32,
    pub splash: i32,
    pub heal: i32,
    pub on_hit: Option<OnHit>,
}

/// Canonical effective stats (see `Ruleset::effective_stats`).
pub fn effective_stats(kind: UnitKind, tier: u8, branch: u8, synergy_level: u8) -> Stats {
    CANON.effective_stats(kind, tier, branch, synergy_level)
}
