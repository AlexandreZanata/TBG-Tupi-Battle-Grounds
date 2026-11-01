//! Regressions for real figure bounds, held-weapon clearance and close-contact presentation.
use super::{
    anim::{Animator, CLIPS},
    figure, kits,
    presentation::pose_world,
    rig::{Node, Socket, NODE_COUNT},
    FORMS,
};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use tbg_core::UnitKind;

#[test]
fn p71_rattle_club_stays_above_the_floor_in_all_forms_and_clips() {
    for (tier, branch) in FORMS {
        let kit = kits::kit(UnitKind::RattleTick).unwrap();
        let spec = (kit.figure)(tier, branch);
        let mesh = figure::build_joint_meshes(&spec)[Node::Prop(Socket::WeaponR).index()]
            .take()
            .unwrap();
        let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("weapon positions missing")
        };
        let mut a = Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            spec.plan,
            Entity::PLACEHOLDER,
            UnitKind::RattleTick,
            0,
        );
        a.form = (tier, branch);
        for clip in CLIPS {
            for step in 0..=32 {
                let time = kit.duration(clip) * step as f32 / 32.0;
                let pose = a.pose(clip, time);
                let world = pose_world(UnitKind::RattleTick, &spec.plan, &pose);
                let low = vertices
                    .iter()
                    .map(|p| {
                        world[Node::Prop(Socket::WeaponR).index()]
                            .transform_point3(Vec3::from(*p))
                            .y
                            * spec.plan.scale
                    })
                    .fold(f32::INFINITY, f32::min);
                assert!(
                    low >= -0.002,
                    "Rattle {tier}/{branch} {clip:?} t={time}: {low}"
                );
            }
        }
    }
}

#[test]
fn p71_body_envelopes_clear_large_custom_figures() {
    let mut meshes = Assets::<Mesh>::default();
    for kind in tbg_core::FULL_POOL {
        for (tier, branch) in FORMS {
            let mut form = super::native_form(
                (kits::kit(kind).unwrap().figure)(tier, branch),
                &mut meshes,
                false,
            );
            (form.overhead, form.occupancy, form.body_radius) =
                super::overhead::height(&form, &meshes, kind, tier, branch);
            assert!(form.overhead.is_finite() && form.overhead > 0.32);
            assert!(form.occupancy.x < form.occupancy.y);
            println!(
                "P71 {kind:?} {tier}/{branch}: overhead={} depth={:?}",
                form.overhead, form.occupancy
            );
        }
    }
}

#[test]
fn shatterback_head_clears_allies_and_enemies_in_every_form() {
    use crate::{
        art::anim::Clip,
        crowd::{Body, Offset, Params, Solver},
    };
    let kind = UnitKind::Shatterback;
    let kit = kits::kit(kind).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    for (tier, branch) in FORMS {
        let mut form = super::native_form((kit.figure)(tier, branch), &mut meshes, false);
        (form.overhead, form.occupancy, form.body_radius) =
            super::overhead::height(&form, &meshes, kind, tier, branch);
        let mesh = meshes
            .get(
                form.meshes[Node::Body(super::rig::Joint::Head).index()]
                    .as_ref()
                    .unwrap(),
            )
            .unwrap();
        let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("head positions missing")
        };
        let lunge = super::melee::strike_profile_for_unit(kind, tier, branch).lunge;
        for degrees in [65.0f32, 90.0, -65.0, -90.0] {
            let yaw = degrees.to_radians();
            let facing = degrees.signum();
            for opposing in [false, true] {
                let root = 8.0;
                let other = root + facing * 0.8;
                let mut turtle = Body {
                    id: 1,
                    anchor_x: root,
                    radius: 0.5,
                    half_length: 0.5,
                    facing,
                    ghost: false,
                };
                turtle.include_envelope(
                    form.occupancy.x,
                    form.occupancy.y + lunge,
                    form.body_radius,
                    yaw,
                );
                let bodies = [
                    turtle,
                    Body {
                        id: 2,
                        anchor_x: other,
                        radius: 0.4,
                        half_length: 0.4,
                        facing: if opposing { -facing } else { facing },
                        ghost: false,
                    },
                ];
                let mut offsets = [Offset::default(); 2];
                let mut solver = Solver::default();
                let params = Params::default();
                for _ in 0..180 {
                    solver.solve(&bodies, &mut offsets, 1.0 / 60.0, &params);
                }
                let head_root = Vec3::new(root + offsets[0].dx, 0.0, offsets[0].dz);
                let enemy = Vec3::new(other + offsets[1].dx, 0.0, offsets[1].dz);
                let rotation = Quat::from_rotation_y(yaw);
                let mut animator = Animator::new(
                    [Entity::PLACEHOLDER; NODE_COUNT],
                    form.plan,
                    Entity::PLACEHOLDER,
                    kind,
                    0,
                );
                animator.form = (tier, branch);
                for clip in [Clip::Idle, Clip::Walk, Clip::AttackMelee] {
                    for step in 0..8 {
                        let pose = animator.pose(clip, kit.duration(clip) * step as f32 / 8.0);
                        let world = pose_world(kind, &form.plan, &pose);
                        for vertex in vertices {
                            let p = world[Node::Body(super::rig::Joint::Head).index()]
                                .transform_point3(Vec3::from(*vertex))
                                * form.plan.scale;
                            let head = head_root
                                + rotation
                                    * (p + if clip == Clip::AttackMelee {
                                        Vec3::Z * lunge
                                    } else {
                                        Vec3::ZERO
                                    });
                            assert!(head.xz().distance(enemy.xz())>=0.38,
                                "Shatterback {tier}/{branch} yaw={degrees} opposing={opposing} {clip:?}: head enters adjacent body");
                        }
                    }
                }
                for (body, offset) in bodies.iter().zip(offsets) {
                    assert!((-params.max_back - 1e-6..=params.max_dx + 1e-6)
                        .contains(&(offset.dx * body.facing)));
                }
            }
        }
    }
}
