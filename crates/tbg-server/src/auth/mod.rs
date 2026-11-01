//! Bounded account sessions outside combat, with one cancellation-safe coordinator (D-97).

use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tbg_core::protocol::Profile;
use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::store::auth_sessions::{self as db, Channel, CredentialHash, Family, FamilyId};
use crate::{
    chan::LobbyCmd,
    config::Config,
    limits::Limits,
    store::{AccountId, Store},
};

mod access;
pub(crate) mod http;
mod origin;
mod rotation;
mod tokens;
pub(crate) use origin::websocket as websocket_origin;
pub use tokens::Clock;

pub(crate) fn valid_origin(value: &str) -> bool {
    origin::canonical(value).is_some()
}

const ACCESS_SECONDS: i64 = 900;
/// P71: a lost refresh reply is recovered by retrying the same operation; the
/// window outlasts any outage a seat survives, so the retry never reads as reuse.
/// Reuse after it still revokes the family.
const RETRY_SECONDS: i64 = 300;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Unauthorized,
    Forbidden,
    BadRequest,
    Unavailable,
    Conflict,
    RecoveryRequired,
    RateLimited,
    NameInvalid,
    NameTaken,
    FamilyLimit,
    Capacity,
    InMatch,
}

impl From<db::Error> for Error {
    fn from(value: db::Error) -> Self {
        match value {
            db::Error::Unauthorized => Self::Unauthorized,
            db::Error::Unavailable => Self::Unavailable,
            db::Error::NameTaken => Self::NameTaken,
            db::Error::FamilyLimit => Self::FamilyLimit,
        }
    }
}

/// Account/seat authority never comes from client-supplied account IDs.
#[derive(Clone)]
pub(crate) struct Binding {
    pub account: AccountId,
    pub family: FamilyId,
    pub profile: Profile,
    pub high_water: i64,
    pub expires_at: i64,
    pub session_expires_at: i64,
    deadline: Instant,
    revoked: Arc<AtomicBool>,
}

impl Binding {
    #[cfg(test)]
    pub(crate) fn fixture(account: AccountId, profile: Profile) -> Self {
        Self {
            account,
            family: [account as u8; 16],
            profile,
            high_water: 0,
            expires_at: i64::MAX,
            session_expires_at: i64::MAX,
            deadline: Instant::now() + Duration::from_secs(900),
            revoked: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn revoked(&self) -> bool {
        self.revoked.load(Ordering::Acquire)
    }
    pub fn valid(&self, now: i64) -> bool {
        !self.revoked() && now < self.expires_at && Instant::now() < self.deadline
    }
    /// P71 (BR-AS07): unrevoked and expired at most `LEASE_GRACE_SECS` ago. Such a
    /// binding may take back its own admitted seat on reconnect, nothing else.
    pub fn resumable(&self, now: i64) -> bool {
        let grace = Duration::from_secs(LEASE_GRACE_SECS as u64);
        !self.revoked()
            && now < self.expires_at.saturating_add(LEASE_GRACE_SECS)
            && Instant::now() < self.deadline + grace
    }
}

/// A drop that outlasts the Access token must not need a refresh to come back:
/// longer than any seat hold (reconnect window plus the P71 pause budget).
pub(crate) const LEASE_GRACE_SECS: i64 = 180;

struct NewFamily {
    name: String,
    key_hash: CredentialHash,
    register: bool,
    device: FamilyId,
    persistent: bool,
    channel: Channel,
}
struct FreshTokens {
    access: [u8; 32],
    refresh: [u8; 32],
}

#[derive(Clone)]
struct Grant {
    access: String,
    refresh: String,
    expires_at: i64,
    family: Family,
    profile: Profile,
    account_key: Option<String>,
    persistent: bool,
}

struct CachedRotation {
    old: CredentialHash,
    operation: CredentialHash,
    successor: CredentialHash,
    until: i64,
    deadline: Instant,
    grant: Grant,
}

struct CachedRegistration {
    fingerprint: CredentialHash,
    until: i64,
    deadline: Instant,
    grant: Grant,
}

struct FamilyFlag {
    account: AccountId,
    revoked: Arc<AtomicBool>,
}

#[derive(Clone)]
pub(crate) enum RevocationScope {
    Families(Vec<FamilyId>),
    Account(AccountId),
}
impl RevocationScope {
    pub fn includes(&self, id: FamilyId, account: AccountId) -> bool {
        match self {
            Self::Families(ids) => ids.contains(&id),
            Self::Account(owner) => *owner == account,
        }
    }
}

#[derive(Default)]
struct State {
    flags: HashMap<FamilyId, FamilyFlag>,
    rotations: HashMap<FamilyId, CachedRotation>,
    registrations: HashMap<CredentialHash, CachedRegistration>,
    family_rates: HashMap<FamilyId, VecDeque<i64>>,
    login_limits: Limits,
    renewal_limits: Limits,
    last_cleanup: i64,
    ip_slots: HashMap<IpAddr, Instant>,
}

pub(crate) struct Service {
    store: Option<Arc<Store>>,
    state: Mutex<State>,
    access: std::sync::RwLock<HashMap<CredentialHash, Binding>>,
    lobby: mpsc::Sender<LobbyCmd>,
    clock: Clock,
    config: Arc<Config>,
    pub work: Arc<Semaphore>,
}

impl Service {
    pub fn new(
        store: Option<Arc<Store>>,
        lobby: mpsc::Sender<LobbyCmd>,
        clock: Clock,
        config: Arc<Config>,
    ) -> Self {
        Self {
            store,
            state: Mutex::new(State::default()),
            access: std::sync::RwLock::new(HashMap::new()),
            lobby,
            clock,
            work: Arc::new(Semaphore::new(config.max_connections)),
            config,
        }
    }
    pub fn start(this: &Arc<Self>) {
        let weak = Arc::downgrade(this);
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(Duration::from_secs(1));
            loop {
                timer.tick().await;
                let Some(service) = weak.upgrade() else {
                    return;
                };
                if let Ok(now) = service.now() {
                    if let Ok(mut state) = service.state.try_lock() {
                        service.prune(&mut state, now);
                        service.maintenance(&mut state, now).await;
                    }
                }
            }
        });
    }

