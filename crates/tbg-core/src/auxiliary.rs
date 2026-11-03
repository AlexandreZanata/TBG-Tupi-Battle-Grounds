//! War Auxiliaries (`Mineiro` and `Mercador`): off-lane economic and shop
//! specialists hired with Energy during `Phase::Running` and evolved through
//! three cumulative tiers (P74, BR-AX01–BR-AX10; tiers BR-AX12–BR-AX15).

use serde::{Deserialize, Serialize};

use crate::content::{ENERGY_SCALE, TICK_HZ};
use crate::intent::Rejection;
use crate::state::{Event, MatchState};
use crate::types::Seat;

/// Number of War Auxiliary slots per seat.
pub const AUXILIARY_SLOTS: usize = 2;
/// Highest War Auxiliary tier (BR-AX12): hire = tier 1, two Evolutions after it.
pub const AUXILIARY_MAX_TIER: u8 = 3;
const TIERS: usize = AUXILIARY_MAX_TIER as usize;

/// The two War Auxiliaries available to both seats in fixed slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuxiliaryKind {
    /// Slot 0: builds a mine over 12 s, then produces Energy for 180 s per tier.
    Miner,
    /// Slot 1: counts conventional champion purchases after hiring and pays out free rerolls (and Energy from tier 2).
    Merchant,
}

impl AuxiliaryKind {
    pub const ALL: [AuxiliaryKind; AUXILIARY_SLOTS] =
        [AuxiliaryKind::Miner, AuxiliaryKind::Merchant];

    pub const fn slot(self) -> u8 {
        match self {
            Self::Miner => 0,
            Self::Merchant => 1,
        }
    }

    pub const fn from_slot(slot: u8) -> Option<Self> {
        match slot {
            0 => Some(Self::Miner),
            1 => Some(Self::Merchant),
            _ => None,
        }
    }
}

/// Per-slot lifecycle state of a War Auxiliary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuxiliaryPhase {
    /// Not yet hired this match; can be hired while `Phase::Running`.
    #[default]
    Available,
    /// Hired and constructing (`Miner` 12 s = 240 ticks).
    Preparing { ticks_left: u16 },
    /// Operational (`Miner` counts down `ticks_left`; `Merchant` tracks `progress` and `activations_used`).
    Active {
        ticks_left: u16,
        progress: u8,
        activations_used: u8,
    },
    /// Exhausted for the remainder of the match; cannot be hired again.
    Completed,
}

/// Public snapshot of one War Auxiliary slot on a seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuxiliarySlotView {
    pub kind: AuxiliaryKind,
    pub phase: AuxiliaryPhase,
    /// `0` while not hired, else `1..=AUXILIARY_MAX_TIER` (BR-AX12).
    pub tier: u8,
}

impl AuxiliarySlotView {
    pub const fn available(kind: AuxiliaryKind) -> Self {
        Self {
            kind,
            phase: AuxiliaryPhase::Available,
            tier: 0,
        }
    }
}

/// Balance parameters for War Auxiliaries (D-33 / P74). Every array is
/// indexed by `tier - 1`; costs are paid per step (hire, then each Evolution).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuxiliaryRule {
    /// Sub-energy paid to reach each Miner tier (hire 8, then 14, then 22 Energy).
    pub miner_cost: [u32; TIERS],
    /// Construction ticks before Miner production starts (12 s = 240 ticks).
    pub miner_prep_ticks: u16,
    /// Active production window in combat ticks; each Evolution restarts it (180 s).
    pub miner_duration_ticks: u16,
    /// Cumulative sub-energy produced per combat second at each tier.
    pub miner_sub_energy_per_sec: [u32; TIERS],
    /// Sub-energy paid to reach each Merchant tier (hire 6, then 10, then 16 Energy).
    pub merchant_cost: [u32; TIERS],
    /// Conventional champion purchases required per activation at each tier.
    pub merchant_purchases_per_activation: [u8; TIERS],
    /// Free rerolls granted per activation at each tier.
    pub merchant_rerolls_per_activation: [u8; TIERS],
    /// Sub-energy refunded per activation at each tier.
    pub merchant_energy_per_activation: [u32; TIERS],
    /// Cumulative activation cap per match at each tier.
    pub merchant_max_activations: [u8; TIERS],
}

