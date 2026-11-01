//! P70 spear grip/contact measurements and sampled body-clearance regressions.

use super::{model, motion, weapon, KIT};
use crate::art::{
    anim::{Animator, Clip, ClipEvent, Pose, CLIPS},
    curves::blend,
    figure,
    kits::{common::Form, Aim, Moment},
    presentation::pose_world,
    rig::{Joint::*, Node, Node::*, Socket, Spring, NODE_COUNT},
    FORMS,
};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use tbg_core::UnitKind;

fn moment(form: (u8, u8), clip: Clip, t: f32) -> Moment {
    Moment {
        clip,
        t,
        tier: form.0,
        branch: form.1,
        aim: None,
        since_attack: 10.0,
        variant: 0,
        clock: t,
        walk: (clip == Clip::Walk).then_some(t / clip.duration()),
        gesture: 10.0,
        growth: 0.0,
        reduction: 0,
        counter: 0,
        armor_flash: 0.0,
        special: None,
        special_ready: false,
    }
}

fn sampled(m: &Moment) -> Pose {
    let mut p = [(Vec3::ZERO, Vec3::ZERO); NODE_COUNT];
    motion::pose(m, &mut p);
    p
}

fn vertices(mesh: &Mesh) -> &[[f32; 3]] {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(v)) => v,
        _ => panic!("Trident review mesh has no positions"),
    }
}

