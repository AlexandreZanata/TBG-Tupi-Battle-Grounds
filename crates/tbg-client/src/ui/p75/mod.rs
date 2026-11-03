//! Shared desktop identity/report/library overlays, with bounded asynchronous account HTTP.
use super::{
    kit,
    social::Name,
    textform::{self, ActiveForm, TextEntered, TextKind},
    UiAction,
};
use crate::{auth::api::Request, i18n::Lang, sync::MatchInfo, GameState};
use bevy::prelude::*;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
mod actions;
mod controls;
mod views;
#[derive(Component)]
pub(super) struct DialogButton;
#[derive(Component)]
pub(super) struct DialogPanel;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Account,
    LostKey,
    Emojis,
    Close,
    EmailVerify,
    EmailResend,
    EmailChange,
    EmailConfirm,
    EmailCancel,
    Hold,
    RecoveryName,
    RecoveryEmail,
    RecoveryRequest,
    RecoveryCode,
    KeyProof,
    NewEmail,
    OldCode,
    NewCode,
    Chat,
    #[cfg(target_os = "android")]
    Compose,
    Mute,
    Emoji(u8),
    MarkReport,
    ReportMatch,
    ReportProfile(Name),
    ReportHistory([u8; 16]),
    ReportEmoji([u8; 16]),
    Reason(u8),
    Comment,
    SendReport,
    Mode,
    Slot(u8),
    Standard(u8),
    EquipCustom([u8; 16]),
    Upload,
    UploadLabel,
    UploadSend,
    Withdraw(i64),
}
static REQUESTS: Mutex<Vec<Action>> = Mutex::new(Vec::new());
static MODAL: AtomicBool = AtomicBool::new(false);
static CAPTURE: AtomicBool = AtomicBool::new(false);
static SUSPEND: Mutex<Option<(Option<u64>, tbg_core::protocol::SuspensionReason)>> =
    Mutex::new(None);
