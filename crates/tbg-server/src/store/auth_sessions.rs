//! Transactional, hash-only device families and refresh history (D-97/BR-AS02–03).

use rusqlite::{params, Connection, OptionalExtension};
use tbg_core::protocol::{AccountError, Profile};

use super::{accounts, ranked, AccountId};

pub type FamilyId = [u8; 16];
pub type CredentialHash = [u8; 32];
pub const FAMILY_SECONDS: i64 = 2_592_000;

pub const TABLES: &str = "
CREATE TABLE auth_sessions (
 id BLOB PRIMARY KEY NOT NULL CHECK(length(id)=16),
 account_id INTEGER NOT NULL REFERENCES accounts(id),
 device_id BLOB NOT NULL CHECK(length(device_id)=16),
 client_kind TEXT NOT NULL CHECK(client_kind IN ('browser','native')),
 keep_signed_in INTEGER NOT NULL CHECK(keep_signed_in IN (0,1)),
 created_at INTEGER NOT NULL CHECK(created_at>=0), expires_at INTEGER NOT NULL CHECK(expires_at>created_at),
 last_used_at INTEGER, revoked_at INTEGER, revoke_reason TEXT
);
CREATE TABLE auth_refresh_tokens (
 token_hash BLOB PRIMARY KEY NOT NULL CHECK(length(token_hash)=32),
 session_id BLOB NOT NULL REFERENCES auth_sessions(id) ON DELETE CASCADE,
 generation INTEGER NOT NULL CHECK(generation>=0), issued_at INTEGER NOT NULL,
 consumed_at INTEGER, operation_hash BLOB CHECK(operation_hash IS NULL OR length(operation_hash)=32),
 CHECK((consumed_at IS NULL AND operation_hash IS NULL) OR
       (consumed_at IS NOT NULL AND operation_hash IS NOT NULL)),
 UNIQUE(session_id,generation)
);
";

/// Idempotent so every open re-verifies the single-current-refresh invariant.
pub const INDEXES: &str = "
CREATE INDEX IF NOT EXISTS auth_sessions_account ON auth_sessions(account_id,expires_at);
CREATE INDEX IF NOT EXISTS auth_sessions_expiry ON auth_sessions(expires_at);
CREATE UNIQUE INDEX IF NOT EXISTS auth_refresh_current ON auth_refresh_tokens(session_id) WHERE consumed_at IS NULL;
CREATE INDEX IF NOT EXISTS auth_refresh_history ON auth_refresh_tokens(session_id,consumed_at);
CREATE INDEX IF NOT EXISTS auth_sessions_revoked ON auth_sessions(revoked_at) WHERE revoked_at IS NOT NULL;
";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Browser,
    Native,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Native => "native",
        }
    }
}

#[derive(Clone)]
pub struct Family {
    pub id: FamilyId,
    pub account: AccountId,
    pub device: FamilyId,
    pub channel: Channel,
    pub persistent: bool,
    pub created_at: i64,
    pub expires_at: i64,
}

pub struct Issuance {
    pub name: String,
    pub key_hash: CredentialHash,
    pub register: bool,
    pub family: FamilyId,
    pub device: FamilyId,
    pub channel: Channel,
    pub persistent: bool,
    pub refresh_hash: CredentialHash,
    pub now: i64,
}

pub enum Request {
    Issue(Issuance),
    IssueEmail {
        issue: Issuance,
        email: super::recovery::service::Enrollment,
    },
    Rotate {
        old_hash: CredentialHash,
        new_hash: CredentialHash,
        operation_hash: CredentialHash,
        channel: Channel,
        now: i64,
    },
    Confirm {
        hash: CredentialHash,
        channel: Channel,
        now: i64,
    },
    LogoutFamily {
        hash: CredentialHash,
        channel: Channel,
    },
    Inspect {
        hash: CredentialHash,
        channel: Channel,
        now: i64,
    },
    Families {
        name: String,
        key_hash: CredentialHash,
        now: i64,
    },
    Revoke {
        families: Vec<FamilyId>,
        now: i64,
        reason: &'static str,
    },
    RevokeAccount {
        account: AccountId,
        now: i64,
    },
    Cleanup {
        now: i64,
    },
    /// P71 (W3): families revoked at or after `since`, oldest first.
    RevokedSince {
        since: i64,
    },
}