fn segment_hits_triangle(start: Vec3, end: Vec3, triangle: &[Vec3]) -> bool {
    let direction = end - start;
    let edge1 = triangle[1] - triangle[0];
    let edge2 = triangle[2] - triangle[0];
    let cross = direction.cross(edge2);
    let determinant = edge1.dot(cross);
    if determinant.abs() < 1e-7 {
        return false;
    }
    let offset = start - triangle[0];
    let u = offset.dot(cross) / determinant;
    let q = offset.cross(edge1);
    let v = direction.dot(q) / determinant;
    let t = edge2.dot(q) / determinant;
    (0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0 && (0.0..=1.0).contains(&t)
}

fn point_triangle_distance(point: Vec3, triangle: &[Vec3; 3]) -> f32 {
    let [a, b, c] = *triangle;
    let normal = (b - a).cross(c - a).normalize_or_zero();
    let height = (point - a).dot(normal);
    let projected = point - normal * height;
    let mut closest = f32::INFINITY;
    let mut inside = normal.length_squared() > 0.0;
    for (from, to) in [(a, b), (b, c), (c, a)] {
        let edge = to - from;
        inside &= edge.cross(projected - from).dot(normal) >= -1e-7;
        let t = ((point - from).dot(edge) / edge.length_squared().max(1e-9)).clamp(0.0, 1.0);
        closest = closest.min(point.distance(from + edge * t));
    }
    if inside {
        height.abs()
    } else {
        closest
    }
}

fn segment_distance(a0: Vec3, a1: Vec3, b0: Vec3, b1: Vec3) -> f32 {
    let (u, v, w) = (a1 - a0, b1 - b0, a0 - b0);
    let (a, b, c, d, e) = (u.dot(u), u.dot(v), v.dot(v), u.dot(w), v.dot(w));
    let denominator = a * c - b * b;
    let mut s = if denominator > 1e-9 {
        ((b * e - c * d) / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let t = (b * s + e) / c.max(1e-9);
    if t < 0.0 {
        s = (-d / a.max(1e-9)).clamp(0.0, 1.0);
    }
    if t > 1.0 {
        s = ((b - d) / a.max(1e-9)).clamp(0.0, 1.0);
    }
    (a0 + u * s).distance(b0 + v * t.clamp(0.0, 1.0))
}

fn segment_triangle_distance(start: Vec3, end: Vec3, triangle: &[Vec3; 3]) -> f32 {
    let mut closest =
        point_triangle_distance(start, triangle).min(point_triangle_distance(end, triangle));
    for (a, b) in [
        (triangle[0], triangle[1]),
        (triangle[1], triangle[2]),
        (triangle[2], triangle[0]),
    ] {
        closest = closest.min(segment_distance(start, end, a, b));
    }
    closest
}

fn inspect_pose(form: (u8, u8), pose: &Pose, meshes: &[Option<Mesh>; NODE_COUNT]) -> (f32, f32) {
    let plan = model::plan(form.0);
    let world = pose_world(UnitKind::Trident, &plan, pose);
    let shaft = world[Prop(Socket::WeaponR).index()];
    let mut grip_error = 0.0f32;
    for (joint, anchor) in [(HandR, Vec3::ZERO), (HandL, weapon::SUPPORT)] {
        let wrist = world[Body(joint).index()].transform_point3(Vec3::ZERO);
        grip_error = grip_error.max(wrist.distance(shaft.transform_point3(anchor)));
    }
    // The hand mesh is 0.10 wide: its center must stay within half that width of the grip.
    assert!(
        grip_error < 0.05,
        "form {form:?}: grip left the hand by {grip_error}"
    );
    let mut clearance = f32::INFINITY;
    for node in [
        Body(Hips),
        Body(Chest),
        Body(Head),
        Secondary(Spring::HeadGear),
        Secondary(Spring::Back),
    ] {
        let Some(mesh) = &meshes[node.index()] else {
            continue;
        };
        let local = world[node.index()].inverse() * shaft;
        clearance = clearance.min(inspect_mesh(form, node, mesh, local));
    }
    (grip_error, clearance)
}

fn inspect_mesh(form: (u8, u8), node: Node, mesh: &Mesh, local: Mat4) -> f32 {
    let mut segments = Vec::new();
    // Center and cardinal surface lines cover shaft thickness, not just its centerline.
    for offset in [
        Vec3::ZERO,
        Vec3::X * 0.025,
        -Vec3::X * 0.025,
        Vec3::Y * 0.025,
        -Vec3::Y * 0.025,
    ] {
        segments.push((
            offset + Vec3::Z * weapon::REAR,
            offset + Vec3::Z * weapon::SHAFT_FRONT,
        ));
    }
    for x in [
        -weapon::width(Form::of(form.0, form.1)),
        0.0,
        weapon::width(Form::of(form.0, form.1)),
    ] {
        segments.push((Vec3::new(x, 0.0, 0.80), Vec3::new(x, 0.0, weapon::TIP.z)));
    }
    let mut clearance = f32::INFINITY;
    for (start, end) in segments {
        let (start, end) = (local.transform_point3(start), local.transform_point3(end));
        for triangle in vertices(mesh).chunks_exact(3) {
            let triangle = [
                Vec3::from(triangle[0]),
                Vec3::from(triangle[1]),
                Vec3::from(triangle[2]),
            ];
            assert!(
                !segment_hits_triangle(start, end, &triangle),
                "form {form:?}: spear intersects {node:?}"
            );
            clearance = clearance.min(segment_triangle_distance(start, end, &triangle));
        }
    }
    clearance
}

#[test]
fn p70_trident_holds_spear_clear_of_body_across_clips_specials_and_blends() {
    // P70 S2: preserve both-hand contact and clear torso/head/jaw/fin volumes in all forms.
    let mut largest_grip_error = 0.0f32;
    let mut smallest_clearance = f32::INFINITY;
    for form in FORMS {
        let meshes = figure::build_joint_meshes(&model::spec(form.0, form.1));
        for clip in CLIPS {
            let mut times = (0..=4)
                .map(|i| KIT.duration(clip) * i as f32 / 4.0)
                .collect::<Vec<_>>();
            times.extend(clip.events().iter().map(|(t, _)| *t));
            for t in times {
                let pose = sampled(&moment(form, clip, t));
                let (grip, clearance) = inspect_pose(form, &pose, &meshes);
                largest_grip_error = largest_grip_error.max(grip);
                smallest_clearance = smallest_clearance.min(clearance);
                // The same shared Euler blend used by Animator must not send the prop through the body.
                for next in [Clip::Idle, Clip::Hit, Clip::Death] {
                    let to = sampled(&moment(form, next, 0.0));
                    for weight in [0.25, 0.5, 0.75] {
                        let (grip, clearance) =
                            inspect_pose(form, &blend(&pose, &to, weight), &meshes);
                        largest_grip_error = largest_grip_error.max(grip);
                        smallest_clearance = smallest_clearance.min(clearance);
                    }
                }
            }
        }
        for t in [0.0, 0.14, 0.18, 0.30, 0.45, 0.62, 0.75] {
            let mut m = moment(form, Clip::Idle, 0.0);
            m.special = Some(t);
            let (grip, clearance) = inspect_pose(form, &sampled(&m), &meshes);
            largest_grip_error = largest_grip_error.max(grip);
            smallest_clearance = smallest_clearance.min(clearance);
        }
    }
    println!("P70 Trident maximum sampled grip error, figure units: {largest_grip_error:.6}");
    println!("P70 Trident minimum sampled spear-line to body-mesh clearance, figure units: {smallest_clearance:.6}");
}

#[test]
fn p70_trident_reports_real_pose_dimensions_and_frontal_contact_at_strike() {
    // ART-DIRECTION §4 / P70 S2: point-led extension reaches its contact at the existing 0.30 s.
    assert_eq!(Clip::AttackMelee.events(), &[(0.30, ClipEvent::Strike)]);
    for form in FORMS {
        let spec = model::spec(form.0, form.1);
        let meshes = figure::build_joint_meshes(&spec);
        let mut animator = Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            spec.plan,
            Entity::PLACEHOLDER,
            UnitKind::Trident,
            0,
        );
        animator.form = form;
        animator.present(Clip::Idle, 0.0);
        let world = pose_world(
            UnitKind::Trident,
            &spec.plan,
            &animator.pose(Clip::Idle, 0.0),
        );
        let (mut low, mut high) = (f32::INFINITY, f32::NEG_INFINITY);
        for (index, mesh) in meshes
            .iter()
            .enumerate()
            .filter_map(|(i, m)| m.as_ref().map(|m| (i, m)))
        {
            if index == Prop(Socket::WeaponR).index() {
                continue;
            }
            for vertex in vertices(mesh) {
                let y = world[index].transform_point3(Vec3::from(*vertex)).y * spec.plan.scale;
                low = low.min(y);
                high = high.max(y);
            }
        }
        let shaft_mesh = meshes[Prop(Socket::WeaponR).index()].as_ref().unwrap();
        let longitudinal = vertices(shaft_mesh)
            .iter()
            .map(|v| v[2])
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), z| {
                (min.min(z), max.max(z))
            });
        println!(
            "P70 Trident form={form:?} height={:.6} weapon_length={:.6} scale={:.6}",
            high - low,
            (longitudinal.1 - longitudinal.0) * spec.plan.scale,
            spec.plan.scale
        );
        for target in [Vec3::new(0.0, 0.4, 1.5), Vec3::new(0.0, 0.4, -1.5)] {
            let mut tips = Vec::new();
            for t in [0.14, 0.30, 0.62] {
                let mut m = moment(form, Clip::AttackMelee, t);
                m.aim = Some(Aim {
                    target,
                    weight: 1.0,
                });
                let pose = sampled(&m);
                inspect_pose(form, &pose, &meshes);
                let world = pose_world(UnitKind::Trident, &spec.plan, &pose);
                let weapon = world[Prop(Socket::WeaponR).index()];
                tips.push(weapon.transform_point3(weapon::TIP));
                let origin = weapon.transform_point3(Vec3::ZERO);
                let axis = weapon.transform_vector3(Vec3::Z).xz().normalize();
                let to = target.xz() - origin.xz();
                assert!(axis.perp_dot(to).abs() < 1e-4, "spear line misses target");
            }
            let forward = target.with_y(0.0).normalize();
            assert!(tips[1].dot(forward) > tips[0].dot(forward));
            assert!(tips[1].dot(forward) > tips[2].dot(forward));
            println!("P70 Trident form={form:?} target_world={:?} tip_load={:?} tip_contact={:?} tip_recovered={:?}",
                target * spec.plan.scale, tips[0] * spec.plan.scale, tips[1] * spec.plan.scale, tips[2] * spec.plan.scale);
        }
    }
}

#[test]
fn p70_trident_calibrates_candidate_world_distances_at_contact() {
    // P70 S2: compare integer lane distances with the actual scaled weapon point, not model length.
    for form in FORMS {
        let spec = model::spec(form.0, form.1);
        for distance in [1100, 1600, 1800, 2000, 2200] {
            let mut m = moment(form, Clip::AttackMelee, 0.30);
            m.aim = Some(Aim {
                target: Vec3::new(0.0, 0.6, distance as f32 / 1000.0) / spec.plan.scale,
                weight: 1.0,
            });
            let pose = sampled(&m);
            let world = pose_world(UnitKind::Trident, &spec.plan, &pose);
            let tip = world[Prop(Socket::WeaponR).index()].transform_point3(weapon::TIP)
                * spec.plan.scale;
            println!("P70 calibrated form={form:?} distance={distance} contact_world={tip:?} longitudinal_delta={:.4}", tip.z-distance as f32/1000.0);
            assert!(tip.is_finite());
        }
    }
}
