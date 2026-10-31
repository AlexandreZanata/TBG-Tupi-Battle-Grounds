//! P60 §4 shadow evaluation over one bounded history read: budget facts for
//! the pure gate and the S1–S4 signals. Pure: no SQL, no clock. Signals are
//! weak evidence; only S1 AND S2 AND S3 queues a match for human Review,
//! and nothing here changes a rating.

use tbg_core::ranked_integrity::{
    gate, BudgetFacts, BudgetPolicy, RatingDisposition, RiskState, HISTORY_UNAVAILABLE, NOT_STARTED,
};

use crate::store::AccountId;

/// Signal thresholds (**Proposed**, versioned with `SIGNAL_VERSION`).
pub const SIGNAL_VERSION: u32 = 1;
const MIN_PAIR_GAMES: u32 = 5;
const SHARE_PCT: u64 = 80;
const NET_TRANSFER: i64 = 40;
const MIN_BEHAVIOR_LOSSES: u32 = 3;
const MIN_NEW_SOURCES: usize = 5;

/// One earlier side from `ranked_sides`, joined with its match outcome and
/// integrity status. `won` is from this side's point of view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryRow {
    pub match_id: i64,
    pub account: AccountId,
    pub opponent: AccountId,
    pub opponent_mature: bool,
    pub effective_delta: i32,
    pub low_activity: bool,
    pub early_surrender: bool,
    /// `Some(true)` win, `Some(false)` loss, `None` draw.
    pub won: Option<bool>,
    pub risk: RiskState,
    pub disposition: RatingDisposition,
}

impl HistoryRow {
    /// Only Applied, non-Invalidated entries count toward budgets.
    fn counts(&self) -> bool {
        self.disposition == RatingDisposition::Applied && self.risk != RiskState::Invalidated
    }
}

/// The match being settled, seats A then B.
#[derive(Clone, Copy, Debug)]
pub struct Current {
    pub accounts: [AccountId; 2],
    pub base: [i32; 2],
    pub winner: Option<usize>,
    pub mature: [bool; 2],
    pub low_activity: [bool; 2],
    pub early_surrender: [bool; 2],
    pub reached_running: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Signals {
    pub s1_relationship: bool,
    pub s2_transfer: bool,
    pub s3_behavior: bool,
    pub s4_newcomers: bool,
    pub pair_games: u32,
    pub donor_games: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evaluation {
    pub facts: BudgetFacts,
    /// What enforce mode would apply; shadow records it only.
    pub would_applied: [i32; 2],
    pub would_bits: u32,
    /// Toward the winner (recipient); default for a draw.
    pub signals: Signals,
    pub risk: RiskState,
}

/// `history` is `None` when the read overflowed its bound or failed: no
/// budget or signal is guessed, the gap is recorded, and Elo still applies.
pub fn evaluate(
    history: Option<&[HistoryRow]>,
    cur: &Current,
    policy: &BudgetPolicy,
) -> Evaluation {
    let mut would_bits = if cur.reached_running { 0 } else { NOT_STARTED };
    let Some(rows) = history else {
        return Evaluation {
            facts: BudgetFacts::default(),
            would_applied: cur.base,
            would_bits: would_bits | HISTORY_UNAVAILABLE,
            signals: Signals::default(),
            risk: RiskState::Normal,
        };
    };
    let facts = budget_facts(rows, cur);
    let decision = gate(cur.base, &facts, policy);
    would_bits |= decision.reason_bits;
    let would_applied = if would_bits == 0 { cur.base } else { [0, 0] };
    let signals = cur
        .winner
        .map(|w| signals(rows, cur, w))
        .unwrap_or_default();
    let review = signals.s1_relationship && signals.s2_transfer && signals.s3_behavior;
    Evaluation {
        facts,
        would_applied,
        would_bits,
        signals,
        risk: if review {
            RiskState::Review
        } else {
            RiskState::Normal
        },
    }
}

fn budget_facts(rows: &[HistoryRow], cur: &Current) -> BudgetFacts {
    let mut facts = BudgetFacts {
        opponent_immature: [!cur.mature[1], !cur.mature[0]],
        ..BudgetFacts::default()
    };
    for (seat, &me) in cur.accounts.iter().enumerate() {
        let opponent = cur.accounts[1 - seat];
        for r in rows.iter().filter(|r| r.account == me && r.counts()) {
            let gain = r.effective_delta.max(0) as u32;
            if r.opponent == opponent {
                facts.pair_positive[seat] = facts.pair_positive[seat].saturating_add(gain);
            }
            if !r.opponent_mature {
                facts.immature_positive[seat] = facts.immature_positive[seat].saturating_add(gain);
            }
        }
    }
    facts
}

/// Signals toward recipient `w` from donor `d`, the current match included.
fn signals(rows: &[HistoryRow], cur: &Current, w: usize) -> Signals {
    let d = 1 - w;
    let (recipient, donor) = (cur.accounts[w], cur.accounts[d]);
    let donor_rows = || rows.iter().filter(move |r| r.account == donor);
    let pair = || donor_rows().filter(move |r| r.opponent == recipient);
    let donor_games = donor_rows().count() as u32 + 1;
    let pair_games = pair().count() as u32 + 1;
    let donor_losses = pair().filter(|r| r.won == Some(false)).count() as u32 + 1;
    let behavior = pair()
        .filter(|r| r.won == Some(false) && (r.low_activity || r.early_surrender))
        .count() as u32
        + u32::from(cur.low_activity[d] || cur.early_surrender[d]);
    let net: i64 = rows
        .iter()
        .filter(|r| r.account == recipient && r.opponent == donor && r.counts())
        .map(|r| i64::from(r.effective_delta))
        .sum::<i64>()
        + i64::from(cur.base[w]);
    let s1 = pair_games >= MIN_PAIR_GAMES
        && u64::from(pair_games) * 100 >= SHARE_PCT * u64::from(donor_games);
    let s2 = net >= NET_TRANSFER
        && pair_games >= MIN_PAIR_GAMES
        && u64::from(donor_losses) * 100 >= SHARE_PCT * u64::from(pair_games);
    Signals {
        s1_relationship: s1,
        s2_transfer: s2,
        s3_behavior: behavior >= MIN_BEHAVIOR_LOSSES,
        s4_newcomers: newcomers(rows, cur, w),
        pair_games,
        donor_games,
    }
}

/// S4: at least 80 % of the recipient's positive inflow from immature
/// opponents, from at least five distinct immature source accounts.
fn newcomers(rows: &[HistoryRow], cur: &Current, w: usize) -> bool {
    let recipient = cur.accounts[w];
    let mut total: u64 = 0;
    let mut immature: u64 = 0;
    let mut sources: Vec<AccountId> = Vec::new();
    let current = (cur.accounts[1 - w], !cur.mature[1 - w], cur.base[w]);
    let earlier = rows
        .iter()
        .filter(|r| r.account == recipient && r.counts())
        .map(|r| (r.opponent, !r.opponent_mature, r.effective_delta));
    for (opponent, is_immature, delta) in earlier.chain(std::iter::once(current)) {
        let gain = delta.max(0) as u64;
        if gain == 0 {
            continue;
        }
        total += gain;
        if is_immature {
            immature += gain;
            if !sources.contains(&opponent) {
                sources.push(opponent);
            }
        }
    }
    total > 0 && immature * 100 >= SHARE_PCT * total && sources.len() >= MIN_NEW_SOURCES
}
