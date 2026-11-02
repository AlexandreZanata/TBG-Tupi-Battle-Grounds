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
mod views;
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
            revision: 0,
            epoch: crate::auth::epoch(),
            suspension: None,
        }
    }
}
pub fn plugin(app: &mut App) {
    app.init_resource::<State>().add_systems(
        Update,
        (input, poll, views::render)
            .chain()
            .after(super::match_chat::keys)
            .before(super::keyboard::navigate),
    );
}
pub(super) fn button(p: &mut ChildSpawnerCommands, lang: Lang, key: &str, action: Action) {
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
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    buttons: Query<(
        &UiAction,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    mut focus: Local<usize>,
) {
    if modal() && !textform::is_open(&form) {
        let visible: Vec<_> = buttons
            .iter()
            .filter(|(a, n, _, v)| v.get() && n.size.x > 0. && matches!(a, UiAction::P75(_)))
            .collect();
        if keys.just_pressed(KeyCode::Escape) {
            request(Action::Close);
        }
        if keys.just_pressed(KeyCode::Tab) && !visible.is_empty() {
            *focus = (*focus + 1) % visible.len();
        }
        if keys.just_pressed(KeyCode::Enter) && !visible.is_empty() {
            if let UiAction::P75(a) = visible[*focus % visible.len()].0 {
                request(*a);
            }
        }
        if mouse.just_released(MouseButton::Left) {
            if let Some(pos) = window.cursor_position() {
                for (a, n, t, _) in visible {
                    if n.contains_point(*t, pos * window.scale_factor()) {
                        if let UiAction::P75(a) = a {
                            request(*a);
                        }
                        break;
                    }
                }
            }
        }
    }
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
        match action {
            Action::Close => {
                s.page = Page::None;
                s.email = default();
                textform::close(&mut form);
            }
            Action::Account => {
                s.page = Page::Account;
                s.data = Value::Null;
                send(&mut s, Purpose::Account, "/api/me/account", "GET", None);
            }
            Action::LostKey => {
                textform::close(&mut form);
                s.page = Page::Recovery;
                s.recovery = default();
            }
            Action::Emojis => {
                s.page = Page::Emojis;
                send(&mut s, Purpose::Library, "/api/me/emoji", "GET", None);
            }
            Action::RecoveryName => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::Name,
                "p75.name_prompt",
                TextKind::P75Text,
            ),
            Action::RecoveryEmail => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::Email,
                "p75.email_prompt",
                TextKind::P75Text,
            ),
            Action::RecoveryRequest => {
                if s.pending.is_none() {
                    s.pending = Some((
                        Purpose::Request,
                        Request::recovery(&s.recovery.name, &s.recovery.email, lang.code()),
                    ));
                    s.notice = "p75.loading";
                }
            }
            Action::RecoveryCode => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::RecoveryCode,
                "p75.code_prompt",
                TextKind::P75Code,
            ),
            Action::EmailVerify => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::Verify,
                "p75.code_prompt",
                TextKind::P75Code,
            ),
            Action::EmailResend => send(
                &mut s,
                Purpose::Action,
                "/api/me/email/resend",
                "POST",
                Some(json!({})),
            ),
            Action::KeyProof => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::Key,
                "account.prompt_key",
                TextKind::P75Secret,
            ),
            Action::NewEmail => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::NewEmail,
                "p75.email_prompt",
                TextKind::P75Text,
            ),
            Action::OldCode => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::OldCode,
                "p75.old_code",
                TextKind::P75Code,
            ),
            Action::NewCode => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::NewCode,
                "p75.new_code",
                TextKind::P75Code,
            ),
            Action::Hold => s.email.hold = !s.email.hold,
            Action::EmailChange => {
                let body = json!({"account_key":s.email.key,"email":s.email.email,"locale":lang.code(),"lost_old":s.email.hold});
                send(&mut s, Purpose::Action, "/api/me/email", "POST", Some(body));
                s.email.key.clear();
            }
            Action::EmailConfirm => {
                let body = json!({"old_code":s.email.old_code,"new_code":s.email.new_code});
                send(
                    &mut s,
                    Purpose::Action,
                    "/api/me/email/change/confirm",
                    "POST",
                    Some(body),
                );
            }
            Action::EmailCancel => send(
                &mut s,
                Purpose::Action,
                "/api/me/email/change/cancel",
                "POST",
                Some(json!({})),
            ),
            Action::Chat => {
                if !info.spectating && *game.get() == GameState::InMatch {
                    chat.focused = true;
                }
            }
            Action::Mute => chat.muted = !chat.muted,
            Action::Emoji(slot) => {
                if !info.spectating && *game.get() == GameState::InMatch {
                    link.send(
                        tbg_core::protocol::ClientMsg::EmojiSend { slot },
                        time.elapsed_secs_f64(),
                    );
                }
            }
            Action::MarkReport => {
                s.marked = true;
                s.notice = "p75.report_after";
            }
            Action::ReportMatch => {
                if let Some(id) = info.result_id {
                    if info.outcome.is_some() && !info.spectating {
                        s.page = Page::Report;
                        s.report = super::report::Flow {
                            context: json!({"context":"match","result_id":hex(&id)}),
                            ..default()
                        };
                    }
                }
            }
            Action::ReportProfile(name) => {
                s.page = Page::Report;
                s.report = super::report::Flow {
                    context: json!({"context":"profile","target":name.text()}),
                    ..default()
                };
            }
            Action::ReportHistory(id) => {
                s.page = Page::Report;
                s.report = super::report::Flow {
                    context: json!({"context":"match","result_id":hex(&id)}),
                    ..default()
                };
            }
            Action::ReportEmoji(emoji) => {
                if let Some(id) = info.result_id {
                    s.page = Page::Report;
                    s.report = super::report::Flow {
                        context: json!({"context":"emoji","result_id":hex(&id),"emoji":hex(&emoji)}),
                        ..default()
                    };
                }
            }
            Action::Reason(reason) => s.report.reason = reason.min(7),
            Action::Comment => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::Comment,
                "p75.comment",
                TextKind::P75Text,
            ),
            Action::SendReport => {
                let body = s.report.body();
                send(
                    &mut s,
                    Purpose::Report,
                    "/api/me/reports",
                    "POST",
                    Some(body),
                );
            }
            Action::Mode => {
                chat.mode = (chat.mode + 1) % 3;
                crate::config::save_flag("p75_chat_off", chat.mode == 2);
                crate::config::save_flag("p75_chat_emoji", chat.mode == 1);
            }
            Action::Slot(slot) => s.library.slot = slot.min(2),
            Action::Standard(id) => {
                let mut slots = s.data["slots"].as_array().cloned().unwrap_or_else(|| {
                    vec![
                        json!({"standard":0}),
                        json!({"standard":1}),
                        json!({"standard":2}),
                    ]
                });
                slots[usize::from(s.library.slot)] = json!({"standard":id});
                send(
                    &mut s,
                    Purpose::Library,
                    "/api/me/emoji/slots",
                    "PUT",
                    Some(json!({"slots":slots})),
                );
            }
            Action::EquipCustom(id) => {
                let mut slots = s.data["slots"].as_array().cloned().unwrap_or_else(|| {
                    vec![
                        json!({"standard":0}),
                        json!({"standard":1}),
                        json!({"standard":2}),
                    ]
                });
                slots[usize::from(s.library.slot)] = json!({"custom":hex(&id)});
                send(
                    &mut s,
                    Purpose::Library,
                    "/api/me/emoji/slots",
                    "PUT",
                    Some(json!({"slots":slots})),
                );
            }
            Action::Upload => s.library.file = Some(crate::platform::file_pick::choose()),
            Action::UploadLabel => prompt(
                &mut s,
                &mut form,
                *lang,
                Await::Label,
                "p75.label",
                TextKind::P75Text,
            ),
            Action::UploadSend => {
                if s.pending.is_none() {
                    if let Some((bytes, name)) = s.library.selected.take() {
                        let mime = if name.to_ascii_lowercase().ends_with(".gif") {
                            "image/gif"
                        } else if name.to_ascii_lowercase().ends_with(".webp") {
                            "image/webp"
                        } else {
                            "image/png"
                        };
                        s.pending = Some((
                            Purpose::Upload,
                            Request::upload(&s.library.label, bytes, mime),
                        ));
                        s.notice = "p75.loading";
                    }
                }
            }
            Action::Withdraw(id) => send(
                &mut s,
                Purpose::Action,
                &format!("/api/me/emoji/{id}"),
                "DELETE",
                Some(json!({})),
            ),
        }
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
fn poll(mut s: ResMut<State>, time: Res<Time>) {
    if s.epoch != crate::auth::epoch() {
        s.pending = None;
        s.epoch = crate::auth::epoch();
        s.data = Value::Null;
        s.email = default();
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
    button(p, lang, "p75.report", Action::ReportProfile(name));
}
