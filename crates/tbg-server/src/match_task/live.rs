//! Progress a listed match reports to the lobby (P47): public numbers only,
//! sent every 20 ticks with `try_send`, so a busy lobby never stalls a match.

use std::time::Instant;

use tbg_core::Seat;

use crate::chan::LobbyCmd;
use crate::live::LiveProgress;

use super::Match;

/// Ticks between progress reports (cosmetic: the panel refreshes at most
/// once a second anyway).
const PROGRESS_EVERY: u32 = 20;

impl Match {
    /// Report progress for a listed match; private matches send nothing.
    pub(super) fn report_progress(
        &self,
        lobby: &tokio::sync::mpsc::Sender<LobbyCmd>,
        started: Instant,
    ) {
        let Some(id) = self.live else {
            return;
        };
        if !self.state.clock.is_multiple_of(PROGRESS_EVERY) {
            return;
        }
        let progress = LiveProgress {
            hp_bp: Seat::BOTH.map(|s| self.hp_bp(s)),
            tiers: Seat::BOTH.map(|s| self.state.player(s).fort_tier),
            elapsed_s: started.elapsed().as_secs() as u32,
            spectators: self.spectators.len() as u16,
        };
        let _ = lobby.try_send(LobbyCmd::LiveProgress { id, progress });
    }

    /// Stronghold HP as basis points of its current maximum (0..=10 000).
    fn hp_bp(&self, seat: Seat) -> u16 {
        let max = i64::from(self.state.stronghold(seat).max_hp.max(1));
        let hp = i64::from(self.state.player(seat).stronghold_hp);
        (hp * 10_000 / max).clamp(0, 10_000) as u16
    }
}
