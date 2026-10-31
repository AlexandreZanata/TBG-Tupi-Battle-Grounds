//! Browser-aligned lobby entry point and private-room panel.

mod cards;
pub(super) mod chrome;
mod home;
mod modes;
mod profile;

use super::{button, data::Data, kit, Account, Action, Lang, Panel, Shell, UiAction};
use bevy::prelude::*;

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
