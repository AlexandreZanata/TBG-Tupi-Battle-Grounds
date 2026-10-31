//! Native window preferences, background networking and local diagnostic logs.

use super::storage::{self, Scope};
use bevy::{
    log::{BoxedLayer, LogPlugin},
    prelude::*,
    window::{MonitorSelection, PresentMode, WindowMode, WindowResizeConstraints},
    winit::WinitSettings,
};

pub fn settings() -> Window {
    let width = preference("window.width").unwrap_or(1440).clamp(1280, 7680);
    let height = preference("window.height").unwrap_or(900).clamp(720, 4320);
    #[cfg(debug_assertions)]
    let (width, height) = crate::config::url_param("capture_size")
        .and_then(|s| {
            s.split_once('x')
                .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)))
        })
        .unwrap_or((width, height));
    Window {
        title: format!("Tupi Battle Grounds · {}", env!("CARGO_PKG_VERSION")),
        name: Some("tupi-battle-grounds".into()),
        resolution: (width, height).into(),
        resize_constraints: WindowResizeConstraints {
            min_width: if cfg!(debug_assertions) {
                320.0
            } else {
                1280.0
            },
            min_height: if cfg!(debug_assertions) { 480.0 } else { 720.0 },
            ..default()
        },
        mode: if crate::config::load_flag("window.fullscreen") {
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        },
        present_mode: if crate::config::load_flag("window.no_vsync") {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        },
        ..default()
    }
}

fn preference(key: &str) -> Option<u32> {
    storage::get(Scope::Local, key)?.parse().ok()
}

pub fn plugin(app: &mut App) {
    app.insert_resource(WinitSettings::game())
        .add_systems(Update, controls);
}

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Single<&mut Window>,
    mut saved: Local<(u32, u32)>,
) {
    if keys.just_pressed(KeyCode::F11)
        || (keys.just_pressed(KeyCode::Enter)
            && keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]))
    {
        toggle_fullscreen(&mut window);
    }
    let size = (window.width() as u32, window.height() as u32);
    if size != *saved && window.mode == WindowMode::Windowed {
        *saved = size;
        storage::set(Scope::Local, "window.width", &size.0.to_string());
        storage::set(Scope::Local, "window.height", &size.1.to_string());
    }
}

pub fn toggle_fullscreen(window: &mut Window) {
    let fullscreen = window.mode == WindowMode::Windowed;
    window.mode = if fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };
    crate::config::save_flag("window.fullscreen", fullscreen);
}

pub fn log_plugin() -> LogPlugin {
    LogPlugin {
        custom_layer: file_layer,
        ..default()
    }
}

fn file_layer(_app: &mut App) -> Option<BoxedLayer> {
    let dir = storage::data_dir()?.join("logs");
    std::fs::create_dir_all(&dir).ok()?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let path = dir.join(format!("run-{stamp}.log"));
    let file = std::fs::File::create(&path).ok()?;
    let mut old: Vec<_> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|f| f.file_name().to_string_lossy().starts_with("run-"))
        .map(|f| f.path())
        .collect();
    old.sort();
    for path in old.iter().take(old.len().saturating_sub(5)) {
        let _ = std::fs::remove_file(path);
    }
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!(
            "Tupi Battle Grounds stopped unexpectedly. Diagnostic log: {}",
            path.display()
        );
        original(info);
    }));
    Some(Box::new(
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(std::sync::Mutex::new(file)),
    ))
}

pub fn asset_path() -> String {
    let path = std::env::current_exe()
        .ok()
        .and_then(|p| Some(p.parent()?.join("assets")));
    path.filter(|p| p.is_dir())
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
        .to_string_lossy()
        .into()
}
