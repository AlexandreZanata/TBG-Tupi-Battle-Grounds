//! Audio and Back follow Android state without pausing online authority or adding grace.

use super::services;
use crate::{
    ui::{
        self,
        shell::{Panel, Shell},
    },
    GameState,
};
use bevy::{
    audio::{AudioSink, AudioSinkPlayback},
    prelude::*,
};

pub fn poll(
    mut queue: ResMut<ui::ActionQueue>,
    mut shell: ResMut<Shell>,
    mut form: NonSendMut<ui::textform::ActiveForm>,
    mut account: ResMut<ui::account::Account>,
    mut toasts: ResMut<ui::Toasts>,
    mut training: ResMut<crate::training::Requests>,
    mut chat: ResMut<ui::match_chat::Chat>,
    state: Res<State<GameState>>,
    lang: Res<crate::i18n::Lang>,
    sinks: Query<&AudioSink>,
    mut previous: Local<bool>,
    mut battle_mode: Local<Option<bool>>,
) {
    let battle = matches!(
        *state.get(),
        GameState::InMatch | GameState::Training | GameState::Result
    );
    if *battle_mode != Some(battle)
        && services::send(
            jni::jni_str!("setBattleViewport"),
            if battle { "true" } else { "false" },
        )
    {
        *battle_mode = Some(battle);
    }
    let focused = services::flag(jni::jni_str!("hasAudioFocus"));
    if !focused || focused != *previous {
        for sink in &sinks {
            if focused {
                sink.play();
            } else {
                sink.pause();
            }
        }
        *previous = focused;
    }
    if let Some(raw) = services::string(jni::jni_str!("drainHostEvents")) {
        let Ok(events) = serde_json::from_str::<Vec<serde_json::Value>>(&raw) else {
            return;
        };
        for event in events.into_iter().take(64) {
            if event["type"] == "back" {
                if ui::textform::is_open(&form) {
                    ui::textform::close(&mut form);
                } else if ui::p75::modal() {
                    ui::p75::request(ui::p75::Action::Close);
                } else if chat.expanded {
                    chat.expanded = false;
                    chat.focused = false;
                } else if *state.get() == GameState::Menu {
                    shell.panel = Panel::Lobby;
                    shell.dirty = true;
                    queue
                        .0
                        .extend([ui::UiAction::ClosePanel, ui::UiAction::CloseRanking]);
                } else if *state.get() == GameState::Queue {
                    queue.0.push(ui::UiAction::Cancel);
                } else if *state.get() == GameState::Training {
                    training.0.push(crate::training::Action::Back);
                } else if *state.get() == GameState::Result {
                    queue.0.push(ui::UiAction::ToMenu);
                } else {
                    queue.0.push(ui::UiAction::Deselect);
                }
            } else if event["type"] == "image-picked" {
                if let (Some(id), Some(name), Some(success)) = (
                    event["id"].as_u64(),
                    event["name"].as_str(),
                    event["success"].as_bool(),
                ) {
                    crate::platform::file_pick::android::finish(id, name, success);
                }
            } else if event["type"] == "link-unavailable" {
                toasts.push_for(lang.key("mobile.link_unavailable"), 4.);
            } else if event["type"] == "backup-export" {
                if let (Some(epoch), Some(name), Some(saved), Some(cancelled)) = (
                    event["epoch"].as_u64(),
                    event["name"].as_str(),
                    event["saved"].as_bool(),
                    event["cancelled"].as_bool(),
                ) {
                    ui::account_backup::export_completed(
                        epoch,
                        name,
                        saved,
                        cancelled,
                        &mut account,
                        &mut toasts,
                    );
                }
            } else if event["type"] == "text-commit" {
                if let (Some(id), Some(text)) = (event["id"].as_u64(), event["text"].as_str()) {
                    ui::mobile::text::commit(&mut form, id, text);
                }
            } else if event["type"] == "text-cancel" {
                if let Some(id) = event["id"].as_u64() {
                    ui::mobile::text::cancel(&form, id);
                }
            }
        }
    }
}
