//! Offline revocation delivery into real lobby queues, never a runtime auth adapter.

use std::collections::HashMap;

use tbg_core::protocol::{Profile, Token};
use tbg_core::Seat;
use tokio::sync::mpsc;

use super::{add_session, lobby};
use crate::chan::{ConnEvent, ConnHandle, MatchCmd};
use crate::lobby::{Lobby, SessionState};
use crate::store::recovery::proof::ConfirmationBrowser;
use crate::store::recovery::tests::Fixture;
use crate::store::recovery::Recovery;

struct Delivery {
    token: Token,
    close: Option<mpsc::OwnedPermit<ConnEvent>>,
    detach: Option<(mpsc::OwnedPermit<MatchCmd>, Seat, u64)>,
}

/// Epoch stamps model the future trusted admission boundary. They are deliberately
/// not represented as integrated production/P65/P66 authentication support.
struct GenerationFixture {
    lobby: Lobby,
    epochs: HashMap<Token, i64>,
}

impl GenerationFixture {
    fn apply(&mut self, authority: &Recovery, account: i64, epoch: i64) -> Result<(), ()> {
        if !authority.epoch_is_current(account, epoch) {
            return Err(());
        }
        let mut deliveries = Vec::new();
        for (token, session) in &self.lobby.sessions {
            if session
                .account
                .as_ref()
                .is_none_or(|(id, _)| *id != account)
                || self.epochs.get(token).is_some_and(|seen| *seen >= epoch)
            {
                continue;
            }
            let close = session
                .conn
                .as_ref()
                .map(|conn| conn.tx.clone().try_reserve_owned())
                .transpose()
                .map_err(|_| ())?;
            let detach = match (&session.state, &session.conn) {
                (SessionState::InMatch { tx, seat }, Some(conn)) => Some((
                    tx.clone().try_reserve_owned().map_err(|_| ())?,
                    *seat,
                    conn.id,
                )),
                _ => None,
            };
            deliveries.push(Delivery {
                token: *token,
                close,
                detach,
            });
        }
        // Reserve all bounded delivery slots before changing session ownership.
        // Failure leaves the durable command pending instead of acknowledging it.
        for delivery in deliveries {
            if let Some((permit, seat, conn_id)) = delivery.detach {
                permit.send(MatchCmd::Detach { seat, conn_id });
            }
            if let Some(permit) = delivery.close {
                permit.send(ConnEvent::Close);
            }
            self.lobby.end_watch(delivery.token);
            self.lobby.leave_waiting(delivery.token);
            self.lobby.sessions.remove(&delivery.token);
            self.epochs.remove(&delivery.token);
        }
        Ok(())
    }
}

fn attach_account(generation: &mut GenerationFixture, token: Token, account: i64, epoch: i64) {
    generation.lobby.sessions.get_mut(&token).unwrap().account = Some((
        account,
        Profile {
            name: "Keeper".into(),
            rating: 1234,
            ranked_games: 10,
            wins: 9,
            losses: 1,
            draws: 0,
        },
    ));
    generation.epochs.insert(token, epoch);
}

#[test]
fn p67_c15_committed_revocation_reaches_two_offline_game_generations() {
    let mut f = Fixture::new();
    f.verify();
    let proof = f.request(1_000_002);
    let browser = ConfirmationBrowser::new().unwrap();
    let done = f
        .recovery
        .rotate(&proof, &browser, &browser.form(), 1_000_003)
        .unwrap();
    let persisted: (i64, i64) = f
        .recovery
        .conn
        .query_row(
            "SELECT account_id,epoch FROM recovery_revocations",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(persisted, (done.account, done.epoch));
    for _generation in ["current", "draining"] {
        let (lobby, _commands) = lobby();
        let mut generation = GenerationFixture {
            lobby,
            epochs: HashMap::new(),
        };
        let old = [1; 16];
        let fresh = [2; 16];
        let unrelated = [3; 16];
        let (match_tx, mut match_rx) = mpsc::channel(8);
        let mut old_conn = add_session(
            &mut generation.lobby,
            old,
            SessionState::InMatch {
                tx: match_tx,
                seat: Seat::B,
            },
        );
        let mut fresh_conn = add_session(&mut generation.lobby, fresh, SessionState::Idle);
        let mut unrelated_conn = add_session(&mut generation.lobby, unrelated, SessionState::Idle);
        attach_account(&mut generation, old, f.account, 0);
        attach_account(&mut generation, fresh, f.account, 1);
        attach_account(&mut generation, unrelated, f.account + 1, 0);
        generation
            .apply(&f.recovery, persisted.0, persisted.1)
            .unwrap();
        assert!(matches!(old_conn.try_recv(), Ok(ConnEvent::Close)));
        assert!(matches!(
            match_rx.try_recv(),
            Ok(MatchCmd::Detach {
                seat: Seat::B,
                conn_id: 1
            })
        ));
        assert!(!generation.lobby.sessions.contains_key(&old));
        assert!(generation.lobby.sessions.contains_key(&fresh));
        assert!(generation.lobby.sessions.contains_key(&unrelated));
        assert!(fresh_conn.try_recv().is_err());
        assert!(unrelated_conn.try_recv().is_err());
        // A duplicate/delayed outbox command cannot remove a fresh session.
        generation
            .apply(&f.recovery, persisted.0, persisted.1)
            .unwrap();
        assert!(generation.lobby.sessions.contains_key(&fresh));
        // Real hello no longer recognizes the old reconnect credential.
        let (tx, _rx) = mpsc::channel(8);
        let resumed = generation
            .lobby
            .hello(
                Some(old),
                ConnHandle::new(10, tx),
                "127.0.0.1".parse().unwrap(),
            )
            .expect("a fresh guest session");
        assert_ne!(resumed.0, old);
        // An old in-flight login reply fails the modeled epoch admission gate.
        assert!(!f.recovery.epoch_is_current(f.account, 0));
    }
    // Recovery commands contain neither match results nor rating corrections.
    assert_eq!(
        f.recovery
            .conn
            .query_row("SELECT COUNT(*) FROM ranked_matches", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn p67_c17_revocation_queue_failure_keeps_sessions_and_durable_retry_intent() {
    let mut f = Fixture::new();
    f.verify();
    let token = f.request(1_000_002);
    let browser = ConfirmationBrowser::new().unwrap();
    let done = f
        .recovery
        .rotate(&token, &browser, &browser.form(), 1_000_003)
        .unwrap();
    let (lobby, _commands) = lobby();
    let mut generation = GenerationFixture {
        lobby,
        epochs: HashMap::new(),
    };
    let old = [1; 16];
    let mut rx = add_session(&mut generation.lobby, old, SessionState::Queued);
    attach_account(&mut generation, old, f.account, 0);
    let sender = generation.lobby.sessions[&old]
        .conn
        .as_ref()
        .unwrap()
        .tx
        .clone();
    for _ in 0..8 {
        sender
            .try_send(ConnEvent::LeaveMatch)
            .unwrap_or_else(|_| panic!("fixture capacity"));
    }
    assert_eq!(
        generation.apply(&f.recovery, done.account, done.epoch),
        Err(())
    );
    assert!(generation.lobby.sessions.contains_key(&old));
    let count: i64 = f
        .recovery
        .conn
        .query_row("SELECT COUNT(*) FROM recovery_revocations", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
    while rx.try_recv().is_ok() {}
    generation
        .apply(&f.recovery, done.account, done.epoch)
        .unwrap();
    assert!(!generation.lobby.sessions.contains_key(&old));
    assert!(matches!(rx.try_recv(), Ok(ConnEvent::Close)));
}
