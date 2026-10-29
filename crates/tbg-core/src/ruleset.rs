//! Every tunable balance number as one value (D-33). The simulation reads
//! numbers only through `MatchState::rules()`, so the offline balance lab
//! can play a variant; the server and the client only ever use `CANON`,
//! which is assembled from `content.rs` (each number is still written once).
//!
//! Structural values stay `const` in `content.rs` and are never tuned:
//! they define units of measure, protocol shapes, or array bounds
//! (`TICK_HZ`, `LANE_LEN`, `SPAWN_OFFSET`, `ENERGY_SCALE`, `BP`, `OFFERS`,
//! `ROTATION_TICKS`, `COUNTDOWN`, `REGULATION`, `OVERTIME`, `WINDUP`,
//! `MIN_INTERVAL`, `REGEN_PERIOD`, `BEETLES_PER_PLAYER`).

use crate::content::{
    FormDef, Stats, SynergyEffect, UnitDef, BEETLES_PER_NESTER, BEETLE_LIFETIME, BP, ENERGY_CAP,
    ENERGY_START, EVOLVE_COST, HATCH_EVERY, HATCH_EVERY_HIVE, MIN_INTERVAL, REGEN_OVERTIME,
    REGEN_RUNNING, STRONGHOLD_DAMAGE, STRONGHOLD_HP, STRONGHOLD_INTERVAL, STRONGHOLD_RANGE,
    SYNERGIES, SYNERGY_THRESHOLDS, UNITS,
};
use crate::types::{Tick, UnitKind, ALL_KINDS};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ruleset {
    pub energy_start: u32,
    pub energy_cap: u32,
    pub regen_running: u32,
    pub regen_overtime: u32,
    /// Cost of evolving to tier 1 and to tier 2.
    pub evolve_cost: [u32; 2],
    pub stronghold_hp: i32,
    pub stronghold_damage: i32,
    pub stronghold_interval: u16,
    pub stronghold_range: i32,
    pub synergy_thresholds: [u8; 2],
    pub synergies: [(SynergyEffect, [i32; 2]); 4],
    pub hatch_every: u16,
    pub hatch_every_hive: u16,
    pub beetles_per_nester: [usize; 2],
    pub beetle_lifetime: Tick,
    /// Every kind, in `ALL_KINDS` order.
    pub units: [UnitDef; ALL_KINDS.len()],
}

/// The only ruleset the server and the client use.
pub static CANON: Ruleset = Ruleset {
    energy_start: ENERGY_START,
    energy_cap: ENERGY_CAP,
    regen_running: REGEN_RUNNING,
    regen_overtime: REGEN_OVERTIME,
    evolve_cost: EVOLVE_COST,
    stronghold_hp: STRONGHOLD_HP,
    stronghold_damage: STRONGHOLD_DAMAGE,
    stronghold_interval: STRONGHOLD_INTERVAL,
    stronghold_range: STRONGHOLD_RANGE,
    synergy_thresholds: SYNERGY_THRESHOLDS,
    synergies: SYNERGIES,
    hatch_every: HATCH_EVERY,
    hatch_every_hive: HATCH_EVERY_HIVE,
    beetles_per_nester: BEETLES_PER_NESTER,
    beetle_lifetime: BEETLE_LIFETIME,
    units: UNITS,
};

impl Ruleset {
    pub fn canon() -> &'static Ruleset {
        &CANON
    }

    pub fn def(&self, kind: UnitKind) -> &UnitDef {
        &self.units[kind as usize]
    }

    /// The form a unit at `tier` (1 or 2) on `branch` (0 = A, 1 = B) has.
    pub fn form_of(&self, kind: UnitKind, tier: u8, branch: u8) -> Option<&FormDef> {
        if tier == 0 || tier > 2 || branch > 1 {
            return None;
        }
        Some(&self.def(kind).forms[usize::from(branch) * 2 + usize::from(tier) - 1])
    }

    /// Hatch timing and per-Nester limit for a Nester's form: Brood (A1)
    /// raises the limit, Hive (A2) also hatches faster.
    pub fn hatch_rule(&self, tier: u8, branch: u8) -> (u16, usize) {
        let brood = branch == 0 && tier >= 1;
        let every = if branch == 0 && tier >= 2 {
            self.hatch_every_hive
        } else {
            self.hatch_every
        };
        (every, self.beetles_per_nester[usize::from(brood)])
    }

    /// Effective stats after evolution forms and the seat's synergy level.
    pub fn effective_stats(
        &self,
        kind: UnitKind,
        tier: u8,
        branch: u8,
        synergy_level: u8,
    ) -> Stats {
        let d = self.def(kind);
        // No allocation: this runs for every unit on every tick.
        let forms = [self.form_of(kind, 1, branch), self.form_of(kind, 2, branch)];
        let active = &forms[..usize::from(tier.min(2))];
        let sum = |f: fn(&FormDef) -> i32| -> i64 {
            active.iter().flatten().map(|x| i64::from(f(x))).sum()
        };
        let (mut hp_bp, mut dmg_bp, mut atk_bp) =
            (sum(|f| f.hp_bp), sum(|f| f.dmg_bp), sum(|f| f.atk_bp));
        if synergy_level > 0 {
            let (effect, bps) = self.synergies[d.faction.index()];
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
            on_hit: active
                .iter()
                .rev()
                .flatten()
                .find_map(|f| f.on_hit)
                .or(d.on_hit),
        }
    }
}
