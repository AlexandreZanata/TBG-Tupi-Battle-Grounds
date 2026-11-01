//! Android window, private settings and lifecycle services; simulation remains shared.

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
        .add_systems(Update, lifecycle::poll);
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

/// Android release endpoints require WSS; local emulator tests are opt-in debug builds.
pub fn endpoint_allowed(raw: &str) -> bool {
    let Ok(url) = url::Url::parse(raw) else {
        return false;
    };
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return false;
    }
    url.scheme() == "wss"
        || (cfg!(feature = "android-loopback")
            && services::flag(jni::jni_str!("isInternalBuild"))
            && url.scheme() == "ws"
            && url.host_str() == Some("127.0.0.1"))
}
