//! Wiring commands to the lab: build the ruleset and context, run, print,
//! and write the run directory with its manifest.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Instant;

use tbg_core::content::TICK_HZ;
use tbg_core::Ruleset;

use crate::cli::Args;
use crate::duels;
use crate::gate::{self, Scope};
use crate::knobs;
use crate::meta;
use crate::policy::Policy;
use crate::report::{self, Out};
use crate::runner::{run_pairs, Matchup};
use crate::search::{self, Budget, Settings, CONFIRMATION, DISCOVERY, SCREENING};
use crate::stats;

struct Ctx {
    rules: &'static Ruleset,
    applied: String,
    budget: Budget,
    pairs: u64,
    seed_start: u64,
    out: Out,
    started: Instant,
}

fn ctx(a: &Args, default_pairs: u64, default_seed: u64) -> Result<Ctx, String> {
    let mut rules = Ruleset::canon().clone();
    let applied = a.str("apply").unwrap_or("").to_string();
    knobs::apply(&mut rules, &applied)?;
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get()) as u64;
    let dir = a
        .str("out")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/balance").join(&a.command));
    Ok(Ctx {
        rules: Box::leak(Box::new(rules)),
        applied,
        budget: Budget {
            threads: a.num("threads", cores)? as usize,
            reaction: a.num("reaction", u64::from(gate::REACTION))? as u32,
        },
        pairs: a.num("pairs", default_pairs)?,
        seed_start: a.num("seed-start", default_seed)?,
        out: Out::new(&dir).map_err(|e| format!("{}: {e}", dir.display()))?,
        started: Instant::now(),
    })
}

impl Ctx {
    fn seeds(&self) -> std::ops::Range<u64> {
        self.seed_start..self.seed_start + self.pairs
    }

    /// Write the manifest; `games` and `game_ticks` give the speed-up.
    fn finish(&self, games: u64, game_ticks: u64) -> Result<(), String> {
        let wall = self.started.elapsed().as_secs_f64();
        let mut m = String::new();
        let _ = writeln!(
            m,
            "command: {}",
            std::env::args().collect::<Vec<_>>().join(" ")
        );
        let _ = writeln!(m, "git_head: {}", report::git_head());
        let _ = writeln!(
            m,
            "ruleset_fingerprint: {:016x}",
            report::fingerprint(self.rules)
        );
        let _ = writeln!(
            m,
            "applied: {}",
            if self.applied.is_empty() {
                "canon"
            } else {
                &self.applied
            }
        );
        let _ = writeln!(
            m,
            "seeds: {}..{} ({} pairs)",
            self.seed_start,
            self.seed_start + self.pairs,
            self.pairs
        );
        let _ = writeln!(m, "threads: {}", self.budget.threads);
        let _ = writeln!(m, "reaction_views: {}", self.budget.reaction);
        let _ = writeln!(m, "wall_s: {wall:.2}");
        if games > 0 {
            let game_s = game_ticks as f64 / f64::from(TICK_HZ);
            let _ = writeln!(m, "games: {games}");
            let _ = writeln!(m, "games_per_s: {:.0}", games as f64 / wall);
            let _ = writeln!(m, "game_seconds_per_wall_second: {:.0}", game_s / wall);
        }
        self.out
            .write("manifest.txt", &m)
            .map_err(|e| e.to_string())?;
        eprintln!("{m}output: {}", self.out.dir.display());
        Ok(())
    }
}

fn games_and_ticks(results: &gate::Results) -> (u64, u64) {
    let games = results.values().map(|v| v.len() as u64 * 2).sum();
    let ticks = results
        .values()
        .flat_map(|v| v.iter().flat_map(|p| p.iter().map(|r| u64::from(r.ticks))))
        .sum();
    (games, ticks)
}

pub fn dispatch(a: &Args) -> Result<(), String> {
    match a.command.as_str() {
        "run" => run(a),
        "duels" => duels_cmd(a),
        "meta" => meta_cmd(a),
        "gate" => gate_cmd(a),
        "throughput" => throughput(a),
        "screen" => screen(a),
        "search" => search_cmd(a),
        "confirm" => confirm(a),
        other => Err(format!("unknown command `{other}` (see --help)")),
    }
}

fn policy(a: &Args, flag: &str) -> Result<Policy, String> {
    let name = a
        .str(flag)
        .ok_or(format!("`--{flag} POLICY` is required"))?;
    Policy::parse(name).ok_or(format!("unknown policy `{name}`"))
}

