//! Output (P16-C step 4): a run directory with a manifest and CSV files,
//! plus a readable table on stdout. Raw outputs stay outside git.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use tbg_core::Ruleset;

use crate::duels::{Matrix, BUDGETS};
use crate::gate::{Row, Unit};
use crate::policy::Policy;

/// FNV-1a over the ruleset's debug form: names a ruleset in manifests.
pub fn fingerprint(r: &Ruleset) -> u64 {
    format!("{r:?}")
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        })
}

pub fn git_head() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or("unknown".into(), |s| s.trim().to_string())
}

pub struct Out {
    pub dir: PathBuf,
}

impl Out {
    pub fn new(dir: &Path) -> std::io::Result<Out> {
        std::fs::create_dir_all(dir)?;
        Ok(Out {
            dir: dir.to_path_buf(),
        })
    }

    pub fn write(&self, name: &str, text: &str) -> std::io::Result<()> {
        std::fs::write(self.dir.join(name), text)
    }
}

fn show(v: f64, unit: Unit) -> String {
    match unit {
        Unit::Percent => format!("{:.1}%", v * 100.0),
        Unit::Seconds => format!("{v:.0} s"),
        Unit::Count => format!("{v:.0}"),
    }
}

/// The gate as an aligned table.
pub fn table(rows: &[Row]) -> String {
    let mut s = String::new();
    for r in rows {
        let ci = r
            .interval
            .map(|(lo, hi)| format!("[{} – {}]", show(lo, r.unit), show(hi, r.unit)))
            .unwrap_or_default();
        let band = format!(
            "{} – {}",
            show(r.band.0, r.unit),
            if r.band.1.is_finite() {
                show(r.band.1, r.unit)
            } else {
                "∞".into()
            }
        );
        let _ = writeln!(
            s,
            "{:<44} {:>9} {:<22} target {:<18} {}",
            r.name,
            show(r.value, r.unit),
            ci,
            band,
            if r.pass() { "PASS" } else { "FAIL" }
        );
    }
    let failed = rows.iter().filter(|r| !r.pass()).count();
    let _ = writeln!(s, "{} rows, {failed} failing", rows.len());
    s
}

pub fn rows_csv(rows: &[Row]) -> String {
    let mut s = String::from("row,value,lo,hi,band_lo,band_hi,pass\n");
    for r in rows {
        let (lo, hi) = r.interval.unwrap_or((f64::NAN, f64::NAN));
        let _ = writeln!(
            s,
            "\"{}\",{},{},{},{},{},{}",
            r.name,
            r.value,
            lo,
            hi,
            r.band.0,
            r.band.1,
            r.pass()
        );
    }
    s
}

pub fn payoff_csv(ps: &[Policy], m: &[Vec<f64>], mix: &[f64]) -> String {
    let mut s = String::from("policy,nash_weight");
    for p in ps {
        let _ = write!(s, ",{}", p.name());
    }
    s.push('\n');
    for (i, p) in ps.iter().enumerate() {
        let _ = write!(s, "{},{:.4}", p.name(), mix[i]);
        for v in &m[i] {
            let _ = write!(s, ",{v:.4}");
        }
        s.push('\n');
    }
    s
}

pub fn duels_csv(m: &Matrix, rules: &Ruleset) -> String {
    let mut s = String::from("budget,form");
    for f in &m.forms {
        let _ = write!(s, ",{}", f.name(rules));
    }
    s.push('\n');
    for (b, budget) in BUDGETS.iter().enumerate() {
        for (i, f) in m.forms.iter().enumerate() {
            let _ = write!(s, "{budget},{}", f.name(rules));
            for v in &m.scores[b][i] {
                match v {
                    Some(v) => {
                        let _ = write!(s, ",{v:.2}");
                    }
                    None => s.push(','),
                }
            }
            s.push('\n');
        }
    }
    s
}

/// One line per flagged duel form, readable.
pub fn duel_summary(m: &Matrix, rules: &Ruleset) -> String {
    let f = m.flags(rules);
    let names = |v: &[usize]| {
        v.iter()
            .map(|&i| m.forms[i].name(rules))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut s = String::new();
    let _ = writeln!(s, "weak everywhere: {}", names(&f.weak));
    let _ = writeln!(s, "oppressive everywhere: {}", names(&f.oppressive));
    let _ = writeln!(s, "without a counter: {}", names(&f.counterless));
    let _ = writeln!(
        s,
        "Evolutions never worth their Energy: {}",
        names(&f.evolution_not_worth)
    );
    let branches: Vec<String> = f
        .branch_without_niche
        .iter()
        .map(|(k, t)| format!("{k:?} tier {t}"))
        .collect();
    let _ = writeln!(s, "branches without a niche: {}", branches.join(", "));
    let _ = writeln!(
        s,
        "average duel score per form against every form (budgets {BUDGETS:?}):"
    );
    for (i, form) in m.forms.iter().enumerate() {
        let avgs: Vec<String> = (0..BUDGETS.len())
            .map(|b| {
                m.average(b, i)
                    .map_or("  -  ".into(), |a| format!("{:>4.0}%", a * 100.0))
            })
            .collect();
        let _ = writeln!(s, "  {:<28} {}", form.name(rules), avgs.join(" "));
    }
    s
}
