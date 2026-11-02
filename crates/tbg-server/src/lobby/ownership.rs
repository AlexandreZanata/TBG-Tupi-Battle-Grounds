//! P66 cross-slot account ownership as the lobby sees it (D-92, §6.2): a
//! sign-in claims the account for this slot and is refused while a live
//! session holds it on another slot; an idle disconnect, a logout or a
//! dropped session releases it; a resumed session re-claims it and is
//! signed out if another slot took the account meanwhile. Single-server
//! mode (no `TBG_SLOT`) skips all of this.

use tbg_core::protocol::{AccountError, ServerMsg, Token};

use super::{send, Lobby};
use crate::chan::{ConnEvent, LobbyCmd};
use crate::names::display_name;
use crate::store::release::{self, OwnerConflict};
use crate::store::{AccountId, Store};

/// The slot a sign-in claims for.
#[derive(Clone)]
pub(super) struct Claim {
    slot: String,
    generation: String,
}

/// Claim `account` for this slot; a conflict reads as "finish your match
/// first" (`Unavailable`), never as a takeover of the other slot's seat.
pub(super) async fn claim(
    store: &Store,
    account: AccountId,
    c: &Claim,
) -> Result<(), AccountError> {
    let c = c.clone();
    let claimed = store
        .release(move |conn| {
            release::claim_owner(conn, account, &c.slot, &c.generation, release::now_unix())
        })
        .await;
    match claimed {
        Some(Ok(Ok(_epoch))) => Ok(()),
        Some(Ok(Err(OwnerConflict::OtherSlot))) => Err(AccountError::Unavailable),
        _ => Err(AccountError::Unavailable),
    }
}

impl Lobby {
    pub(super) fn owner_claim(&self) -> Option<Claim> {
        let id = self.release.identity.as_ref()?;
        Some(Claim {
            slot: id.slot.clone(),
            generation: id.generation.clone(),
        })
    }

    /// Fire-and-forget release of this slot's hold on `account`.
    pub(super) fn release_owner(&self, account: AccountId) {
        let (Some(c), Some(store)) = (self.owner_claim(), self.store.clone()) else {
            return;
        };
        tokio::spawn(async move {
            store
                .release(move |conn| release::release_owner(conn, account, &c.slot))
                .await;
        });
    }

    /// A resumed signed-in session claims its account again.
    pub(super) fn reclaim(&self, token: Token, account: AccountId) {
        let (Some(c), Some(store)) = (self.owner_claim(), self.store.clone()) else {
            return;
        };
        let tx = self.self_tx.clone();
        tokio::spawn(async move {
            if claim(&store, account, &c).await.is_err() {
                let _ = tx.send(LobbyCmd::OwnerLost { token, account }).await;
            }
        });
    }

    /// Another slot holds the account now: sign this session out (BR-P03
    /// across slots). A session in a match keeps playing as a guest name.
    pub(super) fn owner_lost(&mut self, token: Token, account: AccountId) {
        let Some(s) = self.session(&token) else {
            return;
        };
        if s.account.as_ref().is_none_or(|(a, _)| *a != account) {
            return;
        }
        self.cancel_direct(token);
        // The opponent agreed to rematch the account, not the guest left behind.
        self.decline_rematch(token);
        let Some(s) = self.session(&token) else {
            return;
        };
        s.player_challenges_enabled = false;
        s.account = None;
        s.name = display_name();
        send(
            &s.conn,
            ConnEvent::Send(ServerMsg::Error {
                code: tbg_core::protocol::ErrorCode::SessionTakenOver,
            }),
        );
        send(&s.conn, ConnEvent::Send(ServerMsg::LoggedOut));
    }
}
