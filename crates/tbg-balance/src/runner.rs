//! Parallel, order-stable match runner (P16-C step 4). Workers take the next
//! job index from an atomic counter; results are re-ordered by index, so the
//! output is byte-identical for any thread count (contract C3).

use std::sync::atomic::{AtomicUsize, Ordering};

use tbg_core::headless::{self, Moment};
use tbg_core::{ArcanaPreset, MatchState, PlayerIntention, Ruleset, Seat, StrongholdKind};

use crate::metrics::{observe_tick, participant, MatchResult, Side};
use crate::policy::{Controller, Policy};

/// A lab-only change applied before a chosen tick. The online server never
/// uses it; the value engine uses it to price Energy at a point in time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Setup {
    #[default]
    None,
    /// Add sub-energy to one participant at `tick`. `seat` is a participant
    /// index, not a physical seat; `play` maps it after the seat swap.
    Energy {
        seat: usize,
        tick: u32,
        sub_energy: u32,
    },
    /// Stronghold kind per participant (D-46); it follows the participant
    /// on a swap like the Arcana preset.
    Strongholds([StrongholdKind; 2]),
}

impl Setup {
    fn apply(self, state: &mut MatchState) {
        if let Setup::Energy {
            seat,
            tick,
            sub_energy,
        } = self
        {
            if state.tick == tick && seat < 2 {
                state.players[seat].energy = state.players[seat].energy.saturating_add(sub_energy);
            }
        }
    }
}

/// Two participants on one ruleset. Participant 0 is "the row" in reports.
#[derive(Clone, Copy, Debug)]
pub struct Matchup {
    pub rules: &'static Ruleset,
    pub policies: [Policy; 2],
    /// Views between decisions per participant.
    pub reaction: [u32; 2],
    /// Lab-only setup; `None` in every production and gate path.
    pub setup: Setup,
    /// Arcana preset per participant (it follows the participant on a swap).
    pub arcanas: [ArcanaPreset; 2],
}

/// Map `f` over `items` on `threads` workers; results in item order.
pub fn parallel_map<T: Sync, R: Send>(
    items: &[T],
    threads: usize,
    f: impl Fn(&T) -> R + Sync,
) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let mut out: Vec<(usize, R)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads.max(1))
            .map(|_| {
                scope.spawn(|| {
                    let mut local = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(item) = items.get(i) else { break };
                        local.push((i, f(item)));
                    }
                    local
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| {
                // A panicking job must fail the run, never shift results.
                w.join().unwrap_or_else(|e| std::panic::resume_unwind(e))
            })
            .collect()
    });
    out.sort_unstable_by_key(|(i, _)| *i);
    out.into_iter().map(|(_, r)| r).collect()
}

