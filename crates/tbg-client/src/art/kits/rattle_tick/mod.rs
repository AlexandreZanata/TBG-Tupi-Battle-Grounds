//! Rattle's club swings while Tick independently points, grips and throws from the shoulder.
mod model;
use super::{
    common::{moves, moves::*},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node::{Prop, Secondary},
        Socket, Spring,
    },
};
use crate::vfx::impacts::{Burst, Impact};
use bevy::prelude::*;
static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: model::parent,
    springs: false,
    planted_legs: true,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, -0.40, 0.0),
        impact: |_, _| Impact::Crush,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.9);
            p[Prop(Socket::Back).index()].0.x = -55.0 + (m.clock * 0.8).sin() * 10.0;
        }
        Clip::Walk => {
            moves::walk(m, p, 5.0, 2.0);
            p[Secondary(Spring::Back).index()].0.x = (m.clock * 4.0).sin() * 7.0;
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let load = tools::hold(m.t, [0.0, 0.16, 0.18, 0.30]);
            let hit = tools::hold(m.t, [0.18, 0.30, 0.38, 0.70]);
            tools::set(
                p,
                Chest,
                [-8.0 * load + 12.0 * hit, 25.0 * load - 28.0 * hit, 0.0],
            );
            tools::set(p, UpperArmR, [-145.0 * load - 50.0 * hit, 0.0, -12.0]);
            tools::set(p, ForearmR, [-30.0, 0.0, 0.0]);
            tools::set(p, UpperArmL, [-55.0 * load, 0.0, 20.0]);
        }
        Clip::Hit => {
            moves::hit(m, p, 0.8);
            p[Prop(Socket::Back).index()].0.x = -65.0;
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Forward);
            p[Prop(Socket::Back).index()].0.x = -90.0;
            p[Prop(Socket::StringTop).index()].0.x = -90.0;
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Drop),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Burst);
            p[Prop(Socket::Back).index()].0.z = 45.0 * (m.t * 7.0).sin();
        }
        Clip::Victory => moves::victory(m, p, &body, Cheer::Both),
    }
    if let Some(t) = m.special {
        let wind = tools::hold(t, [0.0, 0.15, 0.22, 0.30]);
        let throw = tools::hold(t, [0.18, 0.30, 0.40, 0.70]);
        p[Prop(Socket::StringTop).index()].0.x = -150.0 * wind - 30.0 * throw;
        p[Secondary(Spring::Back).index()].0.x = 20.0 * throw;
    }
    p[Secondary(Spring::HeadGear).index()].0.y = (m.clock * 0.9).sin() * 15.0;
    tools::face(p, m.aim, Vec3::new(0.0, 1.0, 0.2));
}
