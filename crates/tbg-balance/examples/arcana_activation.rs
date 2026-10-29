//! P40 audit: how often each conditional Arcana has something to act on in
//! representative bot matches (GAME-DESIGN §9: "a conditional node that
//! never activates ... is not offering a decision"). It plays bot-vs-bot
//! matches on the canonical rules with the default Arcanas and counts, per
//! sampled combat tick, the opportunities each condition would have used.
//! Opportunities are what the rule could affect, not what a preset gained.
//!
//!   cargo run --release -p tbg-balance --example arcana_activation -- 200

use tbg_core::bot::Bot;
use tbg_core::headless::{self, Moment};
use tbg_core::types::{Archetype, Status};
use tbg_core::{Event, MatchState, Seat, Target, UNIT_KINDS};

#[derive(Default)]
struct Counts {
    ticks: u64,
    /// Ticks a seat has both a Tank and a DPS alive (Combined Arms).
    both_roles: u64,
    /// Ticks a seat has a DPS alive (Shieldwall) / a Tank alive (Battle Rush).
    dps_alive: u64,
    tank_alive: u64,
    /// Ranged attack windups, and those on a slowed/rooted target
    /// (Hunter's Focus) or within two tiles (Quickdraw).
    ranged_windups: u64,
    ranged_on_controlled: u64,
    ranged_close: u64,
    /// Support heal windups, those on an ally below 40 % (Emergency Care) and
    /// on a slowed/rooted ally (Purification).
    heal_windups: u64,
    heal_low: u64,
    heal_on_controlled: u64,
    /// Statuses applied, and those landing on Support/Control (Unshaken).
    statuses: u64,
    statuses_on_support_control: u64,
}

fn archetype(state: &MatchState, kind: tbg_core::UnitKind) -> Archetype {
    debug_assert!(UNIT_KINDS.contains(&kind) || kind.is_summon());
    state.rules().def(kind).archetype
}

fn sample(state: &MatchState, c: &mut Counts) {
    for seat in Seat::BOTH {
        let army = state.army_of(seat);
        c.ticks += 1;
        c.both_roles += u64::from(army.tanks > 0 && army.dps > 0);
        c.dps_alive += u64::from(army.dps > 0);
        c.tank_alive += u64::from(army.tanks > 0);
    }
    for u in state
        .units
        .iter()
        .filter(|u| u.hp > 0 && !u.kind.is_summon())
    {
        let Some((Target::Unit(id), _)) = u.windup else {
            continue;
        };
        let Some(i) = state.unit_index(id) else {
            continue;
        };
        let t = &state.units[i];
        let controlled = matches!(t.status, Status::Slowed { .. } | Status::Rooted { .. });
        match archetype(state, u.kind) {
            Archetype::Ranged if t.seat != u.seat => {
                c.ranged_windups += 1;
                c.ranged_on_controlled += u64::from(controlled);
                c.ranged_close += u64::from((t.x - u.x).abs() <= 2_000);
            }
            Archetype::Support if t.seat == u.seat => {
                c.heal_windups += 1;
                c.heal_low += u64::from(i64::from(t.hp) * 10_000 < 4_000 * i64::from(t.max_hp));
                c.heal_on_controlled += u64::from(controlled);
            }
            _ => {}
        }
    }
}

fn main() {
    let matches: u64 = std::env::args()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let mut c = Counts::default();
    let mut ended = 0u64;
    for seed in 0..matches {
        let mut state = MatchState::new(seed);
        let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
        let applied: std::cell::RefCell<Vec<Event>> = Default::default();
        let done = headless::play(
            &mut state,
            |seat, view| bots[seat.index()].decide(view),
            |m| {
                if let Moment::Stepped { state } = m {
                    if state.tick % 4 == 0 && state.is_live() {
                        sample(state, &mut c);
                    }
                }
            },
        );
        ended += u64::from(done);
        drop(applied);
    }
    // Statuses are counted from events of a second pass over the same seeds.
    for seed in 0..matches {
        let mut state = MatchState::new(seed);
        let mut bots = [Bot::new(seed * 2 + 1), Bot::new(seed * 2 + 2)];
        let mut events = Vec::new();
        let mut decisions = Vec::new();
        while state.phase != tbg_core::Phase::Ended && state.tick < 20 * 60 * 60 {
            if state.tick.is_multiple_of(2) {
                decisions.clear();
                for seat in Seat::BOTH {
                    let view = state.view_for(seat, &events);
                    if let Some(i) = bots[seat.index()].decide(&view) {
                        decisions.push((seat, i));
                    }
                }
                for (seat, i) in decisions.drain(..) {
                    let _ = state.apply_intention(seat, state.clock.max(1), i);
                }
                events.clear();
            }
            events.extend(state.step());
            for e in &events {
                if let Event::StatusApplied { unit } = e {
                    if let Some(i) = state.unit_index(*unit) {
                        c.statuses += 1;
                        let a = archetype(&state, state.units[i].kind);
                        c.statuses_on_support_control +=
                            u64::from(matches!(a, Archetype::Support | Archetype::Control));
                    }
                }
            }
        }
    }
    let pct = |n: u64, d: u64| 100.0 * n as f64 / d.max(1) as f64;
    println!(
        "{matches} bot matches ({ended} ended), default Arcanas, every 4th combat tick sampled"
    );
    println!("seat-ticks sampled: {}", c.ticks);
    println!(
        "Shieldwall    a DPS ally alive:               {:5.1}% of seat-ticks",
        pct(c.dps_alive, c.ticks)
    );
    println!(
        "Battle Rush   a Tank ally alive:              {:5.1}% of seat-ticks",
        pct(c.tank_alive, c.ticks)
    );
    println!(
        "Combined Arms Tank and DPS both alive:        {:5.1}% of seat-ticks",
        pct(c.both_roles, c.ticks)
    );
    println!(
        "Hunter's Focus Ranged attack on controlled:   {:5.1}% of {} Ranged windup ticks",
        pct(c.ranged_on_controlled, c.ranged_windups),
        c.ranged_windups
    );
    println!(
        "Quickdraw     Ranged target within 2 tiles:   {:5.1}% of Ranged windup ticks",
        pct(c.ranged_close, c.ranged_windups)
    );
    println!(
        "Emergency Care heal on an ally below 40% HP:  {:5.1}% of {} Support heal windup ticks",
        pct(c.heal_low, c.heal_windups),
        c.heal_windups
    );
    println!(
        "Purification  heal on a slowed/rooted ally:   {:5.1}% of Support heal windup ticks",
        pct(c.heal_on_controlled, c.heal_windups)
    );
    println!(
        "Unshaken      status landing on Support/Control: {:5.1}% of {} statuses applied",
        pct(c.statuses_on_support_control, c.statuses),
        c.statuses
    );
}
