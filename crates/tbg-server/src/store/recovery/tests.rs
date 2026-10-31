//! P67 C12–C17 synthetic evidence against real account SQL and captured mail.

use std::path::PathBuf;
use std::sync::{Arc, Barrier};

use rusqlite::Connection;

use super::email::normalize;
use super::mail::Message;
use super::proof::{digest, ConfirmationBrowser, Post};
use super::{Accepted, Error, Recovery};

const NOW: i64 = 1_000_000;
const EMAIL: &str = "Player+TBG@example.invalid";

pub(crate) struct Fixture {
    pub(crate) recovery: Recovery,
    pub(crate) account: i64,
    pub(crate) key: String,
}

impl Fixture {
    pub(crate) fn new() -> Self {
        Self::on(Connection::open_in_memory().unwrap())
    }

    fn on(mut conn: Connection) -> Self {
        crate::store::schema::migrate(&mut conn).unwrap();
        let secret = tbg_core::account::encode_secret(&[7; 10]);
        let key = tbg_core::account::format_key("Keeper", &secret);
        let (account, _) =
            crate::store::accounts::register(&mut conn, "Keeper", "keeper", &digest(&secret))
                .unwrap();
        Self {
            recovery: Recovery::fixture(conn).unwrap(),
            account,
            key,
        }
    }

    fn enroll(&mut self) -> String {
        self.recovery
            .enroll(
                self.account,
                &self.key,
                EMAIL,
                "127.0.0.1".parse().unwrap(),
                NOW,
            )
            .unwrap();
        match &self.recovery.mail.messages.last().unwrap().message {
            Message::Verify(token) => token.clone(),
            _ => panic!("expected temporary verification proof"),
        }
    }

    pub(crate) fn verify(&mut self) {
        let token = self.enroll();
        let browser = ConfirmationBrowser::new().unwrap();
        self.recovery
            .verify(&token, &browser, &browser.form(), NOW + 1)
            .unwrap();
    }

    pub(crate) fn request(&mut self, now: i64) -> String {
        assert_eq!(
            self.recovery
                .request("Keeper", EMAIL, "127.0.0.1".parse().unwrap(), now),
            Accepted
        );
        match &self.recovery.mail.messages.last().unwrap().message {
            Message::Recover(token) => token.clone(),
            _ => panic!("expected temporary recovery proof"),
        }
    }

    fn login(&self, key: &str) -> bool {
        let (name, secret) = tbg_core::account::parse_key(key).unwrap();
        crate::store::accounts::login(
            &self.recovery.conn,
            &name.to_ascii_lowercase(),
            &digest(&secret),
        )
        .is_ok()
    }
}

struct TempDb(PathBuf);

impl TempDb {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "tbg-p67-{}.sqlite",
            super::proof::random_hex().unwrap()
        )))
    }
    fn open(&self) -> Connection {
        let conn = Connection::open(&self.0).unwrap();
        conn.busy_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        conn
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.0.display()));
        }
    }
}

#[test]
fn p67_c12_enrollment_requires_fresh_key_and_preserves_account_binding() {
    // BR-CO02: billing email and session possession are not account-key proof.
    let mut f = Fixture::new();
    assert!(!f.recovery.verified(f.account));
    assert_eq!(
        f.recovery.enroll(
            f.account,
            "stolen-session",
            EMAIL,
            "127.0.0.1".parse().unwrap(),
            NOW
        ),
        Err(Error::Denied)
    );
    assert_eq!(
        f.recovery.enroll(
            f.account + 1,
            &f.key,
            EMAIL,
            "127.0.0.1".parse().unwrap(),
            NOW
        ),
        Err(Error::Denied)
    );
    assert!(f.recovery.mail.messages.is_empty());
    let token = f.enroll();
    assert!(!f.recovery.verified(f.account));
    f.recovery
        .request("Keeper", EMAIL, "127.0.0.2".parse().unwrap(), NOW + 1);
    assert_eq!(f.recovery.mail.messages.len(), 1);
    let browser = ConfirmationBrowser::new().unwrap();
    f.recovery
        .verify(&token, &browser, &browser.form(), NOW + 2)
        .unwrap();
    assert!(f.recovery.verified(f.account));
    assert_eq!(f.recovery.mail.messages[0].to, EMAIL);
    assert!(f.login(&f.key));
}

