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
            tools::add(
                pose,
                Joint::Head,
                [
                    3.0 * (m.clock * tempo * std::f32::consts::PI).sin(),
                    0.0,
                    0.0,
                ],
            );
        }
        Clip::Walk => {
            moves::walk(m, pose, -3.0, 2.0);
            let march = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::add(pose, Joint::Head, [-6.0, 0.0, -2.0 * march]);
            tools::add(pose, Joint::Chest, [2.0 * march.abs(), 0.0, 0.0]);
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
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let flourish = tools::hold(m.t.rem_euclid(0.8), [0.28, 0.42, 0.50, 0.72]);
            tools::set(pose, Joint::Head, [-5.0 * flourish, 0.0, 0.0]);
            tools::set(pose, Joint::Spine, [-3.0 * flourish, 0.0, 0.0]);
            let performance = Moment { clock: m.t, ..*m };
            drum(&performance, pose, &p, if f.a { 12.5 } else { 10.0 });
            tools::plant(pose, &p, tools::stance(&p, 1.2, 0.0));
        }
        _ => drum(m, pose, &p, tempo),
    }
}

/// Sticks alternate on the drum head at `tempo` beats per second.
fn drum(m: &Moment, pose: &mut Pose, p: &Proportions, tempo: f32) {
    let beat = m.clock * tempo * 0.5;
    let (_, depth) = model::drum_size(Form::of(m.tier, m.branch));
    let hip_to_chest = tools::chest(pose, p).inverse() * tools::fk(pose, p, &[Joint::Hips]);
    let flourish = if m.clip == Clip::Victory {
        tools::hold(m.t.rem_euclid(0.8), [0.28, 0.42, 0.50, 0.72])
    } else {
        0.0
    };
    for (left, phase) in [(false, 0.0f32), (true, std::f32::consts::PI)] {
        let lift = ((beat * std::f32::consts::TAU + phase).sin() * 0.5 + 0.5).powi(2);
        let grip = Quat::from_rotation_x((60.0 + 25.0 * lift).to_radians());
        let contact = Vec3::new(
            if left { 0.13 } else { -0.025 },
            0.02 + depth * 0.5 + 0.0075,
            0.27,
        );
        let tip =
            hip_to_chest.transform_point3(contact) + Vec3::Y * (0.075 * lift + 0.06 * flourish);
        let target = tip - grip * Vec3::Z * 0.23;
        let pole = Vec3::new(if left { 1.0 } else { -1.0 }, -0.8, -0.3);
        tools::reach(pose, left, target, Some(grip), (p, pole));
    }
}
