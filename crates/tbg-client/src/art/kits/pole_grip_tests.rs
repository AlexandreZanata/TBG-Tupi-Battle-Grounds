//! P70 two-hand pole grips must contact the actual haft throughout their authored attacks.

use super::{kit, Aim, Moment};
use crate::art::{
    anim::{sample, Clip},
    figure,
    presentation::pose_world,
    rig::{Joint, Node, Socket},
    FORMS,
};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use tbg_core::UnitKind;

#[test]
fn p70_pole_hands_stay_on_the_haft_and_the_shaft_clears_head_and_chest() {
    // P70 S3 / ART-DIRECTION §4: the visible support hand must not float off its weapon.
    for kind in [UnitKind::Longaxe, UnitKind::BoneBrute] {
        for (tier, branch) in FORMS {
            if kind == UnitKind::BoneBrute && (tier, branch) != (2, 1) {
                continue;
            }
            let k = kit(kind).expect("reviewed pole kit");
            let spec = (k.figure)(tier, branch);
            let meshes = figure::build_joint_meshes(&spec);
            let (butt, top) = if kind == UnitKind::Longaxe {
                (-0.672, 1.728)
            } else {
                (-0.35, 0.95)
            };
            let mut worst = 0.0_f32;
            for variant in 0..2 {
                for direction in [-1.0, 1.0] {
                    for step in 0..=64 {
                        let t = 0.62 * step as f32 / 64.0;
                        let moment = Moment {
                            clip: Clip::AttackMelee,
                            t,
                            tier,
                            branch,
                            aim: Some(Aim {
                                target: Vec3::new(0.0, 0.6, direction * 3.0),
                                weight: 1.0,
                            }),
                            since_attack: 0.0,
                            variant,
                            clock: t,
                            walk: None,
                            gesture: 10.0,
                            growth: 0.0,
                            reduction: 0,
                            counter: 0,
                            armor_flash: 0.0,
                            special: None,
                            special_ready: false,
                        };
                        worst = worst.max(check_pose(kind, &spec, &meshes, &moment, butt, top));
                    }
                }
            }
            println!(
                "P70 {kind:?} {tier}/{branch} maximum sampled wrist-to-haft axis error: {worst:.6}"
            );
        }
    }
}

fn check_pose(
    kind: UnitKind,
    spec: &figure::FigureSpec,
    meshes: &[Option<Mesh>],
    m: &Moment,
    butt: f32,
    top: f32,
) -> f32 {
    let k = kit(kind).expect("reviewed pole kit");
    let mut pose = sample(m.clip, m.t);
    (k.pose)(m, &mut pose);
    let world = pose_world(kind, &spec.plan, &pose);
    let shaft = world[Node::Prop(Socket::WeaponR).index()];
    let label = format!(
        "{kind:?} {}/{} variant={} t={}",
        m.tier, m.branch, m.variant, m.t
    );
    let gap = check_hands(&world, shaft, butt, top, &label);
    for body in [Joint::Hips, Joint::Chest, Joint::Head] {
        if let Some(mesh) = &meshes[body.index()] {
            check_clearance(
                mesh,
                world[body.index()].inverse() * shaft,
                butt,
                top,
                &format!("{label}: {body:?}"),
            );
        }
    }
    gap
}

fn check_hands(world: &[Mat4], shaft: Mat4, butt: f32, top: f32, label: &str) -> f32 {
    let mut heights = [0.0; 2];
    let mut worst = 0.0_f32;
    for (index, hand) in [Joint::HandR, Joint::HandL].into_iter().enumerate() {
        let wrist = (shaft.inverse() * world[hand.index()]).transform_point3(Vec3::ZERO);
        let gap = Vec2::new(wrist.x, wrist.z).length();
        worst = worst.max(gap);
        assert!(
            gap < 0.035 && wrist.y >= butt && wrist.y <= top,
            "{label}: {hand:?} off haft: {wrist:?}"
        );
        heights[index] = wrist.y;
    }
    assert!(
        heights[1] - heights[0] > 0.10,
        "{label}: two distinct grip anchors required: {heights:?}"
    );
    worst
}

fn check_clearance(mesh: &Mesh, local: Mat4, butt: f32, top: f32, label: &str) {
    let Some(VertexAttributeValues::Float32x3(vertices)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("missing body vertices")
    };
    for offset in [
        Vec3::ZERO,
        Vec3::X * 0.024,
        -Vec3::X * 0.024,
        Vec3::Z * 0.024,
        -Vec3::Z * 0.024,
    ] {
        let a = local.transform_point3(offset + Vec3::Y * butt);
        let b = local.transform_point3(offset + Vec3::Y * top);
        for tri in vertices.chunks_exact(3) {
            assert!(
                !intersects(
                    a,
                    b,
                    Vec3::from(tri[0]),
                    Vec3::from(tri[1]),
                    Vec3::from(tri[2])
                ),
                "{label}: shaft intersects body"
            );
        }
    }
}

fn intersects(a: Vec3, b: Vec3, v0: Vec3, v1: Vec3, v2: Vec3) -> bool {
    let direction = b - a;
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let h = direction.cross(e2);
    let det = e1.dot(h);
    if det.abs() < 1e-7 {
        return false;
    }
    let s = a - v0;
    let u = s.dot(h) / det;
    let q = s.cross(e1);
    let v = direction.dot(q) / det;
    let t = e2.dot(q) / det;
    (0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0 && (0.0..=1.0).contains(&t)
}
