//! Deterministic local Practice policies; the baseline Bot remains unchanged.
use crate::{bot::Bot, Archetype, MatchState, MatchView, Phase, PlayerIntention, Seat};

pub struct PracticeBot {
    baseline: Bot,
    rng: crate::Pcg32,
    difficulty: u8,
    next: u32,
    regen_frac: u64,
}
impl PracticeBot {
    pub fn new(seed: u64, difficulty: u8) -> Self {
        Self {
            baseline: Bot::new(seed),
            rng: crate::Pcg32::new(seed, 62),
            difficulty: difficulty.min(3),
            next: 0,
            regen_frac: 0,
        }
    }
    pub fn decide(&mut self, v: &MatchView) -> Option<PlayerIntention> {
        if self.difficulty == 1 {
            return self.baseline.decide(v);
        }
        if v.phase != Phase::Running {
            return None;
        }
        if let Some(r) = &v.augment_round {
            if r.resolved {
                return None;
            }
            if self.difficulty >= 2 {
                if let Some((i, _)) = r
                    .costs
                    .iter()
                    .enumerate()
                    .filter(|(_, cost)| **cost <= v.energy)
                    .max_by_key(|(i, cost)| (**cost, std::cmp::Reverse(*i)))
                {
                    return Some(PlayerIntention::ChooseAugment {
                        round: r.round,
                        option: i as u8,
                    });
                }
            }
            return Some(PlayerIntention::Forgo { round: r.round });
        }
        if v.tick < self.next {
            return None;
        }
        self.next = v.tick.saturating_add(if self.difficulty == 0 {
            40
        } else if self.difficulty == 2 {
            4
        } else {
            2
        });
        if self.difficulty == 0 {
            let affordable: Vec<_> = v
                .offers
                .iter()
                .enumerate()
                .filter(|(_, k)| crate::content::def(**k).cost <= v.energy)
                .map(|(i, _)| i)
                .collect();
            return (!affordable.is_empty()).then(|| PlayerIntention::Purchase {
                offer: affordable[self.rng.below(affordable.len() as u32) as usize] as u8,
            });
        }
        let own = v.strongholds[v.you.index()];
        let threatened = v.units.iter().any(|u| {
            u.seat != v.you
                && if v.you == Seat::A {
                    u.x < 4_000
                } else {
                    u.x > 12_000
                }
        });
        if threatened
            && own.kind != crate::StrongholdKind::Classic
            && own.tier < 2
            && v.energy >= crate::content::FORTIFY_COST[usize::from(own.tier)]
        {
            return Some(PlayerIntention::Fortify { branch: own.branch });
        }
        let tank = v
            .units
            .iter()
            .any(|u| u.seat == v.you && crate::content::def(u.kind).archetype == Archetype::Tank);
        let ally_count = v
            .units
            .iter()
            .filter(|u| u.seat == v.you && !u.is_summon())
            .count();
        let offer = v
            .offers
            .iter()
            .enumerate()
            .filter(|(_, k)| crate::content::def(**k).cost <= v.energy)
            .max_by_key(|(i, k)| {
                let d = crate::content::def(**k);
                (
                    (!tank && d.archetype == Archetype::Tank),
                    d.cost,
                    std::cmp::Reverse(*i),
                )
            });
        if ally_count < 6 {
            if let Some((i, _)) = offer {
                return Some(PlayerIntention::Purchase { offer: i as u8 });
            }
        }
        if let Some(u) = v
            .units
            .iter()
            .filter(|u| u.seat == v.you && !u.is_summon() && u.tier < 2)
            .find(|u| v.energy >= v.evolve_costs[usize::from(u.tier)])
        {
            return Some(PlayerIntention::Evolve {
                unit: u.id,
                branch: if u.tier == 0 {
                    (u.id % 2) as u8
                } else {
                    u.branch
                },
            });
        }
        offer
            .map(|(i, _)| PlayerIntention::Purchase { offer: i as u8 })
            .or_else(|| self.baseline.decide(v))
    }
    /// Only a caller-owned local Practice state receives this disclosed assistance.
    /// Compute before step, apply after step: identical pre-regen Interest and arena rate.
    pub fn regen_bonus(&mut self, state: &MatchState) -> u32 {
        let regenerates = state.phase == Phase::Running
            || (state.phase == Phase::Countdown && state.tick == crate::content::RUNNING_START);
        if self.difficulty != 3 || !regenerates || state.augments.selecting() {
            return 0;
        }
        let rules = state.rules();
        let bp = rules
            .interest
            .regen_bp(state.players[Seat::B.index()].energy)
            + state.arena_rule().regen_bp;
        self.regen_frac += u64::from(rules.regen_running) * u64::from(bp);
        let gain = (self.regen_frac / 10_000) as u32;
        self.regen_frac %= 10_000;
        gain
    }
}