pub enum Reply {
    LogoutFamily(Family),
    Grant {
        family: Family,
        profile: Profile,
        high_water: i64,
    },
    Consumed {
        family: Family,
        at: i64,
        operation_matches: bool,
        successor: CredentialHash,
    },
    Families(Vec<FamilyId>),
    Revoked(Vec<(FamilyId, i64)>),
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Suspended {
        until: Option<u64>,
        reason: tbg_core::protocol::SuspensionReason,
    },
    RegistrationClosed,
    EmailLimit,
    EmailBlocked,
    EmailInvalid,
    Unauthorized,
    Unavailable,
    NameTaken,
    FamilyLimit,
}

impl super::Store {
    /// Hash-only auth work uses the existing bounded writer, never Tokio SQL.
    pub async fn auth(&self, request: Request) -> Result<Reply, Error> {
        let (reply, rx) = tokio::sync::oneshot::channel();
        self.tx
            .try_send(super::worker::StoreCmd::Auth {
                request: Box::new(request),
                reply,
            })
            .map_err(|_| Error::Unavailable)?;
        rx.await.map_err(|_| Error::Unavailable)?
    }
}

impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::Unavailable
    }
}

impl From<AccountError> for Error {
    fn from(value: AccountError) -> Self {
        match value {
            AccountError::NameTaken => Self::NameTaken,
            AccountError::BadKey => Self::Unauthorized,
            _ => Self::Unavailable,
        }
    }
}

pub(super) fn apply(conn: &mut Connection, request: Request) -> Result<Reply, Error> {
    match request {
        Request::Issue(issue) => issue_family(conn, issue, None),
        Request::IssueEmail { issue, email } => issue_family(conn, issue, Some(email)),
        Request::Rotate {
            old_hash,
            new_hash,
            operation_hash,
            channel,
            now,
        } => rotate(conn, old_hash, new_hash, operation_hash, channel, now),
        Request::Confirm { hash, channel, now } => {
            let row = lookup(conn, hash, channel, now)?;
            if row.consumed.is_some() {
                return Err(Error::Unauthorized);
            }
            grant(conn, row.family)
        }
        Request::LogoutFamily { hash, channel } => logout_family(conn, hash, channel),
        Request::Inspect { hash, channel, now } => {
            let row = lookup(conn, hash, channel, now)?;
            if let Some(at) = row.consumed {
                consumed(conn, row, at, None)
            } else {
                grant(conn, row.family)
            }
        }
        Request::Families {
            name,
            key_hash,
            now,
        } => {
            let (account, _) = accounts::login(conn, &name.to_ascii_lowercase(), &key_hash)?;
            let mut statement=conn.prepare("SELECT id FROM auth_sessions WHERE account_id=?1 AND revoked_at IS NULL AND expires_at>?2")?;
            let rows = statement.query_map(params![account, now], |r| r.get::<_, Vec<u8>>(0))?;
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row?.try_into().map_err(|_| Error::Unavailable)?);
            }
            Ok(Reply::Families(ids))
        }
        Request::Revoke {
            families,
            now,
            reason,
        } => {
            if families.len() > 10 {
                return Err(Error::Unavailable);
            }
            let tx = conn.transaction()?;
            for id in families {
                tx.execute("UPDATE auth_sessions SET revoked_at=COALESCE(revoked_at,?1),revoke_reason=?2 WHERE id=?3",params![now,reason,id.as_slice()])?;
            }
            tx.commit()?;
            Ok(Reply::Done)
        }
        Request::Cleanup { now } => cleanup(conn, now),
        Request::RevokedSince { since } => {
            let mut statement = conn.prepare(
                "SELECT id,revoked_at FROM auth_sessions WHERE revoked_at>=?1 ORDER BY revoked_at LIMIT 256",
            )?;
            let rows = statement.query_map(params![since], |r| {
                Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, i64>(1)?))
            })?;
            let mut revoked = Vec::new();
            for row in rows {
                let (id, at) = row?;
                revoked.push((id.try_into().map_err(|_| Error::Unavailable)?, at));
            }
            Ok(Reply::Revoked(revoked))
        }
        Request::RevokeAccount { account, now } => {
            conn.execute("UPDATE auth_sessions SET revoked_at=?1,revoke_reason='logout_all' WHERE account_id=?2 AND revoked_at IS NULL",params![now,account])?;
            Ok(Reply::Done)
        }
    }
}

