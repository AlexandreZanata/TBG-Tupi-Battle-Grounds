//! Basalt organ pipes, a molten gate and crater, dead trees, lava veins, crystals,
//! plank bridges and glowing rocks for Volcanic Forge.

use super::{back_row, boulder, contact, scatter, shared, Kit};
use crate::{
    arena::{
        theme::{rgb, Theme},
        Part,
    },
    art::mesh::Shape::*,
};
use bevy::prelude::*;

pub(super) fn build(kit: &mut Kit) {
    let p = Theme::Volcanic.palette();
    for (i, (at, s)) in back_row().into_iter().enumerate() {
        if (at.x - 8.0).abs() < 2.0 && at.z < -3.8 {
            continue;
        }
        if i % 4 == 0 {
            shared::dead_tree(&mut kit.props, at, s, p.wood);
        } else {
            basalt(&mut kit.props, at, s);
        }
    }
    let at = Vec3::new(8.0, 0.0, -5.0);
    kit.props.extend([
        (
            Frustum(0.77, 1.65, 1.9),
            p.dark,
            Transform::from_translation(at + Vec3::Y * 0.9),
        ),
        (
            Frustum(0.65, 0.8, 0.3),
            p.stone,
            Transform::from_translation(at + Vec3::Y * 1.93),
        ),
    ]);
    kit.glow.push((
        Cyl(0.65, 0.015),
        rgb(0xffc95a),
        Transform::from_translation(at + Vec3::Y * 2.085),
    ));
    crater_rim(kit);
    for (i, at, s) in scatter(0xf04e).into_iter().filter(|(i, _, _)| i % 3 != 1) {
        if i % 3 == 0 {
            basalt(&mut kit.props, at, s * 0.65);
        } else {
            boulder(
                &mut kit.props,
                at,
                s,
                if i % 2 == 0 { p.dark } else { p.plant },
            );
        }
        if i % 5 == 0 {
            fissure(&mut kit.glow, at, s);
        }
    }
    for (x, z, s) in [
        (0.0, 3.2, 0.7),
        (4.3, 4.0, 1.0),
        (10.3, 4.4, 1.2),
        (16.5, 3.5, 0.9),
    ] {
        basalt(&mut kit.props, Vec3::new(x, 0.0, z), s * 0.48);
        fissure(&mut kit.glow, Vec3::new(x + 0.5, 0.0, z), s);
    }
    for x in [-2.2, 18.2] {
        kit.props.extend([
            (
                Box(0.55, 0.24, 0.5),
                p.dark,
                Transform::from_xyz(x, 0.12, 1.4),
            ),
            (
                Box(0.28, 0.3, 0.3),
                p.stone,
                Transform::from_xyz(x, 0.38, 1.4),
            ),
            (
                Box(0.75, 0.18, 0.35),
                rgb(0x83747b),
                Transform::from_xyz(x, 0.59, 1.4),
            ),
            (
                Cone(0.3, 0.6),
                p.dark,
                Transform::from_xyz(x + 0.25, 0.22, -1.3)
                    .with_rotation(Quat::from_rotation_z(-0.4)),
            ),
        ]);
    }
    lava_veins(&mut kit.glow);
    for (i, (x, z, s)) in [
        (-0.7, 3.3, 0.8),
        (16.7, 3.5, 0.9),
        (3.0, -3.9, 0.6),
        (13.0, -4.0, 0.7),
    ]
    .into_iter()
    .enumerate()
    {
        let at = Vec3::new(x, 0.0, z);
        shared::crystal(&mut kit.glow, at, s, rgb(0xff8a2d), rgb(0xffd27a));
        if i % 2 == 0 {
            basalt(&mut kit.props, at + Vec3::new(0.7, 0.0, 0.2), 0.4);
        }
    }
    // Plank bridges leave both ends of the island for rocky islets in the lava.
    for (x, dir) in [(-2.5, -1.0f32), (18.5, 1.0)] {
        shared::walkway(
            &mut kit.props,
            Vec3::new(x, 0.02, 2.6),
            dir,
            6,
            0,
            p.wood,
            p.dark,
        );
        islet(
            &mut kit.props,
            &mut kit.glow,
            Vec3::new(x + dir * 3.3, 0.0, 2.6),
        );
    }
    // Dark slabs smoulder in the lava all around.
    for (i, (at, size)) in shared::water_sites(0x1a7a, 14, 2.6).into_iter().enumerate() {
        let r = 0.7 + size * 0.9;
        kit.props.push((
            Cyl(r, 0.5),
            p.dark,
            Transform::from_translation(at + Vec3::Y * 0.12),
        ));
        kit.props.push((
            Cyl(r * 0.92, 0.05),
            p.stone,
            Transform::from_translation(at + Vec3::Y * 0.39),
        ));
        kit.glow.push((
            Cyl(r * 1.2, 0.012),
            rgb(0xffa447),
            Transform::from_translation(at + Vec3::Y * 0.045),
        ));
        if i % 3 == 0 {
            basalt(&mut kit.props, at + Vec3::Y * 0.4, 0.45 * size + 0.15);
        }
    }
}

