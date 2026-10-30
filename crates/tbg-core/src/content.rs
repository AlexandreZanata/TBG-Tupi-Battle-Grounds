//! Every balance number of the MVP (docs/GAME-DESIGN.md §5), in integer
//! simulation units: ticks (20/s), milli-tiles, sub-energy (600 = 1 Energy),
//! basis points (10 000 = 100%). No other file may contain balance numbers;
//! the simulation reads them through `ruleset::CANON` (D-33).

use crate::arcana::ArcanaValues;
use crate::ruleset::CANON;
use crate::types::{Archetype, Faction, OnHit, StrongholdKind, Tick, UnitKind};

pub const TICK_HZ: u32 = 20;
pub const LANE_LEN: i32 = 16_000;
pub const SPAWN_OFFSET: i32 = 500;

pub const ENERGY_SCALE: u32 = 600;
pub const ENERGY_START: u32 = 3_000;
/// No gameplay cap: `regen_energy` saturates at `u32::MAX` (BR-E04).
pub const REGEN_RUNNING: u32 = 17;
pub const REGEN_OVERTIME: u32 = 34;

pub const COUNTDOWN: Tick = 60;
pub const RUNNING_START: Tick = COUNTDOWN;

/// Interest (owner revision 2026-10-29, BR-E05–E07): every full 5 Energy a
/// player holds raises the Energy regeneration by 25 %, counting at most 25
/// Energy (5 steps, up to +125 %). Energy itself has no cap.
pub const INTEREST_BAND: u32 = 3_000;
/// Regen bonus per step, in basis points of the base regen.
pub const INTEREST_PER_BAND: u32 = 2_500;
pub const INTEREST_MAX_BANDS: u32 = 5;

/// Augments: four options every 60 s of running time, forever (BR-AG01–08).
pub const AUGMENT_CATALOG: usize = 40;
pub const AUGMENT_OPTIONS: usize = 4;
pub const AUGMENT_ROUND_TICKS: Tick = 1_200;
/// How long a round's selection pause lasts (BR-AG13): 20 s, owner revision.
pub const AUGMENT_WINDOW_TICKS: Tick = 400;
pub const AUGMENT_FIRST_ROUND: Tick = RUNNING_START + AUGMENT_ROUND_TICKS;
pub const AUGMENT_COST: [u32; 4] = [1_800, 3_600, 6_000, 9_000];

/// Five distinct offers (BR-S01, D-32 Proposed). There is no army cap (BR-U07).
pub const OFFERS: usize = 5;
/// Bounty window: 10 s of combat (BR-E08). The Shop never rotates (BR-S04).
pub const BOUNTY_WINDOW_TICKS: Tick = 200;

/// Nester Hatch (BR-U08, Proposed): ticks between hatches, Beetlings alive
/// per Nester (base / Brood and Hive), per player, and their lifetime.
pub const HATCH_EVERY: u16 = 80;
pub const HATCH_EVERY_HIVE: u16 = 60;
pub const BEETLES_PER_NESTER: [usize; 2] = [2, 3];
pub const BEETLES_PER_PLAYER: usize = 6;
pub const BEETLE_LIFETIME: Tick = 200;

pub const STRONGHOLD_HP: i32 = 4_500;
/// P16-C lab evidence (D-33, Proposed): 65 → 53.
pub const STRONGHOLD_DAMAGE: i32 = 53;
pub const STRONGHOLD_INTERVAL: u16 = 20;
pub const STRONGHOLD_RANGE: i32 = 6_000;

/// Fortification price to tier 1 and to tier 2 (BR-SK05): 3 then 4 Energy.
/// Its own price, never the unit `EVOLVE_COST` (Proposed, D-46).
pub const FORTIFY_COST: [u32; 2] = [1_800, 2_400];

/// One Fortification step of a kind's branch. Each field adds to the kind's
/// base while the step is owned; `interval` replaces the base interval when
/// non-zero (the highest owned step wins). `hp_bp` and `dmg_bp` are basis
/// points of the kind's own base, so tier 2 adds to tier 1 (BR-SK06).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fortification {
    pub name: &'static str,
    pub hp_bp: i32,
    pub dmg_bp: i32,
    pub range_add: i32,
    pub interval: u16,
    pub spoils_add: u32,
    pub spoils_cap_add: u32,
    pub aura_bp_add: i32,
    pub aura_range_add: i32,
}

