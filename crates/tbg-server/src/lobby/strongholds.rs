//! Stronghold kind selection (BR-SK01, D-46): free, accepted only while
//! idle, frozen at pairing like the Arcanas; accounts keep their kind.

use tbg_core::protocol::{ServerMsg, StrongholdRejection, Token};
use tbg_core::StrongholdKind;

use super::{send, Lobby, SessionState};
use crate::chan::ConnEvent;
use crate::store::AccountId;

impl Lobby {
    pub(super) fn set_stronghold(&mut self, token: Token, kind: StrongholdKind) {
        let store = self.store.clone();
        let Some(s) = self.session(&token) else {
            return;
        };
        let reply = if !matches!(s.state, SessionState::Idle) {
            ServerMsg::StrongholdRejected {
                reason: StrongholdRejection::NotIdle,
            }
        } else if !kind.selectable() {
            ServerMsg::StrongholdRejected {
                reason: StrongholdRejection::NotSelectable,
            }
        } else {
            s.stronghold = kind;
            if let (Some((id, _)), Some(store)) = (&s.account, store) {
                store.save_stronghold(*id, kind);
            }
            ServerMsg::StrongholdSelected { kind }
        };
        send(&s.conn, ConnEvent::Send(reply));
    }

    /// An account's saved kind arrived after login: it becomes the
    /// session's choice for the next match (a running match keeps its own).
    pub(super) fn stronghold_loaded(
        &mut self,
        token: Token,
        account: AccountId,
        kind: StrongholdKind,
    ) {
        let Some(s) = self.session(&token) else {
            return;
        };
        // A late database reply cannot change an already frozen selection.
        if matches!(s.state, SessionState::Idle)
            && s.account.as_ref().is_some_and(|(id, _)| *id == account)
        {
            s.stronghold = kind;
            send(
                &s.conn,
                ConnEvent::Send(ServerMsg::StrongholdSelected { kind }),
            );
        }
    }
}
