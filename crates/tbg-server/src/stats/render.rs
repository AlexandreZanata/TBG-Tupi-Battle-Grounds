//! P61 §9 internal Markdown report: exact counts, all exclusions, metric
//! definitions and gate status. No account identities, even internally.
//! Descriptive associations only; candidates need D-33 lab evidence.

use std::fmt::Write;

use super::aggregate::{Exposure, Report, Tally};
use super::{wilson, Coverage};

pub struct Header<'a> {
    pub revision: u32,
    pub source_hash: &'a str,
    pub compare: Option<(u32, &'a Report)>,
}

fn pct(v: Option<f64>) -> String {
    v.map_or_else(|| "—".into(), |x| format!("{:.1}%", x * 100.0))
}

fn interval(t: &Tally) -> String {
    wilson(t.w, t.n).map_or_else(
        || "—".into(),
        |(lo, hi)| format!("{:.1}–{:.1}%", lo * 100.0, hi * 100.0),
    )
}

fn gate(t: &Tally) -> &'static str {
    if t.passes_gate() {
        "pass"
    } else {
        "insufficient"
    }
}

fn table(out: &mut String, title: &str, rows: &[Exposure], sides: u64, amount: &str) {
    let _ = writeln!(out, "\n## {title}\n");
    let _ = writeln!(
        out,
        "| Exposure | N | presence | W/D/L | WR (Wilson 95%) | draw | score | Δ vs rest (pp) | residual | {amount} | accounts / pairs / max share | gate (X / rest) | extra |"
    );
    let _ = writeln!(
        out,
        "|---|---:|---:|---|---|---:|---:|---:|---:|---:|---|---|---|"
    );
    for e in rows {
        let t = &e.yes;
        let presence = (sides > 0).then(|| t.n as f64 / sides as f64);
        let extra: Vec<String> = e.extra.iter().map(|(k, v)| format!("{k}={v:.3}")).collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {}/{}/{} | {} ({}) | {} | {} | {} | {} | {} | {} / {} / {}% | {} / {} | {} |",
            e.label,
            t.n,
            pct(presence),
            t.w,
            t.d,
            t.l,
            pct(t.win_rate()),
            interval(t),
            pct(t.draw_rate()),
            pct(t.score_rate()),
            e.delta_pp().map_or_else(|| "—".into(), |d| format!("{d:+.1}")),
            t.residual().map_or_else(|| "—".into(), |r| format!("{r:+.3}")),
            e.amount,
            t.distinct_accounts(),
            t.distinct_pairs(),
            t.max_share_pct(),
            gate(t),
            gate(&e.no),
            extra.join(", "),
        );
    }
}