fn run(a: &Args) -> Result<(), String> {
    let c = ctx(a, 2_000, DISCOVERY)?;
    let pair = (policy(a, "a")?, policy(a, "b")?);
    let m = Matchup {
        rules: c.rules,
        policies: [pair.0, pair.1],
        reaction: [c.budget.reaction; 2],
    };
    let results: gate::Results =
        [(pair, run_pairs(&[m], c.seeds(), c.budget.threads).remove(0))].into();
    let s = gate::scores(&results, pair.0, pair.1).unwrap_or_default();
    let e = stats::mean(&s, stats::RESAMPLES, 99);
    let mut text = format!(
        "{} vs {}: {:.1}% [{:.1} – {:.1}] over {} seed pairs\n",
        pair.0.name(),
        pair.1.name(),
        e.mean * 100.0,
        e.lo * 100.0,
        e.hi * 100.0,
        e.n
    );
    text.push_str(&report::table(&gate::rows(
        &results,
        None,
        stats::RESAMPLES,
    )));
    print!("{text}");
    c.out.write("run.txt", &text).map_err(|e| e.to_string())?;
    let (g, t) = games_and_ticks(&results);
    c.finish(g, t)
}

fn duels_cmd(a: &Args) -> Result<(), String> {
    let c = ctx(a, 1, DISCOVERY)?;
    let m = duels::matrix(c.rules, c.budget.threads);
    let summary = report::duel_summary(&m, c.rules);
    print!("{summary}");
    c.out
        .write("duels.txt", &summary)
        .map_err(|e| e.to_string())?;
    c.out
        .write("duels.csv", &report::duels_csv(&m, c.rules))
        .map_err(|e| e.to_string())?;
    c.finish(0, 0)
}

fn meta_text(results: &gate::Results) -> (String, String) {
    let (ps, m) = gate::payoff(results);
    let mix = meta::nash(&m, gate::NASH_ITERATIONS);
    let vs = meta::against(&m, &mix);
    let mut s = String::from("policy                    nash  vs-mix\n");
    let mut order: Vec<usize> = (0..ps.len()).collect();
    order.sort_by(|&x, &y| mix[y].total_cmp(&mix[x]).then(x.cmp(&y)));
    for i in order {
        let _ = writeln!(
            s,
            "{:<24} {:>5.1}% {:>6.1}%",
            ps[i].name(),
            mix[i] * 100.0,
            vs[i] * 100.0
        );
    }
    (s, report::payoff_csv(&ps, &m, &mix))
}

fn meta_cmd(a: &Args) -> Result<(), String> {
    let c = ctx(a, 256, DISCOVERY)?;
    let results = gate::run(
        c.rules,
        Scope::Full,
        c.seeds(),
        c.budget.reaction,
        c.budget.threads,
    );
    let (text, csv) = meta_text(&results);
    print!("{text}");
    c.out.write("meta.txt", &text).map_err(|e| e.to_string())?;
    c.out.write("payoff.csv", &csv).map_err(|e| e.to_string())?;
    let (g, t) = games_and_ticks(&results);
    c.finish(g, t)
}

/// Rows a release must pass (GAME-DESIGN §6 "Balance lab", required).
pub fn required(name: &str) -> bool {
    name.starts_with("seat A")
        || name.starts_with("heavy vs")
        || name.starts_with("cheap vs")
        || name.starts_with("duel base forms oppressive")
        || name.starts_with("Nash")
}

fn gate_cmd(a: &Args) -> Result<(), String> {
    let c = ctx(a, 1_000, CONFIRMATION)?;
    let results = gate::run(
        c.rules,
        Scope::Full,
        c.seeds(),
        c.budget.reaction,
        c.budget.threads,
    );
    let matrix = duels::matrix(c.rules, c.budget.threads);
    let rows = gate::rows(&results, Some((&matrix, c.rules)), stats::RESAMPLES);
    let table = report::table(&rows);
    let (meta_txt, payoff) = meta_text(&results);
    print!("{table}\n{meta_txt}");
    let w = |n: &str, s: &str| c.out.write(n, s).map_err(|e| e.to_string());
    w("gate.txt", &format!("{table}\n{meta_txt}"))?;
    w("gate.csv", &report::rows_csv(&rows))?;
    w("payoff.csv", &payoff)?;
    w("duels.txt", &report::duel_summary(&matrix, c.rules))?;
    w("duels.csv", &report::duels_csv(&matrix, c.rules))?;
    let (g, t) = games_and_ticks(&results);
    c.finish(g, t)?;
    let failing: Vec<&str> = rows
        .iter()
        .filter(|r| required(&r.name) && !r.pass())
        .map(|r| r.name.as_str())
        .collect();
    if a.flag("strict") && !failing.is_empty() {
        return Err(format!("required rows failing: {}", failing.join("; ")));
    }
    Ok(())
}

