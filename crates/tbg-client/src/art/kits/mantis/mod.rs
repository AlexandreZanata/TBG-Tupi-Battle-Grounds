//! Mantis's prayer idle, planted high-knee gait, opposing scythe cuts and confirmed crossed fourth strike.
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
    gait: Gait {
        stride: 1.1,
        lift: 0.32,
        ..Gait::LIGHT
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, -0.04, 0.30),
        impact: |_, _| Impact::Bite,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    for (s, upper, fore) in [(-1.0, UpperArmR, ForearmR), (1.0, UpperArmL, ForearmL)] {
        tools::set(p, upper, [-55.0, 0.0, s * 25.0]);
        tools::set(p, fore, [-105.0, 0.0, 0.0]);
    }
    match m.clip {
        Clip::Idle => {
            let groom = tools::wave(m.clock, 0.8, 0.1);
            tools::set(p, Chest, [2.0, 0.0, 0.0]);
            tools::set(p, Head, [3.0, groom * 7.0, -2.0 * groom]);
            tools::add(p, ForearmR, [-5.0 * groom.max(0.0), 5.0 * groom, 0.0]);
            tools::add(p, ForearmL, [5.0 * groom.min(0.0), -5.0 * groom, 0.0]);
            tools::plant(p, &body, tools::stance(&body, 1.05, 0.035));
        }
        Clip::Walk => {
            moves::walk(m, p, 4.0, 2.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::add(p, UpperArmL, [5.0 * step, 0.0, 0.0]);
            tools::add(p, UpperArmR, [-5.0 * step, 0.0, 0.0]);
            tools::set(p, Head, [-4.0, step * -3.0, 0.0]);
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let k = tools::hold(m.t, [0.15, 0.30, 0.38, 0.60]);
            let (j, s) = if m.variant == 0 {
                (ForearmR, -1.0)
            } else {
                (ForearmL, 1.0)
            };
            let fold = tools::hold(m.t, [0.0, 0.12, 0.15, 0.30]);
            tools::set(
                p,
                j,
                [
                    -105.0 - 16.0 * fold + 90.0 * k,
                    0.0,
                    s * (35.0 * k - 8.0 * fold),
                ],
            );
            tools::set(
                p,
                Chest,
                [-4.0 * fold + 3.0 * k, s * (20.0 * k - 8.0 * fold), 0.0],
            );
            tools::set(p, Head, [3.0 * fold - 2.0 * k, -s * 5.0 * k, 0.0]);
        }
        Clip::Hit => {
            moves::hit(m, p, 1.0);
            tools::shift(p, Head, [0.0, 0.0, -0.03]);
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            tools::set(p, ForearmL, [-120.0, 0.0, 35.0]);
            tools::set(p, ForearmR, [-120.0, 0.0, -35.0]);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => moves::evolve(m, p, &body, Power::Raise),
        Clip::Victory => {
            let display = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            p[Root.index()] = (Vec3::ZERO, Vec3::ZERO);
            for (upper, fore, s) in [(UpperArmL, ForearmL, 1.0), (UpperArmR, ForearmR, -1.0)] {
                tools::set(
                    p,
                    upper,
                    [-55.0 + 15.0 * display, 0.0, s * (25.0 + 28.0 * display)],
                );
                tools::set(p, fore, [-105.0 + 62.0 * display, 0.0, s * 15.0 * display]);
            }
            tools::set(p, Head, [6.0 * display, 0.0, 0.0]);
            tools::set(p, Chest, [-4.0 * display, 0.0, 0.0]);
            tools::plant(p, &body, tools::stance(&body, 1.05, 0.035));
        }
    }
    if let Some(t) = m.special {
        let cross = tools::hold(t, [0.25, 0.30, 0.45, 0.7]);
        tools::add(p, ForearmR, [35.0 * cross, 0.0, 40.0 * cross]);
        tools::add(p, ForearmL, [35.0 * cross, 0.0, -40.0 * cross]);
    }
    p[Secondary(Spring::HeadGear).index()].0.z = (m.clock * 1.2).sin() * 3.0;
    tools::face(p, m.aim, Vec3::new(0.0, 1.0, 0.3));
}
