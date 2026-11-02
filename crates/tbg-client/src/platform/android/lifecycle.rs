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
    state: Res<State<GameState>>,
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
                } else if *state.get() == GameState::Menu {
                    shell.panel = Panel::Lobby;
                    shell.dirty = true;
                    queue
                        .0
                        .extend([ui::UiAction::ClosePanel, ui::UiAction::CloseRanking]);
                } else if *state.get() == GameState::Queue {
                    queue.0.push(ui::UiAction::Cancel);
                } else if *state.get() == GameState::Result {
                    queue.0.push(ui::UiAction::ToMenu);
                } else {
                    queue.0.push(ui::UiAction::Deselect);
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
