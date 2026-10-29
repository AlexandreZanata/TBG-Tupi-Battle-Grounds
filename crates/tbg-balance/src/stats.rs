//! Uncertainty (P16-C step 7): 95 % intervals by deterministic bootstrap
//! over seed pairs. A pair's two seat-swapped games share a seed, so they
//! are one cluster; never treat them as independent Bernoulli trials.

use tbg_core::Pcg32;

pub const RESAMPLES: u32 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Estimate {
    pub mean: f64,
    pub lo: f64,
    pub hi: f64,
    /// Clusters with a non-zero denominator.
    pub n: usize,
}

impl Estimate {
    pub fn half_width(&self) -> f64 {
        (self.hi - self.lo) / 2.0
    }
}

/// Ratio estimate Σnum / Σden over clusters `(num, den)`, with a percentile
/// bootstrap interval from a fixed PCG32 stream (same input, same output).
pub fn ratio(clusters: &[(f64, f64)], resamples: u32, stream: u64) -> Estimate {
    let n = clusters.iter().filter(|c| c.1 > 0.0).count();
    let total = |pick: &mut dyn FnMut() -> usize| {
        let (mut num, mut den) = (0.0, 0.0);
        for _ in 0..clusters.len() {
            let (a, b) = clusters[pick()];
            num += a;
            den += b;
        }
        if den > 0.0 {
            num / den
        } else {
            f64::NAN
        }
    };
    let mut i = 0;
    let mean = total(&mut || {
        i += 1;
        i - 1
    });
    if clusters.is_empty() || resamples == 0 {
        return Estimate {
            mean,
            lo: mean,
            hi: mean,
            n,
        };
    }
    let mut rng = Pcg32::new(0x5eed_ba1a_9ce0, stream);
    let len = clusters.len() as u32;
    let mut draws: Vec<f64> = (0..resamples)
        .map(|_| total(&mut || rng.below(len) as usize))
        .filter(|v| v.is_finite())
        .collect();
    draws.sort_by(f64::total_cmp);
    let at = |q: f64| {
        draws
            .get(((draws.len().max(1) - 1) as f64 * q).round() as usize)
            .copied()
    };
    Estimate {
        mean,
        lo: at(0.025).unwrap_or(mean),
        hi: at(0.975).unwrap_or(mean),
        n,
    }
}

/// Mean of per-cluster values.
pub fn mean(values: &[f64], resamples: u32, stream: u64) -> Estimate {
    let clusters: Vec<(f64, f64)> = values.iter().map(|&v| (v, 1.0)).collect();
    ratio(&clusters, resamples, stream)
}
