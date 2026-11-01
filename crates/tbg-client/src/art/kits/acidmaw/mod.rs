//! Acidmaw's throat breathing, four-petal spit snap and precise head-space emitter.

mod model;
use super::{
    common::{moves, moves::*},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node::{Body, Prop, Secondary},
        Socket::*,
        Spring,
    },
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
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
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        from: Body(Head),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.36),
        shot: |_, _| ShotKind::Acid,
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            let inhale = (m.clock * 1.4).sin();
            tools::set(p, Chest, [-2.0 * inhale, 0.0, 0.0]);
            tools::set(
                p,
                Head,
                [
                    4.0 + 3.0 * inhale,
                    4.0 * tools::wave(m.clock, 0.5, 1.0),
                    0.0,
                ],
            );
            tools::set(p, UpperArmL, [12.0, 0.0, 14.0]);
            tools::set(p, UpperArmR, [12.0, 0.0, -14.0]);
            tools::plant(p, &body, tools::stance(&body, 1.15, 0.02));
        }
        Clip::Walk => {
            moves::walk(m, p, 4.0, 3.0);
            tools::add(p, ShinL, [-8.0, 0.0, 0.0]);
            tools::add(p, ShinR, [-8.0, 0.0, 0.0]);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(p, Head, [2.0 - 3.0 * step, -2.0 * step, 0.0]);
            tools::set(p, UpperArmL, [14.0, 0.0, 12.0]);
            tools::set(p, UpperArmR, [14.0, 0.0, -12.0]);
        }
        Clip::AttackRanged | Clip::AttackMelee | Clip::Cast => spit(m, p),
        Clip::Hit => {
            moves::hit(m, p, 1.0);
            tools::shift(p, Head, [0.0, -0.08, -0.08]);
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            tools::shift(p, Head, [0.0, -0.35 * tools::window(m.t, 0.0, 0.7), -0.1]);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            jaws(p, tools::hold(m.t, [0.0, 0.25, 0.4, 0.8]) * 35.0);
        }
        Clip::Victory => {
            let fan = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            p[Root.index()] = (Vec3::ZERO, Vec3::ZERO);
            tools::set(p, Head, [-15.0 * fan, 8.0 * fan, 0.0]);
            tools::shift(p, Head, [0.0, 0.035 * fan, 0.0]);
            tools::set(p, UpperArmL, [10.0, 0.0, 18.0 + 8.0 * fan]);
            tools::set(p, UpperArmR, [10.0, 0.0, -18.0 - 8.0 * fan]);
            tools::set(p, ForearmL, [-12.0, 0.0, 0.0]);
            tools::set(p, ForearmR, [-12.0, 0.0, 0.0]);
            jaws(p, 5.0 + 25.0 * fan);
            tools::plant(p, &body, tools::stance(&body, 1.15, 0.02));
        }
    }
    let sac = Secondary(Spring::Back).index();
    p[sac].1.y += (m.clock * 2.2).sin() * 0.012 + f32::from(m.counter) * 0.018;
    if m.clip == Clip::Idle {
        jaws(p, 3.0 + (m.clock * 1.4).sin() * 2.0);
    }
    tools::face(p, m.aim, Vec3::new(0.0, 1.0, 0.3));
}

fn jaws(p: &mut Pose, open: f32) {
    for (n, r) in [
        (WeaponR, Vec3::Y * open),
        (OffhandL, -Vec3::Y * open),
        (Back, -Vec3::X * open),
        (StringTop, Vec3::X * open),
    ] {
        p[Prop(n).index()].0 = r;
    }
}
fn spit(m: &Moment, p: &mut Pose) {
    let coil = tools::hold(m.t, [0.0, 0.13, 0.15, 0.18]);
    let snap = tools::hold(m.t, [0.15, 0.18, 0.23, 0.50]);
    tools::shift(p, Head, [0.0, 0.02 * coil, -0.09 * coil + 0.05 * snap]);
    tools::set(p, Head, [-18.0 * coil + 8.0 * snap, 0.0, 0.0]);
    jaws(p, 36.0 * coil + 9.0 * snap);
    tools::set(p, UpperArmL, [18.0 * coil, 0.0, 14.0]);
    tools::set(p, UpperArmR, [18.0 * coil, 0.0, -14.0]);
    p[Secondary(Spring::Back).index()].1.z += coil * 0.04 - snap * 0.025;
    tools::set(p, Chest, [-6.0 * coil + 6.0 * snap, 0.0, 0.0]);
}
