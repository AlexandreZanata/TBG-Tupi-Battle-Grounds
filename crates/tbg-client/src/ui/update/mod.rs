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
mod view;
#[cfg(target_arch = "wasm32")]
mod web;

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
#[derive(Resource)]
pub struct Updater {
    pub state: UpdState,
    /// `Later` hides the modal for this build until the next launch.
    pub snoozed: Option<String>,
    /// The pending notes receipt to present, as the helper reported it.
    pub notes: Option<serde_json::Value>,
    #[cfg(not(target_arch = "wasm32"))]
    inner: native::Inner,
    #[cfg(target_arch = "wasm32")]
    inner: web::Inner,
    pub(super) opened: bool,
    pub(super) automatic: bool,
    pub(crate) idle: bool,
    pending_play: Option<UiAction>,
}

impl Default for Updater {
    fn default() -> Self {
        Self {
            state: UpdState::Off,
            snoozed: None,
            notes: None,
            inner: Default::default(),
            idle: false,
            pending_play: None,
            opened: false,
            automatic: crate::platform::storage::get(
                crate::platform::storage::Scope::Local,
                "update.automatic",
            )
            .as_deref()
                != Some("0"),
        }
    }
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
        UiAction::UpdateAutomatic
            | UiAction::UpdateNow
            | UiAction::UpdateCheck
            | UiAction::UpdateOpen
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
    mut exit: MessageWriter<AppExit>,
) {
    let mine: Vec<UiAction> = queue.0.iter().copied().filter(ours).collect();
    queue.0.retain(|a| !ours(a));
    for action in mine {
        if action == UiAction::UpdateAutomatic {
            updater.automatic = !updater.automatic;
            crate::platform::storage::set(
                crate::platform::storage::Scope::Local,
                "update.automatic",
                if updater.automatic { "1" } else { "0" },
            );
            continue;
        }
        #[cfg(not(target_arch = "wasm32"))]
        native::apply(&mut updater, action, *state.get(), &mut exit);
        #[cfg(target_arch = "wasm32")]
        web::apply(&mut updater, action);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let online = |a: &UiAction| {
            matches!(
                a,
                UiAction::Quick
                    | UiAction::Ranked
                    | UiAction::CreateRoom
                    | UiAction::JoinRoom(_)
                    | UiAction::CreateChallenge
                    | UiAction::JoinChallenge(_)
                    | UiAction::Watch(_)
            )
        };
        if native::stage_busy(&updater) && queue.0.iter().any(online) {
            updater.pending_play = queue.0.iter().find(|a| online(a)).copied();
            queue.0.retain(|a| !online(a));
            native::apply(
                &mut updater,
                UiAction::UpdateCancel,
                *state.get(),
                &mut exit,
            );
        }
        if !native::stage_busy(&updater) && !updater.applying() {
            if let Some(action) = updater.pending_play.take() {
                queue.0.push(action);
            }
        }
    }
    if updater.applying() {
        queue.0.clear();
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Updater>();
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::plugin(app);
    }
    #[cfg(target_arch = "wasm32")]
    web::plugin(app);
}
