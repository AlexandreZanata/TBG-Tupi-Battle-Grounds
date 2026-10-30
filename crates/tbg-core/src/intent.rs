//! `PlayerIntention` validation and application (BR-A01, BR-A02).
//! Validation order follows docs/phases/P2-core-simulation.md exactly.

use serde::{Deserialize, Serialize};

use crate::content::{Ability, LANE_LEN, OFFERS, SPAWN_OFFSET};
use crate::shop;
use crate::state::{Event, MatchState, Unit};
use crate::types::{Seat, Status, UnitId, UnitKind};

/// A request from a client; never a state change until validated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerIntention {
    Purchase { offer: u8 },
    Evolve { unit: UnitId, branch: u8 },
    ChooseAugment { round: u32, option: u8 },
    Forgo { round: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    NotRunning,
    BadOffer,
    NotEnoughEnergy,
    NotYourUnit,
    UnitGone,
    MaxTier,
    WrongBranch,
    /// Summons never evolve (BR-U08).
    NotEvolvable,
    /// A repeated or out-of-order intention (BR-A02).
    Stale,
    /// The Augment round is closed, unknown, or already resolved.
    RoundClosed,
    BadOption,
    /// The feature is disabled in this ruleset.
    Unavailable,
    /// An Augment selection pauses the battle: only `ChooseAugment` and
    /// `Forgo` are accepted until it resolves (BR-AG12).
    Paused,
}

impl MatchState {
    pub fn apply_intention(
        &mut self,
        seat: Seat,
        seq: u32,
        intent: PlayerIntention,
    ) -> Result<(), Rejection> {
        self.player_mut(seat).last_seq = seq;
        if !self.is_live() {
            return Err(Rejection::NotRunning);
        }
        let paused = self.augments.selecting();
        match intent {
            PlayerIntention::Purchase { .. } | PlayerIntention::Evolve { .. } if paused => {
                Err(Rejection::Paused)
            }
            PlayerIntention::Purchase { offer } => self.purchase(seat, offer),
            PlayerIntention::Evolve { unit, branch } => self.evolve(seat, unit, branch),
            PlayerIntention::ChooseAugment { round, option } => {
                crate::augment::buy(self, seat, round, option)
            }
            PlayerIntention::Forgo { round } => crate::augment::forgo(self, seat, round),
        }
    }

    fn purchase(&mut self, seat: Seat, offer: u8) -> Result<(), Rejection> {
        let slot = usize::from(offer);
        if slot >= OFFERS {
            return Err(Rejection::BadOffer);
        }
        let kind = self.player(seat).offers[slot];
        let cost = self.rules.def(kind).cost;
        // Energy is the only gate: no army cap (BR-U07).
        if !self.player_mut(seat).spend(cost) {
            return Err(Rejection::NotEnoughEnergy);
        }
        let id = self.spawn(seat, kind, 0, 0);
        let haste = crate::augment::deploy_haste_until(&self.player(seat).augments, self.tick);
        if haste > 0 {
            if let Some(idx) = self.unit_index(id) {
                self.units[idx].haste_until = haste;
            }
        }
        // BR-S04: every purchase redraws all five offers, the bought one too.
        let pool = self.rules.pool;
        let p = self.player_mut(seat);
        p.offers = shop::draw_offers(&mut p.rng, pool);
        Ok(())
    }

    /// A new unit in front of the seat's Stronghold, at full HP for its form
    /// and the seat's current Synergy.
    pub(crate) fn spawn(&mut self, seat: Seat, kind: UnitKind, tier: u8, branch: u8) -> UnitId {
        let max_hp = self.stats_for(seat, kind, tier, branch).max_hp;
        let id = self.next_unit_id;
        self.next_unit_id += 1;
        let x = match seat {
            Seat::A => SPAWN_OFFSET,
            Seat::B => LANE_LEN - SPAWN_OFFSET,
        };
        self.units.push(Unit {
            id,
            seat,
            kind,
            tier,
            branch,
            x,
            hp: max_hp,
            max_hp,
            cooldown: 0,
            windup: None,
            status: Status::None,
            summoner: None,
            expires_at: None,
            haste_until: 0,
            heal_cut_until: 0,
            heal_cut_bp: 0,
            second_wind_used: 0,
            action_cd: if self.rules.def(kind).ability(0, 0).contains(&Ability::Hatch) {
                self.rules.hatch_rule(tier, branch).0
            } else {
                0
            },
            fx: crate::state::UnitFx::default(),
        });
        // P46: Shatterback plates start full (never restored afterwards).
        let [a, b] = self.rules.def(kind).ability(0, 0);
        for ability in [a, b] {
            if let Ability::Plates { count, hp, .. } = ability {
                if let Some(idx) = self.unit_index(id) {
                    self.units[idx].fx.plates = count;
                    self.units[idx].fx.plate_hp = hp as i16;
                }
                break;
            }
        }
        self.pending.push(Event::Spawn {
            unit: id,
            seat,
            kind,
        });
        id
    }

    fn evolve(&mut self, seat: Seat, id: UnitId, branch: u8) -> Result<(), Rejection> {
        let idx = self
            .unit_index(id)
            .filter(|&i| self.units[i].hp > 0)
            .ok_or(Rejection::UnitGone)?;
        let u = self.units[idx];
        if u.seat != seat {
            return Err(Rejection::NotYourUnit);
        }
        if u.is_summon() {
            return Err(Rejection::NotEvolvable);
        }
        if u.tier >= 2 {
            return Err(Rejection::MaxTier);
        }
        let branch_ok = if u.tier == 0 {
            branch <= 1
        } else {
            branch == u.branch
        };
        if !branch_ok {
            return Err(Rejection::WrongBranch);
        }
        let discount = crate::augment::evolve_discount(&self.player(seat).augments);
        let cost = self.rules.evolve_cost[usize::from(u.tier)]
            .saturating_sub(discount)
            .max(crate::content::ENERGY_SCALE);
        if !self.player_mut(seat).spend(cost) {
            return Err(Rejection::NotEnoughEnergy);
        }
        let unit = &mut self.units[idx];
        unit.tier += 1;
        unit.branch = branch;
        let tier = unit.tier;
        self.refresh_max_hp(idx);
        self.pending.push(Event::Evolved {
            unit: id,
            tier,
            branch,
        });
        Ok(())
    }
}
