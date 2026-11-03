//! `MobaIntention`: the only way a player (human or bot) affects a match (M-04).

use serde::{Deserialize, Serialize};

use crate::content;
use crate::geom::Pos;
use crate::state::{CommandMode, Event, MobaMatch};
use crate::types::{Destination, Seat, TroopKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobaIntention {
    /// "Comprar e Enviar": buy one Troop and send it to a Destination.
    BuyTroop { troop: TroopKind, to: Destination },
    /// Spend Essence on the next Troop tier.
    Upgrade,
    /// Walk the Commander to a point (then Hold there).
    MoveCommander { to: Pos },
    /// Hand the Commander back to its role's Auto behaviour.
    CommanderAuto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reject {
    MatchOver,
    InvalidSeat,
    NotEnoughGold,
    TroopCap,
    NotEnoughEssence,
    MaxTier,
}

impl MobaMatch {
    /// Validates and applies one intention for `seat`. The seat must come from
    /// the host (connection or bot slot), never from the message (M-05).
    pub fn apply(&mut self, seat: Seat, intention: MobaIntention) -> Result<(), Reject> {
        if self.outcome.is_some() {
            return Err(Reject::MatchOver);
        }
        if !seat.valid() {
            return Err(Reject::InvalidSeat);
        }
        match intention {
            MobaIntention::BuyTroop { troop, to } => self.buy(seat, troop, to),
            MobaIntention::Upgrade => self.upgrade(seat),
            MobaIntention::MoveCommander { to } => {
                self.command(seat, CommandMode::Move(to.clamp_to_map()))
            }
            MobaIntention::CommanderAuto => self.command(seat, CommandMode::Auto),
        }
    }

    fn buy(&mut self, seat: Seat, troop: TroopKind, to: Destination) -> Result<(), Reject> {
        let cost = content::troop_cost(troop);
        if self.player(seat).gold < cost {
            return Err(Reject::NotEnoughGold);
        }
        if self.troops_alive(seat) >= content::TROOP_CAP {
            return Err(Reject::TroopCap);
        }
        let player = self.player_mut(seat);
        player.gold -= cost;
        player.destination = to;
        crate::sim::spawn::troop(self, seat, troop, to);
        self.events.push(Event::TroopSent { seat, troop, to });
        Ok(())
    }

    fn upgrade(&mut self, seat: Seat) -> Result<(), Reject> {
        let player = self.player_mut(seat);
        if player.tier >= content::MAX_TIER {
            return Err(Reject::MaxTier);
        }
        let cost = content::upgrade_cost(player.tier);
        if player.essence < cost {
            return Err(Reject::NotEnoughEssence);
        }
        player.essence -= cost;
        player.tier += 1;
        let tier = player.tier;
        self.events.push(Event::Upgraded { seat, tier });
        Ok(())
    }

    fn command(&mut self, seat: Seat, mode: CommandMode) -> Result<(), Reject> {
        let id = self.player(seat).commander;
        self.player_mut(seat).mode = mode;
        // A dead Commander keeps the order and obeys it after respawning.
        if let Some(slot) = self.slot(id) {
            if let crate::state::Brain::Commander {
                mode: m,
                retreating,
                ..
            } = &mut self.units[slot].brain
            {
                *m = mode;
                *retreating = false;
            }
            self.units[slot].target = 0;
        }
        Ok(())
    }
}
