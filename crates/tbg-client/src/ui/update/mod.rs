//! P66 in-game update experience (BR-UP01–UP04, BR-UP08): the startup
//! check, the update modal at a safe idle point (the lobby), download
//! progress and cancel, "restart to update", and the "what's new" modal
//! of the build that actually runs, shown until acknowledged. Every
//! network, trust and file decision belongs to the launcher's helper; this
//! module only renders its events and asks it to act. Unmanaged runs and
//! the browser show nothing here.

use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use super::{ActionQueue, UiAction};

#[cfg(not(target_arch = "wasm32"))]
mod helper;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
mod view;

/// What the update overlay currently says.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, PartialEq, Default)]
pub enum UpdState {
    /// Not managed by the launcher, or nothing to say.
    #[default]
    Off,
    Checking,
    Available {
        build: String,
        version: String,
        size: u64,
        required: bool,
    },
    Downloading {
        build: String,
        version: String,
        done: u64,
        total: u64,
    },
    Staged {
        version: String,
    },
    /// Exiting so the launcher activates the staged build.
    Applying,
    Error(String),
}

/// The update UI state shared by its systems.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource, Default)]
pub struct Updater {
    pub state: UpdState,
    /// `Later` hides the modal for this build until the next launch.
    pub snoozed: Option<String>,
    /// The pending notes receipt to present, as the helper reported it.
    pub notes: Option<serde_json::Value>,
    inner: native::Inner,
}

#[cfg(not(target_arch = "wasm32"))]
impl Updater {
    /// An update is being applied: no new play may start (U09).
    pub fn applying(&self) -> bool {
        self.state == UpdState::Applying
    }
}

/// Is `action` one of the update overlay's own buttons?
#[cfg(not(target_arch = "wasm32"))]
fn ours(action: &UiAction) -> bool {
    matches!(
        action,
        UiAction::UpdateNow
            | UiAction::UpdateLater
            | UiAction::UpdateCancel
            | UiAction::UpdateRestart
            | UiAction::UpdateNotesOk
    )
}

/// Runs before `actions::apply`: handles the update buttons and, while an
/// update is applying, drops every other action (the Update/Play race).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn take_actions(
    mut queue: ResMut<ActionQueue>,
    mut updater: ResMut<Updater>,
    state: Res<State<crate::GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    let mine: Vec<UiAction> = queue.0.iter().copied().filter(ours).collect();
    queue.0.retain(|a| !ours(a));
    for action in mine {
        native::apply(&mut updater, action, *state.get(), &mut exit);
    }
    if updater.applying() {
        queue.0.clear();
    }
}

/// The browser has no installer to drive (new builds arrive on reload).
#[cfg(target_arch = "wasm32")]
pub(super) fn take_actions() {}

pub fn plugin(app: &mut App) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.init_resource::<Updater>();
        native::plugin(app);
    }
    #[cfg(target_arch = "wasm32")]
    let _ = app;
}
