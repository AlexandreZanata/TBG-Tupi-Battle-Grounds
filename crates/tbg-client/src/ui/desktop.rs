//! Desktop HTML shell visibility and safe profile projection across the DOM bridge.

use bevy::prelude::*;

#[derive(Resource, Default)]
pub(super) struct DesktopLobby(pub bool);

pub fn plugin(app: &mut App) {
    app.init_resource::<DesktopLobby>();
    #[cfg(target_arch = "wasm32")]
    app.add_systems(
        PreUpdate,
        (super::desktop_actions::poll, web::update).chain(),
    );
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use crate::ui::{
        account::{Account, AccountScreen},
        ranking::Ranking,
        textform::{self, ActiveForm},
    };
    use crate::{audio::Sounds, camera, GameState};
    use wasm_bindgen::JsCast;

    fn attr(root: &web_sys::HtmlElement, name: &str, value: &str) {
        if root.get_attribute(name).as_deref() != Some(value) {
            let _ = root.set_attribute(name, value);
        }
    }

    pub(super) fn update(
        window: Single<&Window>,
        state: Res<State<GameState>>,
        account: Res<Account>,
        ranking: Res<Ranking>,
        sounds: Res<Sounds>,
        form: NonSend<ActiveForm>,
        mut lobby: ResMut<DesktopLobby>,
    ) {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("tbg-lobby"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
        let Some(root) = root else {
            lobby.0 = false;
            return;
        };
        lobby.0 = camera::desktop(window.width(), window.height())
            && root.get_attribute("data-ready").as_deref() == Some("true");
        let screen = if !lobby.0 || *state.get() != GameState::Menu {
            "off"
        } else if ranking.open {
            "ranking"
        } else if matches!(account.screen, AccountScreen::SaveKey { .. }) {
            "key"
        } else if textform::is_open(&form) {
            "form"
        } else {
            "lobby"
        };
        root.set_hidden(screen != "lobby");
        attr(&root, "data-screen", screen);
        attr(
            &root,
            "data-muted",
            if sounds.muted { "true" } else { "false" },
        );
        if let Some(profile) = &account.profile {
            attr(&root, "data-player", &profile.name);
            attr(&root, "data-rating", &profile.rating.to_string());
        } else {
            attr(&root, "data-player", "");
            attr(&root, "data-rating", "");
        }
    }
}
