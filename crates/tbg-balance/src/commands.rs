//! Wiring commands to the lab: build the ruleset and context, run, print,
//! and write the run directory with its manifest.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Instant;

use tbg_core::content::TICK_HZ;
use tbg_core::ArcanaPreset;
use tbg_core::Ruleset;

use crate::arcanas;
use crate::cli::Args;
use crate::duels;
use crate::gate::{self, Scope};
use crate::knobs;
use crate::meta;
use crate::policy::Policy;
use crate::report::{self, Out};
use crate::runner::{run_pairs, Matchup, Setup};
use crate::search::{self, Budget, Settings, CONFIRMATION, DISCOVERY, SCREENING};
use crate::stats;
use crate::value::{self, estimate, report as value_report};

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
    fn finish(&self, games: u64, game_ticks: u64, note: &str) -> Result<(), String> {
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
        if !note.is_empty() {
            let _ = writeln!(m, "{note}");
        }
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
        "value" => value_cmd(a),
        "validate" => validate_cmd(a),
        "arcana" => arcana_cmd(a),
        "arcana-named" => arcana_named_cmd(a),
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
        setup: Setup::None,
        arcanas: [ArcanaPreset::NEUTRAL; 2],
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
    c.finish(g, t, "")
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
    c.finish(0, 0, "")
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
    c.finish(g, t, "")
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
    c.finish(g, t, "")?;
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
        setup: Setup::None,
        arcanas: [ArcanaPreset::NEUTRAL; 2],
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
    c.finish(0, 0, "")
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
    let note = format!(
        "screening_block: {SCREENING}..{} (fixed)",
        SCREENING + c.pairs
    );
    c.finish(0, 0, &note)
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
    let screened_pairs = (c.pairs / 4).max(32);
    let note = format!(
        "screening_block: {SCREENING}..{} (fixed)\ndiscovery_block: {DISCOVERY}..{} (fixed)",
        SCREENING + screened_pairs,
        DISCOVERY + c.pairs
    );
    c.finish(0, 0, &note)
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
    c.finish(0, 0, "")
}

fn shadow_points(
    c: &Ctx,
    seeds: std::ops::Range<u64>,
) -> Result<Vec<estimate::ShadowPoint>, String> {
    estimate::shadow_with(
        c.rules,
        &[0, 600, 1_200, 1_800],
        c.budget.reaction,
        seeds,
        c.budget.threads,
    )
}

fn reference_rows(
    c: &Ctx,
    lambda_ref: f64,
    seeds: std::ops::Range<u64>,
) -> Result<Vec<value_report::ReferenceRow>, String> {
    let mut rows = Vec::new();
    for candidate in value::catalog() {
        let cmp = estimate::compare_with(
            c.rules,
            candidate.intervention,
            candidate.policies(),
            c.budget.reaction,
            seeds.clone(),
            c.budget.threads,
        )?;
        let pve = candidate
            .kind
            .map(|_| estimate::pve(&cmp.delta, lambda_ref));
        rows.push(value_report::ReferenceRow {
            name: candidate.name,
            kind: candidate.kind,
            delta: cmp.delta,
            pve,
            lambda_ref,
            seeds: cmp.seed_pairs,
        });
    }
    Ok(rows)
}

