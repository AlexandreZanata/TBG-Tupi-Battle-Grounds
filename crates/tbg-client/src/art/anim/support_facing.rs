//! Friendly support casts share a bounded torso look across locomotion, cast and hit clips.
use super::*;

const MAX_LOOK: f32 = 65.0_f32.to_radians();

#[derive(Default)]
pub(super) struct Facing {
    yaw: f32,
    rear_side: f32,
}

impl Facing {
    pub(super) fn observe(&mut self, aim: Option<Aim>, dt: f32) {
        let goal = aim.map_or(0.0, |aim| {
            let bearing = kits::aim::yaw(Vec3::ZERO, aim.target);
            let bearing = if aim.target.z < 0.0 {
                // The rear branch cut must not switch shoulders on tiny crowd/recoil offsets.
                if self.rear_side == 0.0 {
                    self.rear_side = bearing.signum();
                }
                bearing.abs() * self.rear_side
            } else {
                self.rear_side = 0.0;
                bearing
            };
            bearing.clamp(-MAX_LOOK, MAX_LOOK) * aim.weight
        });
        if aim.is_none() {
            self.rear_side = 0.0;
        }
        let step = MAX_LOOK * dt / AIM_IN;
        self.yaw += (goal - self.yaw).clamp(-step, step);
    }

    pub(super) fn apply(&self, pose: &mut Pose) {
        pose[Joint::Chest.index()].0.y += self.yaw.to_degrees();
        pose[Joint::Head.index()].0.y += self.yaw.to_degrees() * 0.25;
    }
}

pub(super) fn combat(clip: Clip) -> bool {
    matches!(
        clip,
        Clip::Idle | Clip::Walk | Clip::Cast | Clip::AttackMelee | Clip::AttackRanged | Clip::Hit
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::{content::def, Archetype, ALL_KINDS};

    #[test]
    fn support_friendly_casts_do_not_turn_feet_between_heal_hit_idle_and_walk() {
        let supports = ALL_KINDS
            .into_iter()
            .filter(|k| k.is_conventional() && def(*k).archetype == Archetype::Support);
        let mut cases = 0;
        for kind in supports {
            for form in crate::art::FORMS {
                let plan = (kits::kit(kind).unwrap().figure)(form.0, form.1).plan;
                for side in [-1.0_f32, 1.0] {
                    for hz in [30, 120] {
                        let mut anim = Animator::new(
                            [Entity::PLACEHOLDER; NODE_COUNT],
                            plan,
                            Entity::PLACEHOLDER,
                            kind,
                            1,
                        );
                        anim.form = form;
                        anim.aim_is_ally = true;
                        let dt = 1.0 / hz as f32;
                        let at = Vec3::new(8.0, 0.0, 0.0);
                        let frame = Mat4::from_rotation_translation(
                            Quat::from_rotation_y(side * 65.0_f32.to_radians()),
                            at,
                        ) * Mat4::from_scale(Vec3::splat(plan.scale));
                        for step in 0..(hz * 4) {
                            let cycle = step % hz;
                            if cycle == 0 {
                                anim.play(Clip::Cast);
                            }
                            if cycle == hz * 3 / 4 {
                                anim.play_if_free(Clip::Hit);
                            }
                            anim.set_base(if cycle < hz / 8 {
                                Clip::Walk
                            } else {
                                Clip::Idle
                            });
                            let patient =
                                Vec3::new(8.0 - side * 1.2, 0.6, (step as f32 * 0.7).sin() * 0.02);
                            anim.aim_at = Some(patient);
                            anim.observe(at, &frame, dt);
                            let (pose, _) = anim.advance(dt);
                            let forward = ik::quat(pose[Joint::Root.index()].0) * Vec3::Z;
                            assert!(forward.dot(Vec3::Z) > 0.99,
                                "{kind:?} {form:?} side={side} hz={hz} frame={step}: feet turned toward rear patient: {forward:?}");
                            assert_eq!(
                                anim.aim_at,
                                Some(patient),
                                "body posture must not retarget the heal"
                            );
                        }
                        cases += 1;
                    }
                }
            }
        }
        assert_eq!(cases, 200);
    }

    #[test]
    fn support_rear_branch_cut_jitter_does_not_switch_shoulders_and_target_loss_releases() {
        let mut facing = Facing::default();
        for i in 0..240 {
            facing.observe(
                Some(Aim {
                    target: Vec3::new(if i % 2 == 0 { 0.001 } else { -0.001 }, 0.5, -2.0),
                    weight: 1.0,
                }),
                1.0 / 60.0,
            );
            assert!(facing.yaw >= 0.0 && facing.yaw <= MAX_LOOK);
        }
        assert!((facing.yaw - MAX_LOOK).abs() < 0.001);
        for _ in 0..60 {
            facing.observe(None, 1.0 / 60.0);
        }
        assert!(facing.yaw.abs() < 0.001);
        assert_eq!(facing.rear_side, 0.0);
    }

    #[test]
    fn support_friendly_posture_preserves_authored_life_clips_and_non_support_aim() {
        for kind in ALL_KINDS.into_iter().filter(|k| k.is_conventional()) {
            let plan = (kits::kit(kind).unwrap().figure)(0, 0).plan;
            let mut anim = Animator::new(
                [Entity::PLACEHOLDER; NODE_COUNT],
                plan,
                Entity::PLACEHOLDER,
                kind,
                1,
            );
            anim.aim = Some(Aim {
                target: Vec3::new(0.01, 0.6, -2.0),
                weight: 1.0,
            });
            anim.support_facing.observe(anim.aim, 1.0);
            for clip in [Clip::Spawn, Clip::Evolve, Clip::Death, Clip::Victory] {
                anim.aim_is_ally = false;
                let original = anim.pose(clip, 0.35);
                anim.aim_is_ally = true;
                assert_eq!(
                    anim.pose(clip, 0.35),
                    original,
                    "{kind:?} {clip:?} is authored motion"
                );
            }
            if def(kind).archetype != Archetype::Support {
                for clip in [
                    Clip::Idle,
                    Clip::Cast,
                    Clip::AttackMelee,
                    Clip::AttackRanged,
                ] {
                    anim.aim_is_ally = false;
                    let original = anim.pose(clip, 0.3);
                    anim.aim_is_ally = true;
                    assert_eq!(
                        anim.pose(clip, 0.3),
                        original,
                        "non-support {kind:?} must keep its aim"
                    );
                }
            }
        }
    }
}