fn throughput(a: &Args) -> Result<(), String> {
    let c = ctx(a, 1_000, DISCOVERY)?;
    let m = Matchup {
        rules: c.rules,
        policies: [Policy::Bot; 2],
        reaction: [c.budget.reaction; 2],
    };
    let mut text =
        String::from("threads  games/s  speed-up vs 1x (game-seconds per wall-second)\n");
    let mut threads = 1;
    let mut reference = None;
    while threads <= c.budget.threads {
        let t0 = Instant::now();
        let r = run_pairs(&[m], c.seeds(), threads).remove(0);
        let wall = t0.elapsed().as_secs_f64();
        let games = r.len() as f64 * 2.0;
        let ticks: u64 = r
            .iter()
            .flat_map(|p| p.iter().map(|g| u64::from(g.ticks)))
            .sum();
        let speed = ticks as f64 / f64::from(TICK_HZ) / wall;
        // Every thread count must reproduce the same results (C3).
        if *reference.get_or_insert_with(|| r.clone()) != r {
            return Err("results changed with the thread count".into());
        }
        let _ = writeln!(text, "{threads:>7}  {:>7.0}  {speed:>10.0}x", games / wall);
        threads *= 2;
    }
    print!("{text}");
    c.out
        .write("throughput.txt", &text)
        .map_err(|e| e.to_string())?;
    c.finish(0, 0)
}

fn screen(a: &Args) -> Result<(), String> {
    let c = ctx(a, 256, SCREENING)?;
    let mut log = String::new();
    let (base, list) = search::screen(c.rules, c.pairs, c.budget, |line| {
        eprintln!("{line}");
        let _ = writeln!(log, "{line}");
    });
    let mut text = format!("canon loss {base:.1}\nknob,steps,loss\n");
    for s in &list {
        let _ = writeln!(
            text,
            "{},{:+},{:.1}",
            knobs::name(c.rules, s.knob),
            s.steps,
            s.loss
        );
    }
    c.out
        .write("screen.csv", &text)
        .map_err(|e| e.to_string())?;
    c.finish(0, 0)
}

fn search_cmd(a: &Args) -> Result<(), String> {
    let c = ctx(a, 1_024, DISCOVERY)?;
    let settings = Settings {
        generations: a.num("generations", 30)? as u32,
        lambda: a.num("lambda", 16)? as usize,
        pairs: c.pairs,
        seed: a.num("search-seed", 1)?,
    };
    let mut log = String::new();
    let mut say = |line: &str| {
        eprintln!("{line}");
        let _ = writeln!(log, "{line}");
    };
    let (_, screened) = search::screen(c.rules, (c.pairs / 4).max(32), c.budget, &mut say);
    let top = search::ranked(&screened, a.num("top", 24)? as usize);
    let found = search::search(c.rules, &top, &settings, c.budget, &mut say);
    let spec = search::apply_spec(c.rules, &found.changes);
    let canon_rows = search::evaluate(c.rules, DISCOVERY..DISCOVERY + c.pairs, c.budget);
    let mut md = format!(
        "# Balance proposal\n\nloss {:.1} (canon {:.1}), penalty {:.1}\n\n`--apply {spec}`\n\n| Knob | Canon | Proposed |\n|---|---|---|\n",
        found.loss,
        gate::loss(&canon_rows),
        found.penalty
    );
    let proposed = search::build(c.rules, &found.changes);
    for &k in found.changes.keys() {
        let _ = writeln!(
            md,
            "| {} | {} | {} |",
            knobs::name(c.rules, k),
            knobs::get(c.rules, k),
            knobs::get(&proposed, k)
        );
    }
    let _ = writeln!(
        md,
        "\n## Discovery rows: canon\n\n```\n{}```\n\n## Discovery rows: proposal\n\n```\n{}```",
        report::table(&canon_rows),
        report::table(&found.rows)
    );
    println!("{md}");
    c.out.write("proposal.md", &md).map_err(|e| e.to_string())?;
    c.out.write("search.log", &log).map_err(|e| e.to_string())?;
    c.finish(0, 0)
}

fn confirm(a: &Args) -> Result<(), String> {
    let c = ctx(a, 4_096, CONFIRMATION)?;
    if c.applied.is_empty() {
        return Err("`confirm` needs `--apply SPEC`".into());
    }
    let canon = Ruleset::canon();
    let mut text = String::new();
    for (label, rules) in [("canon", canon), ("proposal", c.rules)] {
        let full = gate::run(
            rules,
            Scope::Full,
            c.seeds(),
            c.budget.reaction,
            c.budget.threads,
        );
        let matrix = duels::matrix(rules, c.budget.threads);
        let rows = gate::rows(&full, Some((&matrix, rules)), stats::RESAMPLES);
        let _ = writeln!(
            text,
            "== {label}, reaction {} ==\n{}",
            c.budget.reaction,
            report::table(&rows)
        );
        let (meta_txt, _) = meta_text(&full);
        let _ = writeln!(text, "{meta_txt}");
        for reaction in [1, 10] {
            let suite = gate::run(rules, Scope::Suite, c.seeds(), reaction, c.budget.threads);
            let rows: Vec<_> = gate::rows(&suite, None, stats::RESAMPLES)
                .into_iter()
                .filter(|r| r.name.contains(" vs bot"))
                .collect();
            let _ = writeln!(
                text,
                "-- {label}, reaction {reaction} --\n{}",
                report::table(&rows)
            );
        }
        eprintln!("{label} done");
    }
    print!("{text}");
    c.out
        .write("confirm.txt", &text)
        .map_err(|e| e.to_string())?;
    c.finish(0, 0)
}
