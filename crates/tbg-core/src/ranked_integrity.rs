//! Ranked settlement eligibility (P60 §2/§5, BR-RI, D-87): pure integer
//! gating of the two baseline Elo deltas. It never touches simulation
//! state, clocks or SQL, and never changes the Elo formula: a settlement
//! either keeps both baseline deltas unchanged or applies zero to both.

/// Suspicion axis of one settled match. `Normal -> Review` only by the
/// documented signal combination; `Invalidated` only by a human decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiskState {
    Normal,
    Review,
    Invalidated,
}

/// Settlement axis: a zero-delta equal draw is still `Applied`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingDisposition {
    Applied,
    NoChange,
}

impl RiskState {
    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn from_code(code: u8) -> Option<RiskState> {
        [RiskState::Normal, RiskState::Review, RiskState::Invalidated]
            .get(usize::from(code))
            .copied()
    }
}

impl RatingDisposition {
    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn from_code(code: u8) -> Option<RatingDisposition> {
        [RatingDisposition::Applied, RatingDisposition::NoChange]
            .get(usize::from(code))
            .copied()
    }
}

/// Reason bits (stored, versioned with the policy). Seat A is the low bit
/// of each pair.
pub const PAIR_LIMIT_A: u32 = 1 << 0;
pub const PAIR_LIMIT_B: u32 = 1 << 1;
pub const NEW_SOURCE_LIMIT_A: u32 = 1 << 2;
pub const NEW_SOURCE_LIMIT_B: u32 = 1 << 3;
/// Objective: the match ended before it reached Running (if adopted).
pub const NOT_STARTED: u32 = 1 << 4;
/// History was unavailable or over its row bound: no budget was evaluated.
pub const HISTORY_UNAVAILABLE: u32 = 1 << 5;

/// Positive points already received in the rolling window, per seat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BudgetFacts {
    /// From this exact opponent (Applied, non-Invalidated entries).
    pub pair_positive: [u32; 2],
    /// From all opponents that were immature at their settlement.
    pub immature_positive: [u32; 2],
    /// Whether each seat's opponent is immature now.
    pub opponent_immature: [bool; 2],
}

/// Inclusive ceilings (**Proposed**, owner decision O1: off by default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetPolicy {
    pub pair_limit: u32,
    pub immature_limit: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetDecision {
    /// Either the baseline deltas unchanged or `[0, 0]`.
    pub applied: [i32; 2],
    pub reason_bits: u32,
}

impl BudgetDecision {
    pub fn disposition(&self) -> RatingDisposition {
        if self.reason_bits == 0 {
            RatingDisposition::Applied
        } else {
            RatingDisposition::NoChange
        }
    }
}

/// BR-RI02: only a positive baseline delta can exceed a budget; exactly the
/// limit is allowed; any exceeded budget zeroes **both** deltas. Losses
/// never refund an allowance, and a capped player still loses normally
/// when only their own budget is exhausted and their delta is negative.
pub fn gate(base: [i32; 2], facts: &BudgetFacts, policy: &BudgetPolicy) -> BudgetDecision {
    let mut reason_bits = 0;
    for (seat, &delta) in base.iter().enumerate() {
        let gain = match u32::try_from(delta) {
            Ok(g) if g > 0 => g,
            _ => continue,
        };
        if facts.pair_positive[seat].saturating_add(gain) > policy.pair_limit {
            reason_bits |= PAIR_LIMIT_A << seat;
        }
        if facts.opponent_immature[seat]
            && facts.immature_positive[seat].saturating_add(gain) > policy.immature_limit
        {
            reason_bits |= NEW_SOURCE_LIMIT_A << seat;
        }
    }
    BudgetDecision {
        applied: if reason_bits == 0 { base } else { [0, 0] },
        reason_bits,
    }
}
