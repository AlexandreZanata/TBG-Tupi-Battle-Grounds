//! Fan out converged policy to the bounded set of admitted match tasks.
use super::*;
impl Lobby {
    pub(super) fn social_policy(&mut self) {
        let mut seen: Vec<tokio::sync::mpsc::Sender<MatchCmd>> = Vec::new();
        for s in self.sessions.values() {
            if let SessionState::InMatch { tx, .. } = &s.state {
                if !seen.iter().any(|other| other.same_channel(tx)) {
                    seen.push(tx.clone());
                    let _ = tx.try_send(MatchCmd::SocialPolicy);
                }
            }
        }
    }
}
