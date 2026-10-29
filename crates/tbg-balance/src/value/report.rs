//! Human-readable tables and CSV for the value engine. Raw evidence stays
//! outside git; only small summaries are committed.

use std::fmt::Write as _;

use tbg_core::UnitKind;

use super::estimate::{Pve, ShadowPoint};
use crate::stats::Estimate;

pub struct ReferenceRow {
    pub name: &'static str,
    pub kind: Option<UnitKind>,
    /// Match-score change for the treated participant.
    pub delta: Estimate,
    /// `Some` only for one-sided candidates.
    pub pve: Option<Pve>,
    pub lambda_ref: f64,
    pub seeds: usize,
}

pub fn reference_table(rows: &[ReferenceRow], lambda_ref: f64) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "lambda_ref {lambda_ref:.6} score/Energy; 100 PVE = 1 Energy at the reference\n"
    );
    let _ = writeln!(
        s,
        "{:<32} {:>9} {:>22} {:>10} {:>22}  source",
        "candidate", "d score", "95% interval", "PVE", "95% interval"
    );
    for r in rows {
        let pve = r
            .pve
            .map(|p| format!("{:.1}", p.value))
            .unwrap_or_else(|| "-".into());
        let pve_ci = r
            .pve
            .map(|p| format!("{:.1} .. {:.1}", p.lo, p.hi))
            .unwrap_or_else(|| "-".into());
        let source = if r.kind.is_some() {
            "full-match, one-sided"
        } else {
            "full-match, global"
        };
        let _ = writeln!(
            s,
            "{:<32} {:>8.2}% [{:>7.2}% .. {:>7.2}%] {:>10} {:>22}  {}",
            r.name,
            r.delta.mean * 100.0,
            r.delta.lo * 100.0,
            r.delta.hi * 100.0,
            pve,
            pve_ci,
            source
        );
    }
    let _ = writeln!(
        s,
        "\n{} candidates, {} seed pairs each",
        rows.len(),
        rows.first().map_or(0, |r| r.seeds)
    );
    s
}

pub fn reference_csv(rows: &[ReferenceRow], lambda_ref: f64) -> String {
    let mut s = String::from(
        "candidate,kind,seed_pairs,delta,delta_lo,delta_hi,pve,pve_lo,pve_hi,lambda_ref,source\n",
    );
    for r in rows {
        let kind = r.kind.map(|k| format!("{k:?}")).unwrap_or_default();
        let (pve, lo, hi) =
            r.pve
                .map(|p| (p.value, p.lo, p.hi))
                .unwrap_or((f64::NAN, f64::NAN, f64::NAN));
        let source = if r.kind.is_some() {
            "full-match-one-sided"
        } else {
            "full-match-global"
        };
        let _ = writeln!(
            s,
            "{},{kind},{},{:.6},{:.6},{:.6},{pve:.6},{lo:.6},{hi:.6},{lambda_ref:.6},{source}",
            r.name, r.seeds, r.delta.mean, r.delta.lo, r.delta.hi
        );
    }
    s
}

pub fn shadow_table(points: &[ShadowPoint]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "{:>8}  {:>10}  {:>22}", "tick", "lambda", "95% interval");
    for p in points {
        let _ = writeln!(
            s,
            "{:>8}  {:>10.6}  [{:>8.6} .. {:>8.6}]",
            p.tick, p.lambda, p.lo, p.hi
        );
    }
    s
}

pub struct InteractionRow {
    pub name: &'static str,
    pub base: i64,
    pub a: i64,
    pub b: i64,
    pub both: i64,
}

impl InteractionRow {
    /// Native HP margin: both minus the two single changes and the base.
    pub fn interaction(&self) -> i64 {
        (self.both - self.base) - (self.a - self.base) - (self.b - self.base)
    }
}

pub fn interaction_table(rows: &[InteractionRow]) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "{:<32} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "pair", "base", "A", "B", "A+B", "interaction"
    );
    for r in rows {
        let _ = writeln!(
            s,
            "{:<32} {:>10} {:>10} {:>10} {:>10} {:>10}",
            r.name,
            r.base,
            r.a,
            r.b,
            r.both,
            r.interaction()
        );
    }
    s
}