    pub fn now(&self) -> Result<i64, Error> {
        self.clock.now()
    }
    fn store(&self) -> Result<&Store, Error> {
        self.store.as_deref().ok_or(Error::Unavailable)
    }

    fn prune(&self, state: &mut State, now: i64) {
        if let Ok(mut access) = self.access.write() {
            access.retain(|_, b| b.resumable(now));
        }
        state
            .rotations
            .retain(|_, c| now < c.until && Instant::now() < c.deadline);
        state
            .registrations
            .retain(|_, c| now < c.until && Instant::now() < c.deadline);
        state
            .flags
            .retain(|_, flag| Arc::strong_count(&flag.revoked) > 1);
        state.family_rates.retain(|_, log| {
            log.retain(|at| now.saturating_sub(*at) < 60);
            !log.is_empty()
        });
        state.login_limits.sweep(Instant::now());
        state.renewal_limits.sweep(Instant::now());
    }

    fn capacity(&self, family: Option<FamilyId>) -> Result<(), Error> {
        let access = self.access.read().map_err(|_| Error::Unavailable)?;
        if access.len() >= self.config.max_connections.saturating_mul(4) {
            return Err(Error::Capacity);
        }
        if family.is_some_and(|id| access.values().filter(|b| b.family == id).count() >= 16) {
            return Err(Error::Capacity);
        }
        Ok(())
    }

    fn cache_capacity(&self, state: &State, replacing: Option<FamilyId>) -> Result<(), Error> {
        let count = state.rotations.len() + state.registrations.len();
        if count >= self.config.max_connections
            && replacing.is_none_or(|id| !state.rotations.contains_key(&id))
        {
            return Err(Error::Capacity);
        }
        Ok(())
    }

    fn rate(&self, state: &mut State, ip: IpAddr, login: bool) -> Result<(), Error> {
        state
            .ip_slots
            .retain(|_, at| at.elapsed() < Duration::from_secs(3600));
        if !state.ip_slots.contains_key(&ip)
            && state.ip_slots.len() >= self.config.max_connections.saturating_mul(4)
        {
            return Err(Error::Capacity);
        }
        state.ip_slots.insert(ip, Instant::now());
        let allowed = if login {
            state.login_limits.check_login(
                ip,
                self.config.login_per_ip_min.min(u64::from(u32::MAX)) as u32,
                Instant::now(),
            )
        } else {
            state.renewal_limits.check_api(
                ip,
                self.config.api_per_ip_min.min(u64::from(u32::MAX)) as u32,
                Instant::now(),
            )
        };
        if allowed {
            Ok(())
        } else {
            Err(Error::RateLimited)
        }
    }

    fn family_rate(&self, state: &mut State, family: FamilyId, now: i64) -> Result<(), Error> {
        let rates = state.family_rates.entry(family).or_default();
        rates.retain(|at| now.saturating_sub(*at) < 60);
        if rates.len() >= 10 {
            return Err(Error::RateLimited);
        }
        rates.push_back(now);
        Ok(())
    }

    async fn maintenance(&self, state: &mut State, now: i64) {
        if now.saturating_sub(state.last_cleanup) >= 60 {
            state.last_cleanup = now;
            if let Some(store) = &self.store {
                let _ = store.auth(db::Request::Cleanup { now }).await;
            }
        }
    }

    fn publish(
        &self,
        state: &mut State,
        reply: db::Reply,
        secrets: FreshTokens,
        now: i64,
        account_key: Option<String>,
    ) -> Result<Grant, Error> {
        let db::Reply::Grant {
            family,
            profile,
            high_water,
        } = reply
        else {
            return Err(Error::Unavailable);
        };
        self.capacity(Some(family.id))?;
        let expiry = now
            .checked_add(ACCESS_SECONDS)
            .ok_or(Error::Unavailable)?
            .min(family.expires_at);
        if expiry <= now {
            return Err(Error::Unauthorized);
        }
        let flag = state
            .flags
            .entry(family.id)
            .or_insert_with(|| FamilyFlag {
                account: family.account,
                revoked: Arc::new(AtomicBool::new(false)),
            })
            .revoked
            .clone();
        if flag.load(Ordering::Acquire) {
            return Err(Error::Unauthorized);
        }
        let binding = Binding {
            account: family.account,
            family: family.id,
            profile: profile.clone(),
            high_water,
            expires_at: expiry,
            session_expires_at: family.expires_at,
            deadline: Instant::now() + Duration::from_secs((expiry - now) as u64),
            revoked: flag,
        };
        self.access
            .write()
            .map_err(|_| Error::Unavailable)?
            .insert(tokens::hash(&secrets.access), binding);
        let persistent = family.persistent;
        Ok(Grant {
            access: tokens::hex(&secrets.access),
            refresh: tokens::hex(&secrets.refresh),
            expires_at: expiry,
            family,
            profile,
            account_key,
            persistent,
        })
    }
}
