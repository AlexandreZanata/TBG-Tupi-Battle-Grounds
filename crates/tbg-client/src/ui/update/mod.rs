//! P66 in-game update experience (BR-UP01–UP04, BR-UP08): the startup
//! check, the update modal at a safe idle point (the lobby), download
//! progress and cancel, "restart to update", and the "what's new" modal
//! of the build that actually runs, shown until acknowledged. Every
//! network, trust and file decision belongs to the launcher's helper; this
//! module only renders its events and asks it to act. Unmanaged runs and
//! the browser show nothing here.

use bevy::prelude::*;

use super::{ActionQueue, UiAction};

#[cfg(not(target_arch = "wasm32"))]
mod helper;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
mod view;

/// What the update overlay currently says.
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
#[derive(Resource, Default)]
pub struct Updater {
    pub state: UpdState,
    /// `Later` hides the modal for this build until the next launch.
    pub snoozed: Option<String>,
    /// The pending notes receipt to present, as the helper reported it.
    pub notes: Option<serde_json::Value>,
    #[cfg(not(target_arch = "wasm32"))]
    inner: native::Inner,
}

impl Updater {
    /// An update is being applied: no new play may start (U09).
    pub fn applying(&self) -> bool {
        self.state == UpdState::Applying
    }
}

/// Is `action` one of the update overlay's own buttons?
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
pub(super) fn take_actions(
    mut queue: ResMut<ActionQueue>,
    mut updater: ResMut<Updater>,
    state: Res<State<crate::GameState>>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: MessageWriter<AppExit>,
) {
    let mine: Vec<UiAction> = queue.0.iter().copied().filter(ours).collect();
    queue.0.retain(|a| !ours(a));
    #[cfg(not(target_arch = "wasm32"))]
    for action in mine {
        native::apply(&mut updater, action, *state.get(), &mut exit);
    }
    #[cfg(target_arch = "wasm32")]
    let _ = (mine, &state);
    if updater.applying() {
        queue.0.clear();
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Updater>();
    #[cfg(not(target_arch = "wasm32"))]
    native::plugin(app);
}
