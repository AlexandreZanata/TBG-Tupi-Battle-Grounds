//! Puppetmaster kit: it dances its marionette with small jerks of the
//! control while idle, strolls with a showman's swagger, and points its
//! control at the enemy it turns, the strings' hex leaving the cross on
//! the cast Release (confusing is its attack). Rises from the ground;
//! falls back on death.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Node, Proportions, Socket},
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| model::CROSS,
        shot: |_, _| ShotKind::Puppet,
        flash: |_, _| None,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            work(m, pose, &p, f.b, 1.0);
        }
        Clip::Walk => {
            moves::walk(m, pose, 2.0, 5.0);
            work(m, pose, &p, f.b, 0.5);
        }
        Clip::Cast => moves::strike(m, pose, &p, POINT, 1.1),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Back),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Dance),
        _ => work(m, pose, &p, f.b, 0.0),
    }
}

/// Hold the control before the chest and jerk it so the puppet dances;
/// the second control (branch B) answers on the off beat.
fn work(m: &Moment, pose: &mut Pose, p: &Proportions, both: bool, dance: f32) {
    let jerk = |phase: f32| {
        let a = m.clock * 3.2 + phase;
        Quat::from_rotation_z((12.0 * a.sin() * dance).to_radians())
            * Quat::from_rotation_x((10.0 * (a * 1.7).sin() * dance).to_radians())
    };
    let bob = 0.03 * (m.clock * 6.4).sin().abs() * dance;
    tools::reach(
        pose,
        false,
        Vec3::new(-0.16, 0.3 + bob, 0.3),
        Some(jerk(0.0)),
        (p, Vec3::new(-1.0, -0.5, -0.3)),
    );
    if both {
        tools::reach(
            pose,
            true,
            Vec3::new(0.16, 0.3 + bob, 0.3),
            Some(jerk(1.6)),
            (p, Vec3::new(1.0, -0.5, -0.3)),
        );
    }
}
