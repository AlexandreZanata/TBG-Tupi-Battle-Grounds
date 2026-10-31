//! Per-instance eased clips, crossfades, event delivery, kit layers, aim
//! blending, and two-bone walking whose cadence is the distance travelled.

use super::curves::{blend, events_between};
pub use super::{
    clips::{Clip, CLIPS},
    curves::{sample, ClipEvent, Easing, Pose},
};
use super::{
    feet::{FootLocks, Stride},
    ik,
    kits::{self, Aim, Attack, Gait, Kit, Moment},
    rig::{self, Joint, Proportions, NODE_COUNT},
};
use bevy::prelude::*;
use tbg_core::UnitKind;
const FADE: f32 = 0.1;
/// Seconds for the aim to swing fully in, and to let go.
const AIM_IN: f32 = 0.15;
const AIM_OUT: f32 = 0.3;

#[derive(Component)]
pub struct Animator {
    pub joints: [Entity; NODE_COUNT],
    pub plan: Proportions,
    pub figure: Entity,
    pub native: bool,
    pub rate: f32,
    pub hit_stop: f32,
    pub preview: Option<(Clip, f32)>,
    pub events: Vec<ClipEvent>,
    pub kit: Option<&'static Kit>,
    /// (tier, branch) of the shown form, for kit layers.
    pub form: (u8, u8),
    /// The engaged target in world space; visuals refresh it every frame.
    pub aim_at: Option<Vec3>,
    /// Walk in place at content speed (review tools): feet are not planted.
    pub treadmill: bool,
    pub gait: Gait,
    /// When carried by a carrier (e.g. riding a Cargo Crab), sits mounted in saddle.
    pub carried: bool,
    pub reduction: u16,
    pub counter: u8,
    special: Option<f32>,
    special_ready: bool,
    armor_flash: f32,
    base: Clip,
    base_t: f32,
    action: Option<(Clip, f32)>,
    fade: Option<(Pose, f32)>,
    last: Pose,
    walk_period: f32,
    phase: f32,
    feet: FootLocks,
    aim: Option<Aim>,
    since_attack: f32,
    variant: u8,
    clock: f32,
    last_root: Option<Vec3>,
    travel: f32,
    heading: Vec3,
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
        let gait = Gait::of(kind);
        Self {
            joints,
            plan,
            figure,
            native: false,
            rate: 1.0,
            hit_stop: 0.0,
            preview: None,
            events: Vec::new(),
            kit: kits::kit(kind),
            form: (0, 0),
            aim_at: None,
            treadmill: false,
            gait,
            carried: false,
            reduction: 0,
            counter: 0,
            special: None,
            special_ready: false,
            armor_flash: 0.0,
            base: Clip::Idle,
            base_t: phase * Clip::Idle.duration(),
            action: None,
            fade: None,
            last: sample(Clip::Idle, 0.0),
            walk_period: gait.cycle(plan.leg) * plan.scale / speed.max(0.01),
            phase,
            feet: FootLocks::default(),
            aim: None,
            since_attack: 1e6,
            variant: 0,
            clock: phase * 100.0,
            last_root: None,
            travel: 0.0,
            heading: Vec3::Z,
        }
    }
    /// Snapshots restore persistent armor; only an increase squeezes the plates.
    pub fn set_reduction(&mut self, bp: u16) {
        if bp > self.reduction {
            self.armor_flash = 1.0;
        }
        self.reduction = bp;
    }
    pub fn attack_variant(&self) -> usize {
        usize::from(self.variant & 1)
    }
    pub fn start_special(&mut self) {
        self.special = Some(0.0);
    }
    pub fn resolve_special(&mut self) {
        let contact = f32::from(tbg_core::content::WINDUP) / tbg_core::content::TICK_HZ as f32;
        self.special = Some(self.special.unwrap_or(0.0).max(contact));
    }
    pub fn observe_special(&mut self, ready: bool, left: Option<u16>, rooted: bool) {
        self.special_ready = ready;
        if rooted {
            self.special = None;
            return;
        }
        if let Some(left) = left {
            let elapsed = f32::from(tbg_core::content::WINDUP.saturating_sub(left))
                / tbg_core::content::TICK_HZ as f32;
            self.special = Some(self.special.unwrap_or(0.0).max(elapsed));
        }
    }
    pub fn apply_hit_stop(&mut self, duration_s: f32) {
        self.hit_stop = duration_s;
    }
    pub fn play(&mut self, clip: Clip) {
        if matches!(clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast) {
            self.since_attack = 0.0;
            self.variant ^= 1;
        }
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
    pub fn clip_time(&self) -> (Clip, f32) {
        self.preview
            .or(self.action)
            .unwrap_or((self.base, self.base_t))
    }
    /// The foot the latest footstep put down: phase 0 lands the left foot,
    /// phase 0.5 the right one.
    pub fn stepping_foot(&self) -> Joint {
        let k = (self.base_t / Clip::Walk.duration()).rem_euclid(1.0);
        if (0.25..0.75).contains(&k) {
            Joint::FootR
        } else {
            Joint::FootL
        }
    }
    /// Length of `clip` for this figure (kits may lengthen recoveries).
    pub fn duration(&self, clip: Clip) -> f32 {
        self.kit
            .map_or_else(|| clip.duration(), |k| k.duration(clip))
    }
    fn pose(&self, clip: Clip, t: f32) -> Pose {
        let mut pose = sample(clip, t);
        if let Some(kit) = self.kit {
            let walk = (clip == Clip::Walk).then(|| (t / clip.duration()).rem_euclid(1.0));
            let moment = Moment {
                clip,
                t,
                tier: self.form.0,
                branch: self.form.1,
                aim: self.aim,
                since_attack: self.since_attack,
                variant: self.variant,
                clock: self.clock,
                walk,
                reduction: self.reduction,
                counter: self.counter,
                armor_flash: self.armor_flash,
                special: self.special,
                special_ready: self.special_ready,
            };
            (kit.pose)(&moment, &mut pose);
        }
        pose
    }
    fn action_pose(&mut self, dt: f32) -> Pose {
        let mut pose = self.last;
        if let Some((clip, t)) = self.action {
            self.events = events_between(clip, t, t + dt);
            let t = t + dt;
            self.action = Some((clip, t));
            pose = self.pose(clip, t);
            if !clip.looping() && t >= self.duration(clip) && clip != Clip::Death {
                self.action = None;
                self.fade = Some((pose, FADE));
            }
        }
        pose
    }
    /// Seconds of base-clip time this frame: walking advances by distance.
    fn base_step(&self, dt: f32) -> f32 {
        if self.base != Clip::Walk {
            return dt;
        }
        let cycles = if self.treadmill {
            dt / self.walk_period
        } else {
            self.travel / (self.gait.cycle(self.plan.leg) * self.plan.scale).max(1e-3)
        };
        cycles * Clip::Walk.duration()
    }
    fn advance(&mut self, dt: f32) -> (Pose, Option<f32>) {
        self.events.clear();
        self.armor_flash = (self.armor_flash - dt * 4.0).max(0.0);
        self.special = self.special.map(|t| t + dt).filter(|&t| t < 0.9);
        self.since_attack = (self.since_attack + dt).min(1e6);
        self.clock += dt;
        if let Some((clip, t)) = self.preview {
            return (
                self.pose(clip, t),
                (clip == Clip::Walk).then_some(t / clip.duration()),
            );
        }
        let previous = self.base_t;
        self.base_t += self.base_step(dt);
        let mut pose = self.pose(self.base, self.base_t);
        let mut walk = (self.base == Clip::Walk && self.action.is_none())
            .then_some(self.base_t / Clip::Walk.duration());
        if self.action.is_some() {
            pose = self.action_pose(dt);
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
    /// Distance and direction travelled since the last frame, and the aim
    /// target in figure space with its eased weight.
    fn observe(&mut self, at: Vec3, frame: &Mat4, dt: f32) {
        let moved = self.last_root.map_or(Vec3::ZERO, |p| (at - p).with_y(0.0));
        self.last_root = Some(at);
        // A jump of a whole tile is a spawn or a reconnect, not a step.
        self.travel = if moved.length() < 1.0 {
            moved.length()
        } else {
            0.0
        };
        if self.travel > 1e-5 {
            let local = frame.inverse().transform_vector3(moved).with_y(0.0);
            self.heading = local.normalize_or(Vec3::Z);
        }
        let engaged = self.aim_at.is_some() && self.base != Clip::Walk;
        let goal = if engaged { 1.0 } else { 0.0 };
        let rate = dt / if engaged { AIM_IN } else { AIM_OUT };
        let weight = self.aim.map_or(0.0, |a| a.weight);
        let weight = weight + (goal - weight).clamp(-rate, rate);
        let target = self
            .aim_at
            .map(|w| frame.inverse().transform_point3(w))
            .or(self.aim.map(|a| a.target));
        self.aim = target
            .filter(|_| weight > 0.0)
            .map(|target| Aim { target, weight });
    }
}
pub fn animate(
    time: Res<Time>,
    mut animators: Query<(&mut Animator, &Transform)>,
    mut transforms: Query<&mut Transform, Without<Animator>>,
    mut visibility: Query<&mut Visibility, Without<Animator>>,
) {
    for (mut a, root) in &mut animators {
        let dt = if a.hit_stop > 0.0 {
            let stopped = (time.delta_secs() * a.rate).min(a.hit_stop);
            a.hit_stop -= stopped;
            (time.delta_secs() * a.rate) - stopped
        } else {
            time.delta_secs() * a.rate
        };
        if dt <= 0.0 {
            continue;
        }
        let dt = time.delta_secs() * a.rate;
        let figure = transforms
            .get(a.figure)
            .map_or(Mat4::IDENTITY, |t| t.to_matrix());
        let frame = root.to_matrix() * figure;
        a.observe(root.translation, &frame, dt);
        let (mut pose, walk) = a.advance(dt);
        let anatomy = a.kit.and_then(|kit| kit.anatomy);
        let walk = walk.filter(|_| anatomy.is_none_or(|body| body.planted_legs));
        if a.carried {
            apply_mounted(&mut pose);
            a.feet.reset();
        } else if let Some(phase) = walk {
            let (r, o) = pose[Joint::Root.index()];
            let root_joint = Mat4::from_rotation_translation(ik::quat(r), o);
            let stride = Stride {
                phase,
                dir: a.heading,
                lock: !a.treadmill,
            };
            let (plan, gait) = (a.plan, a.gait);
            a.feet
                .apply(&mut pose, &stride, frame * root_joint, &plan, &gait);
        } else {
            a.feet.reset();
        }
        for node in rig::nodes().filter(|n| {
            !matches!(n, rig::Node::Secondary(_)) || anatomy.is_some_and(|body| !body.springs)
        }) {
            if let Ok(mut t) = transforms.get_mut(a.joints[node.index()]) {
                let (rot, off) = pose[node.index()];
                t.translation = anatomy.map_or_else(
                    || rig::rest(node, &a.plan),
                    |body| (body.rest)(node, &a.plan),
                ) + off;
                t.rotation = ik::quat(rot);
            }
        }
        if let Some(kit) = a.kit {
            show_loaded(kit, &a, &mut visibility);
        }
    }
}

/// Shows a kit's loaded prop (an archer's nocked arrow) only while its shot
/// is loaded, so it leaves the bow at Release instead of staying there.
fn show_loaded(
    kit: &Kit,
    a: &Animator,
    visibility: &mut Query<&mut Visibility, Without<Animator>>,
) {
    let Attack::Shot {
        loaded: Some((node, shown)),
        ..
    } = kit.attack
    else {
        return;
    };
    let (clip, t) = a.clip_time();
    if let Ok(mut v) = visibility.get_mut(a.joints[node.index()]) {
        *v = if shown(clip, t) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Seated saddle posture for units riding a carrier (e.g. Cargo Crab).
fn apply_mounted(pose: &mut Pose) {
    pose[Joint::ThighL.index()].0 = Vec3::new(65.0, -12.0, -14.0);
    pose[Joint::ThighR.index()].0 = Vec3::new(65.0, 12.0, 14.0);
    pose[Joint::ShinL.index()].0 = Vec3::new(-75.0, 0.0, 0.0);
    pose[Joint::ShinR.index()].0 = Vec3::new(-75.0, 0.0, 0.0);
    pose[Joint::FootL.index()].0 = Vec3::new(15.0, 0.0, 0.0);
    pose[Joint::FootR.index()].0 = Vec3::new(15.0, 0.0, 0.0);
    pose[Joint::Hips.index()].1[1] -= 0.05;
}