pub fn markdown(h: &Header, cov: &Coverage, r: &Report) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# Ranked statistics, revision {} (internal)\n",
        h.revision
    );
    let _ = writeln!(
        out,
        "Source hash `{}`. Internal descriptive report (P61 E1): association, not causal \
         strength or skill. Draws are non-wins in WR; score counts a draw as 0.5. Wilson \
         intervals are descriptive under clustering (repeated accounts and pairs). Residual = \
         mean(score − expected) from settlement-time ratings. Gate = P61 §7 publication \
         floor (≥50 sides, ≥5 accounts, ≥3 pairs, max account share ≤40%), applied to the \
         exposure and its complement separately. Not served publicly.",
        h.source_hash
    );
    let _ = writeln!(out, "\n## Coverage\n");
    let _ = writeln!(
        out,
        "| stored | eligible | invalidated | review | no change | not started | missing summary | unreadable summary |\n|---:|---:|---:|---:|---:|---:|---:|---:|\n| {} | {} | {} | {} | {} | {} | {} | {} |",
        cov.stored, cov.eligible, cov.invalidated, cov.review, cov.no_change, cov.not_started,
        cov.missing_summary, cov.unreadable_summary
    );
    let o = &r.overall;
    let _ = writeln!(
        out,
        "\nM = {} matches, S = {} sides. All sides: WR {} ({}), draw {}, score {}. Seat A: WR {}, score {} \
         (seat A is the first ranked waiter: position is confounded with queue behavior).",
        r.matches, r.sides, pct(o.win_rate()), interval(o), pct(o.draw_rate()), pct(o.score_rate()),
        pct(r.seat_a.win_rate()), pct(r.seat_a.score_rate()),
    );
    table(
        &mut out,
        "Units: at least one purchase (all matches)",
        &r.units,
        r.sides,
        "copies",
    );
    let _ = writeln!(
        out,
        "\nLandmark cohort: {} matches reached 60 combat seconds; {} ended earlier and are excluded. \
         Exposure = first purchase by 60 s; conditions on survival to the landmark.",
        r.landmark_matches, r.landmark_excluded
    );
    table(
        &mut out,
        "Units: bought by 60 s (landmark)",
        &r.landmark_units,
        2 * r.landmark_matches,
        "—",
    );
    table(
        &mut out,
        "Evolution: buying sides that reached tier ≥ 1",
        &r.evolution,
        r.sides,
        "copies bought",
    );
    table(
        &mut out,
        "Factions: bought ≥ 1 unit",
        &r.factions,
        r.sides,
        "copies",
    );
    table(
        &mut out,
        "Archetypes: bought ≥ 1 unit",
        &r.archetypes,
        r.sides,
        "copies",
    );
    table(
        &mut out,
        "Synergy: reached level ≥ 1",
        &r.synergy,
        r.sides,
        "level-1 ticks",
    );
    table(
        &mut out,
        "Augments: applied ≥ 1 copy",
        &r.augments,
        r.sides,
        "chosen",
    );
    table(&mut out, "Arcana nodes: rank > 0", &r.arcana, r.sides, "—");
    table(
        &mut out,
        "Strongholds (exclusive per side)",
        &r.strongholds,
        r.sides,
        "—",
    );
    let _ = writeln!(out, "\n## Arenas (assigned, not picked; seat A rates)\n");
    let _ = writeln!(out, "| Arena | matches | frequency | seat A WR | seat A score | forfeit rate | mean wall s |\n|---|---:|---:|---:|---:|---:|---:|");
    for a in &r.arenas {
        let m = a.matches.max(1) as f64;
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            a.label,
            a.matches,
            pct((r.matches > 0).then(|| a.matches as f64 / r.matches as f64)),
            pct(a.seat_a.win_rate()),
            pct(a.seat_a.score_rate()),
            pct((a.matches > 0).then(|| a.forfeits as f64 / m)),
            if a.matches > 0 {
                format!("{:.1}", a.wall_ms_sum as f64 / m / 1000.0)
            } else {
                "—".into()
            },
        );
    }
    let g = &r.general;
    let m = r.matches.max(1) as f64;
    let _ = writeln!(
        out,
        "\n## General\n\nMean wall {:.1} s, mean combat {:.1} combat-ticks; Surrender {}, disconnect forfeit {}, draws {} (of {} matches). No gameplay timeout exists. Matches per UTC day: {}.",
        g.wall_ms_sum as f64 / m / 1000.0,
        g.combat_ticks_sum as f64 / m,
        g.surrenders,
        g.disconnects,
        g.draws,
        r.matches,
        g.per_day.values().map(u64::to_string).collect::<Vec<_>>().join(", "),
    );
    compare(&mut out, h, r);
    out
}

/// Previous-revision delta: same definitions, both rows gated, label as
/// the stable comparison identity; anything else is absent, not zero.
fn compare(out: &mut String, h: &Header, r: &Report) {
    let Some((rev, prev)) = h.compare else {
        return;
    };
    let _ = writeln!(out, "\n## Units vs revision {rev} (descriptive)\n");
    if prev.matches == 0 {
        let _ = writeln!(
            out,
            "Revision {rev} has no eligible data: comparison absent."
        );
        return;
    }
    let _ = writeln!(
        out,
        "| Unit | WR now | WR then | Δ (pp) |\n|---|---:|---:|---:|"
    );
    for e in &r.units {
        let then = prev.units.iter().find(|p| p.label == e.label);
        let delta = then
            .filter(|p| p.yes.passes_gate() && e.yes.passes_gate())
            .and_then(|p| Some((e.yes.win_rate()? - p.yes.win_rate()?) * 100.0));
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            e.label,
            pct(e.yes.win_rate()),
            pct(then.and_then(|p| p.yes.win_rate())),
            delta.map_or_else(|| "absent".into(), |d| format!("{d:+.1}")),
        );
    }
}
