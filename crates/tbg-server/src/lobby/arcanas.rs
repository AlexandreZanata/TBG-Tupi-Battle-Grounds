//! Arcana selection (BR-AR05, BR-AR06, BR-AR08; D-40): accepted only while
//! idle and validated here; pairing copies the preset into the match, so a
//! later edit can never reach a running match. Accounts keep their preset.

use tbg_core::protocol::{ArcanaRejection, ServerMsg, Token};
use tbg_core::ArcanaPreset;

use super::{send, Lobby, SessionState};
use crate::chan::ConnEvent;
use crate::store::AccountId;

impl Lobby {
    pub(super) fn set_arcanas(&mut self, token: Token, ranks: [u8; 6]) {
        let store = self.store.clone();
        let Some(s) = self.session(&token) else {
            return;
        };
        let reply = if !matches!(s.state, SessionState::Idle) {
            ServerMsg::ArcanasRejected {
                reason: ArcanaRejection::NotIdle,
            }
        } else {
            match ArcanaPreset::complete(ranks) {
                Ok(preset) => {
                    s.arcanas = preset;
                    if let (Some((id, _)), Some(store)) = (&s.account, store) {
                        store.save_arcanas(*id, preset);
                    }
                    ServerMsg::ArcanasSelected { preset }
                }
                Err(_) => ServerMsg::ArcanasRejected {
                    reason: ArcanaRejection::BadRanks,
                },
            }
        };
        send(&s.conn, ConnEvent::Send(reply));
    }

    /// An account's saved preset arrived after login: it becomes the
    /// session's choice for the next match (a running match keeps its own).
    pub(super) fn arcanas_loaded(
        &mut self,
        token: Token,
        account: AccountId,
        preset: ArcanaPreset,
    ) {
        let Some(s) = self.session(&token) else {
            return;
        };
        if s.account.as_ref().is_some_and(|(id, _)| *id == account) {
            s.arcanas = preset;
            send(
                &s.conn,
                ConnEvent::Send(ServerMsg::ArcanasSelected { preset }),
            );
        }
    }
}