const fn fort(name: &'static str) -> Fortification {
    Fortification {
        name,
        hp_bp: 0,
        dmg_bp: 0,
        range_add: 0,
        interval: 0,
        spoils_add: 0,
        spoils_cap_add: 0,
        aura_bp_add: 0,
        aura_range_add: 0,
    }
}

/// A Stronghold kind: its base relative to the Classic tower and its two
/// exclusive branches, `branches[branch][tier - 1]` (GAME-DESIGN §10.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindDef {
    pub kind: StrongholdKind,
    pub name: &'static str,
    /// Basis points of `STRONGHOLD_HP` / `STRONGHOLD_DAMAGE`.
    pub hp_bp: i32,
    pub dmg_bp: i32,
    /// Spoils: sub-energy per eligible kill and per-window ceiling (BR-SK02).
    pub spoils: u32,
    pub spoils_cap: u32,
    /// War Banner: damage bp and radius in milli-tiles (BR-SK04).
    pub aura_bp: i32,
    pub aura_range: i32,
    pub branch_names: [&'static str; 2],
    pub branches: [[Fortification; 2]; 2],
}

const fn kind(kind: StrongholdKind, name: &'static str, hp_bp: i32, dmg_bp: i32) -> KindDef {
    KindDef {
        kind,
        name,
        hp_bp,
        dmg_bp,
        spoils: 0,
        spoils_cap: 0,
        aura_bp: 0,
        aura_range: 0,
        branch_names: ["", ""],
        branches: [[fort(""), fort("")], [fort(""), fort("")]],
    }
}

/// Every Stronghold kind in `StrongholdKind` order. Treasury and Command
/// keep the product hypotheses' ratios to the reference tower (4 200/60 and
/// 4 300/60 against 4 500/65) on today's 4 500 HP / 53 damage; Artillery
/// and Bastion moved off theirs after the lab matrix (Proposed, D-46).
#[rustfmt::skip]
pub const STRONGHOLD_KINDS: [KindDef; 5] = [
    kind(StrongholdKind::Classic, "Classic", 0, 0),
    KindDef {
        // 0.15 Energy a kill, at most 0.6 Energy per 10 s window.
        spoils: 90,
        spoils_cap: 360,
        branch_names: ["Armored Vault", "Mint"],
        branches: [
            [Fortification { hp_bp: 1_200, ..fort("Reinforced Vault") },
             Fortification { hp_bp: 1_200, ..fort("Grand Vault") }],
            [Fortification { spoils_add: 30, ..fort("Coinage") },
             // 0.25 Energy a kill; ceiling 0.6 -> 0.9 Energy a window.
             Fortification { spoils_add: 30, spoils_cap_add: 180, ..fort("Royal Mint") }],
        ],
        ..kind(StrongholdKind::Treasury, "Treasury", -700, -800)
    },
    KindDef {
        branch_names: ["Fortress", "Sentinel"],
        branches: [
            [Fortification { hp_bp: 1_500, ..fort("Reinforced Walls") },
             Fortification { hp_bp: 1_500, ..fort("Imperial Fortress") }],
            [Fortification { range_add: 1_000, ..fort("Watchtower") },
             Fortification { dmg_bp: 1_500, ..fort("Royal Sentinel") }],
        ],
        ..kind(StrongholdKind::Bastion, "Bastion", 1_500, -1_000)
    },
    KindDef {
        branch_names: ["Heavy Cannon", "Rapid Fire"],
        branches: [
            [Fortification { dmg_bp: 2_000, ..fort("Heavy Shot") },
             Fortification { dmg_bp: 2_000, ..fort("Great Cannon") }],
            [Fortification { interval: 17, ..fort("Quick Reload") },
             Fortification { interval: 14, ..fort("Advanced Mechanism") }],
        ],
        ..kind(StrongholdKind::Artillery, "Artillery", -1_500, 1_000)
    },
    KindDef {
        aura_bp: 500,
        aura_range: 4_000,
        branch_names: ["War Standard", "Forward Banner"],
        branches: [
            [Fortification { aura_bp_add: 300, ..fort("War Cry") },
             Fortification { aura_bp_add: 300, ..fort("Supreme Command") }],
            [Fortification { aura_range_add: 1_000, ..fort("Outpost") },
             Fortification { aura_range_add: 1_000, ..fort("High Command") }],
        ],
        ..kind(StrongholdKind::Command, "Command", -400, -800)
    },
];

