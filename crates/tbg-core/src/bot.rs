//! Seeded heuristic opponent: practice mode, balance harness, load tests.
//! Acts only from its `MatchView`, like a real client.

use crate::content::{self, ENERGY_SCALE, EVOLVE_COST, SLOT_CAP};
use crate::intent::PlayerIntention;
use crate::rng::Pcg32;
use crate::types::Archetype;
use crate::view::MatchView;

/// Energy at which the bot starts evolving instead of buying.
const EVOLVE_AT: u32 = 8 * ENERGY_SCALE;

pub struct Bot {
    rng: Pcg32,
    wait: u32,
}

impl Bot {
    pub fn new(seed: u64) -> Bot {
        Bot {
            rng: Pcg32::new(seed, 7),
            wait: 0,
        }
    }

    /// Called once per view (10 per second).
    pub fn decide(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        if !matches!(view.phase, crate::Phase::Running | crate::Phase::Overtime) {
            return None;
        }
        if self.wait > 0 {
            self.wait -= 1;
            return None;
        }
        self.wait = 3 + self.rng.below(6);
        self.buy_tank(view)
            .or_else(|| self.buy_best(view))
            .or_else(|| self.evolve(view))
    }

    fn fits(view: &MatchView, offer: usize) -> bool {
        let d = content::def(view.offers[offer]);
        d.cost <= view.energy && view.slots_used + d.slots.count() <= SLOT_CAP
    }

    fn buy_tank(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        let own_tank = view
            .units
            .iter()
            .any(|u| u.seat == view.you && content::def(u.kind).archetype == Archetype::Tank);
        if own_tank {
            return None;
        }
        (0..3)
            .find(|&i| {
                content::def(view.offers[i]).archetype == Archetype::Tank && Self::fits(view, i)
            })
            .map(|i| PlayerIntention::Purchase { offer: i as u8 })
    }

    fn buy_best(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        let reserve = self.rng.below(3) * ENERGY_SCALE;
        (0..3)
            .filter(|&i| Self::fits(view, i))
            .max_by_key(|&i| (content::def(view.offers[i]).cost, std::cmp::Reverse(i)))
            .filter(|&i| view.energy >= content::def(view.offers[i]).cost + reserve)
            .map(|i| PlayerIntention::Purchase { offer: i as u8 })
    }

    fn evolve(&mut self, view: &MatchView) -> Option<PlayerIntention> {
        if view.energy < EVOLVE_AT && view.slots_used < SLOT_CAP {
            return None;
        }
        let unit = view
            .units
            .iter()
            .filter(|u| u.seat == view.you && u.tier < 2)
            .min_by_key(|u| u.id)?;
        if view.energy < EVOLVE_COST[usize::from(unit.tier)] {
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
