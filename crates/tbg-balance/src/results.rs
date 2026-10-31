//! Bounded-run result storage: retain paired observations and aggregate combat counters once.
use crate::{metrics::MatchResult, policy::Policy};
use std::collections::BTreeMap;
use tbg_core::{lab::LabStats, ALL_KINDS};

#[derive(Default)]
pub struct Results {
    pub pairs: BTreeMap<(Policy, Policy), Vec<[MatchResult; 2]>>,
    pub combat: LabStats,
}

impl Results {
    pub fn insert(&mut self, policies: (Policy, Policy), mut pairs: Vec<[MatchResult; 2]>) {
        assert!(!self.pairs.contains_key(&policies), "duplicate matchup");
        for game in pairs.iter_mut().flat_map(|pair| pair.iter_mut()) {
            if let Some(stats) = game.stats.take() {
                self.add_combat(&stats);
            }
        }
        self.pairs.insert(policies, pairs);
    }

    fn add_combat(&mut self, stats: &LabStats) {
        for seat in 0..2 {
            for kind in 0..ALL_KINDS.len() {
                self.combat.dealt[seat][kind] += stats.dealt[seat][kind];
                self.combat.taken[seat][kind] += stats.taken[seat][kind];
                self.combat.healed[seat][kind] += stats.healed[seat][kind];
                self.combat.status_ticks[seat][kind] += stats.status_ticks[seat][kind];
            }
            self.combat.stronghold_dealt[seat] += stats.stronghold_dealt[seat];
        }
    }
}
