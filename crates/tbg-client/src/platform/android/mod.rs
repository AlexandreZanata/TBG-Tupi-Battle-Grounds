//! Android window, private settings and lifecycle services; simulation remains shared.

mod endpoint;
mod gles_shader;
mod lifecycle;
pub mod render;
pub mod services;
use bevy::{
    prelude::*,
    window::{MonitorSelection, WindowMode, WindowResizeConstraints},
    winit::WinitSettings,
};

pub fn settings() -> Window {
    Window {
        title: "TBG Prototype".into(),
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        resizable: true,
        resize_constraints: WindowResizeConstraints {
            min_width: 0.,
            min_height: 0.,
            ..default()
        },
        ..default()
    }
}
pub fn plugin(app: &mut App) {
    if super::storage::get(super::storage::Scope::Local, "video.low").is_none() {
        crate::config::save_flag("video.low", true);
    }
    bevy::log::info!(
        "{} · protocol {}",
        concat!("TBG_ANDROID_SOURCE=", env!("TBG_SOURCE_COMMIT")),
        tbg_core::protocol::PROTOCOL_VERSION
    );
    app.insert_resource(WinitSettings::game())
        .add_plugins(crate::ui::mobile::plugin)
        .add_systems(Update, (lifecycle::poll, gles_shader::adapt));
}
pub fn persistent_preference(key: &str) -> bool {
    matches!(
        key,
        "tbg.lang"
            | "tbg.arcanas"
            | "tbg.stronghold"
            | "tbg.arena"
            | "tbg_muted"
            | "audio.volume"
            | "tbg_reduced_motion"
            | "video.low"
            | "practice.difficulty"
    )
}

/// Android release endpoints require WSS; local tests need both feature and debuggable host.
pub fn endpoint_allowed(raw: &str) -> bool {
    endpoint::allowed(raw, internal_loopback(), None)
}

pub fn socket_endpoint_allowed(raw: &str) -> bool {
    endpoint::allowed(
        raw,
        internal_loopback(),
        Some(&crate::net::admission::contract_id()),
    )
}

fn internal_loopback() -> bool {
    cfg!(feature = "android-loopback") && services::flag(jni::jni_str!("isInternalBuild"))
}