fn issue_family(
    conn: &mut Connection,
    issue: Issuance,
    email: Option<super::recovery::service::Enrollment>,
) -> Result<Reply, Error> {
    let tx = conn.transaction()?;
    if let Some(input) = &email {
        super::recovery::service::registration(&tx, &input.email, &input.locale)
            .map_err(Error::from)?;
    }
    let lower = issue.name.to_ascii_lowercase();
    let (account, profile) = if issue.register {
        accounts::register_in_transaction(&tx, &issue.name, &lower, &issue.key_hash)?
    } else {
        accounts::login(&tx, &lower, &issue.key_hash)?
    };
    if let Some(input) = email {
        super::recovery::service::enroll(&tx, account, &input, issue.now).map_err(Error::from)?;
    }
    guard_ban(&tx, account, issue.now)?;
    let count:u32=tx.query_row("SELECT COUNT(*) FROM auth_sessions WHERE account_id=?1 AND revoked_at IS NULL AND expires_at>?2",params![account,issue.now],|r|r.get(0))?;
    if count >= 10 {
        return Err(Error::FamilyLimit);
    }
    let expires = issue
        .now
        .checked_add(FAMILY_SECONDS)
        .ok_or(Error::Unavailable)?;
    tx.execute("INSERT INTO auth_sessions(id,account_id,device_id,client_kind,created_at,expires_at,last_used_at,keep_signed_in) VALUES(?1,?2,?3,?4,?5,?6,?5,?7)",params![issue.family.as_slice(),account,issue.device.as_slice(),issue.channel.as_str(),issue.now,expires,issue.persistent])?;
    tx.execute("INSERT INTO auth_refresh_tokens(token_hash,session_id,generation,issued_at) VALUES(?1,?2,0,?3)",params![issue.refresh_hash.as_slice(),issue.family.as_slice(),issue.now])?;
    let high_water = ranked::high_water(&tx)?;
    tx.commit()?;
    Ok(Reply::Grant {
        family: Family {
            id: issue.family,
            account,
            device: issue.device,
            channel: issue.channel,
            persistent: issue.persistent,
            created_at: issue.now,
            expires_at: expires,
        },
        profile,
        high_water,
    })
}

struct Row {
    family: Family,
    generation: i64,
    consumed: Option<i64>,
    operation: Option<CredentialHash>,
}

