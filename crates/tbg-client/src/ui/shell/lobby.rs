//! Browser-aligned lobby entry point and private-room panel.

#[cfg(not(target_os = "android"))]
mod cards;
pub(in crate::ui) mod chrome;
#[cfg(not(target_os = "android"))]
mod home;
#[cfg(not(target_os = "android"))]
mod modes;
#[cfg(not(target_os = "android"))]
mod profile;

use super::{button, kit, Action, Lang, UiAction};
#[cfg(not(target_os = "android"))]
use super::{data::Data, Account, Panel, Shell};
use bevy::prelude::*;

#[cfg(not(target_os = "android"))]
pub use home::spawn;

pub fn friends(root: &mut ChildSpawnerCommands, lang: Lang) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("lobby.friend.title"), 32.));
        p.spawn(kit::muted(lang.key("lobby.friend.body"), 18.));
        p.spawn(kit::button(
            lang.key("lobby.friend.create"),
            UiAction::CreateRoom,
            320.,
        ));
        p.spawn(button(
            lang.key("lobby.friend.join"),
            Action::RoomInput,
            320.,
        ));
        p.spawn(kit::button(
            lang.key("lobby.mode.challenge.title"),
            UiAction::CreateChallenge,
            320.,
        ));
    });
}
