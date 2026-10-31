//! Conservative rig envelopes, staggered slots and a seat-aware frontal camera with UI safe space.

use bevy::prelude::*;
use tbg_core::Seat;

pub(super) struct Formation {
    pub positions: Vec<Vec3>,
    pub scale: f32,
    pub camera: Transform,
}

pub(super) fn formation(n: usize, radius: f32, height: f32, aspect: f32, seat: Seat) -> Formation {
    let cols = ((n.max(1) as f32 * 2.).sqrt().ceil() as usize).min(n.max(1));
    let rows = n.max(1).div_ceil(cols);
    let gap = 2. * radius + 0.25;
    let scale = 1f32
        .min(13. / (gap * (cols as f32 + 0.25)))
        .min(5. / (gap * rows as f32));
    let mut positions = Vec::with_capacity(n);
    for i in 0..n {
        let row = i / cols;
        let col = i % cols;
        let width = cols.min(n - row * cols);
        positions.push(Vec3::new(
            8. + (col as f32 - (width - 1) as f32 * 0.5 + (row % 2) as f32 * 0.5) * gap * scale,
            0.12,
            (row as f32 - (rows - 1) as f32 * 0.5) * gap * scale,
        ));
    }
    let dir = Vec3::new(if seat == Seat::A { -0.15 } else { 0.15 }, 0.55, 1.).normalize();
    let right = Vec3::Y.cross(dir).normalize();
    let up = dir.cross(right).normalize();
    let centre = Vec3::new(8., height * scale * 0.5 + 0.12, 0.);
    let mut horizontal = 0.5f32;
    let mut vertical = 0.5f32;
    let mut depth = 0.5f32;
    for p in &positions {
        for x in [-radius, radius] {
            for y in [0., height] {
                for z in [-radius, radius] {
                    let v = *p + Vec3::new(x, y, z) * scale - centre;
                    horizontal = horizontal.max(v.dot(right).abs());
                    vertical = vertical.max(v.dot(up).abs());
                    depth = depth.max(v.dot(dir).abs());
                }
            }
        }
    }
    let tan = (crate::camera::FOV_Y * 0.5).tan();
    let distance =
        (horizontal / (tan * aspect.max(0.5) * 0.88)).max(vertical / (tan * 0.43)) + depth;
    let aim = centre - up * (distance * tan * 0.43);
    let camera = Transform::from_translation(aim + dir * distance).looking_at(aim, Vec3::Y);
    Formation {
        positions,
        scale,
        camera,
    }
}

pub(super) fn facing(from: Vec3, camera: Vec3) -> Quat {
    let d = (camera - from).with_y(0.).normalize_or_zero();
    Quat::from_rotation_y(d.x.atan2(d.z))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn celebration_formation_faces_both_seats_and_keeps_nonoverlapping_envelopes() {
        for n in [0, 1, 3, 64] {
            for seat in Seat::BOTH {
                for aspect in [1366. / 768., 1920. / 1080.] {
                    let f = formation(n, 1.6, 3.0, aspect, seat);
                    assert!(f.scale > 0. && f.scale <= 1.);
                    for (i, p) in f.positions.iter().enumerate() {
                        let forward = facing(*p, f.camera.translation) * Vec3::Z;
                        assert!(
                            forward.dot((f.camera.translation - *p).with_y(0.).normalize()) > 0.999
                        );
                        for q in &f.positions[i + 1..] {
                            assert!(p.distance(*q) >= 3.2 * f.scale);
                        }
                    }
                }
            }
        }
    }
}
