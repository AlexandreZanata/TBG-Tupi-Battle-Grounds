//! Figure-space aiming: the yaw that turns a figure toward its target and
//! the launch direction of the parabolic shots `vfx::shots` flies, so a
//! barrel points along the first instant of its own projectile's flight.

use bevy::prelude::*;

/// A target in figure space (+Z forward) and how far the aim is blended in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aim {
    pub target: Vec3,
    pub weight: f32,
}

/// Apex height of a shot above its chord, as a fraction of its length.
pub fn apex(from: Vec3, to: Vec3, arc: f32) -> f32 {
    from.distance(to) * arc
}

/// Point `t` (0..1) of a shot from `from` to `to` whose apex rises `arc`
/// (fraction of the distance) above the chord.
pub fn flight(from: Vec3, to: Vec3, arc: f32, t: f32) -> Vec3 {
    from.lerp(to, t) + Vec3::Y * 4.0 * apex(from, to, arc) * t * (1.0 - t)
}

/// Unit direction of that flight at `t`.
pub fn tangent(from: Vec3, to: Vec3, arc: f32, t: f32) -> Vec3 {
    let d = to - from + Vec3::Y * 4.0 * apex(from, to, arc) * (1.0 - 2.0 * t);
    d.normalize_or(Vec3::Z)
}

/// Yaw (radians about +Y) that turns +Z from `pivot` toward `target`.
pub fn yaw(pivot: Vec3, target: Vec3) -> f32 {
    let d = target - pivot;
    d.x.atan2(d.z)
}

/// Yaw (radians about +Y, turning about the origin) that puts the line
/// through `muzzle` along `dir` over `target`: a gun carried off-center
/// still lines up with what it shoots at.
pub fn line_yaw(muzzle: Vec3, dir: Vec3, target: Vec3) -> f32 {
    let d = Vec2::new(dir.x, dir.z).normalize_or(Vec2::Y);
    let offset = muzzle.x * d.y - muzzle.z * d.x;
    let t = Vec2::new(target.x, target.z);
    let distance = t.length().max(offset.abs() + 1e-3);
    t.x.atan2(t.y) - (offset / distance).asin() - d.x.atan2(d.y)
}

/// Elevation (radians, up positive) of a direction.
pub fn pitch(dir: Vec3) -> f32 {
    dir.y.atan2(Vec2::new(dir.x, dir.z).length())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flight_starts_and_ends_on_its_points_and_leaves_along_its_tangent() {
        let (from, to) = (Vec3::new(0.0, 1.2, 0.0), Vec3::new(5.0, 0.5, 0.8));
        assert!(flight(from, to, 0.15, 0.0).distance(from) < 1e-5);
        assert!(flight(from, to, 0.15, 1.0).distance(to) < 1e-5);
        let step = (flight(from, to, 0.15, 1e-4) - from).normalize();
        assert!(step.angle_between(tangent(from, to, 0.15, 0.0)) < 1e-2);
        assert!(pitch(tangent(from, to, 0.15, 0.0)) > pitch(to - from));
    }

    #[test]
    fn an_off_center_barrel_turns_onto_its_target() {
        let (muzzle, dir) = (Vec3::new(-0.3, 1.0, 0.6), Vec3::new(0.2, 0.1, 1.0));
        for target in [
            Vec3::new(1.0, 0.5, 5.0),
            Vec3::new(-3.0, 0.2, 2.0),
            Vec3::new(4.0, 0.0, -1.0),
        ] {
            let q = Quat::from_rotation_y(line_yaw(muzzle, dir, target));
            let (m, d) = ((q * muzzle).xz(), (q * dir).xz().normalize());
            let to = target.xz() - m;
            assert!(
                to.perp_dot(d).abs() < 1e-3,
                "{target:?} misses by {}",
                to.perp_dot(d)
            );
            assert!(to.dot(d) > 0.0, "{target:?} is behind the barrel");
        }
    }

    #[test]
    fn yaw_turns_forward_onto_the_target() {
        let pivot = Vec3::new(0.2, 1.0, -0.1);
        for target in [Vec3::new(3.0, 0.0, 1.0), Vec3::new(-2.0, 0.4, 0.5)] {
            let turned = Quat::from_rotation_y(yaw(pivot, target)) * Vec3::Z;
            let flat = (target - pivot).with_y(0.0).normalize();
            assert!(turned.angle_between(flat) < 1e-4);
        }
    }
}
