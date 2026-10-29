//! P40 audit: dose-response of one conditional Arcana value against the
//! default build, with a null control. Run:
//!   cargo run --release -p tbg-balance --example arcana_dose -- <pairs> <threads>
//! The same paired, seat-swapped seeds are used for every row, so rows are
//! comparable; "null" (default vs default) is the lab's noise floor.

use tbg_balance::arcanas;
use tbg_balance::policy::Policy;
use tbg_core::{ArcanaPreset, Ruleset};

fn leak(r: Ruleset) -> &'static Ruleset {
    Box::leak(Box::new(r))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let pairs: u64 = args.next().and_then(|v| v.parse().ok()).unwrap_or(2048);
    let threads: usize = args.next().and_then(|v| v.parse().ok()).unwrap_or(8);
    let precision = ArcanaPreset {
        ranks: [2, 2, 2, 2, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
    };
    let precision_no_deadeye = ArcanaPreset {
        ranks: [2, 2, 2, 2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0],
    };
    let mut rows: Vec<(String, &'static Ruleset, ArcanaPreset)> = Vec::new();
    rows.push((
        "null: default vs default".into(),
        Ruleset::canon(),
        ArcanaPreset::DEFAULT,
    ));
    rows.push((
        "precision without Deadeye (10 pts)".into(),
        Ruleset::canon(),
        precision_no_deadeye,
    ));
    for range in [0, 100, 250, 500] {
        let mut r = Ruleset::canon().clone();
        r.arcana.deadeye_range = range;
        rows.push((
            format!("precision, Deadeye +{range} milli-tiles"),
            leak(r),
            precision,
        ));
    }
    println!("{pairs} seed pairs per row, policy bot, seeds 0..{pairs}");
    for (name, rules, preset) in rows {
        let est = arcanas::against(
            rules,
            &[preset],
            ArcanaPreset::DEFAULT,
            Policy::Bot,
            0..pairs,
            tbg_balance::gate::REACTION,
            threads,
        );
        let e = est[0].1;
        println!(
            "  {name:<40} {:>5.1}% [{:.1} - {:.1}]",
            e.mean * 100.0,
            e.lo * 100.0,
            e.hi * 100.0
        );
    }
}