#[test]
fn p67_c12_normalization_preserves_local_identity_and_rejects_unsupported_addresses() {
    assert_eq!(normalize(" Player+TBG@EXAMPLE.invalid ").unwrap(), EMAIL);
    assert_ne!(normalize("player+TBG@example.invalid").unwrap(), EMAIL);
    assert_ne!(normalize("Player.TBG@example.invalid").unwrap(), EMAIL);
    for bad in [
        "a\r\nBcc:victim@x.invalid",
        "a@xn--example.invalid",
        "a@@x.invalid",
        "ü@x.invalid",
        ".a@x.invalid",
        "a..b@x.invalid",
        "a@-x.invalid",
        "a@localhost",
        "a@x..invalid",
    ] {
        assert_eq!(normalize(bad), Err(Error::Denied));
    }
}

#[test]
fn p67_c13_generic_bounded_dispatch_does_not_lock_or_rotate_the_victim() {
    let mut f = Fixture::new();
    f.verify();
    for name in ["Unknown", "Keeper", "Keeper", "Keeper", "Keeper"] {
        assert_eq!(
            f.recovery
                .request(name, EMAIL, "127.0.0.2".parse().unwrap(), NOW + 2),
            Accepted
        );
    }
    // Enrollment and the unknown request share the destination budget too.
    assert_eq!(f.recovery.mail.messages.len(), 2);
    let count: usize = f
        .recovery
        .conn
        .query_row("SELECT COUNT(*) FROM recovery_dispatch_limits", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(count <= 256);
    assert!(f.login(&f.key));
    assert!(f.recovery.epoch_is_current(f.account, 0));
    let token = match &f.recovery.mail.messages.last().unwrap().message {
        Message::Recover(t) => t.clone(),
        _ => unreachable!(),
    };
    let browser = ConfirmationBrowser::new().unwrap();
    assert!(f
        .recovery
        .rotate(&token, &browser, &browser.form(), NOW + 3)
        .is_ok());
}

#[test]
fn p67_c13_expiry_purpose_and_one_time_proof_are_enforced() {
    let mut f = Fixture::new();
    let token = f.enroll();
    let browser = ConfirmationBrowser::new().unwrap();
    assert!(matches!(
        f.recovery
            .rotate(&token, &browser, &browser.form(), NOW + 1),
        Err(Error::Denied)
    ));
    assert_eq!(
        f.recovery
            .verify(&token, &browser, &browser.form(), NOW + 24 * 3600),
        Err(Error::Denied)
    );
    assert!(f.login(&f.key));
    f.recovery
        .verify(&token, &browser, &browser.form(), NOW + 1)
        .unwrap();
    assert_eq!(
        f.recovery
            .verify(&token, &browser, &browser.form(), NOW + 2),
        Err(Error::Denied)
    );
    let recovery = f.request(NOW + 2);
    assert_eq!(
        f.recovery
            .verify(&recovery, &browser, &browser.form(), NOW + 3),
        Err(Error::Denied)
    );
    assert!(matches!(
        f.recovery
            .rotate(&recovery, &browser, &browser.form(), NOW + 2 + 900),
        Err(Error::Denied)
    ));
}

#[test]
fn p67_c14_scanner_get_and_unprotected_confirmation_do_not_mutate() {
    let mut f = Fixture::new();
    f.verify();
    let token = f.request(NOW + 2);
    for _ in 0..3 {
        assert!(f.recovery.preview(&token, NOW + 3));
    }
    let browser = ConfirmationBrowser::new().unwrap();
    let other_browser = ConfirmationBrowser::new().unwrap();
    for post in [
        Post {
            method: "GET",
            ..browser.form()
        },
        Post {
            origin: "https://attacker.invalid",
            ..browser.form()
        },
        Post {
            nonce: "",
            ..browser.form()
        },
        other_browser.form(),
    ] {
        assert!(matches!(
            f.recovery.rotate(&token, &browser, &post, NOW + 3),
            Err(Error::Denied)
        ));
    }
    assert!(f.login(&f.key));
    assert!(f.recovery.epoch_is_current(f.account, 0));
    let result = f
        .recovery
        .rotate(&token, &browser, &browser.form(), NOW + 3)
        .unwrap();
    assert_eq!(result.account, f.account);
    assert_eq!(result.epoch, 1);
    assert!(f.login(&result.key));
    assert!(!f.login(&f.key));
    assert!(matches!(
        f.recovery
            .rotate(&token, &browser, &browser.form(), NOW + 4),
        Err(Error::Denied)
    ));
}

#[test]
fn p67_c14_parallel_confirmations_commit_exactly_one_rotation() {
    let db = TempDb::new();
    let mut f = Fixture::on(db.open());
    f.verify();
    let token = f.request(NOW + 2);
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let conn = db.open();
            let token = token.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut recovery = Recovery {
                    conn,
                    mail: Default::default(),
                };
                let browser = ConfirmationBrowser::new().unwrap();
                barrier.wait();
                recovery
                    .rotate(&token, &browser, &browser.form(), NOW + 3)
                    .is_ok()
            })
        })
        .collect();
    let successes = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .filter(|success| *success)
        .count();
    assert_eq!(successes, 1);
    let (epoch, revocations): (i64, i64) = f
        .recovery
        .conn
        .query_row(
            "SELECT epoch,(SELECT COUNT(*) FROM recovery_revocations) FROM recovery_identity",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((epoch, revocations), (1, 1));
}