fn value_cmd(a: &Args) -> Result<(), String> {
    let c = ctx(a, 128, DISCOVERY)?;
    let mode = a.str("mode").unwrap_or("reference");
    match mode {
        "shadow" => {
            let points = shadow_points(&c, c.seeds())?;
            let text = value_report::shadow_table(&points);
            print!("{text}");
            c.out
                .write("shadow.txt", &text)
                .map_err(|e| e.to_string())?;
            c.finish(0, 0, "shadow price of one Energy")
        }
        "interactions" => {
            let rows = interaction_rows(&c)?;
            let text = value_report::interaction_table(&rows);
            print!("{text}");
            c.out
                .write("interactions.txt", &text)
                .map_err(|e| e.to_string())?;
            c.finish(0, 0, "deterministic micro-scenario interactions")
        }
        "scenarios" => {
            let mut text = String::new();
            for pair in value::pairs() {
                let r = pair.scenario.run(c.rules);
                let _ = writeln!(
                    text,
                    "{}: ticks {} margin {} unit-damage A {} B {} healing A {} B {}",
                    r.name,
                    r.ticks,
                    r.margin,
                    r.metrics[0].unit_damage,
                    r.metrics[1].unit_damage,
                    r.metrics[0].healing,
                    r.metrics[1].healing
                );
            }
            print!("{text}");
            c.out
                .write("scenarios.txt", &text)
                .map_err(|e| e.to_string())?;
            c.finish(0, 0, "deterministic micro-scenarios")
        }
        "reference" => {
            let points = shadow_points(&c, c.seeds())?;
            let lambda_ref = estimate::reference_lambda(&points)?;
            let rows = reference_rows(&c, lambda_ref, c.seeds())?;
            let text = value_report::reference_table(&rows, lambda_ref);
            let csv = value_report::reference_csv(&rows, lambda_ref);
            print!("{text}");
            c.out
                .write("values.txt", &text)
                .map_err(|e| e.to_string())?;
            c.out.write("values.csv", &csv).map_err(|e| e.to_string())?;
            let games = (rows.len() as u64) * c.pairs * 4;
            c.finish(games, 0, "full-match PVE, paired seeds")
        }
        other => Err(format!(
            "unknown value mode `{other}` (reference, shadow, interactions, scenarios)"
        )),
    }
}

fn interaction_rows(c: &Ctx) -> Result<Vec<value_report::InteractionRow>, String> {
    let mut rows = Vec::new();
    for pair in value::pairs() {
        let base = pair.scenario.run(c.rules);
        let a_world = pair.a.rules_from(c.rules)?.unwrap_or(c.rules);
        let b_world = pair.b.rules_from(c.rules)?.unwrap_or(c.rules);
        let both_world = pair.b.rules_from(a_world)?.unwrap_or(a_world);
        let a = pair.scenario.run(a_world);
        let b = pair.scenario.run(b_world);
        let both = pair.scenario.run(both_world);
        rows.push(value_report::InteractionRow {
            name: pair.name,
            base: base.margin,
            a: a.margin,
            b: b.margin,
            both: both.margin,
        });
    }
    Ok(rows)
}

fn validate_cmd(a: &Args) -> Result<(), String> {
    let c = ctx(a, 128, DISCOVERY)?;
    let points = shadow_points(&c, c.seeds())?;
    let lambda_ref = estimate::reference_lambda(&points)?;
    let discovery = reference_rows(&c, lambda_ref, c.seeds())?;
    let confirmation = reference_rows(&c, lambda_ref, CONFIRMATION..CONFIRMATION + c.pairs)?;
    let mut text = String::from(
        "candidate                          discovery PVE   confirmation PVE   static power\n",
    );
    let mut d = Vec::new();
    let mut f = Vec::new();
    let mut s = Vec::new();
    for (a, b) in discovery.iter().zip(&confirmation) {
        if let (Some(pa), Some(pb)) = (a.pve, b.pve) {
            let sp = value::validate::static_power(
                &value::catalog()
                    .into_iter()
                    .find(|c| c.name == a.name)
                    .ok_or("candidate missing")?,
            );
            d.push(pa.value);
            f.push(pb.value);
            s.push(sp);
            let _ = writeln!(
                text,
                "{:<32} {:>12.1} {:>16.1} {:>14.3}",
                a.name, pa.value, pb.value, sp
            );
        }
    }
    let stability = estimate::spearman(&d, &f);
    let baseline = estimate::spearman(&f, &s);
    let _ = writeln!(text, "\ndiscovery vs confirmation rank: {stability:?}");
    let _ = writeln!(text, "confirmation vs static baseline rank: {baseline:?}");
    print!("{text}");
    c.out
        .write("validation.txt", &text)
        .map_err(|e| e.to_string())?;
    c.finish(0, 0, "P16-D validation")
}

