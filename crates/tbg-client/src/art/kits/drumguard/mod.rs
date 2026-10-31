//! Drumguard kit: a drummer with no attack of his own who beats the march
//! all the time — standing, walking (knees up), even in victory; his
//! bonded ally takes the speed and the cover.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Proportions},
};
use crate::vfx::impacts::Burst;
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::None,
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    // The march quickens on branch A.
    let tempo = if f.a { 9.0 } else { 7.5 };
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            drum(m, pose, &p, tempo);
        }
        Clip::Walk => {
            moves::walk(m, pose, -3.0, 2.0);
            tools::add(pose, Joint::Head, [-6.0, 0.0, 0.0]);
            drum(m, pose, &p, tempo);
        }
        Clip::Hit => {
            moves::hit(m, pose, 1.0);
            drum(m, pose, &p, 0.0);
        }
        Clip::Death => moves::death(m, pose, &p, Fall::Back),
        Clip::Spawn => {
            moves::spawn(m, pose, &p, Entry::March);
            drum(m, pose, &p, tempo);
        }
        Clip::Evolve => {
            moves::evolve(m, pose, &p, Power::Spin);
        }
        Clip::Victory => {
            moves::victory(m, pose, &p, Cheer::Dance);
            drum(m, pose, &p, tempo * 1.4);
        }
        _ => drum(m, pose, &p, tempo),
    }
}

/// Sticks alternate on the drum head at `tempo` beats per second.
fn drum(m: &Moment, pose: &mut Pose, p: &Proportions, tempo: f32) {
    let beat = m.clock * tempo * 0.5;
    for (left, phase) in [(false, 0.0f32), (true, std::f32::consts::PI)] {
        let hit = (beat * std::f32::consts::TAU + phase).sin().max(0.0);
        let x = if left { 0.04 } else { -0.06 };
        let target = Vec3::new(x, -0.02 + 0.09 * hit, 0.27);
        let grip = Quat::from_rotation_x((70.0 - 50.0 * hit).to_radians());
        let pole = Vec3::new(if left { 1.0 } else { -1.0 }, -0.8, -0.3);
        tools::reach(pose, left, target, Some(grip), (p, pole));
    }
}