#[test]
fn p67_c14_lost_completion_requires_fresh_recovery_after_restart() {
    let db = TempDb::new();
    let mut f = Fixture::on(db.open());
    f.verify();
    let token = f.request(NOW + 2);
    let browser = ConfirmationBrowser::new().unwrap();
    drop(
        f.recovery
            .rotate(&token, &browser, &browser.form(), NOW + 3)
            .unwrap(),
    );
    // No stored completion response can retrieve the permanent key.
    f.recovery = Recovery {
        conn: db.open(),
        mail: Default::default(),
    };
    assert!(!f.login(&f.key));
    assert!(matches!(
        f.recovery
            .rotate(&token, &browser, &browser.form(), NOW + 4),
        Err(Error::Denied)
    ));
    let fresh = f.request(NOW + 3601);
    let result = f
        .recovery
        .rotate(&fresh, &browser, &browser.form(), NOW + 3602)
        .unwrap();
    assert_eq!(result.account, f.account);
    assert_eq!(result.epoch, 2);
    assert!(f.login(&result.key));
}

#[test]
fn p67_c15_rotation_preserves_profile_and_source_ownership_and_invalidates_old_epochs() {
    // BR-CO02/BR-C02: Inventory is not implemented; use an explicitly synthetic
    // source-owned fixture table to prove that recovery never transfers IDs.
    let mut f = Fixture::new();
    f.recovery.conn.execute_batch(
        "CREATE TABLE fixture_purchases(account_id INTEGER REFERENCES accounts(id), source TEXT UNIQUE);
         INSERT INTO fixture_purchases VALUES(1,'fixture-source-one');
         UPDATE accounts SET rating=1234,wins=9 WHERE id=1;",
    ).unwrap();
    f.verify();
    let token = f.request(NOW + 2);
    let browser = ConfirmationBrowser::new().unwrap();
    let result = f
        .recovery
        .rotate(&token, &browser, &browser.form(), NOW + 3)
        .unwrap();
    let row: (i64,i64,i64) = f.recovery.conn.query_row(
        "SELECT a.id,a.rating,a.wins FROM accounts a JOIN fixture_purchases p ON p.account_id=a.id",
        [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)),
    ).unwrap();
    assert_eq!(row, (f.account, 1234, 9));
    assert_eq!(result.account, f.account);
    assert!(!f.recovery.epoch_is_current(f.account, 0));
    assert!(f.recovery.epoch_is_current(f.account, 1));
    assert!(!f.recovery.epoch_is_current(f.account + 1, 1));
}

#[test]
fn p67_c16_recent_email_replacement_and_compromise_fail_closed() {
    let mut f = Fixture::new();
    f.verify();
    assert_eq!(
        f.recovery.enroll(
            f.account,
            &f.key,
            "attacker@example.invalid",
            "127.0.0.1".parse().unwrap(),
            NOW + 2
        ),
        Err(Error::TrustedChangeRequired)
    );
    f.recovery.request(
        "Keeper",
        "billing@example.invalid",
        "127.0.0.2".parse().unwrap(),
        NOW + 2,
    );
    assert_eq!(f.recovery.mail.messages.len(), 1);
    let token = f.request(NOW + 3601);
    // A trusted risk signal is injected through fixture SQL, never a player command.
    f.recovery
        .conn
        .execute("UPDATE recovery_identity SET compromised=1", [])
        .unwrap();
    let browser = ConfirmationBrowser::new().unwrap();
    assert!(matches!(
        f.recovery
            .rotate(&token, &browser, &browser.form(), NOW + 3602),
        Err(Error::Denied)
    ));
    f.recovery
        .request("Keeper", EMAIL, "127.0.0.2".parse().unwrap(), NOW + 3602);
    assert_eq!(f.recovery.mail.messages.len(), 2);
    assert!(!f.recovery.verified(f.account));
    assert!(f.login(&f.key));
}

