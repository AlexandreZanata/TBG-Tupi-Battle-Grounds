//! Shared animation library: keyframed joint rotations/offsets in code,
//! valid for every figurine because all share the 12-joint body plan.
//! `Animator` plays a looping base clip (Idle/Walk) plus one-shot actions,
//! crossfading 0.1 s on every change.

use bevy::prelude::*;

use super::clips::Key;
pub use super::clips::{Clip, CLIPS};
use super::figure::{rest, BodyPlan, JOINTS};

/// Per joint: rotation (degrees) and offset from the rest pose.
pub type Pose = [(Vec3, Vec3); 12];

pub fn sample(clip: Clip, seconds: f32) -> Pose {
    let raw = seconds / clip.duration();
    let k = if clip.looping() {
        raw.fract()
    } else {
        raw.min(1.0)
    };
    let mut pose = [(Vec3::ZERO, Vec3::ZERO); 12];
    for (joint, keys) in clip.tracks() {
        pose[joint.index()] = sample_track(keys, k);
    }
    pose
}

fn sample_track(keys: &[Key], k: f32) -> (Vec3, Vec3) {
    let v = |key: &Key| (Vec3::from(key.1), Vec3::from(key.2));
    let Some(next) = keys.iter().position(|key| key.0 >= k) else {
        return keys.last().map_or((Vec3::ZERO, Vec3::ZERO), v);
    };
    if next == 0 {
        return v(&keys[0]);
    }
    let (a, b) = (&keys[next - 1], &keys[next]);
    let t = ((k - a.0) / (b.0 - a.0).max(1e-4)).clamp(0.0, 1.0);
    let s = t * t * (3.0 - 2.0 * t);
    let (ra, oa) = v(a);
    let (rb, ob) = v(b);
    (ra.lerp(rb, s), oa.lerp(ob, s))
}

fn blend(a: &Pose, b: &Pose, t: f32) -> Pose {
    std::array::from_fn(|i| (a[i].0.lerp(b[i].0, t), a[i].1.lerp(b[i].1, t)))
}

const FADE: f32 = 0.1;

#[derive(Component)]
pub struct Animator {
    pub joints: [Entity; 12],
    plan: BodyPlan,
    base: Clip,
    base_t: f32,
    action: Option<(Clip, f32)>,
    fade: Option<(Pose, f32)>,
    last: Pose,
}

impl Animator {
    pub fn new(joints: [Entity; 12], plan: BodyPlan) -> Animator {
        Animator {
            joints,
            plan,
            base: Clip::Idle,
            base_t: 0.0,
            action: None,
            fade: None,
            last: sample(Clip::Idle, 0.0),
        }
    }

    pub fn play(&mut self, clip: Clip) {
        self.fade = Some((self.last, FADE));
        self.action = Some((clip, 0.0));
    }

    /// Play only if no other one-shot is running (hits never cut attacks).
    pub fn play_if_free(&mut self, clip: Clip) {
        if self.action.is_none() {
            self.play(clip);
        }
    }

    pub fn set_base(&mut self, clip: Clip) {
        if self.base != clip {
            self.base = clip;
            self.base_t = 0.0;
            if self.action.is_none() {
                self.fade = Some((self.last, FADE));
            }
        }
    }

    fn advance(&mut self, dt: f32) -> Pose {
        self.base_t += dt;
        let mut pose = sample(self.base, self.base_t);
        if let Some((clip, t)) = &mut self.action {
            *t += dt;
            pose = sample(*clip, *t);
            let (clip, t) = (*clip, *t);
            if !clip.looping() && t >= clip.duration() && clip != Clip::Death {
                self.action = None;
                self.fade = Some((pose, FADE));
            }
        }
        if let Some((from, left)) = &mut self.fade {
            *left -= dt;
            pose = blend(&pose, from, (*left / FADE).clamp(0.0, 1.0));
            if *left <= 0.0 {
                self.fade = None;
            }
        }
        self.last = pose;
        pose
    }
}

pub fn animate(
    time: Res<Time>,
    mut animators: Query<&mut Animator>,
    mut transforms: Query<&mut Transform, Without<Animator>>,
) {
    let dt = time.delta_secs();
    for mut a in &mut animators {
        let pose = a.advance(dt);
        for j in JOINTS {
            if let Ok(mut t) = transforms.get_mut(a.joints[j.index()]) {
                let (rot, off) = pose[j.index()];
                let r = rot * std::f32::consts::PI / 180.0;
                t.translation = rest(j, &a.plan) + off;
                t.rotation = Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z);
            }
        }
    }
}
