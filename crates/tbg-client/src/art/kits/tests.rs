//! Kit contracts: every form builds within the doubled P24 budget, the
//! ART-DIRECTION §4 event times survive the kits' longer clips, and the
//! Siege Walker's bombard points along its own shell's launch tangent.

use bevy::prelude::*;
use tbg_core::UnitKind;

use super::{aim, kit, Attack, Moment};
use crate::art::{
    anim::{sample, Clip, ClipEvent},
    curves::events_between,
    figure, ik, mesh,
    rig::{self, Joint, Node, Socket},
    FORMS,
};
use crate::vfx::shots;

const KITS: [UnitKind; 3] = [
    UnitKind::SiegeWalker,
    UnitKind::BoneBrute,
    UnitKind::Rootcaller,
];

#[test]
fn every_kit_form_builds_within_the_doubled_budget() {
    // ART-DIRECTION §8 / P24: 3 000 triangles per form, doubled by the owner.
    for kind in KITS {
        let k = kit(kind).expect("kit");
        for (tier, branch) in FORMS {
            let spec = (k.figure)(tier, branch);
            let tris: usize = figure::build_joint_meshes(&spec)
                .iter()
                .chain(&figure::build_glow_meshes(&spec))
                .flatten()
                .map(mesh::triangles)
                .sum();
            println!(
                "{kind:?} {tier}/{branch}: {tris} triangles, scale {:.2}",
                spec.plan.scale
            );
            assert!(tris <= 6_000, "{kind:?} {tier}/{branch}: {tris}");
        }
        // Each tier reads bigger (ART-DIRECTION §5).
        let scale = |t| (k.figure)(t, 0).plan.scale;
        assert!(scale(0) < scale(1) && scale(1) < scale(2));
    }
}

#[test]
fn kits_keep_the_simulation_event_times() {
    for kind in KITS {
        let k = kit(kind).expect("kit");
        for (clip, event, at) in [
            (Clip::AttackMelee, ClipEvent::Strike, 0.30),
            (Clip::AttackRanged, ClipEvent::Release, 0.18),
            (Clip::Cast, ClipEvent::Release, 0.22),
        ] {
            assert!(
                k.duration(clip) > at,
                "{kind:?} {clip:?} ends before its event"
            );
            assert_eq!(events_between(clip, at - 0.01, at), [event]);
        }
    }
}

/// Root-joint-space transform of the right-hand weapon socket.
fn socket(pose: &crate::art::anim::Pose, p: &rig::Proportions) -> Mat4 {
    let mut m = Mat4::IDENTITY;
    for j in [
        Joint::Root,
        Joint::Hips,
        Joint::Spine,
        Joint::Chest,
        Joint::UpperArmR,
        Joint::ForearmR,
        Joint::HandR,
    ] {
        let (r, o) = pose[j.index()];
        m *= Mat4::from_rotation_translation(ik::quat(r), rig::rest(Node::Body(j), p) + o);
    }
    let (r, o) = pose[Node::Prop(Socket::WeaponR).index()];
    m * Mat4::from_rotation_translation(ik::quat(r), o)
}

#[test]
fn the_bombard_points_along_its_shells_flight() {
    let k = kit(UnitKind::SiegeWalker).expect("kit");
    let Attack::Shot { muzzle, shot, .. } = k.attack else {
        panic!("Siege Walker shoots");
    };
    for (tier, branch) in FORMS {
        let p = (k.figure)(tier, branch).plan;
        for target in [
            Vec3::new(0.4, 0.4, 3.5),
            Vec3::new(-1.2, 0.3, 2.6),
            Vec3::new(1.6, 0.6, 4.4),
        ] {
            let m = Moment {
                clip: Clip::Idle,
                t: 0.4,
                tier,
                branch,
                aim: Some(aim::Aim {
                    target,
                    weight: 1.0,
                }),
                since_attack: 0.1,
                variant: 0,
                clock: 3.0,
                walk: None,
            };
            let mut pose = sample(Clip::Idle, 0.4);
            (k.pose)(&m, &mut pose);
            let gun = socket(&pose, &p);
            let start = gun.transform_point3(muzzle(tier, branch));
            let barrel = gun.transform_vector3(Vec3::Z).normalize();
            let flight = aim::tangent(start, target, shots::arc(shot(tier, branch)), 0.0);
            let miss = (target - start)
                .xz()
                .perp_dot(barrel.xz().normalize())
                .abs();
            let lift = (aim::pitch(barrel) - aim::pitch(flight)).abs();
            assert!(
                miss < 0.12,
                "{tier}/{branch} {target:?}: barrel passes {miss} beside"
            );
            assert!(
                lift < 0.09,
                "{tier}/{branch} {target:?}: elevation off by {lift}"
            );
        }
    }
}