#[test]
fn p67_c17_mail_and_audit_failures_roll_back_without_secret_leakage() {
    let mut f = Fixture::new();
    f.recovery.mail.unavailable = true;
    assert_eq!(
        f.recovery
            .enroll(f.account, &f.key, EMAIL, "127.0.0.1".parse().unwrap(), NOW),
        Err(Error::Unavailable)
    );
    assert!(!f.recovery.verified(f.account));
    f.recovery.mail.unavailable = false;
    f.verify();
    let token = f.request(NOW + 2);
    f.recovery
        .conn
        .execute_batch(
            "CREATE TRIGGER fixture_audit_failure BEFORE INSERT ON recovery_audit
         BEGIN SELECT RAISE(ABORT,'fixture-canary-never-surface'); END;",
        )
        .unwrap();
    let browser = ConfirmationBrowser::new().unwrap();
    let error = f
        .recovery
        .rotate(&token, &browser, &browser.form(), NOW + 3)
        .err()
        .unwrap();
    assert_eq!(format!("{error:?}"), "Unavailable");
    assert!(f.login(&f.key));
    assert!(f.recovery.preview(&token, NOW + 3));
    assert!(f.recovery.epoch_is_current(f.account, 0));
    assert_eq!(f.recovery.mail.messages.len(), 2);
    f.recovery
        .conn
        .execute_batch("DROP TRIGGER fixture_audit_failure")
        .unwrap();
    let done = f
        .recovery
        .rotate(&token, &browser, &browser.form(), NOW + 4)
        .unwrap();
    let stored: Vec<u8> = f
        .recovery
        .conn
        .query_row(
            "SELECT key_hash FROM accounts WHERE id=?1",
            [f.account],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stored.len(), 32);
    assert_ne!(stored, done.key.as_bytes());
    let mut stmt = f
        .recovery
        .conn
        .prepare("SELECT hex(digest),purpose FROM recovery_challenges")
        .unwrap();
    for row in stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
    {
        let (stored, purpose) = row.unwrap();
        assert_ne!(stored.to_lowercase(), token);
        assert_eq!(purpose, "rotate");
    }
    assert!(matches!(
        f.recovery.mail.messages.last().unwrap().message,
        Message::KeyRotated
    ));
    assert_eq!(f.recovery.mail.messages.last().unwrap().to, EMAIL);
}

#[test]
fn p67_c17_storage_and_mail_backpressure_are_bounded_and_non_enumerating() {
    let mut f = Fixture::new();
    f.verify();
    let token = f.request(NOW + 2);
    f.recovery.mail.unavailable = true;
    let browser = ConfirmationBrowser::new().unwrap();
    assert!(matches!(
        f.recovery
            .rotate(&token, &browser, &browser.form(), NOW + 3),
        Err(Error::Unavailable)
    ));
    assert!(f.login(&f.key));
    assert_eq!(
        f.recovery
            .request("Keeper", EMAIL, "127.0.0.2".parse().unwrap(), NOW + 3),
        Accepted
    );
    assert_eq!(
        f.recovery
            .request("Unknown", EMAIL, "127.0.0.2".parse().unwrap(), NOW + 3),
        Accepted
    );
    f.recovery.mail.unavailable = false;
    f.recovery
        .conn
        .execute_batch("DROP TABLE recovery_dispatch_limits")
        .unwrap();
    assert_eq!(
        f.recovery
            .request("Keeper", EMAIL, "127.0.0.2".parse().unwrap(), NOW + 3),
        Accepted
    );
    assert_eq!(
        f.recovery
            .request("Unknown", EMAIL, "127.0.0.2".parse().unwrap(), NOW + 3),
        Accepted
    );
    assert!(f.login(&f.key));
    // Even the already captured proof fails closed if the account authority is gone.
    f.recovery
        .conn
        .execute_batch("DROP TABLE recovery_revocations")
        .unwrap();
    assert!(matches!(
        f.recovery
            .rotate(&token, &browser, &browser.form(), NOW + 3),
        Err(Error::Unavailable)
    ));
    assert!(f.login(&f.key));
}
