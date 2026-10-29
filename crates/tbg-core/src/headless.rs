//! The server's match loop without a clock (D-33): views at even ticks
//! (10 Hz), each decided intention applied before the next `step()`.
//! Tests, examples, and the balance lab share it, so an offline match is
//! the same match the server would tick every 50 ms — only faster.

use crate::content::MATCH_END;
use crate::intent::{PlayerIntention, Rejection};
use crate::state::{MatchState, Phase};
use crate::types::Seat;
use crate::view::MatchView;

/// Ticks past `MATCH_END` after which a match counts as not ending.
const GRACE: u32 = 5;

/// What a harness can observe while a match is played.
pub enum Moment<'a> {
    /// An intention was processed; its `seq` is the tick it was sent at.
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
/// Returns false if the match did not end in time (a simulation bug).
pub fn play(
    state: &mut MatchState,
    mut decide: impl FnMut(Seat, &MatchView) -> Option<PlayerIntention>,
    mut observe: impl FnMut(Moment<'_>),
) -> bool {
    let mut events = Vec::new();
    while state.phase != Phase::Ended {
        if state.tick > MATCH_END + GRACE {
            return false;
        }
        if state.tick.is_multiple_of(2) {
            for seat in Seat::BOTH {
                let view = state.view_for(seat, &events);
                if let Some(intent) = decide(seat, &view) {
                    let result = state.apply_intention(seat, state.tick, intent);
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
