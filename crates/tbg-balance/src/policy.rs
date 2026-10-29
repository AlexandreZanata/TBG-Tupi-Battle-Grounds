//! The opponent pool (P16-C step 6): small, seeded playing styles that
//! decide only from what a client sees (`MatchView`), never from enemy
//! Energy, offers, seeds, or future draws. `Faction` and `Mono` variants are
//! generated from the roster, so new content joins the pool by itself.

use tbg_core::bot::Bot;
use tbg_core::content::OFFERS;
use tbg_core::{
    Archetype, Faction, MatchView, Pcg32, Phase, PlayerIntention, Ruleset, UnitKind, FACTIONS,
    UNIT_KINDS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// The shipped `tbg_core::bot::Bot` (reference opponent).
    Bot,
    /// Most expensive affordable offer, never evolves (P16-B).
    Heavy,
    /// Cheapest affordable offer, never evolves (P16-B).
    Cheap,
    /// One Tank, then evolve the oldest unit before buying again.
    Evolver,
    /// Bank to the Energy cap, then spend it all.
    Saver,
    /// Offers of one faction until its level-2 Synergy, then `Bot`.
    Faction(Faction),
    /// `Bot`, but buys this unit whenever it is offered and affordable: how
    /// much leaning on one unit helps (a pure one-unit army measured the
    /// policy's handicap, not the unit).
    Favor(UnitKind),
    /// `Bot`, but never buys this unit. The value engine uses it so a
    /// one-sided stat change cannot leak into the control seat's army.
    BotAvoid(UnitKind),
    /// Uniformly random legal action (sanity floor).
    Random,
}

impl Ord for Policy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key().cmp(&other.key())
    }
}

impl PartialOrd for Policy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Policy {
    /// Stable order for result maps: variant, then faction or kind index.
    fn key(self) -> (u8, usize) {
        match self {
            Policy::Bot => (0, 0),
            Policy::Heavy => (1, 0),
            Policy::Cheap => (2, 0),
            Policy::Evolver => (3, 0),
            Policy::Saver => (4, 0),
            Policy::Faction(f) => (5, f.index()),
            Policy::Favor(k) => (6, k as usize),
            Policy::Random => (7, 0),
            Policy::BotAvoid(k) => (8, k as usize),
        }
    }

    /// Strategic styles: the pool the GAME-DESIGN §6 rows are measured on.
    pub fn strategic() -> Vec<Policy> {
        let mut v = vec![Policy::Bot, Policy::Evolver, Policy::Saver];
        v.extend(FACTIONS.map(Policy::Faction));
        v
    }

    /// Every policy except `Random`: the meta-game pool.
    pub fn pool() -> Vec<Policy> {
        let mut v = Policy::strategic();
        v.extend([Policy::Heavy, Policy::Cheap]);
        v.extend(UNIT_KINDS.map(Policy::Favor));
        v
    }

    /// Lab policies that exist only for one-sided value experiments.
    pub fn value_extra() -> Vec<Policy> {
        UNIT_KINDS.into_iter().map(Policy::BotAvoid).collect()
    }

    pub fn name(self) -> String {
        match self {
            Policy::Faction(f) => format!("faction-{f:?}").to_lowercase(),
            Policy::Favor(k) => format!("favor-{k:?}").to_lowercase(),
            Policy::BotAvoid(k) => format!("avoid-{k:?}").to_lowercase(),
            other => format!("{other:?}").to_lowercase(),
        }
    }

    pub fn parse(s: &str) -> Option<Policy> {
        let mut all = Policy::pool();
        all.extend(Policy::value_extra());
        all.push(Policy::Random);
        all.into_iter().find(|p| p.name() == s.to_lowercase())
    }
}

/// One participant: a policy with its own seeded state.
pub struct Controller {
    policy: Policy,
    rules: &'static Ruleset,
    rng: Pcg32,
    bot: Bot,
    /// Views between decisions (1 = every view, 100 ms; 5 = 500 ms).
    reaction: u32,
    wait: u32,
    spending: bool,
}

impl Controller {
    pub fn new(policy: Policy, rules: &'static Ruleset, seed: u64, reaction: u32) -> Controller {
        Controller {
            policy,
            rules,
            rng: Pcg32::new(seed, 11),
            bot: Bot::with_rules(seed, rules),
            reaction: reaction.max(1),
            wait: 0,
            spending: false,
        }
    }

    pub fn decide(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        if !matches!(view.phase, Phase::Running | Phase::Overtime) {
            return None;
        }
        // The shipped bot keeps its own reaction schedule.
        if self.policy == Policy::Bot {
            return self.bot.decide(view);
        }
        if self.wait > 0 {
            self.wait -= 1;
            // A fallback bot keeps its own pace between our decisions.
            return match self.policy {
                Policy::Favor(_) => self.bot.decide(view),
                _ => None,
            };
        }
        self.wait = self.reaction - 1;
        match self.policy {
            Policy::Bot => self.bot.decide(view),
            Policy::Heavy => self.buy_by(view, i64::from),
            Policy::Cheap => self.buy_by(view, |cost| -i64::from(cost)),
            Policy::Evolver => self.evolver(view),
            Policy::Saver => self.saver(view),
            Policy::Faction(f) => self.faction(view, f),
            Policy::Favor(k) => self.favor(view, k),
            Policy::BotAvoid(k) => self.avoid(view, k),
            Policy::Random => self.random(view),
        }
    }

