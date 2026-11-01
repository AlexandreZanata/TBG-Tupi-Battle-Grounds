//! Ravager's alternating tripod crawl and paired hook/scythe attack tracks.

mod model;

use super::{common::moves, tools, Anatomy, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node,
        Node::{Body, Prop},
        Socket::*,
    },
};
use crate::vfx::impacts::{Burst, Impact};
use bevy::prelude::*;

static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: model::parent,
    springs: false,
    planted_legs: false,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Body(ForearmR),
        tip: |_, _| Vec3::new(-0.07, -0.03, 0.43),
        impact: |_, _| Impact::Bite,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

fn turn(p: &mut Pose, n: Node, xyz: [f32; 3]) {
    p[n.index()].0 += Vec3::from(xyz);
}

fn pose(m: &Moment, p: &mut Pose) {
    p.fill((Vec3::ZERO, Vec3::ZERO));
    match m.clip {
        Clip::Idle | Clip::Victory => {
            let scrape = (m.clock * 1.7).sin() * 4.0;
            turn(p, Body(ForearmL), [0.0, scrape, 0.0]);
            turn(p, Body(ForearmR), [0.0, -scrape, 0.0]);
            tools::shift(p, Chest, [0.0, (m.clock * 1.8).sin() * 0.008, 0.0]);
        }
        Clip::Walk => crawl(m, p),
        Clip::AttackMelee | Clip::AttackRanged => strike(m, p),
        Clip::Cast => {}
        Clip::Hit => {
            let h = tools::hold(m.t, [0.0, 0.04, 0.09, 0.25]);
            tools::shift(p, Hips, [0.0, -0.10 * h, 0.0]);
            tools::set(p, Head, [14.0 * h, 0.0, 0.0]);
        }
        Clip::Death => {
            let fold = tools::window(m.t, 0.0, 0.65);
            tools::shift(p, Hips, [0.0, -0.39 * fold, 0.0]);
            for (n, s) in [
                (Body(UpperArmL), 1.0),
                (Body(UpperArmR), -1.0),
                (Body(ThighL), 1.0),
                (Body(ThighR), -1.0),
                (Prop(WeaponR), 1.0),
                (Prop(OffhandL), -1.0),
            ] {
                turn(p, n, [25.0 * fold, 0.0, s * -65.0 * fold]);
            }
        }
        Clip::Spawn | Clip::Evolve => {
            let spread = tools::hold(m.t, [0.0, 0.25, 0.4, 0.9]);
            tools::shift(p, Hips, [0.0, 0.12 * spread, 0.0]);
            for (n, s) in [
                (Body(UpperArmL), 1.0),
                (Body(UpperArmR), -1.0),
                (Prop(WeaponR), 1.0),
                (Prop(OffhandL), -1.0),
            ] {
                turn(p, n, [-15.0 * spread, 0.0, s * -20.0 * spread]);
            }
        }
    }
    let frenzy = f32::from(m.counter) / 4.0;
    tools::add(p, ForearmL, [0.0, frenzy * -12.0, frenzy * -8.0]);
    tools::add(p, ForearmR, [0.0, frenzy * 12.0, frenzy * 8.0]);
    tools::add(p, Head, [frenzy * 8.0, 0.0, 0.0]);
}

fn crawl(m: &Moment, p: &mut Pose) {
    let phase = m.walk.unwrap_or(0.0) * std::f32::consts::TAU;
    for (upper, lower, group) in [
        (Body(UpperArmL), Body(ForearmL), 0.0),
        (Body(ThighR), Body(ShinR), 0.0),
        (Prop(WeaponR), Prop(Back), 0.0),
        (Body(UpperArmR), Body(ForearmR), std::f32::consts::PI),
        (Body(ThighL), Body(ShinL), std::f32::consts::PI),
        (Prop(OffhandL), Prop(StringTop), std::f32::consts::PI),
    ] {
        let step = (phase + group).sin();
        turn(p, upper, [step * 20.0, step * 10.0, 0.0]);
        turn(p, lower, [-step.max(0.0) * 25.0, 0.0, 0.0]);
    }
    tools::shift(p, Hips, [0.0, (phase * 2.0).cos() * 0.012, 0.0]);
}

fn strike(m: &Moment, p: &mut Pose) {
    let coil = tools::hold(m.t, [0.0, 0.18, 0.22, 0.30]);
    let cut = tools::hold(m.t, [0.22, 0.30, 0.37, 0.60]);
    let (hook, blade, s) = (UpperArmL, UpperArmR, 1.0);
    tools::set(p, hook, [-12.0 * coil, -25.0 * coil * s, -10.0 * coil * s]);
    tools::set(
        p,
        blade,
        [
            -48.0 * coil + 20.0 * cut,
            35.0 * coil * s - 30.0 * cut * s,
            0.0,
        ],
    );
    tools::set(
        p,
        Chest,
        [
            8.0 * coil - 6.0 * cut,
            -10.0 * coil * s + 12.0 * cut * s,
            0.0,
        ],
    );
    tools::shift(p, Chest, [0.0, 0.0, -0.04 * coil + 0.09 * cut]);
}
