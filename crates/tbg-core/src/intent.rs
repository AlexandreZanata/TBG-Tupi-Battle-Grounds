//! `PlayerIntention` validation and application (BR-A01, BR-A02).
//! Validation order follows docs/phases/P2-core-simulation.md exactly.

use serde::{Deserialize, Serialize};

use crate::content::{self, EVOLVE_COST, LANE_LEN, OFFERS, SLOT_CAP, SPAWN_OFFSET};
use crate::state::{Event, MatchState, Unit};
use crate::types::{Seat, Status, UnitId};
use crate::{content::effective_stats, shop};

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
    SlotCapReached,
    NotYourUnit,
    UnitGone,
    MaxTier,
    WrongBranch,
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
        let d = content::def(kind);
        if self.player(seat).energy < d.cost {
            return Err(Rejection::NotEnoughEnergy);
        }
        if self.slots_used(seat) + d.slots.count() > SLOT_CAP {
            return Err(Rejection::SlotCapReached);
        }
        let level = self.player(seat).synergy[d.faction.index()];
        let max_hp = effective_stats(kind, 0, 0, level).max_hp;
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
            tier: 0,
            branch: 0,
            x,
            hp: max_hp,
            max_hp,
            cooldown: 0,
            windup: None,
            status: Status::None,
        });
        let p = self.player_mut(seat);
        p.energy -= d.cost;
        p.offers[slot] = shop::draw(&mut p.rng);
        self.pending.push(Event::Spawn {
            unit: id,
            seat,
            kind,
        });
        Ok(())
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
        let cost = EVOLVE_COST[usize::from(u.tier)];
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
