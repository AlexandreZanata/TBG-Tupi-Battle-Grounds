//! Bot-vs-bot balance harness: `cargo run --release -p tbg-core --example balance -- [N]`.
//! Prints the GAME-DESIGN §6 metrics a harness can measure, and step time.

use std::time::Instant;

use tbg_core::bot::Bot;
use tbg_core::content::{def, MATCH_END, TICK_HZ};
use tbg_core::*;

#[derive(Default)]
struct Totals {
    durations: Vec<u32>,
    reasons: [u32; 3],
    faction_wins: [u32; 4],
    faction_games: [u32; 4],
    purchases: [u32; 8],
    evolutions: u32,
    step_ns: u128,
    steps: u128,
    max_step_ns: u128,
    max_units: usize,
    draws: u32,
}

fn main() {
    let n: u64 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(200);
    let mut t = Totals::default();
    for seed in 0..n {
        if !play(seed, &mut t) {
            eprintln!("match {seed} did not end");
            std::process::exit(1);
        }
    }
    report(n, &mut t);
}

/// Returns false if the match did not end in time.
fn play(seed: u64, t: &mut Totals) -> bool {
    let mut s = MatchState::new(seed);
    let mut bots = [Bot::new(2 * seed + 1), Bot::new(2 * seed + 2)];
    let mut bought = [[0u32; 4]; 2];
    let mut events = Vec::new();
    while s.phase != Phase::Ended {
        if s.tick > MATCH_END + 5 {
            return false;
        }
        if s.tick.is_multiple_of(2) {
            for seat in Seat::BOTH {
                let view = s.view_for(seat, &events);
                if let Some(intent) = bots[seat.index()].decide(&view) {
                    let kind = match intent {
                        PlayerIntention::Purchase { offer } => {
                            Some(view.offers[usize::from(offer)])
                        }
                        PlayerIntention::Evolve { .. } => None,
                    };
                    if s.apply_intention(seat, s.tick, intent).is_ok() {
                        match kind {
                            Some(k) => {
                                t.purchases[k as usize] += 1;
                                bought[seat.index()][def(k).faction.index()] += 1;
                            }
                            None => t.evolutions += 1,
                        }
                    }
                }
            }
            events.clear();
        }
        let start = Instant::now();
        events.extend(s.step());
        let ns = start.elapsed().as_nanos();
        t.step_ns += ns;
        t.steps += 1;
        t.max_step_ns = t.max_step_ns.max(ns);
        t.max_units = t.max_units.max(s.units.len());
    }
    let o = s.outcome.expect("ended");
    t.durations.push(s.tick / TICK_HZ);
    t.reasons[o.reason as usize] += 1;
    t.draws += u32::from(o.winner.is_none());
    for seat in Seat::BOTH {
        // Strict majority only; tied armies count as mixed (no faction).
        let counts = bought[seat.index()];
        let top = counts.iter().copied().max().unwrap_or(0);
        if top > 0 && counts.iter().filter(|&&c| c == top).count() == 1 {
            let main = counts.iter().position(|&c| c == top).unwrap_or(0);
            t.faction_games[main] += 1;
            if o.winner == Some(seat) {
                t.faction_wins[main] += 1;
            }
        }
    }
    true
}

fn report(n: u64, t: &mut Totals) {
    t.durations.sort_unstable();
    let pct = |x: u32| 100.0 * f64::from(x) / n as f64;
    let at = |q: f64| t.durations[((t.durations.len() - 1) as f64 * q) as usize];
    println!("matches: {n}");
    println!("duration s: median {} p90 {}", at(0.5), at(0.9));
    println!(
        "ended by: stronghold {:.1}% | time {:.1}% | forfeit {:.1}%",
        pct(t.reasons[0]),
        pct(t.reasons[1]),
        pct(t.reasons[2])
    );
    println!("draws: {:.1}%", pct(t.draws));
    for (i, f) in FACTIONS.iter().enumerate() {
        let games = t.faction_games[i].max(1);
        let rate = 100.0 * f64::from(t.faction_wins[i]) / f64::from(games);
        println!(
            "faction-majority {f:?}: {} armies, win {rate:.1}%",
            t.faction_games[i]
        );
    }
    let total: u32 = t.purchases.iter().sum::<u32>().max(1);
    for (i, k) in UNIT_KINDS.iter().enumerate() {
        let share = 100.0 * f64::from(t.purchases[i]) / f64::from(total);
        println!("purchase share {k:?}: {share:.1}%");
    }
    println!(
        "evolutions per match: {:.1}",
        f64::from(t.evolutions) / n as f64
    );
    println!(
        "step µs: mean {:.2} max {:.1} | max units alive {}",
        t.step_ns as f64 / t.steps as f64 / 1_000.0,
        t.max_step_ns as f64 / 1_000.0,
        t.max_units
    );
}
