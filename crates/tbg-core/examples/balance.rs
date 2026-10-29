//! Bot-vs-bot balance harness:
//! `cargo run --release -p tbg-core --example balance -- [N] [bot|heavy|cheap]`.
//! Prints the GAME-DESIGN §6 metrics a harness can measure, and step time.
//! `heavy` / `cheap` (P16-B) replace seat A with an adversarial spam policy
//! — always buy the most expensive / cheapest affordable offer, never
//! evolve — against the normal bot on seat B.

use std::time::Instant;

use tbg_core::bot::Bot;
use tbg_core::content::{def, RUNNING_START, TICK_HZ};
use tbg_core::headless::{self, Moment};
use tbg_core::*;

#[derive(Default)]
struct Totals {
    durations: Vec<u32>,
    reasons: [u32; 3],
    faction_wins: [u32; 4],
    faction_games: [u32; 4],
    purchases: [u32; 10],
    /// Per kind: (games a seat bought it, games that seat won).
    bought_won: [(u32, u32); 10],
    evolutions: u32,
    comebacks: u32,
    decided_at_2min: u32,
    a_wins: u32,
    max_army: usize,
    step_ns: u128,
    steps: u128,
    max_step_ns: u128,
    max_units: usize,
    draws: u32,
}

#[derive(Clone, Copy, PartialEq)]
enum Policy {
    Bot,
    Heavy,
    Cheap,
}

fn main() {
    let n: u64 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(200);
    let policy = match std::env::args().nth(2).as_deref() {
        Some("heavy") => Policy::Heavy,
        Some("cheap") => Policy::Cheap,
        _ => Policy::Bot,
    };
    let mut t = Totals::default();
    for seed in 0..n {
        if !play(seed, policy, &mut t) {
            eprintln!("match {seed} did not end");
            std::process::exit(1);
        }
    }
    report(n, &mut t);
}

/// Returns false if the match did not end in time.
/// Seat A's spam policy: the most expensive or cheapest affordable offer.
fn spam(view: &MatchView, policy: Policy) -> Option<PlayerIntention> {
    let affordable = (0..view.offers.len()).filter(|&i| def(view.offers[i]).cost <= view.energy);
    let pick = match policy {
        Policy::Heavy => affordable.max_by_key(|&i| def(view.offers[i]).cost),
        _ => affordable.min_by_key(|&i| def(view.offers[i]).cost),
    }?;
    Some(PlayerIntention::Purchase { offer: pick as u8 })
}

fn play(seed: u64, policy: Policy, t: &mut Totals) -> bool {
    let mut s = MatchState::new(seed);
    let mut bots = [Bot::new(2 * seed + 1), Bot::new(2 * seed + 2)];
    let mut bought = [[0u32; 4]; 2];
    let mut kinds = [[false; 10]; 2];
    let mut hp_at_2min = None;
    let mut last = Instant::now();
    let decide = |seat: Seat, view: &MatchView| {
        if seat == Seat::A && policy != Policy::Bot {
            spam(view, policy)
        } else {
            bots[seat.index()].decide(view)
        }
    };
    let ended = headless::play(&mut s, decide, |m| match m {
        Moment::Intention {
            seat,
            view,
            intent,
            result: Ok(()),
        } => match intent {
            PlayerIntention::Purchase { offer } => {
                let k = view.offers[usize::from(offer)];
                t.purchases[k as usize] += 1;
                kinds[seat.index()][k as usize] = true;
                bought[seat.index()][def(k).faction.index()] += 1;
            }
            PlayerIntention::Evolve { .. } => t.evolutions += 1,
        },
        Moment::Intention { .. } => {}
        Moment::Stepped { state } => {
            // Whole tick: step, both views, both decisions.
            let ns = last.elapsed().as_nanos();
            last = Instant::now();
            t.step_ns += ns;
            t.steps += 1;
            t.max_step_ns = t.max_step_ns.max(ns);
            t.max_units = t.max_units.max(state.units.len());
            t.max_army = t
                .max_army
                .max(state.army_size(Seat::A))
                .max(state.army_size(Seat::B));
            if state.tick == RUNNING_START + 120 * TICK_HZ {
                hp_at_2min = Some([0, 1].map(|i| state.players[i].stronghold_hp));
            }
        }
    });
    if !ended {
        return false;
    }
    let o = s.outcome.expect("ended");
    t.durations.push(s.tick / TICK_HZ);
    t.reasons[o.reason as usize] += 1;
    t.draws += u32::from(o.winner.is_none());
    t.a_wins += u32::from(o.winner == Some(Seat::A));
    if let (Some(hp), Some(w)) = (hp_at_2min, o.winner) {
        if hp[0] != hp[1] {
            t.decided_at_2min += 1;
            t.comebacks += u32::from(hp[w.index()] < hp[1 - w.index()]);
        }
    }
    for seat in Seat::BOTH {
        for (k, &had) in kinds[seat.index()].iter().enumerate() {
            if had {
                t.bought_won[k].0 += 1;
                t.bought_won[k].1 += u32::from(o.winner == Some(seat));
            }
        }
    }
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
        let (games, won) = t.bought_won[i];
        let rate = 100.0 * f64::from(won) / f64::from(games.max(1));
        println!("purchase share {k:?}: {share:.1}% | win when bought {rate:.1}%");
    }
    println!(
        "seat A wins: {:.1}% | comebacks (behind at 2:00, then won): {:.1}% of {} | max army {}",
        pct(t.a_wins),
        100.0 * f64::from(t.comebacks) / f64::from(t.decided_at_2min.max(1)),
        t.decided_at_2min,
        t.max_army
    );
    println!(
        "evolutions per match: {:.1}",
        f64::from(t.evolutions) / n as f64
    );
    println!(
        "tick µs (step + views + bots): mean {:.2} max {:.1} | max units alive {}",
        t.step_ns as f64 / t.steps as f64 / 1_000.0,
        t.max_step_ns as f64 / 1_000.0,
        t.max_units
    );
}
