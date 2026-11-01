//! Regressions for real figure bounds, held-weapon clearance and close-contact presentation.
use super::{
    anim::{Animator, Clip, CLIPS},
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
