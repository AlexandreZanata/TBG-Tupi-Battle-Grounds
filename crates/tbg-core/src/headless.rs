//! The server's match loop without a clock (D-33): views at even steps
//! (10 Hz), each decided intention applied before the next `step()`.
//! Tests, examples, and the balance lab share it, so an offline match is
//! the same match the server would tick every 50 ms — only faster.

use crate::intent::{PlayerIntention, Rejection};
use crate::state::{MatchState, Phase};
use crate::types::Seat;
use crate::view::MatchView;

/// Offline safety bound only: online matches have no gameplay time limit.
/// A match still live after six hours is reported as not ending.
const SAFETY_TICKS: u32 = 20 * 60 * 60 * 6;

/// What a harness can observe while a match is played.
pub enum Moment<'a> {
    /// An intention was processed; its `seq` is the step it was sent at.
    Intention {
        seat: Seat,
        view: &'a MatchView,
        intent: PlayerIntention,
        result: Result<(), Rejection>,
    },
    /// A tick finished.
    Stepped { state: &'a MatchState },
}

/// Play `state` to its end. `decide` sees exactly what a client sees;
/// `observe` gets every processed intention and every finished tick.
/// Returns false if the match did not end within the offline safety bound.
pub fn play(
    state: &mut MatchState,
    decide: impl FnMut(Seat, &MatchView) -> Option<PlayerIntention>,
    observe: impl FnMut(Moment<'_>),
) -> bool {
    play_with(state, |_| {}, decide, observe)
}

/// Same loop as `play`, but calls `before_step` before every tick. The
/// offline value engine uses it to grant Energy at a chosen tick; the
/// online game never calls this with a non-trivial hook.
pub fn play_with(
    state: &mut MatchState,
    mut before_step: impl FnMut(&mut MatchState),
    mut decide: impl FnMut(Seat, &MatchView) -> Option<PlayerIntention>,
    mut observe: impl FnMut(Moment<'_>),
) -> bool {
    let mut events = Vec::new();
    while state.phase != Phase::Ended {
        if state.tick > SAFETY_TICKS {
            return false;
        }
        before_step(state);
        // Views follow the selection clock so seats still decide while an
        // Augment pause freezes the combat tick (BR-AG13).
        if state.clock.is_multiple_of(2) {
            for seat in Seat::BOTH {
                let view = state.view_for(seat, &events);
                if let Some(intent) = decide(seat, &view) {
                    let result = state.apply_intention(seat, state.clock.max(1), intent);
                    observe(Moment::Intention {
                        seat,
                        view: &view,
                        intent,
                        result,
                    });
                }
            }
            events.clear();
        }
        events.extend(state.step());
        observe(Moment::Stepped { state });
    }
    true
}
