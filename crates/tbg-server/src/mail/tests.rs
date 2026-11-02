//! IC-16, OP-02 and S10/S11/S13 synthetic mail and dispatch evidence.

use super::*;
use crate::{
    metrics::Metrics,
    store::{mail_outbox, Store},
};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn capture_delivery_uses_one_lease_and_persists_no_code() {
    let path = std::env::temp_dir().join(format!(
        "p75-mail-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let store = Store::open(&path).unwrap();
    let other = Store::open(&path).unwrap();
    let (account, _) = store
        .register("Mailer".into(), "mailer".into(), [7; 32])
        .await
        .unwrap();
    store.release(move |db| {
        db.execute("INSERT INTO recovery_identity(account_id,email,updated_unix) VALUES(?1,'mail@example.invalid',1000)",[account]).unwrap();
        mail_outbox::enqueue(db,"account_verify","account",account,"mail@example.invalid","en",1000).unwrap();
    }).await.unwrap();
    let capture = transport::MailCapture::default();
    let transport = transport::Transport::Capture(capture.clone());
    let settings = settings::Settings::default();
    let metrics = Metrics::default();
    assert!(
        dispatch::once(&store, &transport, &settings, "a", 1000, &metrics)
            .await
            .unwrap()
    );
    assert!(
        !dispatch::once(&other, &transport, &settings, "b", 1000, &metrics)
            .await
            .unwrap()
    );
    assert_eq!(capture.len(), 1);
    let mail = capture.take().unwrap();
    let code = mail.code.unwrap();
    let raw = crate::identity::codes::parse(&code).unwrap();
    let hash = crate::identity::codes::digest(account, "verify", &raw);
    assert!(store
        .release(move |db| db.query_row(
            "SELECT digest=?1 FROM recovery_challenges WHERE account_id=?2",
            rusqlite::params![hash.as_slice(), account],
            |r| r.get::<_, bool>(0)
        ))
        .await
        .unwrap()
        .unwrap());
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(!bytes.windows(code.len()).any(|v| v == code.as_bytes()));
    assert!(!bytes.windows(raw.len()).any(|v| v == raw.as_bytes()));
    assert!(!mail.body.contains("?code="));
    assert_eq!(metrics.mail_sent.load(Ordering::Relaxed), 1);
    drop((store, other, conn));
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn dispatch_retries_then_gives_up_and_queue_is_bounded() {
    let store = Store::open(std::path::Path::new(":memory:")).unwrap();
    store
        .release(|db| {
            assert!(mail_outbox::enqueue(
                db,
                "account_key_rotated",
                "account",
                1,
                "a@example.invalid",
                "en",
                1000
            )
            .unwrap());
            let mut now = 1000;
            for attempt in 1..=8 {
                let row = mail_outbox::lease(db, "a", now).unwrap().unwrap();
                assert!(mail_outbox::lease(db, "b", now).unwrap().is_none());
                mail_outbox::finish(db, row.id, "a", now, Some("smtp_failed")).unwrap();
                let (status, due): (String, i64) = db
                    .query_row(
                        "SELECT status,not_before_unix FROM mail_outbox WHERE id=?1",
                        [row.id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .unwrap();
                assert_eq!(status, if attempt == 8 { "failed" } else { "pending" });
                assert!(due > now);
                now = due;
            }
            assert!(mail_outbox::lease(db, "a", now).unwrap().is_none());
            for _ in 0..2000 {
                assert!(mail_outbox::enqueue(
                    db,
                    "account_key_rotated",
                    "account",
                    1,
                    "a@example.invalid",
                    "en",
                    now
                )
                .unwrap());
            }
            assert!(!mail_outbox::enqueue(
                db,
                "account_key_rotated",
                "account",
                1,
                "a@example.invalid",
                "en",
                now
            )
            .unwrap());
        })
        .await
        .unwrap();
}

#[test]
fn private_configuration_and_capture_never_debug_secrets() {
    let mut config = crate::Config::default();
    config.bootstrap_email = settings::Private("owner-canary@example.invalid".into());
    config.mail.username = settings::Private("smtp-username-canary".into());
    assert!(!format!("{config:?}").contains("canary"));
    assert!(config.mail.load().is_none(), "unconfigured mail stays idle");
    assert_eq!(
        format!("{:?}", transport::MailCapture::default()),
        "MailCapture([private])"
    );
}