fn lookup(
    conn: &Connection,
    hash: CredentialHash,
    channel: Channel,
    now: i64,
) -> Result<Row, Error> {
    let row=conn.query_row("SELECT s.id,s.account_id,s.device_id,s.created_at,s.expires_at,t.generation,t.consumed_at,t.operation_hash,s.keep_signed_in FROM auth_refresh_tokens t JOIN auth_sessions s ON s.id=t.session_id WHERE t.token_hash=?1 AND s.client_kind=?2 AND s.revoked_at IS NULL AND s.expires_at>?3",params![hash.as_slice(),channel.as_str(),now],|r| {
        Ok((r.get::<_,Vec<u8>>(0)?,r.get(1)?,r.get::<_,Vec<u8>>(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get::<_,Option<Vec<u8>>>(7)?,r.get::<_,bool>(8)?))
    }).optional()?.ok_or(Error::Unauthorized)?;
    guard_ban(conn, row.1, now)?;
    Ok(Row {
        family: Family {
            id: row.0.try_into().map_err(|_| Error::Unavailable)?,
            account: row.1,
            device: row.2.try_into().map_err(|_| Error::Unavailable)?,
            channel,
            persistent: row.8,
            created_at: row.3,
            expires_at: row.4,
        },
        generation: row.5,
        consumed: row.6,
        operation: row
            .7
            .map(|v| v.try_into())
            .transpose()
            .map_err(|_| Error::Unavailable)?,
    })
}

fn grant(conn: &Connection, family: Family) -> Result<Reply, Error> {
    let (_, profile) = conn.query_row(
        &format!("SELECT {} FROM accounts WHERE id=?1", accounts::SELECT_COLS),
        params![family.account],
        accounts::profile_from_row,
    )?;
    Ok(Reply::Grant {
        family,
        profile,
        high_water: ranked::high_water(conn)?,
    })
}

fn consumed(
    conn: &Connection,
    row: Row,
    at: i64,
    operation: Option<CredentialHash>,
) -> Result<Reply, Error> {
    let successor: Vec<u8> = conn.query_row(
        "SELECT token_hash FROM auth_refresh_tokens WHERE session_id=?1 AND consumed_at IS NULL",
        params![row.family.id.as_slice()],
        |r| r.get(0),
    )?;
    Ok(Reply::Consumed {
        family: row.family,
        at,
        operation_matches: row.operation == operation,
        successor: successor.try_into().map_err(|_| Error::Unavailable)?,
    })
}

fn rotate(
    conn: &mut Connection,
    old_hash: CredentialHash,
    new_hash: CredentialHash,
    operation_hash: CredentialHash,
    channel: Channel,
    now: i64,
) -> Result<Reply, Error> {
    let tx = conn.transaction()?;
    let row = lookup(&tx, old_hash, channel, now)?;
    if let Some(at) = row.consumed {
        return consumed(&tx, row, at, Some(operation_hash));
    }
    let changed=tx.execute("UPDATE auth_refresh_tokens SET consumed_at=?1,operation_hash=?2 WHERE token_hash=?3 AND consumed_at IS NULL",params![now,operation_hash.as_slice(),old_hash.as_slice()])?;
    if changed != 1 {
        return Err(Error::Unavailable);
    }
    tx.execute("INSERT INTO auth_refresh_tokens(token_hash,session_id,generation,issued_at) VALUES(?1,?2,?3,?4)",params![new_hash.as_slice(),row.family.id.as_slice(),row.generation.checked_add(1).ok_or(Error::Unavailable)?,now])?;
    tx.execute(
        "UPDATE auth_sessions SET last_used_at=?1 WHERE id=?2",
        params![now, row.family.id.as_slice()],
    )?;
    let result = grant(&tx, row.family)?;
    tx.commit()?;
    Ok(result)
}

fn cleanup(conn: &mut Connection, now: i64) -> Result<Reply, Error> {
    let tx = conn.transaction()?;
    let removed=tx.execute("DELETE FROM auth_refresh_tokens WHERE token_hash IN (SELECT t.token_hash FROM auth_sessions s JOIN auth_refresh_tokens t ON s.id=t.session_id WHERE s.expires_at<=?1 LIMIT 256)",params![now])?;
    tx.execute("DELETE FROM auth_sessions WHERE id IN (SELECT s.id FROM auth_sessions s WHERE s.expires_at<=?1 AND NOT EXISTS (SELECT 1 FROM auth_refresh_tokens t WHERE t.session_id=s.id) LIMIT ?2)",params![now,i64::try_from(256_usize.saturating_sub(removed)).map_err(|_| Error::Unavailable)?])?;
    tx.commit()?;
    Ok(Reply::Done)
}

/// A current hash proves only its own logout retry, even after revocation.
/// This path never returns a profile, Access or a renewable grant.
fn logout_family(
    conn: &Connection,
    hash: CredentialHash,
    channel: Channel,
) -> Result<Reply, Error> {
    let row=conn.query_row("SELECT s.id,s.account_id,s.device_id,s.created_at,s.expires_at,s.keep_signed_in FROM auth_refresh_tokens t JOIN auth_sessions s ON s.id=t.session_id WHERE t.token_hash=?1 AND t.consumed_at IS NULL AND s.client_kind=?2",params![hash.as_slice(),channel.as_str()],|r|Ok((r.get::<_,Vec<u8>>(0)?,r.get(1)?,r.get::<_,Vec<u8>>(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional()?.ok_or(Error::Unauthorized)?;
    Ok(Reply::LogoutFamily(Family {
        id: row.0.try_into().map_err(|_| Error::Unavailable)?,
        account: row.1,
        device: row.2.try_into().map_err(|_| Error::Unavailable)?,
        channel,
        created_at: row.3,
        expires_at: row.4,
        persistent: row.5,
    }))
}

impl From<super::recovery::Error> for Error {
    fn from(value: super::recovery::Error) -> Self {
        use super::recovery::Error as E;
        match value {
            E::EmailInvalid => Self::EmailInvalid,
            E::EmailBlocked => Self::EmailBlocked,
            E::EmailLimit => Self::EmailLimit,
            E::RegistrationClosed => Self::RegistrationClosed,
            _ => Self::Unavailable,
        }
    }
}

fn guard_ban(db: &Connection, account: i64, now: i64) -> Result<(), Error> {
    let state = super::moderation::state(db, account).map_err(|_| Error::Unavailable)?;
    if state.banned(now) {
        return Err(Error::Suspended {
            until: state
                .ban_until
                .filter(|v| *v != super::moderation::PERMANENT)
                .map(|v| v as u64),
            reason: state
                .reason
                .unwrap_or(tbg_core::protocol::SuspensionReason::Other),
        });
    }
    Ok(())
}
