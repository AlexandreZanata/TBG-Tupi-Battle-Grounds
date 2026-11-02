//! Seeded heuristic opponent: practice mode, balance harness, load tests.
//! Acts only from its `MatchView`, like a real client.

use crate::content::{ENERGY_SCALE, OFFERS};
use crate::intent::PlayerIntention;
use crate::rng::Pcg32;
use crate::ruleset::{Ruleset, CANON};
use crate::types::{Archetype, StrongholdKind};
use crate::view::MatchView;

/// Energy at which the bot evolves when it found nothing to buy.
const EVOLVE_AT: u32 = 8 * ENERGY_SCALE;
/// Army size from which the bot prefers evolving to buying. A play-style
/// preference, not a limit: it still buys whenever evolving is not possible.
const EVOLVE_ARMY: usize = 6;

pub struct Bot {
    shop: (bool, bool),
    rolls: u8,
    rng: Pcg32,
    wait: u32,
    rules: &'static Ruleset,
}

impl Bot {
    pub fn new(seed: u64) -> Bot {
        Bot::with_rules(seed, &CANON)
    }

    /// A bot reading costs from another ruleset: the balance lab only (D-33).
    pub fn with_rules(seed: u64, rules: &'static Ruleset) -> Bot {
        Bot {
            shop: (true, true),
            rolls: 0,
            rng: Pcg32::new(seed, 7),
            wait: 0,
            rules,
        }
    }

    /// Independent controls for paired lab experiments; practice enables both.
    pub fn shop_options(mut self, reserve: bool, reroll: bool) -> Self {
        self.shop = (reserve, reroll);
        self
    }

    /// Called once per view (10 per second).
    pub fn decide(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        if view.phase != crate::Phase::Running {
            return None;
        }
        // ponytail: the bot forgoes every Augment round so a paused battle
        // resumes as soon as its human opponent confirms (BR-AG13). Upgrade:
        // an Augment preference once the lab measures one (D-33).
        if let Some(round) = &view.augment_round {
            return (!round.resolved).then_some(PlayerIntention::Forgo { round: round.round });
        }
        if self.wait > 0 {
            self.wait -= 1;
            return None;
        }
        self.wait = 3 + self.rng.below(6);
        // PK-01: spend on deployed power before filling an already blocked entrance.
        // This is an opponent preference; players retain unlimited purchases.
        if view.queued > 0 {
            return self
                .fortify(view)
                .or_else(|| self.evolve(view, 0))
                .or_else(|| {
                    crate::bot_shop::decide(
                        view,
                        self.rules,
                        self.shop.0,
                        self.shop.1,
                        self.rolls,
                        |kind| {
                            self.rules.def(kind).damage > 0
                                && crate::content::occupancy::footprint(kind, 0).width
                                    <= crate::content::occupancy::WIDTH / 2
                                && !self
                                    .rules
                                    .def(kind)
                                    .ability(0, 0)
                                    .contains(&crate::content::Ability::NoStronghold)
                        },
                    )
                });
        }
        let big_army = army(view) >= EVOLVE_ARMY;
        let shop = crate::bot_shop::decide(
            view,
            self.rules,
            self.shop.0,
            self.shop.1,
            self.rolls,
            |_| true,
        );
        let action = shop.or_else(|| {
            self.buy_tank(view)
                .or_else(|| big_army.then(|| self.fortify(view)).flatten())
                .or_else(|| big_army.then(|| self.evolve(view, 0)).flatten())
                .or_else(|| self.buy_best(view))
                .or_else(|| self.evolve(view, EVOLVE_AT))
        });
        match action {
            Some(PlayerIntention::Purchase { .. }) => self.rolls = 0,
            Some(PlayerIntention::Reroll { .. }) => self.rolls = self.rolls.saturating_add(1),
            _ => {}
        }
        action
    }

    /// Energy is the only purchase condition (BR-U07).
    fn fits(&self, view: &MatchView, offer: usize) -> bool {
        self.rules.def(view.offers[offer]).cost <= view.energy
    }

