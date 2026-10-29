//! The meta-game (P16-C step 7): a payoff matrix of policy against policy
//! and its symmetric Nash equilibrium by fictitious play. A healthy game
//! keeps several styles in the equilibrium and none above half of it.

/// `payoff[i][j]`: score of policy `i` against `j` (0..1; `[i][i]` = 0.5).
/// Returns the equilibrium mix after `iterations` best responses
/// (deterministic: ties go to the lowest index).
pub fn nash(payoff: &[Vec<f64>], iterations: u32) -> Vec<f64> {
    let n = payoff.len();
    if n == 0 {
        return Vec::new();
    }
    let mut counts = vec![0u64; n];
    // Running payoff of each strategy against the empirical mix.
    let mut value = vec![0.0; n];
    let mut current = 0;
    for _ in 0..iterations {
        counts[current] += 1;
        for (i, v) in value.iter_mut().enumerate() {
            *v += payoff[i][current];
        }
        current = (0..n)
            .max_by(|&a, &b| value[a].total_cmp(&value[b]).then(b.cmp(&a)))
            .unwrap_or(0);
    }
    let total = counts.iter().sum::<u64>().max(1) as f64;
    counts.iter().map(|&c| c as f64 / total).collect()
}

/// Each policy's expected score against the mix.
pub fn against(payoff: &[Vec<f64>], mix: &[f64]) -> Vec<f64> {
    payoff
        .iter()
        .map(|row| row.iter().zip(mix).map(|(p, w)| p * w).sum())
        .collect()
}

/// Policies holding at least 1 % of the mix.
pub fn support(mix: &[f64]) -> usize {
    mix.iter().filter(|&&w| w >= 0.01).count()
}