pub const AUXILIARY_RULE: AuxiliaryRule = AuxiliaryRule {
    miner_cost: [8 * ENERGY_SCALE, 14 * ENERGY_SCALE, 22 * ENERGY_SCALE],
    miner_prep_ticks: (12 * TICK_HZ) as u16,
    miner_duration_ticks: (180 * TICK_HZ) as u16,
    // +0.15, +0.30 and +0.50 Energy/s.
    miner_sub_energy_per_sec: [90, 180, 300],
    merchant_cost: [6 * ENERGY_SCALE, 10 * ENERGY_SCALE, 16 * ENERGY_SCALE],
    merchant_purchases_per_activation: [3, 2, 2],
    merchant_rerolls_per_activation: [1, 1, 2],
    merchant_energy_per_activation: [0, ENERGY_SCALE, 2 * ENERGY_SCALE],
    merchant_max_activations: [4, 8, 12],
};

impl Default for AuxiliaryRule {
    fn default() -> Self {
        AUXILIARY_RULE
    }
}

impl AuxiliaryRule {
    /// Sub-energy to reach `tier` (`1..=AUXILIARY_MAX_TIER`); `None` past the last tier.
    pub fn cost_of(&self, kind: AuxiliaryKind, tier: u8) -> Option<u32> {
        let table = match kind {
            AuxiliaryKind::Miner => &self.miner_cost,
            AuxiliaryKind::Merchant => &self.merchant_cost,
        };
        table.get(usize::from(tier.checked_sub(1)?)).copied()
    }
}

pub const fn default_slot_views() -> [AuxiliarySlotView; AUXILIARY_SLOTS] {
    [
        AuxiliarySlotView::available(AuxiliaryKind::Miner),
        AuxiliarySlotView::available(AuxiliaryKind::Merchant),
    ]
}

pub fn views(state: &MatchState) -> [[AuxiliarySlotView; AUXILIARY_SLOTS]; 2] {
    Seat::BOTH.map(|seat| {
        let p = state.player(seat);
        AuxiliaryKind::ALL.map(|kind| {
            let idx = usize::from(kind.slot());
            AuxiliarySlotView {
                kind,
                phase: p.auxiliaries[idx],
                tier: p.auxiliary_tiers[idx],
            }
        })
    })
}

/// Index into a per-tier table for a hired slot (tier `0` reads tier 1).
fn tier_index(tier: u8) -> usize {
    usize::from(tier.clamp(1, AUXILIARY_MAX_TIER) - 1)
}

/// Validate and apply `PlayerIntention::HireAuxiliary { slot }`: hire at
/// tier 1, or evolve a hired slot one tier (BR-AX01–BR-AX03, BR-AX12–BR-AX13).
pub(crate) fn hire(state: &mut MatchState, seat: Seat, slot: u8) -> Result<(), Rejection> {
    let Some(kind) = AuxiliaryKind::from_slot(slot) else {
        return Err(Rejection::BadAuxiliarySlot);
    };
    let idx = usize::from(slot);
    let tier = state.player(seat).auxiliary_tiers[idx];
    let next_tier = tier + 1;
    let rule = state.rules().auxiliary;
    let Some(cost) = rule.cost_of(kind, next_tier) else {
        return Err(Rejection::AuxiliaryAlreadyHired);
    };
    if !state.player_mut(seat).spend(cost) {
        return Err(Rejection::NotEnoughEnergy);
    }
    let p = state.player_mut(seat);
    p.auxiliary_tiers[idx] = next_tier;
    p.auxiliaries[idx] = next_phase(kind, p.auxiliaries[idx], tier, &rule);
    state
        .pending
        .push(Event::AuxiliaryHired { seat, slot, kind });
    Ok(())
}

/// Phase after a hire or Evolution (BR-AX13): a Miner keeps building if it
/// is still Preparing, otherwise restarts its full production window; a
/// Merchant (re)opens with its activation history kept.
fn next_phase(
    kind: AuxiliaryKind,
    phase: AuxiliaryPhase,
    tier: u8,
    rule: &AuxiliaryRule,
) -> AuxiliaryPhase {
    match (kind, phase) {
        (AuxiliaryKind::Miner, AuxiliaryPhase::Available) => AuxiliaryPhase::Preparing {
            ticks_left: rule.miner_prep_ticks,
        },
        (AuxiliaryKind::Miner, AuxiliaryPhase::Preparing { .. }) => phase,
        (AuxiliaryKind::Miner, _) => AuxiliaryPhase::Active {
            ticks_left: rule.miner_duration_ticks,
            progress: 0,
            activations_used: 0,
        },
        (AuxiliaryKind::Merchant, AuxiliaryPhase::Active { .. }) => phase,
        (AuxiliaryKind::Merchant, AuxiliaryPhase::Completed) => AuxiliaryPhase::Active {
            ticks_left: 0,
            progress: 0,
            activations_used: rule.merchant_max_activations[tier_index(tier)],
        },
        (AuxiliaryKind::Merchant, _) => AuxiliaryPhase::Active {
            ticks_left: 0,
            progress: 0,
            activations_used: 0,
        },
    }
}

