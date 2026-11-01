//! Four articulated venom-hose sections connect the abdomen to the moving pistol.
use crate::art::{
    anim::Pose,
    presentation::pose_world,
    rig::{Joint, Node, Proportions, Socket, Spring},
};
use bevy::prelude::*;

pub const SOCKETS: [Socket; 4] = [
    Socket::OffhandL,
    Socket::Nock,
    Socket::StringTop,
    Socket::StringBottom,
];

/// Root-local transforms keep both ends attached through every authored clip.
pub fn segments(plan: &Proportions, pose: &Pose) -> [Transform; 4] {
    let world = pose_world(tbg_core::UnitKind::Hornet, plan, pose);
    let root = world[Joint::Root.index()].inverse();
    let start = root.transform_point3(
        world[Node::Secondary(Spring::Back).index()]
            .transform_point3(Vec3::new(-0.10, -0.035, -0.18)),
    );
    let end = root.transform_point3(
        world[Node::Prop(Socket::WeaponR).index()].transform_point3(Vec3::new(-0.075, 0.03, 0.015)),
    );
    let control = start.lerp(end, 0.5) + Vec3::new(-0.22, -0.18, 0.0);
    let point = |t: f32| start * (1.0 - t).powi(2) + control * (2.0 * t * (1.0 - t)) + end * t * t;
    std::array::from_fn(|i| {
        let a = point(i as f32 / 4.0);
        let b = point((i + 1) as f32 / 4.0);
        let d = b - a;
        Transform::from_translation((a + b) * 0.5)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, d.normalize_or(Vec3::Y)))
            .with_scale(Vec3::new(1.0, d.length() + 0.015, 1.0))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p71_hose_sections_remain_connected_in_all_five_forms_and_clips() {
        let kit = crate::art::kits::kit(tbg_core::UnitKind::Hornet).unwrap();
        for (tier, branch) in crate::art::FORMS {
            let plan = (kit.figure)(tier, branch).plan;
            let mut a = crate::art::anim::Animator::new(
                [Entity::PLACEHOLDER; crate::art::rig::NODE_COUNT],
                plan,
                Entity::PLACEHOLDER,
                tbg_core::UnitKind::Hornet,
                1,
            );
            a.form = (tier, branch);
            for clip in [
                crate::art::anim::Clip::Idle,
                crate::art::anim::Clip::Walk,
                crate::art::anim::Clip::AttackRanged,
                crate::art::anim::Clip::Death,
            ] {
                for step in 0..=16 {
                    let pose = a.pose(clip, clip.duration() * step as f32 / 16.0);
                    let sections = segments(&plan, &pose);
                    for pair in sections.windows(2) {
                        let tip = pair[0].transform_point(Vec3::Y * 0.5);
                        let base = pair[1].transform_point(-Vec3::Y * 0.5);
                        assert!(tip.distance(base) < 0.02);
                    }
                }
            }
        }
    }
}
