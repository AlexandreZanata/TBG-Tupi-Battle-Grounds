//! Deterministic local Practice policies; the baseline Bot remains unchanged.
use crate::{bot::Bot, Archetype, MatchState, MatchView, Phase, PlayerIntention, Seat};

pub struct PracticeBot {
    baseline: Bot,
    difficulty: u8,
    next: u32,
    regen_frac: u64,
}
impl PracticeBot {
    pub fn new(seed: u64, difficulty: u8) -> Self {
        Self {
            baseline: Bot::new(seed),
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
            return (!r.resolved).then_some(PlayerIntention::Forgo { round: r.round });
        }
        if v.tick < self.next {
            return None;
        }
        self.next = v
            .tick
            .saturating_add(if self.difficulty == 0 { 40 } else { 2 });
        if self.difficulty == 0 {
            return self.baseline.decide(v);
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
        if self.difficulty != 3
            || !(state.phase == Phase::Running
                || (state.phase == Phase::Countdown
                    && state.tick + 1 >= crate::content::RUNNING_START))
            || state.augments.selecting()
        {
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
