//! Battle chat log/input, mute and three-slot cosmetic delivery; never gameplay authority.
use super::{kit, UiAction};
use crate::{i18n::Lang, link::MatchLink, sync::MatchInfo, GameState};
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use std::collections::VecDeque;
use tbg_core::protocol::{ChatPermission, ClientMsg, MatchSocial, ServerMsg};
static CONSUMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn consumed() -> bool {
    CONSUMED.load(std::sync::atomic::Ordering::Relaxed)
}
#[derive(Resource, Default)]
pub struct Inbox(pub Vec<ServerMsg>);
#[derive(Resource, Default)]
pub struct Chat {
    pub policy: Option<MatchSocial>,
    pub lines: VecDeque<(f64, tbg_core::Seat, String)>,
    pub bubbles: [Option<(u8, f64)>; 2],
    pub focused: bool,
    pub edit: kit::edit::Edit,
    pub muted: bool,
    pub mode: u8,
    pub refusal: bool,
    result: Option<[u8; 16]>,
}
#[derive(Component)]
struct Root;
pub fn plugin(app: &mut App) {
    app.init_resource::<Inbox>()
        .insert_resource(Chat {
            mode: stored_mode(),
            ..default()
        })
        .add_systems(
            Update,
            (receive, keys, render)
                .chain()
                .before(super::keyboard::navigate),
        );
}
fn stored_mode() -> u8 {
    if crate::config::load_flag("p75_chat_off") {
        2
    } else if crate::config::load_flag("p75_chat_emoji") {
        1
    } else {
        0
    }
}
pub fn allow(c: char) -> bool {
    (' '..='~').contains(&c) || ('\u{a0}'..='\u{ff}').contains(&c)
}
fn receive(
    mut chat: ResMut<Chat>,
    mut inbox: ResMut<Inbox>,
    info: Res<MatchInfo>,
    time: Res<Time>,
) {
    if chat.result != info.result_id {
        *chat = Chat {
            result: info.result_id,
            mode: stored_mode(),
            ..default()
        };
    }
    let now = time.elapsed_secs_f64();
    for message in inbox.0.drain(..) {
        match message {
            ServerMsg::MatchSocial(p) => chat.policy = Some(*p),
            ServerMsg::Chat { from, text }
                if chat.mode == 0 && (!chat.muted || Some(from) == info.you) =>
            {
                let name = if Some(from) == info.you {
                    &info.you_name
                } else {
                    &info.opponent
                };
                chat.lines.push_back((now, from, format!("{name}: {text}")));
                while chat.lines.len() > 50 {
                    chat.lines.pop_front();
                }
            }
            ServerMsg::Emoji { from, slot }
                if chat.mode < 2 && (!chat.muted || Some(from) == info.you) =>
            {
                chat.bubbles[from.index()] = Some((slot, now))
            }
            ServerMsg::SocialRefused { .. } => chat.refusal = true,
            ServerMsg::AccountSuspended { until_unix, reason } => {
                super::p75::suspend(until_unix, reason)
            }
            _ => {}
        }
    }
}
#[allow(clippy::too_many_arguments)] // Bevy injects the independent input, identity and modal resources.
pub(crate) fn keys(
    mut chat: ResMut<Chat>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    info: Res<MatchInfo>,
    mut link: NonSendMut<MatchLink>,
    time: Res<Time>,
    form: NonSend<super::textform::ActiveForm>,
) {
    CONSUMED.store(chat.focused, std::sync::atomic::Ordering::Relaxed);
    if *state.get() != GameState::InMatch
        || info.spectating
        || chat.mode == 2
        || !matches!(&*link, MatchLink::Net(_))
        || super::textform::is_open(&form)
        || super::p75::modal()
    {
        events.clear();
        chat.focused = false;
        return;
    }
    let now = time.elapsed_secs_f64();
    if !chat.focused {
        events.clear();
        if keys.just_pressed(KeyCode::Enter) && !super::keyboard::tabbed() {
            chat.focused = chat
                .policy
                .as_ref()
                .is_some_and(|p| p.chat == ChatPermission::Allowed);
        } else {
            for (key, slot) in [
                (KeyCode::Digit1, 0),
                (KeyCode::Digit2, 1),
                (KeyCode::Digit3, 2),
            ] {
                if keys.just_pressed(key) && chat.policy.as_ref().is_some_and(|p| p.emojis) {
                    link.send(ClientMsg::EmojiSend { slot }, now);
                }
            }
        }
        return;
    }
    let ctrl = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    for e in events.read().filter(|e| e.state.is_pressed()) {
        match &e.logical_key {
            Key::Escape => {
                chat.focused = false;
                chat.edit = default();
            }
            Key::Enter => {
                let text = chat.edit.text.trim().to_owned();
                if !text.is_empty() {
                    link.send(ClientMsg::ChatSend { text }, now);
                }
                chat.edit = default();
                chat.focused = false;
            }
            Key::Backspace => chat.edit.backspace(),
            Key::Delete => chat.edit.delete(),
            Key::ArrowLeft => {
                let at = chat.edit.cursor.saturating_sub(1);
                chat.edit.move_to(at, false);
            }
            Key::ArrowRight => {
                let at = chat.edit.cursor + 1;
                chat.edit.move_to(at, false);
            }
            #[cfg(not(target_arch = "wasm32"))]
            Key::Character(c) if ctrl && c.eq_ignore_ascii_case("v") => {
                if let Some(text) = crate::platform::paste() {
                    chat.edit.insert(&text, 120, allow);
                }
            }
            Key::Character(c) if !ctrl => chat.edit.insert(c, 120, allow),
            _ => {}
        }
    }
}
#[allow(clippy::too_many_arguments)] // One presentation system reads the seat policy and existing HUD context.
fn render(
    mut commands: Commands,
    roots: Query<Entity, With<Root>>,
    chat: Res<Chat>,
    info: Res<MatchInfo>,
    state: Res<State<GameState>>,
    lang: Res<Lang>,
    time: Res<Time>,
    link: NonSend<MatchLink>,
    mut shown: Local<String>,
) {
    let active = chat.mode < 2
        && *state.get() == GameState::InMatch
        && !info.spectating
        && matches!(&*link, MatchLink::Net(_));
    let signature = format!(
        "{active}:{}:{}:{}:{}:{}:{}",
        chat.focused,
        chat.edit.display(false),
        chat.lines.len(),
        chat.refusal,
        chat.muted,
        (time.elapsed_secs() / 0.25) as u64
    );
    if *shown == signature {
        return;
    }
    *shown = signature;
    for e in &roots {
        commands.entity(e).despawn();
    }
    if !active {
        return;
    }
    let now = time.elapsed_secs_f64();
    commands
        .spawn((
            Root,
            super::hud::Hud::PrivateControls,
            GlobalZIndex(25),
            Node {
                position_type: PositionType::Absolute,
                right: percent(1.),
                top: percent(9.),
                width: percent(20.),
                max_height: percent(58.),
                padding: UiRect::all(px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            BackgroundColor(kit::PANEL.with_alpha(if chat.focused { 0.98 } else { 0.75 })),
        ))
        .with_children(|p| {
            p.spawn(kit::heading(lang.key("p75.chat"), 18.));
            let lines: Vec<_> = chat
                .lines
                .iter()
                .filter(|(at, from, _)| {
                    chat.mode == 0
                        && (chat.focused || now - at < 12.)
                        && (!chat.muted || Some(*from) == info.you)
                })
                .rev()
                .take(8)
                .collect();
            for (_, _, line) in lines.into_iter().rev() {
                p.spawn(kit::muted(line, 14.));
            }
            if chat.mode == 0 {
                if chat
                    .policy
                    .as_ref()
                    .is_some_and(|p| p.chat == ChatPermission::Allowed)
                {
                    p.spawn(kit::button(
                        if chat.focused {
                            chat.edit.display(false)
                        } else {
                            lang.key("p75.chat_enter")
                        },
                        UiAction::P75(super::p75::Action::Chat),
                        230.,
                    ));
                } else {
                    let key = match chat.policy.as_ref().map(|p| p.chat) {
                        Some(ChatPermission::Guest) => "p75.chat_guest",
                        Some(ChatPermission::Unverified) => "p75.unverified",
                        Some(ChatPermission::Restricted { .. }) => "p75.chat_restricted",
                        _ => "p75.chat_disabled",
                    };
                    p.spawn(kit::muted(lang.key(key), 14.));
                }
            }
            p.spawn(kit::row()).with_children(|r| {
                for slot in 0..3 {
                    let allowed = chat.policy.as_ref().is_some_and(|p| p.emojis);
                    r.spawn(kit::button(
                        (slot + 1).to_string(),
                        if allowed {
                            UiAction::P75(super::p75::Action::Emoji(slot))
                        } else {
                            UiAction::Disabled
                        },
                        64.,
                    ))
                    .with_children(|button| {
                        if let Some(policy) = &chat.policy {
                            let seat = info.you.map(|s| s.index()).unwrap_or(0);
                            crate::emoji::spawn(
                                button,
                                policy.loadouts[seat][slot as usize].clone(),
                                slot,
                                now,
                                30.,
                            );
                        }
                    });
                }
            });
            p.spawn(kit::button(
                lang.key(if chat.muted { "p75.unmute" } else { "p75.mute" }),
                UiAction::P75(super::p75::Action::Mute),
                230.,
            ));
            if crate::auth::snapshot().access.is_some()
                && tbg_core::account::valid_name_shape(&info.opponent)
            {
                p.spawn(kit::button(
                    lang.key("p75.report_after"),
                    UiAction::P75(super::p75::Action::MarkReport),
                    230.,
                ));
            }
            if chat.refusal {
                p.spawn(kit::muted(lang.key("p75.social_refused"), 13.));
            }
        });
    if let Some(policy) = &chat.policy {
        for seat in tbg_core::Seat::BOTH {
            if let Some((slot, at)) = chat.bubbles[seat.index()]
                .filter(|(_, at)| now - at < 2.5 && (!chat.muted || Some(seat) == info.you))
            {
                commands
                    .spawn((
                        Root,
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(if seat == tbg_core::Seat::A { 10. } else { 60. }),
                            top: percent(10.),
                            ..default()
                        },
                    ))
                    .with_children(|p| {
                        crate::emoji::spawn(
                            p,
                            policy.loadouts[seat.index()][slot as usize].clone(),
                            slot,
                            at,
                            64.,
                        )
                    });
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p75_chat_input_filters_only_supported_characters() {
        assert!(allow('ç'));
        assert!(!allow('\n'));
        assert!(!allow('\u{202e}'));
    }
}
