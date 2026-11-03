//! Place the battle inside measured mobile HUD bounds without cropping the scene surface.

use super::{ArenaCamera, FOV_Y, LANE_CENTER};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct AboveBattle;
#[derive(Component)]
pub(crate) struct BelowBattle;

pub(super) fn frame(
    window: Single<&Window>,
    camera: Single<(&mut Transform, &mut Projection), With<ArenaCamera>>,
    upper: Single<(&ComputedNode, &UiGlobalTransform), With<AboveBattle>>,
    lower: Single<(&ComputedNode, &UiGlobalTransform), With<BelowBattle>>,
) {
    let surface = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );
    let (top_node, top_transform) = *upper;
    let (bottom_node, bottom_transform) = *lower;
    let top = top_transform.translation.y + top_node.size.y * 0.5;
    let bottom = bottom_transform.translation.y - bottom_node.size.y * 0.5;
    if surface.min_element() <= 0. || top_node.size.x <= 0. || bottom <= top {
        return;
    }
    let (mut transform, mut projection) = camera.into_inner();
    // Rebuild the canonical pose each frame: resizing or overlays cannot accumulate a pan.
    super::frame(
        &mut transform,
        &mut projection,
        window.width(),
        window.height(),
    );
    let distance =
        transform.translation.distance(LANE_CENTER) * (surface.x / top_node.size.x).max(1.);
    let center = Vec2::new(top_transform.translation.x, (top + bottom) * 0.5) / surface;
    let tan = (FOV_Y * 0.5).tan();
    let shift = transform.right() * ((1. - 2. * center.x) * distance * tan * surface.x / surface.y)
        + transform.up() * ((2. * center.y - 1.) * distance * tan);
    // Aim through the bodies rather than their floor anchors so their feet remain above Shop.
    transform.translation = LANE_CENTER + Vec3::Y * 0.7 + transform.back() * distance + shift;
}
