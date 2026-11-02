//! War Auxiliaries (`Mineiro` and `Mercador`): off-lane economic and shop
//! specialists hired with Energy during `Phase::Running` (P74, BR-AX01–BR-AX10).

use serde::{Deserialize, Serialize};

use crate::content::{ENERGY_SCALE, TICK_HZ};
use crate::intent::Rejection;
use crate::state::{Event, MatchState};
use crate::types::Seat;

/// Number of War Auxiliary slots per seat.
pub const AUXILIARY_SLOTS: usize = 2;

/// The two War Auxiliaries available to both seats in fixed slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuxiliaryKind {
    /// Slot 0: builds a mine over 12 s, then produces +0.06 Energy/s for 180 s.
    Miner,
    /// Slot 1: counts conventional champion purchases after hiring; every 3 grants 1 free reroll.
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
}

impl AuxiliarySlotView {
    pub const fn available(kind: AuxiliaryKind) -> Self {
        Self {
            kind,
            phase: AuxiliaryPhase::Available,
        }
    }
}

/// Balance parameters for War Auxiliaries (D-33 / P74).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuxiliaryRule {
    /// Sub-energy cost to hire the Miner (6 Energy = 3,600 sub-energy).
    pub miner_cost: u32,
    /// Construction ticks before Miner production starts (12 s = 240 ticks).
    pub miner_prep_ticks: u16,
    /// Active production window in combat ticks (180 s = 3,600 ticks).
    pub miner_duration_ticks: u16,
    /// Flat sub-energy produced per combat second (0.06 Energy/s = 36 sub-energy/s).
    pub miner_sub_energy_per_sec: u32,
    /// Sub-energy cost to hire the Merchant (4 Energy = 2,400 sub-energy).
    pub merchant_cost: u32,
    /// Conventional champion purchases after hiring required per free reroll.
    pub merchant_purchases_per_reroll: u8,
    /// Maximum free-reroll activations per match before the Merchant completes.
    pub merchant_max_activations: u8,
}

pub const AUXILIARY_RULE: AuxiliaryRule = AuxiliaryRule {
    miner_cost: 6 * ENERGY_SCALE,
    miner_prep_ticks: (12 * TICK_HZ) as u16,
    miner_duration_ticks: (180 * TICK_HZ) as u16,
    miner_sub_energy_per_sec: 36,
    merchant_cost: 4 * ENERGY_SCALE,
    merchant_purchases_per_reroll: 3,
    merchant_max_activations: 3,
};

impl Default for AuxiliaryRule {
    fn default() -> Self {
        AUXILIARY_RULE
    }
}

impl AuxiliaryRule {
    pub const fn cost_of(&self, kind: AuxiliaryKind) -> u32 {
        match kind {
            AuxiliaryKind::Miner => self.miner_cost,
            AuxiliaryKind::Merchant => self.merchant_cost,
        }
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
        let phases = state.player(seat).auxiliaries;
        [
            AuxiliarySlotView {
                kind: AuxiliaryKind::Miner,
                phase: phases[0],
            },
            AuxiliarySlotView {
                kind: AuxiliaryKind::Merchant,
                phase: phases[1],
            },
        ]
    })
}

/// Validate and apply `PlayerIntention::HireAuxiliary { slot }` (BR-AX01–BR-AX03).
pub(crate) fn hire(state: &mut MatchState, seat: Seat, slot: u8) -> Result<(), Rejection> {
    let Some(kind) = AuxiliaryKind::from_slot(slot) else {
        return Err(Rejection::BadAuxiliarySlot);
    };
    let idx = usize::from(slot);
    if state.player(seat).auxiliaries[idx] != AuxiliaryPhase::Available {
        return Err(Rejection::AuxiliaryAlreadyHired);
    }
    let rule = state.rules().auxiliary;
    let cost = rule.cost_of(kind);
    if !state.player_mut(seat).spend(cost) {
        return Err(Rejection::NotEnoughEnergy);
    }
    let next_phase = match kind {
        AuxiliaryKind::Miner => AuxiliaryPhase::Preparing {
            ticks_left: rule.miner_prep_ticks,
        },
        AuxiliaryKind::Merchant => AuxiliaryPhase::Active {
            ticks_left: 0,
            progress: 0,
            activations_used: 0,
        },
    };
    state.player_mut(seat).auxiliaries[idx] = next_phase;
    state
        .pending
        .push(Event::AuxiliaryHired { seat, slot, kind });
    Ok(())
}

