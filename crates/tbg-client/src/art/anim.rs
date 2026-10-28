//! Per-instance eased clips, crossfades, event delivery, and foot-locked v2 walking.

use super::curves::{blend, events_between};
pub use super::{
    clips::{Clip, CLIPS},
    curves::{sample, ClipEvent, Easing, Pose},
};
use super::{
    feet::FootLocks,
    rig::{self, Proportions, NODE_COUNT},
};
use bevy::prelude::*;
use tbg_core::UnitKind;
const FADE: f32 = 0.1;

#[derive(Component)]
pub struct Animator {
    pub joints: [Entity; NODE_COUNT],
    pub plan: Proportions,
    pub figure: Entity,
    pub native: bool,
    pub rate: f32,
    pub preview: Option<(Clip, f32)>,
    pub events: Vec<ClipEvent>,
    base: Clip,
    base_t: f32,
    action: Option<(Clip, f32)>,
    fade: Option<(Pose, f32)>,
    last: Pose,
    walk_period: f32,
    phase: f32,
    feet: FootLocks,
}
impl Animator {
    pub fn new(
        joints: [Entity; NODE_COUNT],
        plan: Proportions,
        figure: Entity,
        kind: UnitKind,
        id: u32,
    ) -> Self {
        let speed = tbg_core::content::effective_stats(kind, 0, 0, 0).speed as f32 / 1000.0;
        let phase = id.wrapping_mul(2654435761) as f32 / u32::MAX as f32;
        Self {
            joints,
            plan,
            figure,
            native: false,
            rate: 1.0,
            preview: None,
            events: Vec::new(),
            base: Clip::Idle,
            base_t: phase * Clip::Idle.duration(),
            action: None,
            fade: None,
            last: sample(Clip::Idle, 0.0),
            walk_period: plan.leg * 0.8 * plan.scale / speed.max(0.01),
            phase,
            feet: FootLocks::default(),
        }
    }
    pub fn play(&mut self, clip: Clip) {
        self.fade = Some((self.last, FADE));
        self.action = Some((clip, 0.0));
    }
    pub fn play_if_free(&mut self, clip: Clip) {
        if self.action.is_none() {
            self.play(clip);
        }
    }
    pub fn set_base(&mut self, clip: Clip) {
        if self.base != clip {
            self.base = clip;
            self.base_t = self.phase * clip.duration();
            self.feet.reset();
            if self.action.is_none() {
                self.fade = Some((self.last, FADE));
            }
        }
    }
    fn advance(&mut self, dt: f32) -> (Pose, Option<f32>) {
        self.events.clear();
        if let Some((clip, t)) = self.preview {
            return (
                sample(clip, t),
                (clip == Clip::Walk).then_some(t / clip.duration()),
            );
        }
        let step = if self.base == Clip::Walk {
            dt * Clip::Walk.duration() / self.walk_period
        } else {
            dt
        };
        let previous = self.base_t;
        self.base_t += step;
        let mut pose = sample(self.base, self.base_t);
        let mut walk = (self.base == Clip::Walk && self.action.is_none())
            .then_some(self.base_t / Clip::Walk.duration());
        if let Some((clip, t)) = &mut self.action {
            self.events = events_between(*clip, *t, *t + dt);
            *t += dt;
            pose = sample(*clip, *t);
            if !clip.looping() && *t >= clip.duration() && *clip != Clip::Death {
                self.action = None;
                self.fade = Some((pose, FADE));
            }
        } else {
            self.events = events_between(self.base, previous, self.base_t);
        }
        if let Some((from, left)) = &mut self.fade {
            *left -= dt;
            pose = blend(&pose, from, (*left / FADE).clamp(0.0, 1.0));
            if *left <= 0.0 {
                self.fade = None;
            }
            walk = None;
        }
        self.last = pose;
        (pose, walk)
    }
}
pub fn animate(
    time: Res<Time>,
    mut animators: Query<(&mut Animator, &Transform)>,
    mut transforms: Query<&mut Transform, Without<Animator>>,
) {
    for (mut a, root) in &mut animators {
        let dt = time.delta_secs() * a.rate;
        let (mut pose, walk) = a.advance(dt);
        if let Some(phase) = walk.filter(|_| a.native) {
            if let Ok(figure) = transforms.get(a.figure) {
                let frame = root.to_matrix() * figure.to_matrix();
                let plan = a.plan;
                a.feet.apply(&mut pose, phase, frame, &plan);
            }
        } else {
            a.feet.reset();
        }
        for node in rig::nodes().filter(|n| !matches!(n, rig::Node::Secondary(_))) {
            if let Ok(mut t) = transforms.get_mut(a.joints[node.index()]) {
                let (rot, off) = pose[node.index()];
                let r = rot * std::f32::consts::PI / 180.0;
                t.translation = rig::rest(node, &a.plan) + off;
                t.rotation = Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z);
            }
        }
    }
}
