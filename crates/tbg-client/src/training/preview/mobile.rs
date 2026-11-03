//! Frame the unchanged preview actor inside the measured mobile stage rectangle.
use bevy::prelude::*;

pub(super) fn frame(
    window: &Window,
    viewport: (&ComputedNode, &UiGlobalTransform),
    radius: f32,
    height: f32,
    zoom: f32,
) -> Transform {
    let (node, transform) = viewport;
    let surface = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );
    let fraction = (node.size / surface).clamp(Vec2::splat(0.01), Vec2::ONE);
    let aspect = surface.x / surface.y.max(1.);
    let tan = (crate::camera::FOV_Y * 0.5).tan();
    let back = Vec3::new(0., 0.22, 1.).normalize();
    let up = Vec3::new(0., back.z, -back.y);
    // Include the actor's depth in the nearest point and projected height.
    let vertical = height * up.y + 2. * radius * up.z.abs();
    let fit = (vertical / (2. * tan * fraction.y)).max(radius / (tan * aspect * fraction.x));
    let distance = (radius + fit * 1.12).max(10.8) / zoom;
    let center = transform.translation / surface;
    let shift = Vec3::X * ((1. - 2. * center.x) * distance * tan * aspect)
        + up * ((2. * center.y - 1.) * distance * tan);
    let aim = Vec3::new(8., super::super::stage::FLOOR + height * 0.5, 0.) + shift;
    Transform::from_translation(aim + back * distance).looking_at(aim, Vec3::Y)
}
