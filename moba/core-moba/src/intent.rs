//! `MobaIntention`: the only way a player (human or bot) affects a match (M-04).

use serde::{Deserialize, Serialize};

use crate::content;
use crate::geom::Pos;
use crate::state::{Event, MobaMatch};
use crate::types::{Destination, Seat};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobaIntention {
    /// Buy the champion in Shop `slot` and send it to a Destination.
    BuyTroop { slot: u8, to: Destination },
    /// Pay to redraw every unlocked Shop offer (M-19).
    Reroll,
    /// Lock or unlock a Shop slot so redraws keep it (free).
    ToggleLock { slot: u8 },
    /// Spend Essence on the next Troop tier.
    Upgrade,
    /// Reserved P76 wire variant; rejected by the strategy-only rules (M-65).
    MoveCommander { to: Pos },
    /// Reserved P76 wire variant; rejected by M-65.
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
    InvalidSlot,
    ManualControlDisabled,
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
            MobaIntention::BuyTroop { slot, to } => self.buy(seat, slot, to),
            MobaIntention::Reroll => self.reroll(seat),
            MobaIntention::ToggleLock { slot } => self.toggle_lock(seat, slot),
            MobaIntention::Upgrade => self.upgrade(seat),
            MobaIntention::MoveCommander { .. } | MobaIntention::CommanderAuto => {
                Err(Reject::ManualControlDisabled)
            }
        }
    }

    fn buy(&mut self, seat: Seat, slot: u8, to: Destination) -> Result<(), Reject> {
        let troop = *self
            .player(seat)
            .offers
            .get(slot as usize)
            .ok_or(Reject::InvalidSlot)?;
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
        // As in the 1v1 Shop, a purchase redraws every offer (BR-S04),
        // except locked ones; the bought slot unlocks.
        player.locked &= !(1 << slot);
        player.offers = crate::shop::redraw(
            &mut player.shop_rng,
            &player.offers,
            player.locked,
            &content::slot_rules(seat.role()),
        );
        crate::sim::spawn::troop(self, seat, troop, to);
        self.events.push(Event::TroopSent { seat, troop, to });
        Ok(())
    }

    fn reroll(&mut self, seat: Seat) -> Result<(), Reject> {
        let player = self.player_mut(seat);
        if player.gold < content::REROLL_COST {
            return Err(Reject::NotEnoughGold);
        }
        player.gold -= content::REROLL_COST;
        player.offers = crate::shop::redraw(
            &mut player.shop_rng,
            &player.offers,
            player.locked,
            &content::slot_rules(seat.role()),
        );
        Ok(())
    }

    fn toggle_lock(&mut self, seat: Seat, slot: u8) -> Result<(), Reject> {
        if slot as usize >= content::OFFERS {
            return Err(Reject::InvalidSlot);
        }
        self.player_mut(seat).locked ^= 1 << slot;
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
}