    fn cost(&self, view: &MatchView, offer: usize) -> u32 {
        self.rules.def(view.offers[offer]).cost
    }

    fn affordable(&self, view: &MatchView) -> impl Iterator<Item = usize> + '_ {
        let energy = view.energy;
        let costs: [u32; OFFERS] = std::array::from_fn(|i| self.rules.def(view.offers[i]).cost);
        (0..OFFERS).filter(move |&i| costs[i] <= energy)
    }

    /// The affordable offer with the highest `score(cost)`; lowest slot on ties.
    fn buy_by(&self, view: &MatchView, score: fn(u32) -> i64) -> Option<PlayerIntention> {
        self.affordable(view)
            .max_by_key(|&i| (score(self.cost(view, i)), std::cmp::Reverse(i)))
            .map(purchase)
    }

    /// Oldest own unit below tier 2 matching `keep`; tier-0 branch is random.
    fn evolve_oldest(
        &mut self,
        view: &MatchView,
        keep: impl Fn(UnitKind) -> bool,
    ) -> Option<Result<PlayerIntention, u32>> {
        let unit = view
            .units
            .iter()
            .filter(|u| u.seat == view.you && u.tier < 2 && !u.kind.is_summon() && keep(u.kind))
            .min_by_key(|u| u.id)?;
        let cost = self.rules.evolve_cost[usize::from(unit.tier)];
        if view.energy < cost {
            return Some(Err(cost));
        }
        let branch = if unit.tier == 0 {
            self.rng.below(2) as u8
        } else {
            unit.branch
        };
        Some(Ok(PlayerIntention::Evolve {
            unit: unit.id,
            branch,
        }))
    }

    fn evolver(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        let rules = self.rules;
        let has_tank = view
            .units
            .iter()
            .any(|u| u.seat == view.you && rules.def(u.kind).archetype == Archetype::Tank);
        if !has_tank {
            let tank = self
                .affordable(view)
                .find(|&i| rules.def(view.offers[i]).archetype == Archetype::Tank);
            if let Some(i) = tank {
                return Some(purchase(i));
            }
        }
        match self.evolve_oldest(view, |_| true) {
            Some(Ok(intent)) => Some(intent),
            // Saving for the next Evolution.
            Some(Err(_)) => None,
            None => self.buy_by(view, i64::from),
        }
    }

    fn saver(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        if view.energy >= self.rules.energy_cap {
            self.spending = true;
        }
        if !self.spending {
            return None;
        }
        let buy = self.buy_by(view, i64::from);
        self.spending = buy.is_some();
        buy
    }

    fn faction(&mut self, view: &MatchView, f: Faction) -> Option<PlayerIntention> {
        let rules = self.rules;
        let level2 = view.synergy[view.you.index()][f.index()] >= 2;
        if level2 {
            return self.bot.decide(view);
        }
        let of_f = |i: &usize| rules.def(view.offers[*i]).faction == f;
        if let Some(i) = self
            .affordable(view)
            .filter(of_f)
            .max_by_key(|&i| (self.cost(view, i), std::cmp::Reverse(i)))
        {
            return Some(purchase(i));
        }
        let offered = (0..OFFERS).any(|i| of_f(&i));
        // Without a faction offer, avoid wasting Energy at the cap.
        (!offered && view.energy >= rules.energy_cap)
            .then(|| self.buy_by(view, |cost| -i64::from(cost)))
            .flatten()
    }

    fn favor(&mut self, view: &MatchView, k: UnitKind) -> Option<PlayerIntention> {
        let pick = self.affordable(view).find(|&i| view.offers[i] == k);
        match pick {
            Some(i) => Some(purchase(i)),
            None => self.bot.decide(view),
        }
    }

    fn avoid(&mut self, view: &MatchView, k: UnitKind) -> Option<PlayerIntention> {
        match self.bot.decide(view) {
            Some(PlayerIntention::Purchase { offer }) if view.offers[usize::from(offer)] == k => {
                self.affordable(view)
                    .filter(|&i| view.offers[i] != k)
                    .max_by_key(|&i| (self.cost(view, i), std::cmp::Reverse(i)))
                    .map(purchase)
            }
            other => other,
        }
    }

    fn random(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        let offers: Vec<usize> = self.affordable(view).collect();
        let own: Vec<_> = view
            .units
            .iter()
            .filter(|u| u.seat == view.you && u.tier < 2 && !u.kind.is_summon())
            .filter(|u| view.energy >= self.rules.evolve_cost[usize::from(u.tier)])
            .map(|u| (u.id, u.tier, u.branch))
            .collect();
        let pick = self.rng.below((offers.len() + own.len() + 1) as u32) as usize;
        if pick < offers.len() {
            return Some(purchase(offers[pick]));
        }
        let (unit, tier, branch) = *own.get(pick - offers.len())?;
        let branch = if tier == 0 {
            self.rng.below(2) as u8
        } else {
            branch
        };
        Some(PlayerIntention::Evolve { unit, branch })
    }
}

fn purchase(offer: usize) -> PlayerIntention {
    PlayerIntention::Purchase { offer: offer as u8 }
}
