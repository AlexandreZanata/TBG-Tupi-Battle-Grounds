//! Re-running a recorded match (D-33): the server records every intention
//! and forfeit with the tick it was applied at; replaying them on the same
//! seed must give the same state, whatever clock drove the original.

use serde::{Deserialize, Serialize};

use crate::arcana::ArcanaPreset;
use crate::content::MATCH_END;
use crate::intent::PlayerIntention;
use crate::state::{MatchState, Phase};
use crate::types::{Seat, Tick};

/// One recorded command, applied before the step of `tick`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayOp {
    pub tick: Tick,
    pub seat: Seat,
    /// `None` is a forfeit (surrender or reconnect window expired).
    pub intent: Option<(u32, PlayerIntention)>,
}

/// Everything needed to rebuild a match; the seed stays server-side (BR-S05).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    pub seed: u64,
    /// D-40: both presets frozen at pairing.
    pub arcanas: [ArcanaPreset; 2],
    pub ops: Vec<ReplayOp>,
    /// `MatchState::hash()` when the original match ended.
    pub final_hash: u64,
}

impl Replay {
    pub fn record(
        &mut self,
        state: &MatchState,
        seat: Seat,
        intent: Option<(u32, PlayerIntention)>,
    ) {
        self.ops.push(ReplayOp {
            tick: state.tick,
            seat,
            intent,
        });
    }

    /// Rebuild the match on the canonical ruleset and the recorded presets.
    pub fn run(&self) -> MatchState {
        let mut state = MatchState::with_arcanas(self.seed, self.arcanas);
        let mut ops = self.ops.iter().peekable();
        loop {
            while let Some(op) = ops.next_if(|op| op.tick == state.tick) {
                match op.intent {
                    Some((seq, intent)) => {
                        let _ = state.apply_intention(op.seat, seq, intent);
                    }
                    None => state.forfeit(op.seat),
                }
            }
            if state.phase == Phase::Ended || state.tick > MATCH_END + 5 {
                return state;
            }
            state.step();
        }
    }
}