pub fn request(a: Action) {
    if let Ok(mut q) = REQUESTS.lock() {
        if q.len() < 64 {
            q.push(a);
        }
    }
}
pub fn modal() -> bool {
    MODAL.load(Ordering::Relaxed)
}
pub fn captured() -> bool {
    CAPTURE.load(Ordering::Relaxed)
}
pub fn suspend(until: Option<u64>, reason: tbg_core::protocol::SuspensionReason) {
    if let Ok(mut s) = SUSPEND.lock() {
        *s = Some((until, reason));
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Page {
    None,
    Account,
    Recovery,
    Emojis,
    Report,
    Suspended,
}
#[derive(Clone, Copy)]
enum Await {
    Name,
    Email,
    RecoveryCode,
    Verify,
    Key,
    NewEmail,
    OldCode,
    NewCode,
    Comment,
    Label,
}
#[derive(Clone, Copy)]
enum Purpose {
    Account,
    Library,
    Action,
    Request,
    Report,
    Upload,
}
#[derive(Resource)]
pub(super) struct State {
    page: Page,
    data: Value,
    notice: &'static str,
    pending: Option<(Purpose, Request)>,
    awaiting: Option<Await>,
    recovery: super::recovery::Flow,
    email: super::account_email::Flow,
    report: super::report::Flow,
    library: super::emoji_library::Library,
    marked: bool,
    eligibility: Option<([u8; 16], bool)>,
    eligibility_request: Option<Request>,
    revision: u64,
    epoch: u64,
    suspension: Option<(Option<u64>, tbg_core::protocol::SuspensionReason)>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            page: Page::None,
            data: Value::Null,
            notice: "",
            pending: None,
            awaiting: None,
            recovery: default(),
            email: default(),
            report: default(),
            library: default(),
            marked: false,
            eligibility: None,
            eligibility_request: None,
            revision: 0,
            epoch: crate::auth::epoch(),
            suspension: None,
        }
    }
}
pub fn plugin(app: &mut App) {
    app.init_resource::<State>().add_systems(
        Update,
        (controls::input, input, poll, views::render)
            .chain()
            .after(super::match_chat::keys)
            .before(super::keyboard::navigate),
    );
}
pub(super) fn button(p: &mut ChildSpawnerCommands, lang: Lang, key: &str, action: Action) {
    p.spawn((
        kit::button(lang.key(key), UiAction::P75(action), 230.),
        DialogButton,
    ));
}
pub(super) fn toolbar_button(p: &mut ChildSpawnerCommands, lang: Lang, key: &str, action: Action) {
    p.spawn(kit::button(lang.key(key), UiAction::P75(action), 230.));
}
fn send(s: &mut State, purpose: Purpose, path: &str, method: &'static str, body: Option<Value>) {
    if s.pending.is_none() {
        s.pending = Some((purpose, Request::new(path.into(), method, body, None, true)));
        s.notice = "p75.loading";
        s.revision += 1;
    }
}
fn prompt(
    s: &mut State,
    form: &mut ActiveForm,
    lang: Lang,
    awaiting: Await,
    key: &str,
    kind: TextKind,
) {
    s.awaiting = Some(awaiting);
    textform::open(form, kind, &lang.key(key));
}
#[allow(clippy::too_many_arguments)]
fn input(
    mut s: ResMut<State>,
    mut chat: ResMut<super::match_chat::Chat>,
    info: Res<MatchInfo>,
    game: Res<bevy::prelude::State<GameState>>,
    lang: Res<Lang>,
    mut form: NonSendMut<ActiveForm>,
    mut entered: MessageReader<TextEntered>,
    mut link: NonSendMut<crate::link::MatchLink>,
    time: Res<Time>,
) {
    for e in entered.read().filter(|e| {
        matches!(
            e.kind,
            TextKind::P75Text | TextKind::P75Code | TextKind::P75Secret
        )
    }) {
        let Some(target) = s.awaiting.take() else {
            continue;
        };
        let value = e.value.clone();
        match target {
            Await::Name => s.recovery.name = value,
            Await::Email => s.recovery.email = value,
            Await::RecoveryCode => {
                let input = json!({"name":s.recovery.name,"code":value});
                crate::auth::request(crate::auth::Command::Recover(input.to_string()));
                s.page = Page::None;
            }
            Await::Verify => send(
                &mut s,
                Purpose::Action,
                "/api/me/email/verify",
                "POST",
                Some(json!({"code":value})),
            ),
            Await::Key => s.email.key = value,
            Await::NewEmail => s.email.email = value,
            Await::OldCode => s.email.old_code = value,
            Await::NewCode => s.email.new_code = value,
            Await::Comment => s.report.comment = value,
            Await::Label => s.library.label = value,
        }
        textform::close(&mut form);
        s.revision += 1;
    }
    let actions = REQUESTS
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    for action in actions {
        s.revision += 1;
        actions::apply(
            action,
            &mut actions::Context {
                state: &mut s,
                chat: &mut chat,
                form: &mut form,
                info: &info,
                game: *game.get(),
                lang: *lang,
                link: &mut link,
                now: time.elapsed_secs_f64(),
            },
        );
    }

    MODAL.store(s.page != Page::None, Ordering::Relaxed);
    CAPTURE.store(
        s.page != Page::None
            || chat.focused
            || super::match_chat::consumed()
            || textform::is_open(&form),
        Ordering::Relaxed,
    );
}
fn poll(
    mut s: ResMut<State>,
    time: Res<Time>,
    info: Res<MatchInfo>,
    game: Res<bevy::prelude::State<GameState>>,
    link: NonSend<crate::link::MatchLink>,
) {
    if *game.get() == GameState::Result
        && !info.spectating
        && matches!(&*link, crate::link::MatchLink::Net(_))
        && crate::auth::snapshot().access.is_some()
    {
        if let Some(id) = info.result_id {
            if s.eligibility.map(|v| v.0) != Some(id) {
                s.eligibility = Some((id, false));
                s.eligibility_request = Some(Request::new(
                    format!("/api/players/{}", info.opponent),
                    "GET",
                    None,
                    None,
                    false,
                ));
            }
        }
    }
    if let Some(reply) = s
        .eligibility_request
        .as_mut()
        .and_then(|r| r.take(time.elapsed_secs_f64()))
    {
        if let Some((_, eligible)) = s.eligibility.as_mut() {
            *eligible = reply.is_ok_and(|r| r.status == 200);
        }
        s.eligibility_request = None;
        s.revision += 1;
    }

    if s.epoch != crate::auth::epoch() {
        s.revision += 1;
        s.pending = None;
        s.epoch = crate::auth::epoch();
        s.data = Value::Null;
        s.eligibility = None;
        s.eligibility_request = None;
        s.email = default();
        s.library = default();
        s.report = default();
        s.recovery = default();
        if matches!(s.page, Page::Account | Page::Emojis | Page::Report) {
            s.page = Page::None;
        }
    }
    if let Some(suspension) = SUSPEND.lock().ok().and_then(|mut s| s.take()) {
        s.suspension = Some(suspension);
        s.page = Page::Suspended;
        s.revision += 1;
    }
    let picked = s.library.file.as_ref().and_then(|f| f.lock().ok()?.take());
    if let Some(picked) = picked {
        s.library.file = None;
        match picked {
            Ok(file) => s.library.selected = Some(file),
            Err(_) => s.notice = "p75.upload_invalid",
        }
        s.revision += 1;
    }
    let result = s
        .pending
        .as_mut()
        .and_then(|(_, r)| r.take(time.elapsed_secs_f64()));
    let Some(result) = result else {
        return;
    };
    let purpose = s.pending.take().map(|p| p.0).unwrap_or(Purpose::Action);
    s.revision += 1;
    match result {
        Ok(reply) if (200..300).contains(&reply.status) => match purpose {
            Purpose::Account | Purpose::Library => {
                if reply.value.get("uploads").is_some() || matches!(purpose, Purpose::Account) {
                    s.data = reply.value;
                    if s.notice == "p75.loading" {
                        s.notice = "";
                    }
                } else {
                    send(&mut s, Purpose::Library, "/api/me/emoji", "GET", None);
                }
            }
            Purpose::Request => {
                s.recovery.requested = true;
                s.notice = "p75.recovery_generic";
            }
            Purpose::Report => s.notice = "p75.report_sent",
            Purpose::Upload => {
                s.notice = "p75.upload_pending";
                send(&mut s, Purpose::Library, "/api/me/emoji", "GET", None);
            }
            Purpose::Action => {
                s.notice = "p75.saved";
                if s.page == Page::Account {
                    send(&mut s, Purpose::Account, "/api/me/account", "GET", None);
                } else if s.page == Page::Emojis {
                    send(&mut s, Purpose::Library, "/api/me/emoji", "GET", None);
                }
            }
        },
        Ok(reply) => {
            s.notice = match reply.value["error"].as_str() {
                Some("duplicate") => "p75.duplicate",
                Some("rate_limited") => "p75.rate_limited",
                Some("code_invalid") => "p75.code_invalid",
                Some("unverified") => "p75.unverified",
                Some("uploads_disabled") => "p75.uploads_disabled",
                Some("queue_full") => "p75.queue_full",
                _ => "p75.error",
            };
        }
        Err(_) => s.notice = "p75.error",
    }
}
pub(super) fn hex(id: &[u8; 16]) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn report_button(p: &mut ChildSpawnerCommands, name: Name, lang: Lang) {
    let account = crate::auth::snapshot();
    if account.access.is_some()
        && account
            .profile
            .is_some_and(|profile| !profile.name.eq_ignore_ascii_case(&name.text()))
    {
        toolbar_button(p, lang, "p75.report", Action::ReportProfile(name));
    }
}