/// Arcanas (GAME-DESIGN §9, Proposed values active by D-40): bp per rank of
/// the six original nodes, in serialization order Guard, Assault, Piercing,
/// Endurance, Resilience, Renewal.
pub const ARCANA_BP: [i32; 6] = [400, 300, 300, 400, 400, 500];

/// Conditional Arcanas (BR-AR05–AR10, P40): the initial Proposed values of
/// the nine new nodes.
pub const ARCANA_VALUES: ArcanaValues = ArcanaValues {
    shieldwall_dr_bp: 600,
    battle_rush_move_bp: 1_200,
    combined_arms_dr_bp: 500,
    combined_arms_dmg_bp: 800,
    hunters_focus_dmg_bp: 1_200,
    quickdraw_atk_bp: 1_500,
    quickdraw_distance: 2_000,
    // P40 audit: +250 milli-tiles (docs/media/p40/deadeye-dose.txt).
    deadeye_range: 250,
    unshaken_duration_bp: 2_500,
    emergency_care_heal_bp: 1_500,
    emergency_care_below_bp: 4_000,
    purification_root_cut_bp: 5_000,
};

pub const WINDUP: u16 = 6;
pub const MIN_INTERVAL: u16 = 5;
/// Cost of evolving to tier 1 and to tier 2 (P16-C, Proposed: tier 1 3 → 2).
pub const EVOLVE_COST: [u32; 2] = [1_200, 2_400];
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
    unit(UnitKind::Shieldbearer, "Shieldbearer", Humans, Tank, 2_400,
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
        [300, 29, 28, 3_500, 50, 0, 0], slow(4_000, 40), [
        FormDef { on_hit: slow(6_000, 40), ..form("Withering") },
        FormDef { dmg_bp: 6_000, ..form("Doom") },
        FormDef { splash_add: 1_000, ..form("Plague") },
        FormDef { dmg_bp: 4_000, splash_add: 500, ..form("Pestilence") },
    ]),
    unit(UnitKind::Sawbot, "Sawbot", Machines, Dps, 2_400,
        [231, 55, 21, 800, 65, 0, 0], None, [
        FormDef { atk_bp: 4_000, ..form("Overclock") },
        FormDef { atk_bp: 4_000, speed_bp: 3_000, ..form("Shredder") },
        FormDef { hp_bp: 6_000, ..form("Plating") },
        FormDef { hp_bp: 6_000, splash_add: 800, ..form("Juggernaut") },
    ]),
    unit(UnitKind::SiegeWalker, "Siege Walker", Machines, Ranged, 3_000,
        [700, 85, 50, 6_000, 33, 1_500, 0], None, [
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
    unit(UnitKind::Rampart, "Rampart", Machines, Tank, 3_600,
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
    /// Static attack-speed basis points already folded into `interval`.
    pub atk_bp: i32,
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

/// Augment rarity, in cost order; the index is the serialization contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AugmentRarity {
    Bronze,
    Silver,
    Gold,
    Diamond,
}

impl AugmentRarity {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn cost(self) -> u32 {
        AUGMENT_COST[self.index()]
    }
}

/// Which conventional units an Augment affects. Summons only match
/// `Beetlings` (BR-AG10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AugmentFilter {
    Conventional,
    Archetype(Archetype),
    Faction(Faction),
    Evolved,
    Beetlings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageAnchor {
    OwnRampart,
    OwnStronghold,
}

/// Closed effect list (one enum, one hook per kind; no generic engine).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AugmentEffect {
    /// Static stat bonus. `summon_add` is used only by `hive_awakening`.
    UnitStat {
        filter: AugmentFilter,
        hp_bp: i32,
        dmg_bp: i32,
        atk_bp: i32,
        move_bp: i32,
        heal_bp: i32,
        summon_add: u8,
    },
    StrongholdStat {
        dmg_bp: i32,
    },
    VsStronghold {
        filter: AugmentFilter,
        bp: i32,
    },
    VsArchetype {
        archetype: Archetype,
        bp: i32,
    },
    Bounty {
        per_kill: u32,
        cap: u32,
    },
    EvolveDiscount {
        amount: u32,
    },
    Lifesteal {
        bp: i32,
    },
    LastStand {
        below_bp: i32,
        shield_bp: i32,
        ticks: u16,
    },
    Comeback {
        bp: i32,
    },
    DeployHaste {
        move_bp: i32,
        ticks: u16,
    },
    KillMend {
        hp: i32,
        cap: i32,
        window: u16,
    },
    DamageTakenAura {
        anchor: DamageAnchor,
        radius: i32,
        bp: i32,
        below_bp: i32,
    },
    HealBonus {
        bp: i32,
        below_bp: i32,
    },
    StatusDuration {
        bp: i32,
    },
    TowerFury {
        bp: i32,
        max_stacks: u8,
        ticks: u16,
    },
    HealCut {
        bp: i32,
        ticks: u16,
    },
    Executioner {
        below_bp: i32,
        bp: i32,
    },
    SecondWind,
    Convergence {
        hp_bp: i32,
        dmg_bp: i32,
    },
}

/// Who an Augment is for (UI text and the future lab policy).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AugmentTag {
    Universal,
    Archetype(Archetype),
    Faction(Faction),
    Kind(UnitKind),
    Beetlings,
    TwoFactions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AugmentDef {
    /// Index in `AUGMENTS`; never renumbered (serialization contract).
    pub id: u8,
    pub name: &'static str,
    pub rarity: AugmentRarity,
    pub tag: AugmentTag,
    pub effect: AugmentEffect,
    pub text: &'static str,
}

const fn stat(
    filter: AugmentFilter,
    hp_bp: i32,
    dmg_bp: i32,
    atk_bp: i32,
    move_bp: i32,
    heal_bp: i32,
) -> AugmentEffect {
    AugmentEffect::UnitStat {
        filter,
        hp_bp,
        dmg_bp,
        atk_bp,
        move_bp,
        heal_bp,
        summon_add: 0,
    }
}

const fn aug(
    id: u8,
    name: &'static str,
    rarity: AugmentRarity,
    tag: AugmentTag,
    effect: AugmentEffect,
    text: &'static str,
) -> AugmentDef {
    AugmentDef {
        id,
        name,
        rarity,
        tag,
        effect,
        text,
    }
}

/// The 40 Augments, in the GAME-DESIGN §10.8 index order.
#[rustfmt::skip]
pub const AUGMENTS: [AugmentDef; AUGMENT_CATALOG] = [
    aug(0, "Forced March", AugmentRarity::Bronze, AugmentTag::Universal,
        AugmentEffect::DeployHaste { move_bp: 4_000, ticks: 100 },
        "Units you buy move +40% for 5 s"),
    aug(1, "Field Kit", AugmentRarity::Bronze, AugmentTag::Universal,
        stat(AugmentFilter::Conventional, 400, 0, 0, 0, 0),
        "Conventional units +4% max HP"),
    aug(2, "Sentry", AugmentRarity::Bronze, AugmentTag::Universal,
        AugmentEffect::StrongholdStat { dmg_bp: 1_000 },
        "Your Stronghold +10% damage"),
    aug(3, "Steady Aim", AugmentRarity::Bronze, AugmentTag::Archetype(Archetype::Ranged),
        stat(AugmentFilter::Archetype(Archetype::Ranged), 0, 800, 0, 0, 0),
        "Ranged +8% damage"),
    aug(4, "Reinforced Shields", AugmentRarity::Bronze, AugmentTag::Archetype(Archetype::Tank),
        stat(AugmentFilter::Archetype(Archetype::Tank), 800, 0, 0, 0, 0),
        "Tanks +8% max HP"),
    aug(5, "First Aid", AugmentRarity::Silver, AugmentTag::Archetype(Archetype::Support),
        stat(AugmentFilter::Conventional, 0, 0, 0, 0, 1_000),
        "Direct and splash heals +10%"),
    aug(6, "Drilled Ranks", AugmentRarity::Silver, AugmentTag::Faction(Faction::Humans),
        stat(AugmentFilter::Faction(Faction::Humans), 0, 800, 0, 0, 0),
        "Humans +8% damage"),
    aug(7, "Frenzied Bones", AugmentRarity::Silver, AugmentTag::Faction(Faction::Undead),
        stat(AugmentFilter::Faction(Faction::Undead), 0, 0, 800, 0, 0),
        "Undead +8% attack speed"),
    aug(8, "Tuned Gears", AugmentRarity::Silver, AugmentTag::Faction(Faction::Machines),
        stat(AugmentFilter::Faction(Faction::Machines), 0, 0, 800, 0, 0),
        "Machines +8% attack speed"),
    aug(9, "Hardy Sap", AugmentRarity::Silver, AugmentTag::Faction(Faction::Plants),
        stat(AugmentFilter::Faction(Faction::Plants), 800, 0, 0, 0, 0),
        "Plants +8% max HP"),
    aug(10, "Plunder", AugmentRarity::Silver, AugmentTag::Universal,
        AugmentEffect::Bounty { per_kill: 90, cap: 270 },
        "Bounty 0.15 Energy per kill (cap 0.45)"),
    aug(11, "Stone Mend", AugmentRarity::Silver, AugmentTag::Universal,
        AugmentEffect::KillMend { hp: 50, cap: 150, window: 600 },
        "Stronghold heals 50 HP per kill (cap 150/30 s)"),
    aug(12, "Field Promotion", AugmentRarity::Silver, AugmentTag::Universal,
        AugmentEffect::EvolveDiscount { amount: 600 },
        "Evolutions cost 1 Energy less (minimum 1)"),
    aug(13, "Veterans", AugmentRarity::Silver, AugmentTag::Universal,
        stat(AugmentFilter::Evolved, 800, 400, 0, 0, 0),
        "Evolved units +8% max HP, +4% damage"),
    aug(14, "Shield Wall", AugmentRarity::Silver, AugmentTag::Kind(UnitKind::Rampart),
        AugmentEffect::DamageTakenAura { anchor: DamageAnchor::OwnRampart, radius: 1_500, bp: 800, below_bp: 0 },
        "Units near a living Rampart take -8% damage"),
    aug(15, "Siege Arrows", AugmentRarity::Silver, AugmentTag::Archetype(Archetype::Ranged),
        AugmentEffect::VsStronghold { filter: AugmentFilter::Archetype(Archetype::Ranged), bp: 1_500 },
        "Ranged +15% damage to Strongholds"),
    aug(16, "Armored Brood", AugmentRarity::Silver, AugmentTag::Beetlings,
        stat(AugmentFilter::Beetlings, 2_500, 0, 0, 0, 0),
        "Beetlings +25% max HP"),
    aug(17, "Triage", AugmentRarity::Silver, AugmentTag::Archetype(Archetype::Support),
        AugmentEffect::HealBonus { bp: 2_500, below_bp: 4_000 },
        "+25% direct heal below 40% HP"),
    aug(18, "Lingering Hex", AugmentRarity::Silver, AugmentTag::Archetype(Archetype::Control),
        AugmentEffect::StatusDuration { bp: 1_500 },
        "Slow and root you apply last +15%"),
    aug(19, "Shield Step", AugmentRarity::Silver, AugmentTag::Archetype(Archetype::Tank),
        stat(AugmentFilter::Archetype(Archetype::Tank), 500, 0, 0, 1_200, 0),
        "Tanks +12% move, +5% max HP"),
    aug(20, "War Leech", AugmentRarity::Gold, AugmentTag::Universal,
        AugmentEffect::Lifesteal { bp: 800 },
        "Lifesteal 8% of HP removed"),
    aug(21, "Conquerors", AugmentRarity::Gold, AugmentTag::Universal,
        AugmentEffect::VsStronghold { filter: AugmentFilter::Conventional, bp: 1_200 },
        "Conventional units +12% damage to Strongholds"),
    aug(22, "Tower Fury", AugmentRarity::Gold, AugmentTag::Universal,
        AugmentEffect::TowerFury { bp: 500, max_stacks: 3, ticks: 240 },
        "Stronghold kills: +5% damage, 3 stacks, 12 s"),
    aug(23, "Entrenched", AugmentRarity::Gold, AugmentTag::Universal,
        AugmentEffect::DamageTakenAura { anchor: DamageAnchor::OwnStronghold, radius: 5_000, bp: 1_000, below_bp: 5_000 },
        "Below 50% HP, units near it take -10% damage"),
    aug(24, "Grievous Wounds", AugmentRarity::Gold, AugmentTag::Universal,
        AugmentEffect::HealCut { bp: 4_000, ticks: 40 },
        "Your hits cut healing 40% for 2 s"),
    aug(25, "Giant Slayer", AugmentRarity::Gold, AugmentTag::Universal,
        AugmentEffect::VsArchetype { archetype: Archetype::Tank, bp: 1_500 },
        "+15% damage to enemy Tanks"),
    aug(26, "Executioner", AugmentRarity::Gold, AugmentTag::Universal,
        AugmentEffect::Executioner { below_bp: 3_500, bp: 2_000 },
        "+20% damage to targets below 35% HP"),
    aug(27, "Hunter's Instinct", AugmentRarity::Gold, AugmentTag::Archetype(Archetype::Dps),
        stat(AugmentFilter::Archetype(Archetype::Dps), 0, 0, 1_000, 800, 0),
        "DPS +10% attack speed, +8% move"),
    aug(28, "Iron Vanguard", AugmentRarity::Gold, AugmentTag::Archetype(Archetype::Tank),
        stat(AugmentFilter::Archetype(Archetype::Tank), 1_500, 0, 0, 1_000, 0),
        "Tanks +15% max HP, +10% move"),
    aug(29, "Spellweaver", AugmentRarity::Gold, AugmentTag::Archetype(Archetype::Control),
        stat(AugmentFilter::Archetype(Archetype::Control), 0, 0, 2_000, 0, 0),
        "Control +20% attack speed"),
    aug(30, "Last Bastion", AugmentRarity::Diamond, AugmentTag::Universal,
        AugmentEffect::LastStand { below_bp: 2_500, shield_bp: 1_200, ticks: 300 },
        "First time below 25% HP: shield for 15 s"),
    aug(31, "Desperate Assault", AugmentRarity::Diamond, AugmentTag::Universal,
        AugmentEffect::Comeback { bp: 1_500 },
        "Behind on HP: +15% damage to Strongholds"),
    aug(32, "Legacy", AugmentRarity::Diamond, AugmentTag::Universal,
        stat(AugmentFilter::Evolved, 1_500, 1_000, 0, 0, 0),
        "Evolved units +15% max HP, +10% damage"),
    aug(33, "Second Wind", AugmentRarity::Diamond, AugmentTag::Universal,
        AugmentEffect::SecondWind,
        "Each conventional unit survives its first lethal hit"),
    aug(34, "Convergence", AugmentRarity::Diamond, AugmentTag::TwoFactions,
        AugmentEffect::Convergence { hp_bp: 800, dmg_bp: 800 },
        "Two active Synergies: units +8% HP and damage"),
    aug(35, "Royal Banner", AugmentRarity::Diamond, AugmentTag::Faction(Faction::Humans),
        stat(AugmentFilter::Faction(Faction::Humans), 1_200, 1_000, 0, 0, 0),
        "Humans +12% max HP, +10% damage"),
    aug(36, "Deathless Legion", AugmentRarity::Diamond, AugmentTag::Faction(Faction::Undead),
        stat(AugmentFilter::Faction(Faction::Undead), 1_000, 0, 1_500, 0, 0),
        "Undead +15% attack speed, +10% max HP"),
    aug(37, "Iron Legion", AugmentRarity::Diamond, AugmentTag::Faction(Faction::Machines),
        stat(AugmentFilter::Faction(Faction::Machines), 0, 1_500, 1_000, 0, 0),
        "Machines +15% damage, +10% attack speed"),
    aug(38, "Bountiful Harvest", AugmentRarity::Diamond, AugmentTag::Faction(Faction::Plants),
        stat(AugmentFilter::Faction(Faction::Plants), 1_000, 0, 0, 0, 1_500),
        "Plants +10% max HP, +15% direct heal"),
    aug(39, "Hive Awakening", AugmentRarity::Diamond, AugmentTag::Beetlings,
        AugmentEffect::UnitStat { filter: AugmentFilter::Beetlings, hp_bp: 0, dmg_bp: 2_500, atk_bp: 0, move_bp: 0, heal_bp: 0, summon_add: 1 },
        "Beetlings +25% damage; summon limit +1"),
];

pub fn augment_def(id: u8) -> Option<&'static AugmentDef> {
    AUGMENTS.get(usize::from(id))
}

/// Static Augment sums for one unit, before the single BR-X01 truncation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AugmentBonus {
    pub hp_bp: i64,
    pub dmg_bp: i64,
    pub atk_bp: i64,
    pub move_bp: i64,
    pub heal_bp: i64,
}
