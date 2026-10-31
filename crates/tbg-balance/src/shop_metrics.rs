//! Reservation conversion age and Reroll spending observed from accepted intentions.
use crate::metrics::Side;
use tbg_core::{content::OFFERS, MatchView, PlayerIntention};

#[derive(Default)]
pub(crate) struct Journal {
    shop: u32,
    started: [Option<u32>; OFFERS],
    streak: u8,
}
impl Journal {
    pub fn record(&mut self, side: &mut Side, view: &MatchView, intent: PlayerIntention) {
        match intent {
            PlayerIntention::Reserve { offer, .. } => {
                self.started[usize::from(offer)] = Some(self.shop);
                side.reservations += 1;
            }
            PlayerIntention::Purchase { offer } => {
                let i = usize::from(offer);
                if let Some(start) = self.started[i].take() {
                    side.reserved_bought += 1;
                    side.reserve_wait_shops += self.shop.saturating_sub(start);
                }
                self.streak = 0;
                self.redraw(view, Some(i));
            }
            PlayerIntention::Reroll { cost } => {
                side.rerolls += 1;
                side.reroll_energy += u64::from(cost);
                side.spent += u64::from(cost);
                self.streak = self.streak.saturating_add(1);
                side.max_reroll_streak = side.max_reroll_streak.max(self.streak);
                self.redraw(view, None);
            }
            _ => {}
        }
    }
    fn redraw(&mut self, view: &MatchView, bought: Option<usize>) {
        self.shop = self.shop.saturating_add(1);
        for (i, start) in self.started.iter_mut().enumerate() {
            if Some(i) == bought || view.reserve[i] == 0 {
                *start = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::{ArcanaPreset, MatchState, Phase, Seat};
    #[test]
    fn release_then_purchase_counts_conversion_without_inventing_wait() {
        let mut state = MatchState::with_preview(1, [ArcanaPreset::NEUTRAL; 2]);
        state.phase = Phase::Running;
        state.players[0].energy = 10_000;
        let kind = state.players[0].offers[0];
        let mut side = Side::default();
        let mut journal = Journal::default();
        for intent in [
            PlayerIntention::Reserve { offer: 0, kind },
            PlayerIntention::Release { offer: 0 },
            PlayerIntention::Purchase { offer: 0 },
        ] {
            let view = state.view_for(Seat::A, &[]);
            state.apply_intention(Seat::A, 0, intent).unwrap();
            journal.record(&mut side, &view, intent);
        }
        assert_eq!(
            (
                side.reservations,
                side.reserved_bought,
                side.reserve_wait_shops
            ),
            (1, 1, 0)
        );
    }
}
