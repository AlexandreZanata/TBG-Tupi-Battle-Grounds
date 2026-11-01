//! Rattle's club swings while Tick independently aims and fires a pistol from the shoulder.
mod model;
mod tick;
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
            let chat = tools::hold((m.clock * 0.23).rem_euclid(1.0), [0.12, 0.28, 0.42, 0.64]);
            p[Prop(Socket::Back).index()].0.x = -55.0 - 35.0 * chat;
            tools::add(p, Head, [-5.0 * chat, 18.0 * chat, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, p, 5.0, 2.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            p[Secondary(Spring::Back).index()].0.x = -4.0 * step;
            p[Secondary(Spring::Back).index()].0.z = -4.0 * step;
            p[Prop(Socket::Back).index()].0.x = -45.0;
            tools::add(p, Head, [0.0, 4.0 * step, 0.0]);
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
            p[Secondary(Spring::HeadGear).index()].0.x = 12.0 * hit;
            p[Secondary(Spring::Back).index()].0.z = -5.0 * load + 5.0 * hit;
            p[Prop(Socket::Back).index()].0.x = -45.0 - 25.0 * load;
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
        Clip::Victory => {
            p.fill((Vec3::ZERO, Vec3::ZERO));
            let t = m.t.rem_euclid(0.8);
            let brother = tools::hold(t, [0.0, 0.14, 0.28, 0.42]);
            let tick = tools::hold(t, [0.32, 0.46, 0.60, 0.78]);
            tools::set(p, UpperArmR, [8.0, 0.0, -18.0]);
            tools::set(p, ForearmR, [-20.0, 0.0, 0.0]);
            tools::set(p, UpperArmL, [-35.0 - 40.0 * brother, 0.0, 30.0]);
            tools::set(p, ForearmL, [-70.0, 0.0, 0.0]);
            tools::set(p, Head, [-3.0, 12.0 * tick, 0.0]);
            p[Prop(Socket::Back).index()].0.x = -50.0 - 85.0 * tick;
            p[Prop(Socket::StringTop).index()].0.x = -40.0;
            p[Secondary(Spring::Back).index()].0.z = -4.0 * brother;
            tools::plant(p, &body, tools::stance(&body, 1.3, 0.03));
        }
    }
    p[Secondary(Spring::HeadGear).index()].0.y = (m.clock * 0.9).sin() * 15.0;
    // A downward club on a dangling hand crossed the floor even in Idle.
    // Keep one chest-space frame and solve the holding wrist to it.
    let active = matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast);
    let load = if active {
        tools::hold(m.t, [0.0, 0.16, 0.18, 0.30])
    } else {
        0.0
    };
    let hit = if active {
        tools::hold(m.t, [0.18, 0.30, 0.38, 0.70])
    } else {
        0.0
    };
    let rotation = Quat::from_rotation_x((-45.0 - 100.0 * load - 20.0 * hit).to_radians());
    let mut origin = Vec3::new(-0.30, 0.05, 0.22);
    let world = crate::art::presentation::pose_world(tbg_core::UnitKind::RattleTick, &body, p);
    let chest = world[Chest.index()];
    let mut low = f32::INFINITY;
    for x in [-0.18, 0.18] {
        for y in [-0.55, 0.13] {
            for z in [-0.12, 0.12] {
                low = low.min(
                    chest
                        .transform_point3(origin + rotation * Vec3::new(x, y, z))
                        .y,
                );
            }
        }
    }
    if low < 0.04 {
        origin += chest.inverse().transform_vector3(Vec3::Y * (0.04 - low));
    }
    p[Prop(Socket::WeaponR).index()] = (crate::art::ik::degrees(rotation), origin);
    tools::reach(
        p,
        false,
        origin,
        Some(rotation),
        (&body, Vec3::new(-1.0, -0.2, 0.0)),
    );
    tools::face(p, m.aim, Vec3::new(0.0, 1.0, 0.2));
    tick::pose(m, p, &body);
}
