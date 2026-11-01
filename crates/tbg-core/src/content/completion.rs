//! Initial integer ability parameters for the seventeen local-preview champions.

use crate::UnitKind;

/// Printed base range used to identify spark-consuming melee attacks.
pub const SPARK_MELEE_RANGE: i32 = 1_600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ability {
    Finisher {
        below_bp: u16,
        bonus_bp: u16,
    },
    Echo {
        heal: u16,
        ticks: u16,
    },
    Sweep {
        every: u8,
        damage: u16,
        radius: i32,
        targets: u8,
    },
    Pounce {
        walk_ticks: u16,
        bonus_bp: u16,
    },
    Pierce {
        every: u8,
        ratio_bp: u16,
        behind: i32,
    },
    Regrowth {
        heal: u16,
        pulses: u8,
        gap: u16,
    },
    Interrupt {
        period: u16,
        lock_ticks: u16,
        windup: u16,
    },
    Spark {
        every: u8,
        damage: u16,
        ticks: u16,
    },
    Eclipse {
        high_bp: u16,
        solar_bp: u16,
        lunar_bp: u16,
    },
    Crusher {
        max_hp_bp: u16,
        cap: u16,
    },
    Courage {
        heal_per_enemy: u16,
        enemies: u8,
        radius: i32,
    },
    Beam {
        period: u16,
        damage: u16,
        radius: i32,
        targets: u8,
    },
    Bleeder {
        damage: u16,
        pulses: u8,
    },
    DeathPop {
        damage: u16,
        radius: i32,
        targets: u8,
    },
    Opportunity {
        every: u8,
        bonus_bp: u16,
    },
    Channel {
        pulses: u8,
        gap: u16,
        first_bp: u16,
    },
    Rhythm {
        period: u16,
        bp: u16,
        ticks: u16,
        radius: i32,
        targets: u8,
    },
}

pub const fn forms(kind: UnitKind) -> [[super::Ability; 2]; 5] {
    [
        form(kind, 0),
        form(kind, 1),
        form(kind, 2),
        form(kind, 3),
        form(kind, 4),
    ]
}

const fn form(kind: UnitKind, slot: u8) -> [super::Ability; 2] {
    let a = if slot == 1 {
        1
    } else if slot == 2 {
        2
    } else {
        0
    };
    let b = if slot == 3 {
        1
    } else if slot == 4 {
        2
    } else {
        0
    };
    let value = match kind {
        UnitKind::Gravecutter => Ability::Finisher {
            below_bp: 3_500,
            bonus_bp: 3_000 + a * 1_000,
        },
        UnitKind::LastlightCantor => Ability::Echo {
            heal: 18 + b * 6,
            ticks: 80,
        },
        UnitKind::Cragclaw => Ability::Sweep {
            every: 3,
            damage: 22 + a * 8,
            radius: 900 + b as i32 * 350,
            targets: if b == 2 { 3 } else { 2 },
        },
        UnitKind::Hushfang => Ability::Pounce {
            walk_ticks: 20,
            bonus_bp: 5_000 + a * 1_500,
        },
        UnitKind::Spinehurler => Ability::Pierce {
            every: 3,
            ratio_bp: 6_000 + b * 1_500,
            behind: 1_600,
        },
        UnitKind::MireTender => Ability::Regrowth {
            heal: 5 + b * 3,
            pulses: 4,
            gap: 20,
        },
        UnitKind::Gorgehowl => Ability::Interrupt {
            period: 160 - a * 20,
            lock_ticks: 80,
            windup: 2,
        },
        UnitKind::Sunlancer => Ability::Spark {
            every: 3,
            damage: 16 + b * 6,
            ticks: 60,
        },
        UnitKind::EclipseDancer => Ability::Eclipse {
            high_bp: 5_000,
            solar_bp: 1_500 + a * 500,
            lunar_bp: 600 + b * 300,
        },
        UnitKind::Starbreaker => Ability::Crusher {
            max_hp_bp: 400 + a * 100,
            cap: 50 + a * 10,
        },
        UnitKind::Dawnmender => Ability::Courage {
            heal_per_enemy: 8 + b * 2,
            enemies: 2,
            radius: 1_200,
        },
        UnitKind::Prismancer => Ability::Beam {
            period: 160,
            damage: 45 + a * 15,
            radius: 800 + b as i32 * 300,
            targets: 3,
        },
        UnitKind::Leafcutter => Ability::Bleeder {
            damage: 4 + a,
            pulses: 3,
        },
        UnitKind::Blastbeetle => Ability::DeathPop {
            damage: 60 + a * 15,
            radius: 900,
            targets: 3,
        },
        UnitKind::Needlewing => Ability::Opportunity {
            every: 3,
            bonus_bp: 3_000 + a * 1_000,
        },
        UnitKind::NectarNurse => Ability::Channel {
            pulses: 4 + b as u8,
            gap: 5,
            first_bp: 10_000 + a * 3_000,
        },
        UnitKind::Stridulator => Ability::Rhythm {
            period: 160 - b * 20,
            bp: 2_000,
            ticks: 30 + a * 10,
            radius: 700,
            targets: 2,
        },
        _ => return [super::Ability::None; 2],
    };
    [super::Ability::Completion(value), super::Ability::None]
}
