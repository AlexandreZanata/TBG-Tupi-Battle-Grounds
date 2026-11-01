//! One application builder shared by desktop, browser and Android.

use crate::*;
use bevy::asset::AssetMetaCheck;

pub fn run() {
    // P66: the release tooling stamps descriptors with the exact contract.
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    if std::env::args().any(|arg| arg == "--contract") {
        println!("{}", net::admission::contract_id());
        return;
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    if std::env::args().any(|arg| arg == "--version") {
        println!(
            "Tupi Battle Grounds {} · protocol {} · source {}",
            env!("CARGO_PKG_VERSION"),
            tbg_core::protocol::PROTOCOL_VERSION,
            option_env!("TBG_SOURCE_COMMIT").unwrap_or("development")
        );
        return;
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    if std::env::args().any(|arg| arg == "--build-info") {
        println!(
            "{}",
            serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"protocol":tbg_core::protocol::PROTOCOL_VERSION,"contract":net::admission::contract_id(),"source_commit":option_env!("TBG_SOURCE_COMMIT").unwrap_or("development"),"target":env!("TBG_BUILD_TARGET"),"profile":if cfg!(debug_assertions){"debug"}else{"release"},"features":if cfg!(feature="ui-acceptance"){vec!["ui-acceptance"]}else{vec![]}})
        );
        return;
    }
    // One synthetic probe of the shared vault; P66 harnesses used the older flag name.
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "ui-acceptance"
    ))]
    if std::env::args().any(|arg| arg == "--auth-vault-probe" || arg == "--resume-vault-probe") {
        platform::credentials::probe();
        return;
    }
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "ui-acceptance"
    ))]
    if std::env::args().any(|arg| arg == "--auth-native-journey") {
        auth::acceptance_journey();
        return;
    }
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    if let Err(message) = platform::window::validate_assets() {
        eprintln!("TBG: {message}");
        std::process::exit(2);
    }
    // P66 resume restoration now runs inside the auth coordinator's first restore.
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    let window = platform::window::settings();
    #[cfg(target_arch = "wasm32")]
    let window = Window {
        title: "Tupi Battle Grounds".into(),
        canvas: Some("#bevy".into()),
        fit_canvas_to_parent: true,
        ..default()
    };
    #[cfg(target_os = "android")]
    let window = platform::android::settings();
    let plugins = DefaultPlugins;
    #[cfg(target_os = "android")]
    let plugins = plugins.set(platform::android::render::plugin());
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    let plugins = plugins.set(platform::window::log_plugin());
    let exit = App::new()
        .add_plugins(
            plugins
                .set(WindowPlugin {
                    primary_window: Some(window),
                    ..default()
                })
                // Static hosts answer unknown `.meta` paths with index.html
                // (200), which the asset loader would try to parse.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
                    file_path: platform::window::asset_path(),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(if cfg!(tbg_desktop) {
            Color::srgb_u8(7, 15, 22)
        } else {
            art::palette::SKY_TOP
        }))
        .insert_resource(Time::<Fixed>::from_hz(20.0))
        .init_state::<GameState>()
        .add_plugins((
            art::plugin,
            camera::plugin,
            arena::plugin,
            overlay::plugin,
            link::plugin,
            visuals::plugin,
            sync::plugin,
            stress::plugin,
            ui::plugin,
            celebration::plugin,
            autoplay::plugin,
            gallery::plugin,
            i18n::plugin,
            vfx::plugin,
            audio::plugin,
        ))
        .add_plugins(auth::plugin)
        .add_plugins(training::plugin)
        .add_plugins(platform_plugin)
        .run();
    // P66: the launcher reads the exit code (75 applies a staged update).
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    if let AppExit::Error(code) = exit {
        std::process::exit(i32::from(code.get()));
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    let _ = exit;
}

fn platform_plugin(app: &mut App) {
    #[cfg(all(
        not(target_os = "android"),
        any(debug_assertions, feature = "ui-acceptance")
    ))]
    capture::plugin(app);
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    platform::window::plugin(app);
    #[cfg(target_os = "android")]
    platform::android::plugin(app);
    #[cfg(target_arch = "wasm32")]
    let _ = app;
}
