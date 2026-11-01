//! One auth coordinator drives native vaults, browser cookies, socket identity and UI state.

use bevy::prelude::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, LazyLock, Mutex, RwLock,
};
use tbg_core::protocol::{AccessToken, Profile, ServerMsg};

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
mod native_store;
mod transport;
mod types;
#[cfg(target_arch = "wasm32")]
mod web;
pub use types::{Failure, Status};
use types::{Grant, Outcome};

#[derive(Clone)]
pub struct Snapshot {
    pub status: Status,
    pub epoch: u64,
    pub access: Option<AccessToken>,
    pub family: Option<String>,
    pub profile: Option<Profile>,
    pub remember: bool,
    pub secure: bool,
    pub legacy_available: bool,
    pub error: Option<Failure>,
    pub manual_claim: bool,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            status: Status::Restoring,
            epoch: 1,
            access: None,
            family: None,
            profile: None,
            remember: false,
            secure: false,
            legacy_available: false,
            error: None,
            manual_claim: false,
        }
    }
}
static VIEW: LazyLock<RwLock<Snapshot>> = LazyLock::new(|| RwLock::new(Snapshot::default()));
static EPOCH: LazyLock<Arc<AtomicU64>> = LazyLock::new(|| Arc::new(AtomicU64::new(1)));
static COMMAND: Mutex<Option<Command>> = Mutex::new(None);

pub enum Command {
    Login(String),
    Register(String),
    Migrate(String),
    Refresh,
    Backup,
    Logout,
    ToggleRemember,
    ContinueHere,
    Guest,
}