/// Uneven basalt teeth surround the molten crater; no straight architectural portal.
fn crater_rim(kit: &mut Kit) {
    let p = Theme::Volcanic.palette();
    for (i, (x, z, s)) in [
        (3.6, -4.7, 1.25),
        (5.1, -4.8, 1.7),
        (10.9, -4.8, 1.7),
        (12.4, -4.7, 1.25),
    ]
    .into_iter()
    .enumerate()
    {
        basalt(&mut kit.props, Vec3::new(x, 0.0, z), s);
        kit.glow.push((
            Box(0.065, s * 1.2, 0.035),
            rgb(0xffa52e),
            Transform::from_xyz(x + 0.15, s * 0.65, z + 0.34),
        ));
        shared::crystal(
            &mut kit.glow,
            Vec3::new(x, 0.0, z + 0.8),
            0.50 + i as f32 * 0.06,
            rgb(0xf98122),
            rgb(0xffcf61),
        );
    }
    for i in 0..7 {
        let a = i as f32 * std::f32::consts::TAU / 7.0;
        kit.props.push((
            Frustum(0.24, 0.39, 0.55),
            p.dark,
            Transform::from_xyz(
                8.0 + a.cos() * 0.88,
                2.05 + (i % 3) as f32 * 0.12,
                -5.0 + a.sin() * 0.62,
            )
            .with_rotation(Quat::from_rotation_z(a.cos() * 0.22)),
        ));
    }
    for (from, to) in [
        (Vec3::new(-2.8, 1.5, -3.0), Vec3::new(-5.2, 0.4, -5.3)),
        (Vec3::new(18.8, 1.5, -3.0), Vec3::new(21.2, 0.4, -5.3)),
    ] {
        kit.props.push((
            Frustum(0.36, 0.65, 3.1),
            p.dark,
            Transform::from_xyz(from.x, 0.05, from.z),
        ));
        kit.props.push((
            Frustum(0.50, 0.80, 1.9),
            p.dark,
            Transform::from_xyz(to.x, -0.45, to.z),
        ));
        for i in 0..13 {
            let t = i as f32 / 12.0;
            let at = from.lerp(to, t) - Vec3::Y * (t * (1.0 - t) * 1.1);
            kit.props.push((
                Ring(0.025, 0.12, 360.0),
                rgb(0x554753),
                Transform::from_translation(at).with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    std::f32::consts::FRAC_PI_2,
                    (i % 2) as f32 * std::f32::consts::FRAC_PI_2,
                    -0.6,
                )),
            ));
        }
    }
}

/// A rock that carries the end of a bridge, with a glowing lava halo.
fn islet(props: &mut Vec<Part>, glow: &mut Vec<Part>, at: Vec3) {
    let p = Theme::Volcanic.palette();
    props.push((
        Cyl(0.95, 1.4),
        p.dark,
        Transform::from_translation(at + Vec3::Y * -0.68),
    ));
    props.push((
        Cyl(1.0, 0.06),
        p.stone,
        Transform::from_translation(at + Vec3::Y * 0.0),
    ));
    glow.push((
        Cyl(1.3, 0.012),
        rgb(0xffa447),
        Transform::from_translation(at + Vec3::Y * -1.355),
    ));
}

/// Molten veins along the front and back strips of the island, away from the lane.
fn lava_veins(glow: &mut Vec<Part>) {
    for (a, b) in [
        ((0.5, 3.6), (2.6, 4.4)),
        ((5.4, 4.7), (7.8, 3.9)),
        ((9.0, 4.4), (11.9, 4.8)),
        ((13.4, 3.9), (16.4, 4.3)),
        ((0.0, -3.2), (2.4, -3.9)),
        ((13.6, -3.9), (16.2, -3.3)),
    ] {
        let (a, b) = (Vec2::new(a.0, a.1), Vec2::new(b.0, b.1));
        let points: [Vec2; 5] = std::array::from_fn(|i| {
            a.lerp(b, i as f32 / 4.0)
                + Vec2::new(
                    0.0,
                    if i == 0 || i == 4 {
                        0.0
                    } else if i % 2 == 0 {
                        0.11
                    } else {
                        -0.11
                    },
                )
        });
        for pair in points.windows(2) {
            vein(glow, pair[0], pair[1], 0.055);
        }
    }
}

fn basalt(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let p = Theme::Volcanic.palette();
    contact(parts, at, s * 0.7, rgb(0x38303c));
    for (dx, dz, h, r) in [
        (0.0, 0.0, 1.8, 0.34),
        (0.4, 0.1, 1.2, 0.28),
        (-0.32, 0.1, 0.8, 0.24),
    ] {
        let at = at + Vec3::new(dx, h * 0.5, dz) * s;
        parts.push((Cyl(r * s, h * s), p.dark, Transform::from_translation(at)));
        parts.push((
            Cyl(r * s * 1.01, 0.04 * s),
            p.stone,
            Transform::from_translation(at + Vec3::Y * h * s * 0.49),
        ));
    }
}

fn fissure(parts: &mut Vec<Part>, at: Vec3, s: f32) {
    let points: [Vec2; 4] = std::array::from_fn(|i| {
        Vec2::new(
            at.x + (i as f32 - 1.5) * 0.4 * s,
            at.z + (i % 2) as f32 * 0.12,
        )
    });
    for pair in points.windows(2) {
        vein(parts, pair[0], pair[1], 0.04);
    }
}

fn vein(parts: &mut Vec<Part>, a: Vec2, b: Vec2, width: f32) {
    let mid = (a + b) * 0.5;
    let d = b - a;
    parts.push((
        Box(d.length() + 0.025, 0.012, width),
        rgb(0xf49c33),
        Transform::from_xyz(mid.x, -0.014, mid.y)
            .with_rotation(Quat::from_rotation_y(-d.y.atan2(d.x))),
    ));
}
