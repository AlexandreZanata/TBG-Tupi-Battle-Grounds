//! Measure the one complete duel row C7 reads; every opponent, budget and seat order is retained.
use tbg_balance::duels::{self, Form, Matrix};
use tbg_core::Ruleset;

/// Other rows are deliberately unmeasured. Only query the target row or its weak/oppressive flag.
pub fn measure(rules: &Ruleset, forms: &[Form], target: usize) -> Matrix {
    assert!(target < forms.len() && forms.len() > 1);
    let mut scores = vec![vec![vec![None; forms.len()]; forms.len()]; duels::BUDGETS.len()];
    for (other, form) in forms.iter().enumerate().filter(|(i, _)| *i != target) {
        // Reuse the reference builder's exact two-seat normalization and fieldability rules.
        let pair = duels::matrix_of(rules, vec![forms[target], *form], 4);
        for (budget, row) in scores.iter_mut().enumerate() {
            row[target][target] = pair.scores[budget][0][0];
            row[target][other] = pair.scores[budget][0][1];
            row[other][target] = pair.scores[budget][1][0];
        }
    }
    Matrix {
        forms: forms.to_vec(),
        scores,
    }
}
