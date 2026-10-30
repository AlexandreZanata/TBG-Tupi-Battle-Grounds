//! Two-bone hinge IK shared by legs and arms, returned as pose rotations.

use bevy::prelude::*;

/// Rotations (upper relative to its parent, lower relative to the upper
/// bone) that bring the end of a chain hanging along -Y from `start` to
/// `target`. The middle joint bends toward `pole`; the hinge stays a pure
/// rotation about the bone's own X so knees and elbows never twist.
pub fn two_bone(start: Vec3, target: Vec3, upper: f32, lower: f32, pole: Vec3) -> (Quat, Quat) {
    let delta = target - start;
    let distance = delta.length().clamp(1e-3, (upper + lower) * 0.9999);
    let direction = delta.try_normalize().unwrap_or(Vec3::NEG_Y);
    let side = (pole - direction * pole.dot(direction))
        .try_normalize()
        .unwrap_or_else(|| direction.any_orthonormal_vector());
    let along = (upper * upper - lower * lower + distance * distance) / (2.0 * distance);
    let joint = direction * along + side * (upper * upper - along * along).max(0.0).sqrt();
    let hinge = side.cross(direction).normalize_or_zero();
    let bone = |dir: Vec3| {
        let y = -dir.normalize_or(Vec3::NEG_Y);
        Quat::from_mat3(&Mat3::from_cols(hinge, y, hinge.cross(y)))
    };
    let u = bone(joint);
    (u, u.inverse() * bone(direction * distance - joint))
}

/// Euler XYZ degrees, the pose format `anim::animate` reads back.
pub fn degrees(q: Quat) -> Vec3 {
    let (x, y, z) = q.to_euler(EulerRot::XYZ);
    Vec3::new(x, y, z) * (180.0 / std::f32::consts::PI)
}

/// The quaternion of a pose rotation in Euler XYZ degrees.
pub fn quat(degrees: Vec3) -> Quat {
    let r = degrees * (std::f32::consts::PI / 180.0);
    Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn end(start: Vec3, (u, l): (Quat, Quat), upper: f32, lower: f32) -> Vec3 {
        start + u * (Vec3::NEG_Y * upper + l * Vec3::NEG_Y * lower)
    }

    #[test]
    fn reaches_targets_and_bends_toward_the_pole() {
        for target in [
            Vec3::new(0.0, -0.3, 0.1),
            Vec3::new(0.05, -0.25, -0.12),
            Vec3::new(-0.1, -0.2, 0.15),
        ] {
            let r = two_bone(Vec3::ZERO, target, 0.19, 0.19, Vec3::Z);
            assert!((end(Vec3::ZERO, r, 0.19, 0.19) - target).length() < 1e-4);
            let knee = r.0 * Vec3::NEG_Y * 0.19;
            let mid = target * 0.5;
            assert!(knee.z > mid.z, "knee {knee:?} bends forward of {mid:?}");
        }
    }

    #[test]
    fn a_straight_hanging_chain_is_the_rest_pose() {
        let (u, l) = two_bone(Vec3::ZERO, Vec3::NEG_Y * 0.379, 0.19, 0.19, Vec3::Z);
        assert!(u.angle_between(Quat::IDENTITY) < 0.15);
        assert!(l.angle_between(Quat::IDENTITY) < 0.3);
    }
}
