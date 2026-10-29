//! Arcanas (GAME-DESIGN §9, BR-AR01–BR-AR13): a free, twelve-point
//! pre-match choice across three five-node families. The six original
//! nodes keep their serialization indices and flat percentage bonuses;
//! the nine new nodes add conditional combat behavior. Validation, the
//! per-seat stats and the conditional hooks live here; per-rank values are
//! `Ruleset` data (`arcana_bp` and `arcana`), so the lab can tune them.

use serde::{Deserialize, Serialize};

use crate::types::{Archetype, Status};

/// Node order is a serialization contract (GAME-DESIGN §9).
pub const GUARD: usize = 0;
pub const ASSAULT: usize = 1;
pub const PIERCING: usize = 2;
pub const ENDURANCE: usize = 3;
pub const RESILIENCE: usize = 4;
pub const RENEWAL: usize = 5;
pub const SHIELDWALL: usize = 6;
pub const BATTLE_RUSH: usize = 7;
pub const COMBINED_ARMS: usize = 8;
pub const HUNTERS_FOCUS: usize = 9;
pub const QUICKDRAW: usize = 10;
pub const DEADEYE: usize = 11;
pub const UNSHAKEN: usize = 12;
pub const EMERGENCY_CARE: usize = 13;
pub const PURIFICATION: usize = 14;
pub const NODES: usize = 15;
/// A complete preset spends exactly this many points (BR-AR02).
pub const BUDGET: u8 = 12;
/// Original nodes take two ranks; children and supremes take one.
pub const MAX_RANK: u8 = 2;
pub const MAX: [u8; NODES] = [2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1];
/// Point cost of one rank of each node. A supreme costs two (BR-AR03).
pub const COST: [u8; NODES] = [1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 2, 1, 1, 2];
/// `(node, minimum rank)` a node needs before it can hold a rank. Keeping
/// this table beside the node order makes validation and the editor's
/// explanations read from one place.
pub const PREREQUISITES: [&[(usize, u8)]; NODES] = [
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[(GUARD, 1)],
    &[(ASSAULT, 1)],
    &[(GUARD, 2), (ASSAULT, 2), (SHIELDWALL, 1), (BATTLE_RUSH, 1)],
    &[(PIERCING, 1)],
    &[(ENDURANCE, 1)],
    &[
        (PIERCING, 2),
        (ENDURANCE, 2),
        (HUNTERS_FOCUS, 1),
        (QUICKDRAW, 1),
    ],
    &[(RESILIENCE, 1)],
    &[(RENEWAL, 1)],
    &[
        (RESILIENCE, 2),
        (RENEWAL, 2),
        (UNSHAKEN, 1),
        (EMERGENCY_CARE, 1),
    ],
];

/// Fifteen ranks in node order. Only complete presets are valid online.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArcanaPreset {
    pub ranks: [u8; NODES],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcanaError {
    /// A rank above the node maximum, an unmet prerequisite, or a total
    /// other than exactly twelve points.
    BadRanks,
}

impl ArcanaPreset {
    /// The complete default every player starts with (BR-AR08): all six
    /// original nodes at rank two, no conditional node preselected.
    pub const DEFAULT: ArcanaPreset = ArcanaPreset {
        ranks: [2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };
    /// No bonus: tests, the balance lab, and diagnostics only.
    pub const NEUTRAL: ArcanaPreset = ArcanaPreset { ranks: [0; NODES] };

    /// A complete preset (BR-AR02/03), or why not. Never panics on input.
    pub fn complete(ranks: [u8; NODES]) -> Result<ArcanaPreset, ArcanaError> {
        let preset = ArcanaPreset { ranks };
        if !preset.ranks_valid() || preset.spent() != u32::from(BUDGET) {
            return Err(ArcanaError::BadRanks);
        }
        Ok(preset)
    }

    /// The schema-1 six-node preset extended to twelve points: keep every
    /// legacy rank, then raise the original nodes to rank two in index
    /// order. No child or supreme is selected automatically, so migration
    /// never grants a conditional behavior the player did not choose.
    pub fn migrate_legacy(old: [u8; 6]) -> ArcanaPreset {
        let mut ranks = [0u8; NODES];
        for (i, &rank) in old.iter().enumerate() {
            ranks[i] = rank.min(MAX[i]);
        }
        let mut spent = cost_of(&ranks);
        for rank in ranks.iter_mut().take(6) {
            while spent < u32::from(BUDGET) && *rank < MAX_RANK {
                *rank += 1;
                spent += 1;
            }
        }
        ArcanaPreset { ranks }
    }

    /// True when every rank fits its node and every held rank satisfies
    /// its prerequisites, regardless of budget.
    pub fn ranks_valid(&self) -> bool {
        PREREQUISITES.iter().enumerate().all(|(node, needs)| {
            self.ranks[node] <= MAX[node]
                && (self.ranks[node] == 0
                    || needs.iter().all(|&(need, rank)| self.ranks[need] >= rank))
        })
    }

    /// Total points this preset costs (BR-AR03: a supreme costs two).
    pub fn spent(&self) -> u32 {
        cost_of(&self.ranks)
    }
}

fn cost_of(ranks: &[u8; NODES]) -> u32 {
    ranks
        .iter()
        .zip(COST)
        .map(|(&rank, cost)| u32::from(rank) * u32::from(cost))
        .sum()
}

/// Balance values for the nine conditional nodes. They live in `Ruleset`
/// so the offline lab can evaluate alternatives without touching online
/// play; the original six nodes keep their per-rank values in `arcana_bp`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArcanaValues {
    pub shieldwall_dr_bp: i32,
    pub battle_rush_move_bp: i32,
    pub combined_arms_dr_bp: i32,
    pub combined_arms_dmg_bp: i32,
    pub hunters_focus_dmg_bp: i32,
    pub quickdraw_atk_bp: i32,
    /// Maximum target distance in milli-tiles for Quickdraw.
    pub quickdraw_distance: i32,
    /// Flat attack range in milli-tiles for Deadeye.
    pub deadeye_range: i32,
    pub unshaken_duration_bp: i32,
    pub emergency_care_heal_bp: i32,
    pub emergency_care_below_bp: i32,
    pub purification_root_cut_bp: i32,
}