pub fn epoch() -> u64 {
    EPOCH.load(Ordering::Acquire)
}
pub fn snapshot() -> Snapshot {
    VIEW.read().map(|v| v.clone()).unwrap_or_default()
}
pub fn request(command: Command) -> bool {
    let Ok(mut slot) = COMMAND.lock() else {
        return false;
    };
    if slot.is_some() {
        return false;
    }
    if matches!(
        command,
        Command::Login(_)
            | Command::Register(_)
            | Command::Migrate(_)
            | Command::Logout
            | Command::Guest
    ) {
        let next = EPOCH.fetch_add(1, Ordering::AcqRel) + 1;
        #[cfg(target_arch = "wasm32")]
        web::invalidate(next);
        if let Ok(mut view) = VIEW.write() {
            view.epoch = next;
            view.access = None;
            view.profile = None;
            view.manual_claim = false;
            view.status = if matches!(command, Command::Logout) {
                Status::LogoutPending
            } else {
                Status::Restoring
            };
        }
    }
    *slot = Some(command);
    true
}
pub fn take_manual_claim() -> bool {
    VIEW.write()
        .map(|mut v| std::mem::take(&mut v.manual_claim))
        .unwrap_or(false)
}
pub fn migration_key() -> Option<String> {
    LEGACY.lock().ok()?.clone()
}
static LEGACY: Mutex<Option<String>> = Mutex::new(None);
pub fn socket_status(status: tbg_core::protocol::AuthStatus) {
    if let Ok(mut view) = VIEW.write() {
        view.status = match status {
            tbg_core::protocol::AuthStatus::Active => {
                if view.remember && !view.secure {
                    Status::PersistenceUnavailable
                } else {
                    Status::Authenticated
                }
            }
            tbg_core::protocol::AuthStatus::Conflict => Status::Conflict,
            tbg_core::protocol::AuthStatus::ReauthenticationRequired => Status::ReauthRequired,
        };
    }
}
pub fn authentication_required(reason: tbg_core::protocol::AuthRefusal) {
    match reason {
        tbg_core::protocol::AuthRefusal::Expired => {
            let _ = request(Command::Refresh);
        }
        _ => {
            if let Ok(mut view) = VIEW.write() {
                view.status = Status::ReauthRequired;
                view.error = Some(Failure::InvalidKey);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Restore,
    Login,
    Register,
    Refresh,
    Backup,
    Logout,
}
type ResultSlot = Arc<Mutex<Option<Result<Outcome, Failure>>>>;
struct Pending {
    epoch: u64,
    kind: Kind,
    slot: ResultSlot,
}

#[derive(Resource)]
struct Session {
    backup_after_login: bool,
    origin: Result<String, Failure>,
    pending: Option<Pending>,
    next: Option<Command>,
    refresh: Option<String>,
    family: Option<String>,
    account: Option<String>,
    device: Option<String>,
    access: Option<AccessToken>,
    deadline: f64,
    retry_at: f64,
    secure: bool,
    remember: bool,
}

pub fn plugin(app: &mut App) {
    let mut session = Session {
        backup_after_login: false,
        origin: transport::origin(&crate::config::server_url()),
        pending: None,
        next: None,
        refresh: None,
        family: None,
        account: None,
        device: None,
        access: None,
        deadline: 0.,
        retry_at: 0.,
        secure: false,
        remember: false,
    };
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(origin) = &session.origin {
        let legacy = native::legacy(origin);
        if let Ok(mut view) = VIEW.write() {
            view.legacy_available = legacy.is_some();
        }
        if let Ok(mut stored) = LEGACY.lock() {
            *stored = legacy;
        }
    }
    session.start(Kind::Restore, None);
    app.insert_resource(session).add_systems(PreUpdate, tick);
}

fn tick(
    mut session: ResMut<Session>,
    time: Res<Time<Real>>,
    mut account: ResMut<crate::ui::account::Account>,
    mut inbox: ResMut<crate::ui::account::AccountInbox>,
    mut link: NonSendMut<crate::link::MatchLink>,
) {
    let now = time.elapsed_secs_f64();
    if let Some(command) = COMMAND.lock().ok().and_then(|mut q| q.take()) {
        session.command(command);
    }
    let complete = session
        .pending
        .as_ref()
        .and_then(|p| p.slot.lock().ok()?.take());
    if let Some(result) = complete {
        if let Some(pending) = session.pending.take() {
            session.settle(
                pending.epoch,
                pending.kind,
                result,
                now,
                &mut account,
                &mut inbox,
            );
        }
    }
    if session.pending.is_none() {
        if let Some(command) = session.next.take() {
            session.command(command);
        } else if session.access.is_some()
            && matches!(
                snapshot().status,
                Status::Authenticated
                    | Status::Refreshing
                    | Status::PersistenceUnavailable
                    | Status::Conflict
            )
            && now >= session.deadline - 60.
            && now >= session.retry_at
        {
            session.start(Kind::Refresh, None);
        }
    }
    let view = snapshot();
    let restoring = view.status == Status::Restoring;
    if account.auto_login != restoring {
        account.auto_login = restoring;
    }
    if view.access.is_some()
        && !matches!(*link, crate::link::MatchLink::Net(_))
        && matches!(*link, crate::link::MatchLink::Idle)
    {
        *link = crate::link::MatchLink::Net(Box::new(crate::net::NetLink::new(
            crate::net::Mode::Probe,
            now,
        )));
    }
    if (view.status == Status::LogoutPending
        || (view.status == Status::Guest && view.access.is_none()))
        && account.profile.is_some()
    {
        account.profile = None;
    }
}

impl Session {
    fn command(&mut self, command: Command) {
        match command {
            Command::ToggleRemember => {
                if self
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.kind != Kind::Restore)
                {
                    return;
                }
                self.remember = !self.remember;
                if let Ok(mut view) = VIEW.write() {
                    view.remember = self.remember;
                }
                return;
            }
            Command::ContinueHere => {
                if let Ok(mut view) = VIEW.write() {
                    view.manual_claim = true;
                }
                return;
            }
            Command::Guest => {
                self.backup_after_login = false;
                self.next = None;
                if self.pending.is_some() {
                    self.next = Some(Command::Guest);
                    return;
                }
                if self.access.is_some() || self.refresh.is_some() {
                    self.start(Kind::Logout, None);
                    return;
                }
                if let Ok(mut view) = VIEW.write() {
                    view.status = Status::Guest;
                    view.access = None;
                    view.profile = None;
                    view.family = None;
                }
                return;
            }
            _ => {}
        }
        if self.pending.is_some() {
            self.next = Some(command);
            return;
        }
        match command {
            Command::Migrate(key) => {
                self.backup_after_login = true;
                self.remember = true;
                self.start(Kind::Login, Some(key));
            }
            Command::Login(key) => {
                self.backup_after_login = false;
                self.start(Kind::Login, Some(key));
            }
            Command::Register(name) => {
                self.backup_after_login = false;
                self.start(Kind::Register, Some(name));
            }
            Command::Refresh => self.start(Kind::Refresh, None),
            Command::Backup => self.start(Kind::Backup, None),
            Command::Logout => self.start(Kind::Logout, None),
            _ => {}
        }
    }

    fn start(&mut self, kind: Kind, input: Option<String>) {
        let slot: ResultSlot = Arc::new(Mutex::new(None));
        let current = epoch();
        let result = match &self.origin {
            Ok(origin) => Ok(origin.clone()),
            Err(error) => Err(*error),
        };
        if let Ok(mut view) = VIEW.write() {
            view.epoch = current;
            view.remember = self.remember;
            view.status = match kind {
                Kind::Restore | Kind::Login | Kind::Register => Status::Restoring,
                Kind::Refresh => Status::Refreshing,
                Kind::Logout => Status::LogoutPending,
                Kind::Backup => view.status,
            };
        }
        match result {
            Err(error) => {
                if let Ok(mut target) = slot.lock() {
                    *target = Some(Err(error));
                }
            }
            Ok(origin) => self.launch(kind, input, origin, current, slot.clone()),
        }
        self.pending = Some(Pending {
            epoch: current,
            kind,
            slot,
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn launch(
        &self,
        kind: Kind,
        input: Option<String>,
        origin: String,
        current: u64,
        slot: ResultSlot,
    ) {
        let operation = match kind {
            Kind::Restore => native::Operation::Restore,
            Kind::Login => native::Operation::Login(input.unwrap_or_default()),
            Kind::Register => native::Operation::Register(input.unwrap_or_default()),
            Kind::Refresh => native::Operation::Refresh,
            Kind::Backup => native::Operation::Backup,
            Kind::Logout => native::Operation::Logout,
        };
        let work = native::Work {
            origin,
            operation,
            remember: self.remember,
            refresh: self.refresh.clone(),
            family: self.family.clone(),
            account: self.account.clone(),
            device: self.device.clone(),
            access: self.access,
            epoch: current,
            current_epoch: EPOCH.clone(),
        };
        let result = slot.clone();
        if std::thread::Builder::new()
            .name("tbg-auth".into())
            .spawn(move || {
                let value = native::run(work);
                if let Ok(mut slot) = result.lock() {
                    *slot = Some(value);
                }
            })
            .is_err()
        {
            if let Ok(mut slot) = slot.lock() {
                *slot = Some(Err(Failure::Unavailable));
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn launch(
        &self,
        kind: Kind,
        input: Option<String>,
        origin: String,
        current: u64,
        slot: ResultSlot,
    ) {
        web::start(kind, input, origin, current, self.remember, slot);
    }

    fn settle(
        &mut self,
        issued_epoch: u64,
        kind: Kind,
        result: Result<Outcome, Failure>,
        now: f64,
        account: &mut crate::ui::account::Account,
        inbox: &mut crate::ui::account::AccountInbox,
    ) {
        if issued_epoch == epoch() {
            self.complete(kind, result, now, account, inbox);
            return;
        }
        match result {
            Ok(Outcome::SignedIn { grant, device, .. }) => {
                self.access = Some(grant.access);
                self.refresh = grant.refresh;
                self.family = Some(grant.family);
                self.account = Some(grant.profile.name);
                self.device = Some(device);
                if matches!(self.next, Some(Command::Logout | Command::Guest)) {
                    self.next = None;
                }
                // A worker can finish just before cancellation. Do not drop its
                // revocation proof or allow its durable family to restore later.
                self.start(Kind::Logout, None);
            }
            Ok(Outcome::LogoutPending(_)) => {
                if matches!(self.next, Some(Command::Logout | Command::Guest)) {
                    self.next = None;
                }
                self.start(Kind::Logout, None);
            }
            Err(Failure::Cancelled) => {
                self.access = None;
                self.refresh = None;
                self.family = None;
                if matches!(self.next, Some(Command::Logout | Command::Guest)) {
                    self.next = None;
                }
                if let Ok(mut view) = VIEW.write() {
                    view.status = Status::Guest;
                    view.access = None;
                    view.profile = None;
                    view.family = None;
                }
            }
            _ => {}
        }
    }

    fn complete(
        &mut self,
        kind: Kind,
        result: Result<Outcome, Failure>,
        now: f64,
        account: &mut crate::ui::account::Account,
        inbox: &mut crate::ui::account::AccountInbox,
    ) {
        match result {
            Ok(Outcome::SignedIn {
                grant,
                secure,
                device,
            }) => self.signed_in(kind, grant, secure, device, now, inbox),
            Ok(Outcome::Guest { legacy }) => {
                if let Ok(mut slot) = LEGACY.lock() {
                    *slot = legacy.clone();
                }
                if let Ok(mut view) = VIEW.write() {
                    view.status = Status::Guest;
                    view.legacy_available = legacy.is_some();
                }
            }
            Ok(Outcome::Backup { migrated }) => {
                if migrated {
                    if let Ok(mut legacy) = LEGACY.lock() {
                        *legacy = None;
                    }
                    if let Ok(mut view) = VIEW.write() {
                        view.legacy_available = false;
                    }
                }
                account.screen = crate::ui::account::AccountScreen::None;
            }
            Ok(Outcome::LoggedOut) => {
                self.access = None;
                self.refresh = None;
                self.family = None;
                if let Ok(mut view) = VIEW.write() {
                    view.status = Status::Guest;
                    view.access = None;
                    view.family = None;
                    view.profile = None;
                }
                inbox.push(ServerMsg::LoggedOut);
            }
            Ok(Outcome::LogoutPending(error)) => {
                if let Ok(mut view) = VIEW.write() {
                    view.status = Status::LogoutPending;
                    view.error = Some(error);
                    view.access = None;
                    view.profile = None;
                }
            }
            Err(error) => {
                self.retry_at = now + 60.;
                if let Ok(mut view) = VIEW.write() {
                    view.error = Some(error);
                    view.status = if kind == Kind::Logout {
                        Status::LogoutPending
                    } else if kind == Kind::Refresh && self.access.is_some() {
                        Status::PersistenceUnavailable
                    } else {
                        Status::ReauthRequired
                    };
                }
                account.pending = None;
            }
        }
    }

    fn signed_in(
        &mut self,
        kind: Kind,
        grant: Grant,
        secure: bool,
        device: String,
        now: f64,
        inbox: &mut crate::ui::account::AccountInbox,
    ) {
        self.deadline = now + grant.remaining as f64;
        self.access = Some(grant.access);
        self.refresh = grant.refresh;
        self.family = Some(grant.family.clone());
        self.account = Some(grant.profile.name.clone());
        self.device = Some(device);
        self.secure = secure;
        if kind == Kind::Restore {
            self.remember = grant.persistent && secure;
        }
        if let Ok(mut view) = VIEW.write() {
            view.status = if !secure && (self.remember || cfg!(target_arch = "wasm32")) {
                Status::PersistenceUnavailable
            } else {
                Status::Authenticated
            };
            view.epoch = epoch();
            view.access = self.access;
            view.family = self.family.clone();
            view.profile = Some(grant.profile.clone());
            view.secure = secure;
            view.remember = self.remember;
            view.error = None;
            view.manual_claim = kind == Kind::Login;
        }
        if self.backup_after_login && kind == Kind::Login {
            self.backup_after_login = false;
            self.next = Some(Command::Backup);
        }
        if let Some(key) = grant.account_key {
            inbox.push(ServerMsg::Registered {
                key,
                profile: grant.profile,
            });
        } else if kind != Kind::Refresh {
            inbox.push(ServerMsg::LoggedIn {
                profile: grant.profile,
            });
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(not(target_arch = "wasm32"), feature = "ui-acceptance"))]
mod acceptance;
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-acceptance"))]
pub fn acceptance_journey() {
    match acceptance::run() {
        Ok(report) => println!("{report}"),
        Err(error) => {
            eprintln!("Synthetic native auth acceptance failed: {error:?}");
            std::process::exit(2);
        }
    }
}
