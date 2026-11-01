//! One rig-aware celebration resolver for Result and future P62 Dance previews.

pub mod bounds;

use super::{
    anim::{Clip, Pose},
    kits::{self, Moment},
    rig::Joint,
};
use tbg_core::UnitKind;

pub type Signature = fn(&Moment, &mut Pose);

#[derive(Clone, Copy)]
pub struct CelebrationMotion {
    pub clip: Clip,
    pub duration: f32,
    pub looping: bool,
    pub signature: Option<Signature>,
}

pub fn celebration_for(kind: UnitKind, tier: u8, branch: u8) -> CelebrationMotion {
    let clip = Clip::Victory;
    CelebrationMotion {
        clip,
        duration: kits::kit(kind).map_or_else(|| clip.duration(), |k| k.duration(clip)),
        looping: true,
        signature: signature_for(kind, tier, branch),
    }
}

/// Register future character/form content here; current standard is the existing kit Victory.
fn signature_for(_kind: UnitKind, _tier: u8, _branch: u8) -> Option<Signature> {
    None
}

pub(super) fn finish(motion: CelebrationMotion, moment: &Moment, pose: &mut Pose) {
    if let Some(signature) = motion.signature {
        (signature)(moment, pose);
    }
    let (rotation, offset) = &mut pose[Joint::Root.index()];
    rotation.y = 0.;
    offset.x = offset.x.clamp(-0.15, 0.15);
    offset.z = offset.z.clamp(-0.15, 0.15);
    offset.y = offset.y.clamp(-0.03, 0.22);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::{
        anim::Animator,
        rig::{BodyPlan, NODE_COUNT},
        FORMS,
    };
    use bevy::prelude::*;
    #[test]
    fn celebration_all_current_forms_have_finite_eventless_bounded_standard_motion() {
        let mut covered = 0;
        for kind in tbg_core::ALL_KINDS
            .into_iter()
            .filter(|k| *k != UnitKind::Crate)
        {
            for (tier, branch) in FORMS {
                let motion = celebration_for(kind, tier, branch);
                assert!(motion.duration > 0. && motion.looping && motion.clip.events().is_empty());
                let plan = kits::kit(kind).map_or_else(
                    || BodyPlan::Humanoid.proportions(),
                    |k| (k.figure)(tier, branch).plan,
                );
                let mut a = Animator::new(
                    [Entity::PLACEHOLDER; NODE_COUNT],
                    plan,
                    Entity::PLACEHOLDER,
                    kind,
                    1,
                );
                a.form = (tier, branch);
                for i in 0..16 {
                    let t = motion.duration * i as f32 / 16.;
                    a.celebrate(motion, t);
                    let pose = a.pose(motion.clip, t);
                    assert!(pose.iter().all(|(r, p)| r.is_finite() && p.is_finite()));
                    let (r, p) = pose[Joint::Root.index()];
                    assert_eq!(r.y, 0.);
                    assert!(p.x.abs() <= 0.15 && p.z.abs() <= 0.15);
                }
                covered += 1;
            }
        }
        assert_eq!(covered, (tbg_core::ALL_KINDS.len() - 1) * FORMS.len());
    }
    #[test]
    fn celebration_signature_seam_is_invoked_and_unsafe_root_motion_is_constrained() {
        fn future(_: &Moment, pose: &mut Pose) {
            pose[Joint::Chest.index()].0.z = 13.;
            pose[Joint::Root.index()].1.x = 999.;
        }
        let mut motion = celebration_for(UnitKind::BoneBrute, 2, 1);
        motion.signature = Some(future);
        let plan = BodyPlan::Large.proportions();
        let mut a = Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            plan,
            Entity::PLACEHOLDER,
            UnitKind::BoneBrute,
            1,
        );
        a.celebrate(motion, 0.2);
        let pose = a.pose(motion.clip, 0.2);
        assert_eq!(pose[Joint::Chest.index()].0.z, 13.);
        assert_eq!(pose[Joint::Root.index()].1.x, 0.15);
    }
    #[test]
    fn celebration_reduced_pose_is_static_and_attack_events_are_never_delivered() {
        let kind = UnitKind::BoneBrute;
        let plan = BodyPlan::Large.proportions();
        let mut a = Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            plan,
            Entity::PLACEHOLDER,
            kind,
            1,
        );
        a.play(Clip::AttackMelee);
        a.apply_hit_stop(0.3);
        a.aim_at = Some(Vec3::X);
        a.celebrate(celebration_for(kind, 0, 0), 0.3);
        let first = a.advance(0.016).0;
        for dt in [0.033, 0.1, 1., 2.] {
            assert_eq!(a.advance(dt).0, first);
            assert!(a.events.is_empty());
        }
        assert!(a.aim_at.is_none());
        assert_eq!(a.hit_stop, 0.);
    }
}
