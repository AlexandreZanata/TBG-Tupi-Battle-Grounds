//! Sequential immutable P70 pose sheets from the real renderer, without combat cues.

use crate::gallery::polish::{Review, PAGE_COUNT};
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured},
};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

#[derive(Resource, Default)]
struct Pending(bool);

pub fn plugin(app: &mut App) {
    app.init_resource::<Pending>().add_systems(Update, capture);
}

fn capture(
    mut commands: Commands,
    review: Option<Res<Review>>,
    mut pending: ResMut<Pending>,
    window: Single<&Window>,
    loading: Res<crate::ui::native_loading::Status>,
    mut ready_frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(review) = review else { return };
    if review.page >= PAGE_COUNT {
        exit.write(AppExit::Success);
        return;
    }
    *ready_frames = if loading.boot_ready && !loading.scene_waiting {
        ready_frames.saturating_add(1)
    } else {
        0
    };
    // Let the loading veil fade and transform/shadow pipelines settle.
    if pending.0 || review.settled_frames < 12 || *ready_frames < 30 {
        return;
    }
    let Some(directory) = crate::config::url_param("capture_polish") else {
        return;
    };
    let prefix = PathBuf::from(directory).join(format!("{:02}", review.page));
    let png = prefix.with_extension("png");
    let sidecar = prefix.with_extension("json");
    if png.exists() || sidecar.exists() {
        error!("P70 refuses to overwrite a capture: {}", prefix.display());
        pending.0 = true;
        exit.write(AppExit::error());
        return;
    }
    let mut metadata = review.metadata();
    metadata["viewport"] = serde_json::json!([window.physical_width(), window.physical_height()]);
    metadata["dpi"] = serde_json::json!(window.scale_factor());
    metadata["quality"] =
        serde_json::json!(crate::config::url_param("quality").unwrap_or_else(|| "standard".into()));
    metadata["image"] = serde_json::json!(png);
    let result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&sidecar)
        .and_then(|mut file| file.write_all(metadata.to_string().as_bytes()));
    if let Err(error) = result {
        error!("P70 metadata write failed: {error}");
        pending.0 = true;
        exit.write(AppExit::error());
        return;
    }
    pending.0 = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(png))
        .observe(
            |_: On<ScreenshotCaptured>,
             mut review: ResMut<Review>,
             mut pending: ResMut<Pending>| {
                review.page += 1;
                review.settled_frames = 0;
                pending.0 = false;
            },
        );
}
