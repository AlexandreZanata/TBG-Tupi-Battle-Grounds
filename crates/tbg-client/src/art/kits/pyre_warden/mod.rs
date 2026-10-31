//! Pyre Warden's hunched drag, braced nozzle pulse and furnace fold; authoritative flame is a separate event.
mod model;
use super::{
    common::{moves, moves::*},
    tools, Attack, Gait, Kit, Moment,
};
use crate::{
    art::{
        anim::{Clip, Pose},
        rig::{Joint::*, Node::Secondary, Spring},
    },
    vfx::impacts::Burst,
};
use bevy::prelude::*;
pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::None,
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => moves::idle(m, p, &body, 1.1),
        Clip::Walk => moves::walk(m, p, 13.0, 3.0),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let k = tools::hold(m.t, [0.0, 0.26, 0.30, 0.46]);
            tools::set(p, Chest, [12.0 - 6.0 * k, 0.0, 0.0]);
            tools::shift(p, Hips, [0.0, -0.03 * k, -0.03 * k]);
        }
        Clip::Hit => moves::hit(m, p, 0.45),
        Clip::Death => moves::death(m, p, &body, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => moves::evolve(m, p, &body, Power::Raise),
        Clip::Victory => moves::victory(m, p, &body, Cheer::Both),
    }
    if m.clip != Clip::Death {
        tools::reach(
            p,
            false,
            Vec3::new(-0.13, 0.12, 0.28),
            Some(Quat::IDENTITY),
            (&body, Vec3::new(-1.0, -0.1, 0.0)),
        );
        tools::reach(
            p,
            true,
            Vec3::new(-0.10, 0.10, 0.46),
            Some(Quat::IDENTITY),
            (&body, Vec3::new(1.0, -0.1, 0.0)),
        );
        tools::add(p, Chest, [9.0, 0.0, 0.0]);
    }
    p[Secondary(Spring::Back).index()].0.x += (m.clock * 1.5).sin() * 1.5;
    tools::face(p, m.aim, Vec3::new(0.0, 0.8, 0.3));
}