/// Basis points a preset adds to one unit's base attributes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArcanaBonus {
    pub hp_bp: i64,
    pub dmg_bp: i64,
    pub heal_bp: i64,
    pub range_add: i32,
}

/// Eligibility by base `Archetype` (BR-AR04): War for Tank and DPS,
/// Precision for Ranged, Vitality for Support and Control; Renewal only
/// for a unit with an existing heal. `per_rank` is `Ruleset::arcana_bp`.
pub fn bonus(
    preset: &ArcanaPreset,
    per_rank: &[i32; 6],
    values: &ArcanaValues,
    archetype: Archetype,
    has_heal: bool,
) -> ArcanaBonus {
    let at = |node: usize| i64::from(preset.ranks[node]) * i64::from(per_rank[node]);
    match archetype {
        Archetype::Tank | Archetype::Dps => ArcanaBonus {
            hp_bp: at(GUARD),
            dmg_bp: at(ASSAULT),
            heal_bp: 0,
            range_add: 0,
        },
        Archetype::Ranged => ArcanaBonus {
            hp_bp: at(ENDURANCE),
            dmg_bp: at(PIERCING),
            heal_bp: 0,
            range_add: if preset.ranks[DEADEYE] > 0 {
                values.deadeye_range
            } else {
                0
            },
        },
        Archetype::Support | Archetype::Control => ArcanaBonus {
            hp_bp: at(RESILIENCE),
            dmg_bp: 0,
            heal_bp: if has_heal { at(RENEWAL) } else { 0 },
            range_add: 0,
        },
    }
}

/// Shieldwall and Combined Arms damage reduction for a conventional Tank.
/// `tank_ally` and `dps_ally` are the owner's cached living counts; the
/// caller combines this with Augment reduction and its global cap.
pub fn damage_reduction_bp(
    preset: &ArcanaPreset,
    values: &ArcanaValues,
    archetype: Archetype,
    tank_ally: bool,
    dps_ally: bool,
) -> i32 {
    if archetype != Archetype::Tank {
        return 0;
    }
    let mut bp = 0;
    if preset.ranks[SHIELDWALL] > 0 && dps_ally {
        bp += values.shieldwall_dr_bp;
    }
    if preset.ranks[COMBINED_ARMS] > 0 && tank_ally && dps_ally {
        bp += values.combined_arms_dr_bp;
    }
    bp
}

/// Combined Arms (DPS) and Hunter's Focus (Ranged vs a slowed/rooted
/// target) damage bonus. The caller adds it to Augment situational damage
/// before the single truncation (BR-AR05).
pub fn damage_bonus_bp(
    preset: &ArcanaPreset,
    values: &ArcanaValues,
    archetype: Archetype,
    target: Status,
    tank_ally: bool,
) -> i32 {
    let mut bp = 0;
    if archetype == Archetype::Dps && preset.ranks[COMBINED_ARMS] > 0 && tank_ally {
        bp += values.combined_arms_dmg_bp;
    }
    if archetype == Archetype::Ranged
        && preset.ranks[HUNTERS_FOCUS] > 0
        && matches!(target, Status::Slowed { .. } | Status::Rooted { .. })
    {
        bp += values.hunters_focus_dmg_bp;
    }
    bp
}

/// Battle Rush movement bonus for a conventional DPS while a Tank lives.
pub fn move_bonus_bp(
    preset: &ArcanaPreset,
    values: &ArcanaValues,
    archetype: Archetype,
    tank_ally: bool,
) -> i32 {
    if archetype == Archetype::Dps && preset.ranks[BATTLE_RUSH] > 0 && tank_ally {
        values.battle_rush_move_bp
    } else {
        0
    }
}

