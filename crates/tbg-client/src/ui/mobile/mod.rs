//! Native landscape composition and touch adapters reuse the existing UI actions.

pub mod input;
#[cfg(target_os = "android")]
pub(in crate::ui) mod insets;
#[cfg(target_os = "android")]
pub mod layout;
#[cfg(target_os = "android")]
pub mod shell;
#[cfg(target_os = "android")]
pub mod text;
#[cfg(target_os = "android")]
use bevy::prelude::*;
#[cfg(target_os = "android")]
pub fn plugin(app: &mut App) {
    app.init_resource::<input::Pointer>()
        .init_resource::<layout::Metrics>()
        .add_systems(PreUpdate, input::collect.after(bevy::input::InputSystems))
        .add_systems(Update, (text::poll, layout::measure))
        .add_systems(
            PostUpdate,
            insets::apply.before(bevy::ui::UiSystems::Prepare),
        );
}
