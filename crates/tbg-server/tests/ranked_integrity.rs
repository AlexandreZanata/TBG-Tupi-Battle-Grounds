//! P60 shadow evaluation (GAME-DESIGN §14 BR-RI, D-87): attack fixtures
//! from P60 §3 over the pure evaluator. Repetition alone stays Normal;
//! only S1 AND S2 AND S3 queues Review; budgets are recorded as `would_*`
//! and never change who is paired or the baseline Elo.

use tbg_core::ranked_integrity::{
    BudgetPolicy, RatingDisposition, RiskState, HISTORY_UNAVAILABLE, NEW_SOURCE_LIMIT_A,
    NOT_STARTED, PAIR_LIMIT_A,
};
use tbg_server::ranked::policy::RankedPolicy;
use tbg_server::ranked::signals::{evaluate, Current, HistoryRow};

const POLICY: BudgetPolicy = BudgetPolicy {
    pair_limit: 120,
    immature_limit: 120,
};
const A: i64 = 1;
const B: i64 = 2;

/// Seat A (account 1) beats seat B (account 2) now.
fn current(base: [i32; 2], mature: [bool; 2], b_idle: bool) -> Current {
    Current {
        accounts: [A, B],
        base,
        winner: Some(0),
        mature,
        low_activity: [false, b_idle],
        early_surrender: [false, false],
        reached_running: true,
    }
}

/// Both sides of `n` earlier matches where A beat B.
fn a_beats_b(n: usize, gain: i32, b_idle: bool) -> Vec<HistoryRow> {
    (0..n as i64)
        .flat_map(|m| {
            [
                row(m, A, B, true, gain, false, Some(true)),
                row(m, B, A, true, -gain, b_idle, Some(false)),
            ]
        })
        .collect()
}

fn row(
    match_id: i64,
    account: i64,
    opponent: i64,
    opponent_mature: bool,
    effective_delta: i32,
    low_activity: bool,
    won: Option<bool>,
) -> HistoryRow {
    HistoryRow {
        match_id,
        account,
        opponent,
        opponent_mature,
        effective_delta,
        low_activity,
        early_surrender: false,
        won,
        risk: RiskState::Normal,
        disposition: RatingDisposition::Applied,
    }
}

#[test]
fn a12_repetition_alone_stays_normal() {
    // Honest friends: many games, both play, A wins most.
    let mut rows = a_beats_b(8, 12, false);
    rows.extend([
        row(90, B, A, true, 12, false, Some(true)),
        row(90, A, B, true, -12, false, Some(false)),
    ]);
    let e = evaluate(
        Some(&rows),
        &current([12, -12], [true, true], false),
        &POLICY,
    );
    assert!(e.signals.s1_relationship && e.signals.s2_transfer);
    assert!(!e.signals.s3_behavior);
    assert_eq!(e.risk, RiskState::Normal, "no behavior evidence: Normal");
}

#[test]
fn a2_idle_donor_relationship_is_queued_for_review() {
    let rows = a_beats_b(4, 12, true);
    let e = evaluate(
        Some(&rows),
        &current([12, -12], [true, true], true),
        &POLICY,
    );
    assert!(e.signals.s1_relationship && e.signals.s2_transfer && e.signals.s3_behavior);
    assert_eq!(e.risk, RiskState::Review);
    // Shadow: the evaluation never zeroes the baseline here (48+12 <= 120).
    assert_eq!(e.would_applied, [12, -12]);
}

#[test]
fn pair_budget_would_zero_both_and_exact_limit_passes() {
    // 9 earlier wins of +12 = 108; +12 now = exactly 120: allowed.
    let rows = a_beats_b(9, 12, false);
    let e = evaluate(
        Some(&rows),
        &current([12, -12], [true, true], false),
        &POLICY,
    );
    assert_eq!((e.would_bits, e.would_applied), (0, [12, -12]));
    // One more point is over: both would be zero, never a partial award.
    let e = evaluate(
        Some(&rows),
        &current([13, -13], [true, true], false),
        &POLICY,
    );
    assert_eq!((e.would_bits, e.would_applied), (PAIR_LIMIT_A, [0, 0]));
}

#[test]
fn a3_disposable_donors_hit_the_newcomer_budget_and_s4() {
    // Recipient A took +20 from each of six fresh accounts (immature).
    let rows: Vec<HistoryRow> = (0..6)
        .map(|m| row(m, A, 10 + m, false, 20, false, Some(true)))
        .collect();
    let e = evaluate(
        Some(&rows),
        &current([20, -20], [true, false], false),
        &POLICY,
    );
    assert_eq!(e.facts.immature_positive[0], 120);
    assert_eq!(e.would_bits & NEW_SOURCE_LIMIT_A, NEW_SOURCE_LIMIT_A);
    assert!(e.signals.s4_newcomers);
    assert_eq!(e.risk, RiskState::Normal, "S4 alone never queues Review");
}

#[test]
fn invalidated_and_no_change_entries_never_consume_budget() {
    let mut rows = a_beats_b(10, 12, false);
    for r in rows.iter_mut() {
        r.risk = RiskState::Invalidated;
    }
    let e = evaluate(
        Some(&rows),
        &current([12, -12], [true, true], false),
        &POLICY,
    );
    assert_eq!(e.facts.pair_positive, [0, 0]);
    for r in rows.iter_mut() {
        r.risk = RiskState::Normal;
        r.disposition = RatingDisposition::NoChange;
    }
    let e = evaluate(
        Some(&rows),
        &current([12, -12], [true, true], false),
        &POLICY,
    );
    assert_eq!(e.facts.pair_positive, [0, 0]);
}

#[test]
fn unavailable_history_and_unstarted_matches_are_recorded_not_guessed() {
    let mut cur = current([20, -20], [true, true], false);
    cur.reached_running = false;
    let e = evaluate(None, &cur, &POLICY);
    assert_eq!(e.would_bits, HISTORY_UNAVAILABLE | NOT_STARTED);
    assert_eq!(e.would_applied, [20, -20], "no budget is guessed");
    assert_eq!(e.risk, RiskState::Normal);
}

#[test]
fn integrity_settings_are_validated_at_start_up() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| {
            pairs
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| v.to_string())
        }
    };
    assert_eq!(
        RankedPolicy::from_lookup(env(&[])),
        Ok(RankedPolicy::default())
    );
    assert!(RankedPolicy::from_lookup(env(&[("TBG_RANKED_INTEGRITY", "enforce")])).is_err());
    assert!(RankedPolicy::from_lookup(env(&[("TBG_RANKED_INTEGRITY", "off")])).is_err());
    assert!(RankedPolicy::from_lookup(env(&[("TBG_RANKED_PAIR_CAP", "0")])).is_err());
    assert!(RankedPolicy::from_lookup(env(&[("TBG_RANKED_WINDOW_SECS", "x")])).is_err());
    let p = RankedPolicy::from_lookup(env(&[("TBG_RANKED_PAIR_CAP", "200")])).unwrap();
    assert_eq!(p.pair_cap, 200);
}
