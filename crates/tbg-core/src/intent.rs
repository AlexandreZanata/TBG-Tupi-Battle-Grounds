//! `PlayerIntention` validation and application (BR-A01, BR-A02).
//! Validation order follows docs/phases/P2-core-simulation.md exactly.

use serde::{Deserialize, Serialize};

use crate::content::{LANE_LEN, OFFERS, SPAWN_OFFSET};
use crate::shop;
use crate::state::{Event, MatchState, Unit};
use crate::types::{Seat, Status, UnitId, UnitKind};

/// A request from a client; never a state change until validated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerIntention {
    Purchase { offer: u8 },
    Evolve { unit: UnitId, branch: u8 },
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
        match intent {
            PlayerIntention::Purchase { offer } => self.purchase(seat, offer),
            PlayerIntention::Evolve { unit, branch } => self.evolve(seat, unit, branch),
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
        if self.player(seat).energy < cost {
            return Err(Rejection::NotEnoughEnergy);
        }
        self.spawn(seat, kind, 0, 0);
        let p = self.player_mut(seat);
        p.energy -= cost;
        shop::refill(&mut p.offers, slot, &mut p.rng);
        Ok(())
    }

    /// A new unit in front of the seat's Stronghold, at full HP for its form
    /// and the seat's current Synergy.
    pub(crate) fn spawn(&mut self, seat: Seat, kind: UnitKind, tier: u8, branch: u8) -> UnitId {
        let level = self.player(seat).synergy[self.rules.def(kind).faction.index()];
        let max_hp = self.rules.effective_stats(kind, tier, branch, level).max_hp;
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
            hatch_cd: if kind == UnitKind::Nester {
                self.rules.hatch_rule(tier, branch).0
            } else {
                0
            },
        });
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
        if u.kind.is_summon() {
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
        let cost = self.rules.evolve_cost[usize::from(u.tier)];
        if self.player(seat).energy < cost {
            return Err(Rejection::NotEnoughEnergy);
        }
        self.player_mut(seat).energy -= cost;
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
