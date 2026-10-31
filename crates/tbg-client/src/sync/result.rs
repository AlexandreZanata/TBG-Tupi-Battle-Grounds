//! Accept one confirmed terminal record per match; cosmetic errors never replace its Outcome.

use super::MatchInfo;
use tbg_core::{celebration::CastPayload, protocol::ResultId, Outcome};

impl MatchInfo {
    pub(super) fn accept_result(
        &mut self,
        id: ResultId,
        tick: u32,
        outcome: Outcome,
        payload: Option<&CastPayload>,
    ) -> bool {
        if self.result_id != Some(id) || self.outcome.is_some() {
            return false;
        }
        self.last_tick = tick;
        self.celebration = outcome
            .winner
            .filter(|w| Some(*w) == self.you && !self.spectating)
            .and_then(|w| payload.and_then(|p| p.decode(w)));
        self.outcome = Some(outcome);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::{
        celebration::{CelebrationCast, CONTENT_VERSION},
        EndReason, Seat,
    };
    #[test]
    fn celebration_result_is_bound_to_the_current_match_and_accepted_once() {
        for seat in Seat::BOTH {
            let mut info = MatchInfo {
                result_id: Some([63; 16]),
                you: Some(seat),
                ..Default::default()
            };
            let outcome = Outcome {
                winner: Some(seat),
                reason: EndReason::Forfeit,
            };
            let payload = CastPayload::new(&CelebrationCast {
                content_version: CONTENT_VERSION,
                winner: seat,
                entries: vec![],
            })
            .unwrap();
            assert!(!info.accept_result([62; 16], 1, outcome, Some(&payload)));
            assert!(info.accept_result([63; 16], 321, outcome, Some(&payload)));
            assert!(info.celebration.is_some());
            assert!(!info.accept_result([63; 16], 999, outcome, Some(&payload)));
            assert_eq!(info.last_tick, 321);
        }
    }
    #[test]
    fn celebration_loss_draw_spectator_and_unusable_cosmetics_keep_the_real_result() {
        for (winner, spectating, valid) in [
            (Some(Seat::B), false, true),
            (None, false, true),
            (Some(Seat::A), true, true),
            (Some(Seat::A), false, false),
        ] {
            let mut info = MatchInfo {
                result_id: Some([63; 16]),
                you: Some(Seat::A),
                spectating,
                ..Default::default()
            };
            let outcome = Outcome {
                winner,
                reason: EndReason::Forfeit,
            };
            let payload = CastPayload::new(&CelebrationCast {
                content_version: CONTENT_VERSION,
                winner: Seat::A,
                entries: vec![],
            })
            .unwrap();
            assert!(info.accept_result([63; 16], 100, outcome, valid.then_some(&payload)));
            assert_eq!(info.outcome, Some(outcome));
            assert!(info.celebration.is_none());
        }
    }
}