/// Advance War Auxiliaries by one live combat tick (BR-AX04–BR-AX06).
/// Called after `MatchState::regen_energy` so `InterestRule::regen_bp` never
/// multiplies Miner output and before unit combat so Energy is available.
pub(crate) fn tick(state: &mut MatchState, ev: &mut Vec<Event>) {
    let rule = state.rules().auxiliary;
    let cap = state.rules().energy_cap;
    for seat in Seat::BOTH {
        let p = &mut state.players[seat.index()];
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
                let total_now =
                    (elapsed * u64::from(rule.miner_sub_energy_per_sec)) / u64::from(TICK_HZ);
                let total_prev =
                    ((elapsed - 1) * u64::from(rule.miner_sub_energy_per_sec)) / u64::from(TICK_HZ);
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

/// Record a conventional champion purchase for the seat's Merchant (BR-AX07–BR-AX10).
pub(crate) fn on_champion_purchase(state: &mut MatchState, seat: Seat) {
    let rule = state.rules().auxiliary;
    let p = state.player_mut(seat);
    if let AuxiliaryPhase::Active {
        ticks_left,
        progress,
        activations_used,
    } = p.auxiliaries[1]
    {
        let next_progress = progress.saturating_add(1);
        if next_progress >= rule.merchant_purchases_per_reroll {
            let next_used = activations_used.saturating_add(1);
            p.free_rerolls = p.free_rerolls.saturating_add(1);
            p.auxiliaries[1] = if next_used >= rule.merchant_max_activations {
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
        } else {
            p.auxiliaries[1] = AuxiliaryPhase::Active {
                ticks_left,
                progress: next_progress,
                activations_used,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArcanaPreset, Phase, PlayerIntention};

    fn running_match() -> MatchState {
        let mut s = MatchState::with_preview(77, [ArcanaPreset::NEUTRAL; 2]);
        while s.phase != Phase::Running {
            s.step();
        }
        s
    }

    #[test]
    fn br_ax01_to_ax06_miner_hires_once_prepares_12s_and_produces_exact_unmultiplied_energy() {
        let mut s = running_match();
        s.players[0].energy = 6 * ENERGY_SCALE;
        assert_eq!(
            s.apply_intention(Seat::A, 1, PlayerIntention::HireAuxiliary { slot: 2 }),
            Err(Rejection::BadAuxiliarySlot)
        );
        s.apply_intention(Seat::A, 2, PlayerIntention::HireAuxiliary { slot: 0 })
            .unwrap();
        assert_eq!(s.players[0].energy, 0);
        assert_eq!(
            s.players[0].auxiliaries[0],
            AuxiliaryPhase::Preparing { ticks_left: 240 }
        );
        assert_eq!(
            s.apply_intention(Seat::A, 3, PlayerIntention::HireAuxiliary { slot: 0 }),
            Err(Rejection::AuxiliaryAlreadyHired)
        );

        let mut twin = running_match();
        twin.players[0].energy = 0;

        for _ in 0..240 {
            s.step();
            twin.step();
        }
        assert_eq!(
            s.players[0].auxiliaries[0],
            AuxiliaryPhase::Active {
                ticks_left: 3_600,
                progress: 0,
                activations_used: 0,
            }
        );
        assert_eq!(
            s.players[0].energy, twin.players[0].energy,
            "no extra Energy during the 12 s preparation window"
        );

        // Give both seats maximum Interest (25+ Energy) so any Interest leak would multiply Miner output.
        s.players[0].energy = 30 * ENERGY_SCALE;
        twin.players[0].energy = 30 * ENERGY_SCALE;
        for _ in 0..3_600 {
            // Keep both seats in the same Interest band and avoid Augment pauses.
            s.augments.next_open_at = u32::MAX;
            twin.augments.next_open_at = u32::MAX;
            s.step();
            twin.step();
        }
        assert_eq!(s.players[0].auxiliaries[0], AuxiliaryPhase::Completed);
        assert_eq!(
            s.players[0].energy - twin.players[0].energy,
            6_480,
            "Miner produces exactly 10.8 Energy (6,480 sub-energy) over 180 s without Interest multiplication"
        );
        assert_eq!(
            s.apply_intention(Seat::A, 4, PlayerIntention::HireAuxiliary { slot: 0 }),
            Err(Rejection::AuxiliaryAlreadyHired)
        );
    }

    #[test]
    fn br_ax07_to_ax10_merchant_counts_post_hire_purchases_grants_three_free_rerolls_and_preserves_streak(
    ) {
        let mut s = running_match();
        s.players[0].energy = 100 * ENERGY_SCALE;
        // Pre-hire purchase must not advance Merchant progress.
        s.apply_intention(Seat::A, 1, PlayerIntention::Purchase { offer: 0 })
            .unwrap();
        // Paid reroll raises streak to 1.
        let paid_cost = s.view_for(Seat::A, &[]).reroll_cost.unwrap();
        assert_eq!(paid_cost, 600);
        s.apply_intention(Seat::A, 2, PlayerIntention::Reroll { cost: paid_cost })
            .unwrap();
        assert_eq!(s.players[0].reroll_streak, 1);

        // Hire Merchant (slot 1, cost 4 Energy = 2,400 sub-energy).
        let before_hire = s.players[0].energy;
        s.apply_intention(Seat::A, 3, PlayerIntention::HireAuxiliary { slot: 1 })
            .unwrap();
        assert_eq!(s.players[0].energy, before_hire - 4 * ENERGY_SCALE);
        assert_eq!(
            s.players[0].auxiliaries[1],
            AuxiliaryPhase::Active {
                ticks_left: 0,
                progress: 0,
                activations_used: 0,
            }
        );

        // Make 3 purchases -> 1 free reroll granted.
        for seq in 4..=6 {
            s.apply_intention(Seat::A, seq, PlayerIntention::Purchase { offer: 0 })
                .unwrap();
        }
        assert_eq!(s.players[0].free_rerolls, 1);
        assert_eq!(s.view_for(Seat::A, &[]).reroll_cost, Some(0));

        // Do another paid reroll after spending the free one: first spend the free reroll.
        let energy_before_free = s.players[0].energy;
        s.apply_intention(Seat::A, 7, PlayerIntention::Reroll { cost: 0 })
            .unwrap();
        assert_eq!(s.players[0].energy, energy_before_free);
        assert_eq!(s.players[0].free_rerolls, 0);
        assert_eq!(
            s.players[0].reroll_streak, 0,
            "free reroll does not increment reroll_streak"
        );

        // Accumulate the remaining 2 activations (6 more purchases) and complete at 3 total activations.
        for seq in 8..=13 {
            s.apply_intention(Seat::A, seq, PlayerIntention::Purchase { offer: 0 })
                .unwrap();
        }
        assert_eq!(s.players[0].free_rerolls, 2);
        assert_eq!(s.players[0].auxiliaries[1], AuxiliaryPhase::Completed);

        // Further purchases after Completed grant no extra free rerolls.
        for seq in 14..=16 {
            s.apply_intention(Seat::A, seq, PlayerIntention::Purchase { offer: 0 })
                .unwrap();
        }
        assert_eq!(s.players[0].free_rerolls, 2);
    }
}
