//! Camera-projected body envelopes keep HP and Evolution markers above the current silhouette.
use super::UnitVisual;
use crate::{art::anim::Animator, camera::ArenaCamera};
use bevy::prelude::*;

/// Project current body bounds so long heads and rotated silhouettes never cover HP.
pub(super) fn billboards(
    camera: Single<(&Camera, &Transform), With<ArenaCamera>>,
    units: Query<(&UnitVisual, &Animator, &Transform), (With<UnitVisual>, Without<ArenaCamera>)>,
    bounds: Query<&bevy::camera::primitives::Aabb>,
    mut transforms: Query<&mut Transform, (Without<ArenaCamera>, Without<UnitVisual>)>,
) {
    use crate::art::rig::{self, Node, Socket};
    let (camera, eye) = *camera;
    let Some(viewport) = camera.logical_viewport_rect() else {
        return;
    };
    let clip = camera.clip_from_view() * eye.to_matrix().inverse();
    for (v, anim, root) in &units {
        if v.dying {
            continue;
        }
        let figure = transforms
            .get(v.figure)
            .map_or(Mat4::IDENTITY, Transform::to_matrix);
        let joints =
            crate::art::presentation::pose_world(v.kind(), &anim.plan, &anim.current_pose());
        let frame = root.to_matrix() * figure;
        let top = rig::nodes()
            .filter(|node| {
                !matches!(
                    node,
                    Node::Prop(
                        Socket::WeaponR
                            | Socket::OffhandL
                            | Socket::Nock
                            | Socket::StringTop
                            | Socket::StringBottom
                    )
                )
            })
            .filter_map(|node| {
                let aabb = bounds.get(anim.joints[node.index()]).ok()?;
                let world = frame * joints[node.index()];
                projected_top(
                    clip,
                    viewport,
                    world,
                    aabb.center.into(),
                    aabb.half_extents.into(),
                )
            })
            .reduce(f32::min);
        if let Ok(mut bar) = transforms.get_mut(v.bar) {
            bar.rotation = eye.rotation;
            if let Some(top) = top {
                bar.translation.y = overhead_height(clip, viewport, root.translation, top - 8.0);
            }
        }
    }
}

fn projected_top(clip: Mat4, viewport: Rect, frame: Mat4, center: Vec3, half: Vec3) -> Option<f32> {
    let mut top: Option<f32> = None;
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let point = frame.transform_point3(center + half * Vec3::new(x, y, z));
                if let Some(y) = screen_y(clip, viewport, point) {
                    top = Some(top.map_or(y, |old| old.min(y)));
                }
            }
        }
    }
    top
}

fn overhead_height(clip: Mat4, viewport: Rect, root: Vec3, screen_top: f32) -> f32 {
    let (mut low, mut high) = (0.0, 12.0);
    for _ in 0..10 {
        let middle = (low + high) * 0.5;
        let above =
            screen_y(clip, viewport, root + Vec3::Y * middle).is_some_and(|y| y <= screen_top);
        if above {
            high = middle;
        } else {
            low = middle;
        }
    }
    high
}

fn screen_y(clip: Mat4, viewport: Rect, point: Vec3) -> Option<f32> {
    let h = clip * point.extend(1.0);
    if h.w <= 0.0 {
        return None;
    }
    let ndc = h.truncate() / h.w;
    (ndc.is_finite() && (0.0..=1.0).contains(&ndc.z))
        .then_some((1.0 - ndc.y) * 0.5 * viewport.height() + viewport.min.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::CameraProjection;
    #[test]
    fn p71_long_body_projection_clears_hp_where_height_only_anchor_intersects() {
        let eye =
            Transform::from_xyz(8.0, 19.73, 13.81).looking_at(Vec3::new(8.0, 0.0, 0.0), Vec3::Y);
        let clip = PerspectiveProjection {
            fov: 35f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            ..default()
        }
        .get_clip_from_view()
            * eye.to_matrix().inverse();
        let viewport = Rect::from_corners(Vec2::ZERO, Vec2::new(1920.0, 1080.0));
        let root = Vec3::new(8.0, 0.0, 0.0);
        let top = projected_top(
            clip,
            viewport,
            Mat4::from_translation(root),
            Vec3::new(0.0, 0.8, 0.0),
            Vec3::new(0.5, 0.8, 2.0),
        )
        .unwrap();
        assert!(screen_y(clip, viewport, root + Vec3::Y * 1.92).unwrap() > top);
        let h = overhead_height(clip, viewport, root, top - 8.0);
        assert!(screen_y(clip, viewport, root + Vec3::Y * h).unwrap() <= top - 7.9);
    }
}