/// Confirmation block for the named builds and the retired six-point default
/// against the twelve-point default, with a larger paired sample (P40).
fn arcana_named_cmd(a: &Args) -> Result<(), String> {
    let c = ctx(a, 512, CONFIRMATION)?;
    let policy = match a.str("a") {
        Some(_) => policy(a, "a")?,
        None => Policy::Bot,
    };
    let mut presets: Vec<(&str, ArcanaPreset)> = arcanas::NAMED
        .iter()
        .map(|(name, ranks)| (*name, ArcanaPreset { ranks: *ranks }))
        .collect();
    presets.push((
        "legacy-6pt",
        ArcanaPreset {
            ranks: arcanas::LEGACY_DEFAULT,
        },
    ));
    let ran: Vec<ArcanaPreset> = presets.iter().map(|(_, p)| *p).collect();
    let rows = arcanas::against(
        c.rules,
        &ran,
        ArcanaPreset::DEFAULT,
        policy,
        c.seeds(),
        c.budget.reaction,
        c.budget.threads,
    );
    let mut text = format!(
        "named builds vs default {:?}, policy {}, {} seed pairs each\n",
        ArcanaPreset::DEFAULT.ranks,
        policy.name(),
        c.pairs
    );
    for ((name, _), (_, e)) in presets.iter().zip(rows.iter()) {
        let _ = writeln!(
            text,
            "  {name:<16} {:>5.1}% [{:.1} – {:.1}]",
            e.mean * 100.0,
            e.lo * 100.0,
            e.hi * 100.0
        );
    }
    print!("{text}");
    c.out
        .write("arcana-named.txt", &text)
        .map_err(|e| e.to_string())?;
    c.finish(
        (ran.len() as u64) * c.pairs * 2,
        0,
        "P40 named confirmation",
    )
}

/// Every complete Arcana preset and the named fixtures against the default
/// (D-40): the evidence BR-AR's acceptance hypotheses ask for.
fn arcana_cmd(a: &Args) -> Result<(), String> {
    let c = ctx(a, 256, CONFIRMATION)?;
    let policy = match a.str("a") {
        Some(_) => policy(a, "a")?,
        None => Policy::Bot,
    };
    let all = arcanas::complete();
    let run = |presets: &[ArcanaPreset]| {
        arcanas::against(
            c.rules,
            presets,
            ArcanaPreset::DEFAULT,
            policy,
            c.seeds(),
            c.budget.reaction,
            c.budget.threads,
        )
    };
    let mut rows = run(&all);
    rows.sort_by(|x, y| y.1.mean.total_cmp(&x.1.mean));
    let show = |p: &ArcanaPreset, e: &stats::Estimate| {
        format!(
            "{:?}  {:>5.1}% [{:.1} – {:.1}]",
            p.ranks,
            e.mean * 100.0,
            e.lo * 100.0,
            e.hi * 100.0
        )
    };
    let mut text = format!(
        "{} complete presets vs default {:?}, policy {}, {} seed pairs each\n",
        rows.len(),
        ArcanaPreset::DEFAULT.ranks,
        policy.name(),
        c.pairs
    );
    let above = rows.iter().filter(|r| r.1.lo > 0.55).count();
    let below = rows.iter().filter(|r| r.1.hi < 0.45).count();
    let _ = writeln!(
        text,
        "interval above 55 %: {above} · interval below 45 %: {below}"
    );
    let _ = writeln!(text, "\nnamed fixtures:");
    let named: Vec<ArcanaPreset> = arcanas::NAMED
        .iter()
        .map(|n| ArcanaPreset { ranks: n.1 })
        .collect();
    for ((name, _), (p, e)) in arcanas::NAMED.iter().zip(run(&named)) {
        let _ = writeln!(text, "  {name:<11} {}", show(&p, &e));
    }
    let neutral = run(&[ArcanaPreset::NEUTRAL]);
    let _ = writeln!(
        text,
        "  {:<11} {}  (diagnostic, never valid online)",
        "neutral",
        show(&neutral[0].0, &neutral[0].1)
    );
    let _ = writeln!(text, "\nbest 10:");
    for (p, e) in rows.iter().take(10) {
        let _ = writeln!(text, "  {}", show(p, e));
    }
    let _ = writeln!(text, "worst 10:");
    for (p, e) in rows.iter().rev().take(10) {
        let _ = writeln!(text, "  {}", show(p, e));
    }
    print!("{text}");
    c.out
        .write("arcana.txt", &text)
        .map_err(|e| e.to_string())?;
    let games = (all.len() + named.len() + 1) as u64 * c.pairs * 2;
    c.finish(games, 0, "")
}
