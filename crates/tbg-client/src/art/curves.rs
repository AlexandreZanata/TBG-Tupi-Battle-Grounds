//! Cubic key interpolation, clip sampling, and event crossings including loop boundaries.

use super::{
    clips::{Clip, Key},
    rig::NODE_COUNT,
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Easing {
    Linear,
    CubicIn,
    CubicOut,
    CubicInOut,
    Hold,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipEvent {
    Strike,
    Release,
    Footstep,
}
pub type Pose = [(Vec3, Vec3); NODE_COUNT];

impl Easing {
    pub fn sample(self, t: f32) -> f32 {
        // Same cubic curves as eased_motion.rs; avoid enabling bevy_curve/animation.
        match self {
            Self::Linear => t,
            Self::CubicIn => t * t * t,
            Self::CubicOut => 1.0 - (1.0 - t).powi(3),
            Self::CubicInOut if t < 0.5 => 4.0 * t * t * t,
            Self::CubicInOut => 1.0 - (-2.0 * t + 2.0).powi(3) * 0.5,
            Self::Hold => {
                if t >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}
pub fn sample(clip: Clip, seconds: f32) -> Pose {
    let raw = seconds / clip.duration();
    let k = if clip.looping() {
        raw.rem_euclid(1.0)
    } else {
        raw.clamp(0.0, 1.0)
    };
    let mut pose = [(Vec3::ZERO, Vec3::ZERO); NODE_COUNT];
    for (joint, keys) in clip.tracks() {
        pose[joint.index()] = sample_track(keys, k);
    }
    pose
}
pub(super) fn sample_track(keys: &[Key], k: f32) -> (Vec3, Vec3) {
    let v = |key: &Key| (Vec3::from(key.1), Vec3::from(key.2));
    let Some(next) = keys.iter().position(|key| key.0 >= k) else {
        return keys.last().map_or((Vec3::ZERO, Vec3::ZERO), v);
    };
    if next == 0 {
        return v(&keys[0]);
    }
    let (a, b) = (&keys[next - 1], &keys[next]);
    let t = ((k - a.0) / (b.0 - a.0).max(1e-4)).clamp(0.0, 1.0);
    let (ra, oa) = v(a);
    let (rb, ob) = v(b);
    let t = a.3.sample(t);
    (ra.lerp(rb, t), oa.lerp(ob, t))
}
pub fn events_between(clip: Clip, from: f32, to: f32) -> Vec<ClipEvent> {
    let duration = clip.duration();
    let start = if clip.looping() {
        (from / duration).floor() as i32
    } else {
        0
    };
    let end = if clip.looping() {
        (to / duration).floor() as i32
    } else {
        0
    };
    let mut events = Vec::new();
    for cycle in start..=end {
        for &(t, event) in clip.events() {
            let at = cycle as f32 * duration + t;
            if at > from && at <= to {
                events.push(event);
            }
        }
    }
    events
}
pub fn blend(a: &Pose, b: &Pose, t: f32) -> Pose {
    std::array::from_fn(|i| (a[i].0.lerp(b[i].0, t), a[i].1.lerp(b[i].1, t)))
}
