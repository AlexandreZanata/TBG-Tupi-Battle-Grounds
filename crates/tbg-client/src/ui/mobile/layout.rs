//! Measure Android density against Bevy coordinates and handle unsupported portrait surfaces.

use crate::{i18n::Lang, platform::android::services, ui::kit};
use bevy::prelude::*;
#[derive(Resource, Default)]
pub struct Metrics {
    pub density: f32,
    /// Extra safe insets inside the surface, in Android dp (left, top, right, bottom).
    pub safe: Vec4,
}
pub fn measure(
    window: Single<&Window>,
    mut metrics: ResMut<Metrics>,
    mut last: Local<(u32, u32)>,
    mut rotate: Local<Option<Entity>>,
    mut commands: Commands,
    lang: Res<Lang>,
) {
    if let Some(raw) = services::string(jni::jni_str!("viewportSnapshot")) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(density) = value["density"]
                .as_f64()
                .filter(|v| v.is_finite() && *v > 0.)
            {
                metrics.density = density as f32;
                let n = |key: &str| value[key].as_f64().unwrap_or(0.) as f32;
                let outside_right = n("width_px") - n("surface_x_px") - n("surface_width_px");
                let outside_bottom = n("height_px") - n("surface_y_px") - n("surface_height_px");
                metrics.safe = Vec4::new(
                    (n("left_px") - n("surface_x_px")).max(0.),
                    (n("top_px") - n("surface_y_px")).max(0.),
                    (n("right_px") - outside_right).max(0.),
                    (n("bottom_px").max(n("ime_bottom_px")) - outside_bottom).max(0.),
                ) / metrics.density;
                if *last != (window.physical_width(), window.physical_height()) {
                    bevy::log::info!("Android viewport {raw}");
                }
            }
        }
    }
    let size = (window.physical_width(), window.physical_height());
    if *last != size && metrics.density > 0. {
        *last = size;
        bevy::log::info!("Android surface {}x{}; Bevy scale {}; Android density {}; effective 48dp control {:.1}px",size.0,size.1,window.scale_factor(),metrics.density,48.*metrics.density);
    }
    let portrait = window.height() > window.width();
    if portrait && rotate.is_none() {
        *rotate = Some(
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(100),
                        height: percent(100),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(kit::INK),
                    GlobalZIndex(10000),
                    children![kit::heading(lang.key("android.rotate"), 22.)],
                ))
                .id(),
        );
    } else if !portrait {
        if let Some(entity) = rotate.take() {
            commands.entity(entity).despawn();
        }
    }
}