    fn buy_tank(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        let own_tank = view
            .units
            .iter()
            .any(|u| u.seat == view.you && self.rules.def(u.kind).archetype == Archetype::Tank);
        if own_tank {
            return None;
        }
        (0..OFFERS)
            .find(|&i| {
                self.rules.def(view.offers[i]).archetype == Archetype::Tank && self.fits(view, i)
            })
            .map(|i| PlayerIntention::Purchase { offer: i as u8 })
    }

    fn buy_best(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        let reserve = self.rng.below(3) * ENERGY_SCALE;
        (0..OFFERS)
            .filter(|&i| self.fits(view, i))
            .max_by_key(|&i| (self.rules.def(view.offers[i]).cost, std::cmp::Reverse(i)))
            .filter(|&i| view.energy >= self.rules.def(view.offers[i]).cost + reserve)
            .map(|i| PlayerIntention::Purchase { offer: i as u8 })
    }

    /// Evolve the oldest own unit below tier 2 once Energy reaches `at`.
    fn evolve(&mut self, view: &MatchView, at: u32) -> Option<PlayerIntention> {
        if view.energy < at {
            return None;
        }
        let unit = view
            .units
            .iter()
            .filter(|u| u.seat == view.you && u.tier < 2 && !u.is_summon())
            .filter(|u| view.queued == 0 || evolution_space(view, u))
            .min_by_key(|u| u.id)?;
        // The seat's real price (Augment and arena discounts, ARENAS-G).
        if view.energy < view.evolve_costs[usize::from(unit.tier.min(1))] {
            return None;
        }
        let branch = if unit.tier == 0 {
            self.rng.below(2) as u8
        } else {
            unit.branch
        };
        Some(PlayerIntention::Evolve {
            unit: unit.id,
            branch,
        })
    }
}

/// Conservative visible-space preference; the server still validates every Evolution.
fn evolution_space(view: &MatchView, unit: &crate::UnitView) -> bool {
    use crate::content::occupancy::{footprint, CELL, GAP, WIDTH};
    let reserved = |width: i32| (width + CELL - 1) / CELL * CELL;
    let next = footprint(unit.kind, unit.tier + 1);
    let width = reserved(next.width);
    let z = i32::from(unit.lane_z);
    if z.abs() + (width + 1) / 2 > WIDTH / 2 {
        return false;
    }
    !view.units.iter().any(|other| {
        if other.id == unit.id
            || other.seat != unit.seat
            || other.hp <= 0
            || other.fx & (1 << 14) != 0
        {
            return false;
        }
        let size = footprint(other.kind, other.tier);
        let dx = (other.x - unit.x) * unit.seat.dir();
        let gap = if dx >= 0 {
            next.front + size.back
        } else {
            size.front + next.back
        } + GAP;
        dx.abs() < gap && (i32::from(other.lane_z) - z).abs() * 2 < width + reserved(size.width)
    })
}

impl Bot {
    /// Fortify its own tower once the army is big (BR-SK05). Never on the
    /// Classic reference tower, so the lab's Classic baselines keep the same
    /// bot (D-33).
    fn fortify(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        let own = view.strongholds[view.you.index()];
        if own.kind == StrongholdKind::Classic || own.tier >= 2 {
            return None;
        }
        if view.energy < self.rules.fortify_cost[usize::from(own.tier)] {
            return None;
        }
        let branch = if own.tier == 0 {
            self.rng.below(2) as u8
        } else {
            own.branch
        };
        Some(PlayerIntention::Fortify { branch })
    }
}

/// Own living conventional units in the view (summons excluded, BR-U08).
pub fn army(view: &MatchView) -> usize {
    view.units
        .iter()
        .filter(|u| u.seat == view.you && !u.is_summon())
        .count()
}

/// Shared conservative Shop policy for offline preference experiments.
pub fn shop_decision(
    view: &MatchView,
    rules: &Ruleset,
    reserve: bool,
    reroll: bool,
    rolls: u8,
    wanted: impl Fn(crate::UnitKind) -> bool,
) -> Option<PlayerIntention> {
    crate::bot_shop::decide(view, rules, reserve, reroll, rolls, wanted)
}
