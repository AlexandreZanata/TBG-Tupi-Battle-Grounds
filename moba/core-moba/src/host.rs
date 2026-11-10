//! The per-tick loop every host (server room or offline client) runs.

use crate::bot;
use crate::intent::{MobaIntention, Reject};
use crate::state::{Controller, MobaMatch};
use crate::types::Seat;

/// Clears last tick's events, applies queued human intentions (only for
/// human seats), adds bot intentions for bot seats, then steps once.
/// Returns the rejected human intentions so hosts can tell their senders.
pub fn tick(m: &mut MobaMatch, inputs: &mut Vec<(Seat, MobaIntention)>) -> Vec<(Seat, Reject)> {
    m.events.clear();
    let mut rejects = Vec::new();
    for (seat, intention) in inputs.drain(..) {
        if seat.valid() && m.player(seat).controller == Controller::Human {
            if let Err(reject) = m.apply(seat, intention) {
                rejects.push((seat, reject));
            }
        }
    }
    for seat in Seat::all() {
        if m.player(seat).controller == Controller::Bot {
            if let Some(intention) = bot::decide(m, seat) {
                // Bots only choose affordable actions; a rejection is harmless.
                let _ = m.apply(seat, intention);
            }
        }
    }
    m.step();
    rejects
}
