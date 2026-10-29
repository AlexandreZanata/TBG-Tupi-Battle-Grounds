//! Arcanas (GAME-DESIGN §9, BR-AR01–BR-AR10; active by D-40): a free,
//! six-point pre-match choice that adds a percentage to a unit's base max
//! HP, damage, or healing, by base `Archetype`. Validation and the per-seat
//! bonus live here; the per-rank values are `Ruleset` data (`arcana_bp`).

use serde::{Deserialize, Serialize};

use crate::types::Archetype;

/// Node order is a serialization contract (GAME-DESIGN §9).
pub const GUARD: usize = 0;
pub const ASSAULT: usize = 1;
pub const PIERCING: usize = 2;
pub const ENDURANCE: usize = 3;
pub const RESILIENCE: usize = 4;
pub const RENEWAL: usize = 5;
pub const NODES: usize = 6;
/// A complete preset spends exactly this many points (BR-AR02).
pub const BUDGET: u8 = 6;
pub const MAX_RANK: u8 = 2;

/// Six ranks in node order. Only complete presets are valid online.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArcanaPreset {
    pub ranks: [u8; NODES],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcanaError {
    /// A rank above 2 or a total other than exactly 6 points.
    BadRanks,
}

impl ArcanaPreset {
    /// The complete default every player starts with (BR-AR08).
    pub const DEFAULT: ArcanaPreset = ArcanaPreset {
        ranks: [2, 0, 2, 0, 2, 0],
    };
    /// No bonus: tests, the balance lab, and diagnostics only.
    pub const NEUTRAL: ArcanaPreset = ArcanaPreset { ranks: [0; NODES] };

    /// A complete preset (BR-AR02), or why not. Never panics on any input.
    pub fn complete(ranks: [u8; NODES]) -> Result<ArcanaPreset, ArcanaError> {
        let total: u32 = ranks.iter().map(|&r| u32::from(r)).sum();
        if ranks.iter().any(|&r| r > MAX_RANK) || total != u32::from(BUDGET) {
            return Err(ArcanaError::BadRanks);
        }
        Ok(ArcanaPreset { ranks })
    }
}

/// Basis points a preset adds to one unit's base attributes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArcanaBonus {
    pub hp_bp: i64,
    pub dmg_bp: i64,
    pub heal_bp: i64,
}

/// Eligibility by base `Archetype` (BR-AR03): War for Tank and DPS,
/// Precision for Ranged, Vitality for Support and Control; Renewal only
/// for a unit with an existing heal. `per_rank` is `Ruleset::arcana_bp`.
pub fn bonus(
    preset: &ArcanaPreset,
    per_rank: &[i32; NODES],
    archetype: Archetype,
    has_heal: bool,
) -> ArcanaBonus {
    let at = |node: usize| i64::from(preset.ranks[node]) * i64::from(per_rank[node]);
    match archetype {
        Archetype::Tank | Archetype::Dps => ArcanaBonus {
            hp_bp: at(GUARD),
            dmg_bp: at(ASSAULT),
            heal_bp: 0,
        },
        Archetype::Ranged => ArcanaBonus {
            hp_bp: at(ENDURANCE),
            dmg_bp: at(PIERCING),
            heal_bp: 0,
        },
        Archetype::Support | Archetype::Control => ArcanaBonus {
            hp_bp: at(RESILIENCE),
            dmg_bp: 0,
            heal_bp: if has_heal { at(RENEWAL) } else { 0 },
        },
    }
}