/// One game. `swapped` puts participant 1 on seat A; each participant keeps
/// its own Shop stream and policy seed in both games of a pair.
pub fn play(m: &Matchup, seed: u64, swapped: bool) -> MatchResult {
    let mut state = MatchState::lab(seed, m.rules, m.arcanas);
    if let Setup::Strongholds(kinds) = m.setup {
        state = state.with_strongholds(kinds);
    }
    let seat_a = usize::from(swapped);
    if swapped {
        state.players.swap(0, 1);
    }
    // Keep the setup attached to the participant, not to the physical seat.
    let setup = match m.setup {
        Setup::Energy {
            seat,
            tick,
            sub_energy,
        } => Setup::Energy {
            seat: if swapped { 1 - seat } else { seat },
            tick,
            sub_energy,
        },
        other => other,
    };
    let mut ctl: [Controller; 2] = std::array::from_fn(|p| {
        Controller::new(
            m.policies[p],
            m.rules,
            seed.wrapping_mul(4) + p as u64 + 1,
            m.reaction[p],
        )
    });
    let mut sides: [Side; 2] = Default::default();
    let mut hp = None;
    let mut shop_journals: [crate::shop_metrics::Journal; 2] = Default::default();
    let ended = headless::play_with(
        &mut state,
        |state| setup.apply(state),
        |seat, view| ctl[participant(seat, seat_a)].decide(view),
        |moment| match moment {
            Moment::Intention {
                seat,
                view,
                intent,
                result: Ok(()),
            } => {
                let index = participant(seat, seat_a);
                let side = &mut sides[index];
                shop_journals[index].record(side, view, intent);
                match intent {
                    PlayerIntention::Purchase { offer } => {
                        let i = usize::from(offer);
                        let cost = m.rules.def(view.offers[i]).cost;
                        side.purchases[view.offers[i] as usize] += 1;
                        side.spent += u64::from(cost);
                        side.expensive_bought += u32::from(
                            (6 * tbg_core::content::ENERGY_SCALE
                                ..=7 * tbg_core::content::ENERGY_SCALE)
                                .contains(&cost),
                        );
                    }
                    PlayerIntention::Evolve { unit, branch } => {
                        if let Some(u) = view.units.iter().find(|u| u.id == unit) {
                            side.evolutions[u.kind as usize][usize::from(branch.min(1))] += 1;
                            side.spent += u64::from(view.evolve_costs[usize::from(u.tier.min(1))]);
                        }
                    }
                    PlayerIntention::ChooseAugment { option, .. } => {
                        side.spent += u64::from(
                            view.augment_round
                                .as_ref()
                                .and_then(|r| r.costs.get(usize::from(option)))
                                .copied()
                                .unwrap_or(0),
                        );
                    }
                    PlayerIntention::Fortify { .. } => {
                        let tier = usize::from(view.strongholds[seat.index()].tier.min(1));
                        side.spent += u64::from(m.rules.fortify_cost[tier]);
                    }
                    PlayerIntention::Reserve { .. }
                    | PlayerIntention::Release { .. }
                    | PlayerIntention::Reroll { .. }
                    | PlayerIntention::Forgo { .. } => {}
                }
            }
            Moment::Intention { .. } => {}
            Moment::Stepped { state } => observe_tick(state, seat_a, &mut sides, &mut hp),
        },
    );
    assert!(ended, "match {seed} did not end: simulation bug");
    let outcome = state.outcome.expect("an ended match has an outcome");
    // Counters are per seat; the report is per participant (P42).
    let mut stats = tbg_core::lab::LabStats::default();
    for seat in Seat::BOTH {
        let p = participant(seat, seat_a);
        stats.dealt[p] = state.lab.dealt[seat.index()];
        stats.taken[p] = state.lab.taken[seat.index()];
        stats.healed[p] = state.lab.healed[seat.index()];
        stats.status_ticks[p] = state.lab.status_ticks[seat.index()];
        stats.stronghold_dealt[p] = state.lab.stronghold_dealt[seat.index()];
    }
    stats.last_progress = state.lab.last_progress;
    stats.longest_stall = state.lab.longest_stall;
    MatchResult {
        winner: outcome.winner.map(|w: Seat| participant(w, seat_a)),
        reason: outcome.reason,
        ticks: state.tick,
        hp_at_checkpoint: hp,
        sides,
        seat_a,
        stats,
        stall: state.lab.stall_at_end(state.tick),
    }
}

/// Both games of every seed for every matchup: `[matchup][seed] -> pair`.
pub fn run_pairs(
    matchups: &[Matchup],
    seeds: std::ops::Range<u64>,
    threads: usize,
) -> Vec<Vec<[MatchResult; 2]>> {
    let jobs: Vec<(usize, u64)> = (0..matchups.len())
        .flat_map(|m| seeds.clone().map(move |s| (m, s)))
        .collect();
    let pairs = parallel_map(&jobs, threads, |&(m, s)| {
        [play(&matchups[m], s, false), play(&matchups[m], s, true)]
    });
    let per = (seeds.end - seeds.start) as usize;
    let mut it = pairs.into_iter();
    (0..matchups.len())
        .map(|_| it.by_ref().take(per).collect())
        .collect()
}
