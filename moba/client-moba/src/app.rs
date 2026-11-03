//! Application assembly: window, screens and the feature plugins.

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

use crate::{camera, capture, config, hud, i18n, input, link, menu, palette, scene, units};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Menu,
    Match,
}

pub fn run() {
    if std::env::args().any(|a| a == "--version") {
        println!(
            "TBG Arena 5v5 {} · MOBA protocol {}",
            env!("CARGO_PKG_VERSION"),
            tbg_moba_core::protocol::MOBA_PROTOCOL_VERSION
        );
        return;
    }
    let launch = config::Launch::from_args();
    let window = Window {
        title: "TBG · Arena 5v5".into(),
        // Matches StartupWMClass in the Linux desktop entry.
        name: Some("tbg-moba".into()),
        resolution: (1600u32, 900u32).into(),
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy".into()),
        #[cfg(target_arch = "wasm32")]
        fit_canvas_to_parent: true,
        ..default()
    };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(window),
                    ..default()
                })
                // Every asset is embedded or generated; never probe for `.meta` files.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(palette::SKY))
        .insert_resource(launch.lang)
        .insert_resource(launch)
        .init_state::<Screen>()
        .add_plugins((
            i18n::plugin,
            scene::plugin,
            camera::plugin,
            link::plugin,
            units::plugin,
            hud::plugin,
            input::plugin,
            menu::plugin,
            capture::plugin,
        ))
        .run();
}
