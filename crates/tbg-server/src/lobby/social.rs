//! Fan out converged policy to the bounded set of admitted match tasks.
use super::*;
impl Lobby {
    pub(super) fn social_policy(&mut self) {
        let mut seen = std::collections::HashSet::new();
        for s in self.sessions.values() {
            if let SessionState::InMatch { tx, .. } = &s.state {
                let id = tx as *const _ as usize;
                if seen.insert(id) {
                    let _ = tx.try_send(MatchCmd::SocialPolicy);
                }
            }
        }
    }
}
