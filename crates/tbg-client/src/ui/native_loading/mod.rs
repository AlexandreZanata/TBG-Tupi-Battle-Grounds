//! Linux presentation readiness: shared asset cache, GPU-aware placeholders and arena transitions.

mod assets;
mod fields;
mod gpu;
mod scene;

use bevy::prelude::*;

#[derive(Component)]
struct Decoration;

#[derive(Resource, Default)]
pub(crate) struct Status {
    pub boot_ready: bool,
    pub scene_waiting: bool,
    pub images: usize,
    pub texts: usize,
    pub failed: usize,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Status>();
    assets::plugin(app);
    gpu::plugin(app);
    scene::plugin(app);
    fields::plugin(app);
}

fn forced() -> bool {
    cfg!(debug_assertions)
        && crate::config::url_param("capture").is_some()
        && crate::config::url_param("capture_loading").is_some()
}

fn ink(alpha: f32) -> Color {
    Color::srgba(0.06, 0.105, 0.14, alpha)
}

pub(in crate::ui) fn rows(parent: &mut ChildSpawnerCommands, count: usize) {
    for i in 0..count {
        parent
            .spawn((
                Node {
                    width: percent(100),
                    min_height: px(30),
                    margin: UiRect::bottom(px(5)),
                    border_radius: BorderRadius::all(px(4)),
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(px(12)),
                    ..default()
                },
                BackgroundColor(ink(1.)),
                Decoration,
            ))
            .with_children(|row| {
                for width in [7., 33., 14., 14., 14.] {
                    row.spawn((
                        Node {
                            width: percent(width),
                            height: px(8),
                            margin: UiRect::right(px(10)),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(
                            0.28,
                            0.36,
                            0.39,
                            0.45 - (i % 3) as f32 * 0.04,
                        )),
                        Decoration,
                    ));
                }
            });
    }
}