/// Advance War Auxiliaries by one live combat tick (BR-AX04–BR-AX06).
/// Called after `MatchState::regen_energy` so `InterestRule::regen_bp` never
/// multiplies Miner output and before unit combat so Energy is available.
pub(crate) fn tick(state: &mut MatchState, ev: &mut Vec<Event>) {
    let rule = state.rules().auxiliary;
    let cap = state.rules().energy_cap;
    for seat in Seat::BOTH {
        let p = &mut state.players[seat.index()];
        let per_sec = rule.miner_sub_energy_per_sec[tier_index(p.auxiliary_tiers[0])];
        match p.auxiliaries[0] {
            AuxiliaryPhase::Preparing { ticks_left } => {
                let next = ticks_left.saturating_sub(1);
                if next == 0 {
                    p.auxiliaries[0] = AuxiliaryPhase::Active {
                        ticks_left: rule.miner_duration_ticks,
                        progress: 0,
                        activations_used: 0,
                    };
                } else {
                    p.auxiliaries[0] = AuxiliaryPhase::Preparing { ticks_left: next };
                }
            }
            AuxiliaryPhase::Active {
                ticks_left,
                progress,
                activations_used,
            } if ticks_left > 0 => {
                let elapsed = u64::from(rule.miner_duration_ticks.saturating_sub(ticks_left)) + 1;
                let total_now = (elapsed * u64::from(per_sec)) / u64::from(TICK_HZ);
                let total_prev = ((elapsed - 1) * u64::from(per_sec)) / u64::from(TICK_HZ);
                let gain = (total_now - total_prev) as u32;
                p.energy = p.energy.saturating_add(gain);
                if let Some(cap) = cap {
                    p.energy = p.energy.min(cap);
                }
                let next = ticks_left - 1;
                if next == 0 {
                    p.auxiliaries[0] = AuxiliaryPhase::Completed;
                    ev.push(Event::AuxiliaryTriggered {
                        seat,
                        kind: AuxiliaryKind::Miner,
                    });
                } else {
                    p.auxiliaries[0] = AuxiliaryPhase::Active {
                        ticks_left: next,
                        progress,
                        activations_used,
                    };
                }
            }
            _ => {}
        }
    }
}

/// Record a conventional champion purchase for the seat's Merchant
/// (BR-AX07–BR-AX10, tier payouts BR-AX14).
pub(crate) fn on_champion_purchase(state: &mut MatchState, seat: Seat) {
    let rule = state.rules().auxiliary;
    let cap = state.rules().energy_cap;
    let p = state.player_mut(seat);
    let AuxiliaryPhase::Active {
        ticks_left,
        progress,
        activations_used,
    } = p.auxiliaries[1]
    else {
        return;
    };
    let t = tier_index(p.auxiliary_tiers[1]);
    let next_progress = progress.saturating_add(1);
    if next_progress < rule.merchant_purchases_per_activation[t] {
        p.auxiliaries[1] = AuxiliaryPhase::Active {
            ticks_left,
            progress: next_progress,
            activations_used,
        };
        return;
    }
    let next_used = activations_used.saturating_add(1);
    p.free_rerolls = p
        .free_rerolls
        .saturating_add(rule.merchant_rerolls_per_activation[t]);
    p.energy = p
        .energy
        .saturating_add(rule.merchant_energy_per_activation[t]);
    if let Some(cap) = cap {
        p.energy = p.energy.min(cap);
    }
    p.auxiliaries[1] = if next_used >= rule.merchant_max_activations[t] {
        AuxiliaryPhase::Completed
    } else {
        AuxiliaryPhase::Active {
            ticks_left,
            progress: 0,
            activations_used: next_used,
        }
    };
    state.pending.push(Event::AuxiliaryTriggered {
        seat,
        kind: AuxiliaryKind::Merchant,
    });
}

#[cfg(test)]
mod tests;
