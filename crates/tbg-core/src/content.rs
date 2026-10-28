//! Every balance number of the MVP (docs/GAME-DESIGN.md §5), in integer
//! simulation units: ticks (20/s), milli-tiles, sub-energy (600 = 1 Energy),
//! basis points (10 000 = 100%). No other file may contain balance numbers.

use crate::types::{Archetype, Faction, OnHit, SlotCount, Tick, UnitKind};

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

pub const OFFERS: usize = 3;
pub const ROTATION_TICKS: Tick = 200;
pub const SLOT_CAP: u8 = 8;

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
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub struct UnitDef {
    pub kind: UnitKind,
    pub name: &'static str,
    pub faction: Faction,
    pub archetype: Archetype,
    pub slots: SlotCount,
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
    slots: SlotCount,
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
        slots,
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
use SlotCount::{One, Two};

/// The roster, in `UNIT_KINDS` order.
#[rustfmt::skip]
pub const UNITS: [UnitDef; 8] = [
    unit(UnitKind::Shieldbearer, "Shieldbearer", Humans, Tank, One, 1_800,
        [650, 30, 24, 800, 40, 0, 0], None, [
        FormDef { hp_bp: 5_000, ..form("Bulwark") },
        FormDef { hp_bp: 5_000, on_hit: slow(3_000, 30), ..form("Fortress") },
        FormDef { dmg_bp: 4_000, speed_bp: 2_000, ..form("Vanguard") },
        FormDef { dmg_bp: 4_000, splash_add: 1_000, ..form("Champion") },
    ]),
    unit(UnitKind::Archer, "Archer", Humans, Ranged, One, 1_800,
        [220, 45, 20, 4_500, 50, 0, 0], None, [
        FormDef { range_add: 1_500, dmg_bp: 2_000, ..form("Longbow") },
        FormDef { dmg_bp: 6_000, ..form("Marksman") },
        FormDef { atk_bp: 4_000, ..form("Volley") },
        FormDef { splash_add: 1_000, ..form("Arrow Rain") },
    ]),
    unit(UnitKind::BoneBrute, "Bone Brute", Undead, Tank, Two, 2_400,
        [1_300, 75, 30, 1_000, 30, 0, 0], None, [
        FormDef { hp_bp: 5_000, ..form("Ossified") },
        FormDef { hp_bp: 5_000, on_hit: slow(3_000, 30), ..form("Bone Wall") },
        FormDef { dmg_bp: 5_000, ..form("Butcher") },
        FormDef { dmg_bp: 3_000, splash_add: 1_200, ..form("Reaper") },
    ]),
    unit(UnitKind::Hexer, "Hexer", Undead, Control, One, 2_400,
        [300, 30, 28, 3_500, 50, 0, 0], slow(4_000, 40), [
        FormDef { on_hit: slow(6_000, 40), ..form("Withering") },
        FormDef { dmg_bp: 6_000, ..form("Doom") },
        FormDef { splash_add: 1_000, ..form("Plague") },
        FormDef { dmg_bp: 4_000, splash_add: 500, ..form("Pestilence") },
    ]),
    unit(UnitKind::Sawbot, "Sawbot", Machines, Dps, One, 1_800,
        [270, 55, 21, 800, 65, 0, 0], None, [
        FormDef { atk_bp: 4_000, ..form("Overclock") },
        FormDef { atk_bp: 4_000, speed_bp: 3_000, ..form("Shredder") },
        FormDef { hp_bp: 6_000, ..form("Plating") },
        FormDef { hp_bp: 6_000, splash_add: 800, ..form("Juggernaut") },
    ]),
    unit(UnitKind::SiegeWalker, "Siege Walker", Machines, Ranged, Two, 3_000,
        [700, 85, 50, 6_000, 30, 1_500, 0], None, [
        FormDef { range_add: 2_000, ..form("Longshot") },
        FormDef { dmg_bp: 5_000, ..form("Artillery") },
        FormDef { splash_add: 1_000, ..form("Shrapnel") },
        FormDef { on_hit: root(16), ..form("Concussion") },
    ]),
    unit(UnitKind::Bloomkeeper, "Bloomkeeper", Plants, Support, One, 1_800,
        [280, 0, 30, 3_000, 45, 0, 40], None, [
        FormDef { heal_bp: 5_000, ..form("Blossom") },
        FormDef { splash_add: 1_500, ..form("Grove") },
        FormDef { hp_bp: 5_000, dmg_add: 25, ..form("Bramble") },
        FormDef { on_hit: root(12), ..form("Thornwarden") },
    ]),
    unit(UnitKind::Rootcaller, "Rootcaller", Plants, Control, One, 2_400,
        [300, 20, 32, 3_000, 45, 0, 0], root(16), [
        FormDef { on_hit: root(32), ..form("Deep Roots") },
        FormDef { dmg_bp: 8_000, ..form("Strangle") },
        FormDef { splash_add: 1_000, ..form("Spores") },
        FormDef { range_add: 1_000, on_hit: root(28), ..form("Overgrowth") },
    ]),
];

pub fn def(kind: UnitKind) -> &'static UnitDef {
    &UNITS[kind as usize]
}

/// The form a unit at `tier` (1 or 2) on `branch` (0 = A, 1 = B) has.
pub fn form_of(kind: UnitKind, tier: u8, branch: u8) -> Option<&'static FormDef> {
    if tier == 0 || tier > 2 || branch > 1 {
        return None;
    }
    Some(&def(kind).forms[usize::from(branch) * 2 + usize::from(tier) - 1])
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

pub fn effective_stats(kind: UnitKind, tier: u8, branch: u8, synergy_level: u8) -> Stats {
    let d = def(kind);
    let forms = [form_of(kind, 1, branch), form_of(kind, 2, branch)];
    let active: Vec<&FormDef> = forms
        .iter()
        .take(usize::from(tier.min(2)))
        .flatten()
        .copied()
        .collect();
    let sum = |f: fn(&FormDef) -> i32| -> i64 { active.iter().map(|x| i64::from(f(x))).sum() };
    let (mut hp_bp, mut dmg_bp, mut atk_bp) =
        (sum(|f| f.hp_bp), sum(|f| f.dmg_bp), sum(|f| f.atk_bp));
    if synergy_level > 0 {
        let (effect, bps) = SYNERGIES[d.faction.index()];
        let bp = i64::from(bps[usize::from(synergy_level.min(2)) - 1]);
        match effect {
            SynergyEffect::MaxHp => hp_bp += bp,
            SynergyEffect::AttackSpeed => atk_bp += bp,
            SynergyEffect::Damage => dmg_bp += bp,
            SynergyEffect::Regen => {}
        }
    }
    let scale = |base: i64, bp: i64| (base * (BP + bp) / BP) as i32;
    let interval = i64::from(d.interval) * BP / (BP + atk_bp);
    Stats {
        max_hp: scale(i64::from(d.hp), hp_bp),
        damage: scale(i64::from(d.damage) + sum(|f| f.dmg_add), dmg_bp),
        interval: (interval as u16).max(MIN_INTERVAL),
        range: d.range + sum(|f| f.range_add) as i32,
        speed: scale(i64::from(d.speed), sum(|f| f.speed_bp)),
        splash: d.splash + sum(|f| f.splash_add) as i32,
        heal: scale(i64::from(d.heal), sum(|f| f.heal_bp)),
        on_hit: active.iter().rev().find_map(|f| f.on_hit).or(d.on_hit),
    }
}
