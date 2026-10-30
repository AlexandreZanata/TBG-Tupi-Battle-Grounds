//! Allow-listed HTML actions enter the same queue as Bevy buttons.

use super::{ActionQueue, UiAction};
use crate::GameState;
use bevy::prelude::*;

pub(super) fn poll(mut queue: ResMut<ActionQueue>, state: Res<State<GameState>>) {
    let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("tbg-lobby"))
    else {
        return;
    };
    let Some(value) = root.get_attribute("data-action") else {
        return;
    };
    let _ = root.remove_attribute("data-action");
    if *state.get() != GameState::Menu {
        return;
    }
    if let Some(action) = action(&value) {
        queue.0.push(action);
    }
}

fn action(value: &str) -> Option<UiAction> {
    Some(match value {
        "practice" => UiAction::Practice,
        "quick" => UiAction::Quick,
        "ranked" => UiAction::Ranked,
        "challenge" => UiAction::CreateChallenge,
        "create" => UiAction::CreateRoom,
        "register" => UiAction::CreateAccount,
        "login" => UiAction::HaveKey,
        "logout" => UiAction::Logout,
        "copy-key" => UiAction::CopyKey,
        "download-key" => UiAction::DownloadKey,
        "ranking" => UiAction::OpenRanking,
        "close-ranking" => UiAction::CloseRanking,
        "close-panel" => UiAction::ClosePanel,
        "sound" => UiAction::ToggleSound,
        "menu" => UiAction::Disabled,
        // P41: the language selector pokes `data-action="lang:pt-BR"`.
        _ if value.starts_with("lang:") => {
            return crate::i18n::Lang::from_code(&value["lang:".len()..]).map(UiAction::SetLang);
        }
        _ => {
            let code = value.strip_prefix("join:")?;
            if !code
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            {
                return None;
            }
            UiAction::JoinRoom(code.as_bytes().try_into().ok()?)
        }
    })
}
