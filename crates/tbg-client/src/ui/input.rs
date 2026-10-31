//! Tap/click → `UiAction`, without Bevy picking. A press and release in
//! the same frame (fast phone taps, synthetic clicks) never registered
//! with hover-based picking; this hit-tests the release position directly:
//! buttons first, then the nearest unit on screen (own or enemy, for its
//! status panel) or your own Stronghold (its Fortification panel), else
//! "tap outside".

use bevy::prelude::*;

use super::{ActionQueue, UiAction};
use crate::camera::ArenaCamera;

/// How close (logical px, before UI scale) a tap must be to a unit.
const UNIT_TAP_RADIUS: f32 = 42.0;
/// The same for your Stronghold, which is far bigger on screen; its anchor is
/// lifted to the middle of the tower.
const TOWER_TAP_RADIUS: f32 = 120.0;
const TOWER_ANCHOR: f32 = 1.2;
/// Seconds a press must last to count as a long press.
const LONG_PRESS: f64 = 0.5;

#[allow(clippy::too_many_arguments)] // one system reading every tap source
pub(super) fn taps(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Single<&Window>,
    scale: Res<UiScale>,
    buttons: Query<(
        &UiAction,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    units: Query<(&UiAction, &GlobalTransform), Without<ComputedNode>>,
    camera: Single<(&Camera, &GlobalTransform), With<ArenaCamera>>,
    mut queue: ResMut<ActionQueue>,
    (time, mut pressed_at): (Res<Time>, Local<f64>),
) {
    let now = time.elapsed_secs_f64();
    if mouse.just_pressed(MouseButton::Left) || touches.iter_just_pressed().next().is_some() {
        *pressed_at = now;
    }
    let long_press = now - *pressed_at >= LONG_PRESS;
    let mut points: Vec<Vec2> = touches.iter_just_released().map(|t| t.position()).collect();
    if mouse.just_released(MouseButton::Left) {
        points.extend(window.cursor_position());
    }
    for p in points {
        let physical = p * window.scale_factor();
        let hit = buttons
            .iter()
            // Buttons never overlap in this UI, so the first hit is the one.
            .find(|(_, node, t, vis)| vis.get() && node.contains_point(**t, physical))
            .map(|(a, ..)| *a);
        let action = match hit {
            // Holding a shop card shows its details instead of buying.
            Some(UiAction::Buy(i)) if long_press => Some(UiAction::Inspect(i)),
            Some(a) => Some(a),
            None => nearest_unit(p, &units, &camera, scale.0),
        };
        queue.0.push(action.unwrap_or(UiAction::Deselect));
    }
}

fn nearest_unit(
    p: Vec2,
    units: &Query<(&UiAction, &GlobalTransform), Without<ComputedNode>>,
    camera: &(&Camera, &GlobalTransform),
    scale: f32,
) -> Option<UiAction> {
    let (cam, cam_t) = camera;
    // Nearest by distance relative to each target's own tap size, so the big
    // tower never steals a tap from a unit standing in front of it.
    units
        .iter()
        .filter_map(|(a, t)| {
            let (anchor, radius) = match a {
                UiAction::SelectUnit(_) => (0.5, UNIT_TAP_RADIUS),
                UiAction::SelectTower => (TOWER_ANCHOR, TOWER_TAP_RADIUS),
                _ => return None,
            };
            let screen = cam
                .world_to_viewport(cam_t, t.translation() + Vec3::Y * anchor)
                .ok()?;
            let ratio = screen.distance(p) / (radius * scale);
            (ratio <= 1.0).then_some((ratio, *a))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, a)| a)
}