/// Quickdraw attack-speed bonus for a conventional Ranged unit whose
/// current target is within two tiles.
pub fn attack_speed_bp(
    preset: &ArcanaPreset,
    values: &ArcanaValues,
    archetype: Archetype,
    distance: i32,
) -> i32 {
    if archetype == Archetype::Ranged
        && preset.ranks[QUICKDRAW] > 0
        && distance <= values.quickdraw_distance
    {
        values.quickdraw_atk_bp
    } else {
        0
    }
}

/// Unshaken lowers only the incoming status duration of Support and
/// Control units, never a slow's intensity (BR-AR07).
pub fn shorten_status(
    preset: &ArcanaPreset,
    values: &ArcanaValues,
    archetype: Archetype,
    ticks: u16,
) -> u16 {
    if !matches!(archetype, Archetype::Support | Archetype::Control) || preset.ranks[UNSHAKEN] == 0
    {
        return ticks;
    }
    let cut = values.unshaken_duration_bp.clamp(0, 9_000);
    (i64::from(ticks) * (10_000 - i64::from(cut)) / 10_000).max(1) as u16
}

/// Emergency Care efficiency bonus, evaluated on the target's HP before
/// the heal and using integer comparison (BR-AR08).
pub fn emergency_heal_bp(
    preset: &ArcanaPreset,
    values: &ArcanaValues,
    hp: i32,
    max_hp: i32,
) -> i32 {
    if preset.ranks[EMERGENCY_CARE] == 0 || max_hp <= 0 {
        return 0;
    }
    if i64::from(hp) * 10_000 < i64::from(values.emergency_care_below_bp) * i64::from(max_hp) {
        values.emergency_care_heal_bp
    } else {
        0
    }
}

/// Purification cleanses a slow or halves the remaining root after a real
/// heal; `None` for a supreme that is not held (BR-AR09).
pub fn purify_status(
    preset: &ArcanaPreset,
    values: &ArcanaValues,
    status: Status,
) -> Option<Status> {
    if preset.ranks[PURIFICATION] == 0 {
        return None;
    }
    let cut = values.purification_root_cut_bp.clamp(0, 9_000);
    Some(match status {
        Status::Slowed { .. } => Status::None,
        Status::Rooted { ticks_left } => Status::Rooted {
            ticks_left: (i64::from(ticks_left) * (10_000 - i64::from(cut)) / 10_000)
                .max(1)
                .min(i64::from(u16::MAX)) as u16,
        },
        Status::None => Status::None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranks() -> [u8; NODES] {
        [0; NODES]
    }

    #[test]
    fn every_new_node_names_its_prerequisites() {
        for (node, needs) in PREREQUISITES.iter().enumerate().skip(6) {
            assert!(!needs.is_empty(), "node {node} has no prerequisite");
            for &(need, rank) in needs.iter() {
                assert!(need < node, "node {node} depends on a later node");
                assert!(rank <= MAX[need]);
            }
        }
    }

    #[test]
    fn children_and_supremes_are_rejected_without_prerequisites() {
        let mut bad = ranks();
        bad[SHIELDWALL] = 1;
        bad[ASSAULT] = 2;
        bad[PIERCING] = 2;
        bad[ENDURANCE] = 2;
        bad[RESILIENCE] = 2;
        bad[RENEWAL] = 2;
        assert_eq!(bad.iter().map(|&r| u32::from(r)).sum::<u32>(), 11);
        assert_eq!(ArcanaPreset::complete(bad), Err(ArcanaError::BadRanks));
        let mut supreme = ranks();
        supreme[COMBINED_ARMS] = 1;
        supreme[GUARD] = 2;
        supreme[ASSAULT] = 2;
        supreme[SHIELDWALL] = 1;
        supreme[BATTLE_RUSH] = 1;
        assert_eq!(supreme.iter().map(|&r| u32::from(r)).sum::<u32>(), 7);
        assert!(ArcanaPreset::complete(supreme).is_err());
        supreme[PIERCING] = 2;
        supreme[ENDURANCE] = 2;
        assert!(ArcanaPreset::complete(supreme).is_ok());
    }

    #[test]
    fn legacy_migration_preserves_ranks_and_fills_originals() {
        assert_eq!(
            ArcanaPreset::migrate_legacy([2, 0, 2, 0, 2, 0]).ranks,
            ArcanaPreset::DEFAULT.ranks
        );
        assert_eq!(
            ArcanaPreset::migrate_legacy([1, 1, 1, 1, 1, 1]).ranks,
            ArcanaPreset::DEFAULT.ranks
        );
        let partial = ArcanaPreset::migrate_legacy([3, 0, 0, 0, 0, 0]);
        assert_eq!(partial.ranks[GUARD], 2);
        assert_eq!(partial.spent(), u32::from(BUDGET));
    }
}
