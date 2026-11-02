//! Executable offline recovery contract on the real account table, compiled only in tests.

use std::net::IpAddr;

use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use tbg_core::account::{encode_secret, format_key, parse_key};

use super::AccountId;

mod challenges;
pub(crate) mod email;
mod limits;
pub(crate) mod mail;
pub(crate) mod proof;
#[cfg(test)]
pub(crate) mod tests;

use challenges::{challenge, consume, issue};
use email::normalize;
use limits::admit;
use mail::{Capture, Message};
use proof::{digest, ConfirmationBrowser, Post};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Denied,
    TrustedChangeRequired,
    Unavailable,
}

impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::Unavailable
    }
}

// No plaintext key or token derives Debug or gets persisted in an outbox.
pub(crate) struct Completion {
    pub(crate) account: AccountId,
    pub(crate) epoch: i64,
    pub(crate) key: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Accepted;

pub(crate) struct Recovery {
    pub(crate) conn: Connection,
    pub(crate) mail: Capture,
}

impl Recovery {
    /// Callers are isolated fixtures. Live Store::open never calls this installer.
    pub(crate) fn fixture(mut conn: Connection) -> Result<Self, Error> {
        super::schema::migrate(&mut conn)?;
        Ok(Self {
            conn,
            mail: Capture::default(),
        })
    }

    pub(crate) fn enroll(
        &mut self,
        account: AccountId,
        key: &str,
        address: &str,
        ip: IpAddr,
        now: i64,
    ) -> Result<(), Error> {
        let email = normalize(address)?;
        self.mail.reserve()?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        fresh_key(&tx, account, key)?;
        tx.execute(
            "INSERT OR IGNORE INTO recovery_identity(account_id) VALUES (?1)",
            [account],
        )?;
        let (verified, compromised): (Option<String>, bool) = tx.query_row(
            "SELECT verified_email,compromised FROM recovery_identity WHERE account_id=?1",
            [account],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if compromised || verified.is_some() {
            return Err(Error::TrustedChangeRequired);
        }
        admit(&tx, &account.to_string(), &email, ip, now)?;
        let token = issue(&tx, account, "verify", &email, now, 24 * 3600)?;
        tx.commit()?;
        self.mail.deliver(email, Message::Verify(token));
        Ok(())
    }

    /// Every request, including throttle/mail/DB failure, has the same public result.
    pub(crate) fn request(&mut self, name: &str, email: &str, ip: IpAddr, now: i64) -> Accepted {
        let _ = self.try_request(name, email, ip, now);
        Accepted
    }

    fn try_request(
        &mut self,
        name: &str,
        address: &str,
        ip: IpAddr,
        now: i64,
    ) -> Result<(), Error> {
        if !tbg_core::account::valid_name_shape(name) {
            return Err(Error::Denied);
        }
        let email = normalize(address)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        admit(&tx, &name.to_ascii_lowercase(), &email, ip, now)?;
        let account: Option<AccountId> = tx
            .query_row(
                "SELECT a.id FROM accounts a JOIN recovery_identity i ON a.id=i.account_id
             WHERE a.name_lower=?1 AND i.verified_email=?2 AND i.compromised=0",
                params![name.to_ascii_lowercase(), email],
                |r| r.get(0),
            )
            .optional()?;
        let token = match account {
            Some(account) => {
                self.mail.reserve()?;
                Some(issue(&tx, account, "rotate", &email, now, 15 * 60)?)
            }
            None => None,
        };
        tx.commit()?;
        if let Some(token) = token {
            self.mail.deliver(email, Message::Recover(token));
        }
        Ok(())
    }

    /// GET/preview does not create, consume, rotate, revoke, or deliver anything.
    pub(crate) fn preview(&self, token: &str, now: i64) -> bool {
        if token.len() != 64 {
            return false;
        }
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM recovery_challenges c JOIN recovery_identity i
             ON c.account_id=i.account_id WHERE c.digest=?1 AND c.consumed=0 AND c.expires>?2
             AND c.epoch=i.epoch AND i.compromised=0)",
                params![digest(token.as_bytes()).as_slice(), now],
                |r| r.get(0),
            )
            .unwrap_or(false)
    }

    pub(crate) fn verify(
        &mut self,
        token: &str,
        browser: &ConfirmationBrowser,
        post: &Post<'_>,
        now: i64,
    ) -> Result<(), Error> {
        browser.validate(post)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (account, email, epoch) = challenge(&tx, token, "verify", now)?;
        let changed = tx.execute(
            "UPDATE recovery_identity SET verified_email=?2 WHERE account_id=?1
             AND verified_email IS NULL AND compromised=0",
            params![account, email],
        )?;
        if changed != 1 {
            return Err(Error::TrustedChangeRequired);
        }
        consume(&tx, account)?;
        tx.execute(
            "INSERT INTO recovery_audit VALUES (?1,?2,'verified')",
            params![account, epoch],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn rotate(
        &mut self,
        token: &str,
        browser: &ConfirmationBrowser,
        post: &Post<'_>,
        now: i64,
    ) -> Result<Completion, Error> {
        browser.validate(post)?;
        self.mail.reserve()?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (account, email, epoch) = challenge(&tx, token, "rotate", now)?;
        let next = epoch.checked_add(1).ok_or(Error::Unavailable)?;
        let name: String =
            tx.query_row("SELECT name FROM accounts WHERE id=?1", [account], |r| {
                r.get(0)
            })?;
        let mut bytes = [0u8; 10];
        getrandom::fill(&mut bytes).map_err(|_| Error::Unavailable)?;
        let secret = encode_secret(&bytes);
        let key = format_key(&name, &secret);
        tx.execute(
            "UPDATE accounts SET key_hash=?2 WHERE id=?1",
            params![account, digest(&secret).as_slice()],
        )?;
        tx.execute(
            "UPDATE recovery_identity SET epoch=?2 WHERE account_id=?1",
            params![account, next],
        )?;
        consume(&tx, account)?;
        tx.execute(
            "INSERT INTO recovery_audit VALUES (?1,?2,'rotated')",
            params![account, next],
        )?;
        tx.execute(
            "INSERT INTO recovery_revocations VALUES (?1,?2)",
            params![account, next],
        )?;
        tx.commit()?;
        self.mail.deliver(email, Message::KeyRotated);
        Ok(Completion {
            account,
            epoch: next,
            key,
        })
    }

    pub(crate) fn epoch_is_current(&self, account: AccountId, epoch: i64) -> bool {
        self.conn
            .query_row(
                "SELECT epoch=?2 AND compromised=0 FROM recovery_identity WHERE account_id=?1",
                params![account, epoch],
                |r| r.get(0),
            )
            .unwrap_or(false)
    }

    pub(crate) fn verified(&self, account: AccountId) -> bool {
        self.conn.query_row(
            "SELECT verified_email IS NOT NULL AND compromised=0 FROM recovery_identity WHERE account_id=?1",
            [account], |r| r.get(0),
        ).unwrap_or(false)
    }
}

fn fresh_key(conn: &Connection, account: AccountId, key: &str) -> Result<(), Error> {
    let (name, secret) = parse_key(key).ok_or(Error::Denied)?;
    let (id, _) = super::accounts::login(conn, &name.to_ascii_lowercase(), &digest(&secret))
        .map_err(|_| Error::Denied)?;
    if id != account {
        return Err(Error::Denied);
    }
    Ok(())
}
