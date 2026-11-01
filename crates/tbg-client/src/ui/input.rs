//! Tap/click → `UiAction`, without Bevy picking. A press and release in
//! the same frame (fast phone taps, synthetic clicks) never registered
//! with hover-based picking; this hit-tests the release position directly:
//! buttons first, then the nearest unit on screen (own or enemy, for its
//! status panel) or your own Stronghold (its Fortification panel), else
//! "tap outside".

use bevy::prelude::*;

use super::{ActionQueue, UiAction};
use crate::camera::ArenaCamera;

type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        &'static UiAction,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        &'static bevy::ui::ComputedStackIndex,
        Option<&'static bevy::ui::CalculatedClip>,
    ),
>;

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
    form: NonSend<super::textform::ActiveForm>,
    mouse: Res<ButtonInput<MouseButton>>,
    #[cfg(target_os = "android")] pointer: Res<super::mobile::input::Pointer>,
    touches: Res<Touches>,
    window: Single<&Window>,
    scale: Res<UiScale>,
    buttons: Buttons,
    units: Query<(&UiAction, &GlobalTransform), Without<ComputedNode>>,
    camera: Single<(&Camera, &GlobalTransform), With<ArenaCamera>>,
    mut queue: ResMut<ActionQueue>,
    (time, mut pressed_at): (Res<Time>, Local<f64>),
) {
    #[cfg(target_os = "android")]
    if window.height() > window.width() {
        return;
    }
    if super::textform::is_open(&form) {
        return;
    }
    let now = time.elapsed_secs_f64();
    if mouse.just_pressed(MouseButton::Left) || touches.iter_just_pressed().next().is_some() {
        *pressed_at = now;
    }
    #[cfg(not(target_os = "android"))]
    let long_press = now - *pressed_at >= LONG_PRESS;
    #[cfg(target_os = "android")]
    let long_press = pointer.tap.is_some_and(|t| t.2);
    #[cfg(not(target_os = "android"))]
    let mut points: Vec<Vec2> = touches.iter_just_released().map(|t| t.position()).collect();
    #[cfg(target_os = "android")]
    let mut points: Vec<Vec2> = pointer.tap.map(|t| t.1).into_iter().collect();
    if mouse.just_released(MouseButton::Left) && {
        #[cfg(target_os = "android")]
        {
            !pointer.touch_frame
        }
        #[cfg(not(target_os = "android"))]
        {
            true
        }
    } {
        points.extend(window.cursor_position());
    }
    for p in points {
        let physical = p * window.scale_factor();
        let hit = buttons
            .iter()
            // Respect clipping and draw order when a modal covers a panel.
            .filter(|(_, node, t, vis, _, clip)| {
                vis.get()
                    && {
                        #[cfg(target_os = "android")]
                        {
                            super::mobile::input::hit(
                                node,
                                **t,
                                physical,
                                window.scale_factor() * scale.0,
                            )
                        }
                        #[cfg(not(target_os = "android"))]
                        {
                            node.contains_point(**t, physical)
                        }
                    }
                    && {
                        #[cfg(target_os = "android")]
                        {
                            pointer.tap.is_none_or(|tap| {
                                super::mobile::input::hit(
                                    node,
                                    **t,
                                    tap.0 * window.scale_factor(),
                                    window.scale_factor() * scale.0,
                                )
                            })
                        }
                        #[cfg(not(target_os = "android"))]
                        {
                            true
                        }
                    }
                    && clip.is_none_or(|c| c.contains_point(physical))
            })
            .max_by_key(|(_, _, _, _, stack, _)| stack.0)
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
