//! Per-champion action accents and celebrations over shared event-timed locomotion.
use super::super::{
    common::moves::{self, Entry, Fall, Power},
    tools, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::Joint::*,
};
use bevy::prelude::*;
use tbg_core::UnitKind::{self, *};
pub fn pose(k: UnitKind, m: &Moment, p: &mut Pose) {
    let plan = super::plan(k, m.tier);
    match m.clip {
        Clip::Idle => moves::idle(m, p, &plan, if k == BasaltBulwark { 0.9 } else { 0.5 }),
        Clip::Walk => moves::walk(m, p, if k == FlintRunner { 12.0 } else { 4.0 }, 2.0),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => action(k, m, p, &plan),
        Clip::Hit => moves::hit(m, p, 0.65),
        Clip::Spawn => moves::spawn(
            m,
            p,
            &plan,
            match k {
                BasaltBulwark | CinderMaul => Entry::Drop,
                FlintRunner | PebbleSlinger => Entry::Pop,
                ClayBinder => Entry::Rise,
                _ => Entry::March,
            },
        ),
        Clip::Evolve => moves::evolve(
            m,
            p,
            &plan,
            if matches!(k, ClayBinder | HideMender) {
                Power::Raise
            } else {
                Power::Burst
            },
        ),
        Clip::Death => moves::death(
            m,
            p,
            &plan,
            match k {
                BasaltBulwark | HideMender => Fall::Kneel,
                FlintRunner | HookBrawler => Fall::Forward,
                RivetBallista => Fall::Sink,
                _ => Fall::Back,
            },
        ),
        Clip::Victory => {
            victory(k, m, p);
            return;
        }
    }
    if !matches!(m.clip, Clip::Spawn | Clip::Evolve | Clip::Death) {
        hold(k, m, p, &plan);
    }
}
fn hold(k: UnitKind, m: &Moment, p: &mut Pose, plan: &crate::art::rig::Proportions) {
    let beat = if matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast) {
        tools::hold(m.t, [0.0, 0.12, 0.25, 0.50])
    } else {
        0.0
    };
    let (right, left) = match k {
        BasaltBulwark => (
            Vec3::new(-0.25, 0.02, 0.20 + beat * 0.15),
            Vec3::new(0.27, 0.11, 0.15),
        ),
        CinderMaul => (
            Vec3::new(-0.23, 0.03 + beat * 0.20, 0.13 + beat * 0.16),
            Vec3::new(0.23, 0.07, 0.14),
        ),
        HookBrawler => (
            Vec3::new(-0.22, 0.09, 0.19 + beat * 0.18),
            Vec3::new(0.22, 0.11, 0.12),
        ),
        FlintRunner => (
            Vec3::new(-0.20, -0.02, 0.12 + beat * 0.20),
            Vec3::new(0.20, 0.02, 0.14),
        ),
        ClayBinder => (
            Vec3::new(-0.23, 0.03, 0.17),
            Vec3::new(0.22, 0.13 + beat * 0.06, 0.21),
        ),
        GustDrummer => (
            Vec3::new(-0.12, 0.19 + beat * 0.08, 0.25),
            Vec3::new(0.12, 0.19 + (1.0 - beat) * 0.06, 0.25),
        ),
        HideMender => (
            Vec3::new(-0.18, 0.13, 0.25 + beat * 0.04),
            Vec3::new(0.15, 0.11, 0.23),
        ),
        PebbleSlinger => (
            Vec3::new(-0.16, 0.16, 0.24),
            Vec3::new(0.12, 0.13, 0.09 - beat * 0.05),
        ),
        _ => (
            Vec3::new(-0.12, 0.17, 0.24 - beat * 0.04),
            Vec3::new(0.12, 0.12, 0.28),
        ),
    };
    tools::reach(
        p,
        false,
        right,
        Some(Quat::IDENTITY),
        (plan, Vec3::new(-0.6, -0.8, 0.5)),
    );
    tools::reach(
        p,
        true,
        left,
        Some(Quat::IDENTITY),
        (plan, Vec3::new(0.6, -0.8, 0.5)),
    );
}
fn action(k: UnitKind, m: &Moment, p: &mut Pose, plan: &crate::art::rig::Proportions) {
    let hit = tools::hold(m.t, [0.05, 0.18, 0.30, 0.52]);
    tools::face(p, m.aim, Vec3::ZERO);
    tools::plant(p, plan, tools::stance(plan, 1.15, 0.03));
    let (pitch, yaw) = match k {
        BasaltBulwark => (8.0, 10.0),
        CinderMaul => (20.0, -14.0),
        HookBrawler => (5.0, 22.0),
        FlintRunner => (12.0, -20.0),
        ClayBinder => (-8.0, 6.0),
        GustDrummer => (10.0, 0.0),
        HideMender => (-4.0, -5.0),
        PebbleSlinger => (-6.0, 10.0),
        _ => (4.0, 0.0),
    };
    tools::add(p, Chest, [pitch * hit, yaw * hit, 0.0]);
    if k == FlintRunner {
        tools::shift(p, Hips, [0.0, -0.035 * hit, 0.05 * hit]);
    }
}
fn victory(k: UnitKind, m: &Moment, p: &mut Pose) {
    let plan = super::plan(k, m.tier);
    let t = m.t.min(3.0);
    let settle = 1.0 - tools::window(t, 2.45, 3.0);
    let pulse = (t * std::f32::consts::TAU * 1.5).sin() * settle;
    moves::idle(&Moment { aim: None, ..*m }, p, &plan, 0.5);
    match k {
        BasaltBulwark => {
            tools::set(p, UpperArmL, [-65.0, 0.0, 12.0]);
            tools::shift(p, Hips, [0.0, -0.025 * pulse.abs(), 0.0]);
            tools::add(p, Head, [-8.0 * pulse, 0.0, 0.0]);
        }
        CinderMaul => {
            tools::set(p, UpperArmL, [-65.0 + 18.0 * pulse, 0.0, 15.0]);
            tools::set(p, UpperArmR, [-65.0 - 18.0 * pulse, 0.0, -15.0]);
        }
        HookBrawler => {
            tools::set(p, ForearmR, [-80.0, 0.0, 0.0]);
            tools::set(p, HandR, [0.0, 0.0, 45.0 * pulse]);
            tools::add(p, Chest, [0.0, 12.0 * pulse, 0.0]);
        }
        FlintRunner => {
            tools::shift(p, Root, [0.0, 0.06 * pulse.max(0.0), 0.0]);
            tools::set(p, UpperArmR, [-110.0, 0.0, -10.0]);
            tools::add(p, Head, [0.0, 0.0, 12.0 * pulse]);
        }
        ClayBinder => {
            tools::set(p, UpperArmL, [-100.0, 0.0, 20.0]);
            tools::set(p, ForearmL, [-30.0, 0.0, 0.0]);
            tools::add(p, Chest, [0.0, 0.0, 7.0 * pulse]);
        }
        GustDrummer => {
            hold(
                k,
                &Moment {
                    clip: Clip::Cast,
                    t: t % 0.5,
                    ..*m
                },
                p,
                &plan,
            );
            tools::add(p, Head, [10.0 * pulse, 0.0, 0.0]);
        }
        HideMender => {
            tools::set(p, UpperArmR, [-80.0, 0.0, -15.0]);
            tools::set(p, ForearmR, [-45.0, 0.0, 0.0]);
            tools::add(p, HandR, [0.0, 20.0 * pulse, 0.0]);
            tools::add(p, Head, [-10.0, 0.0, 0.0]);
        }
        PebbleSlinger => {
            tools::set(p, UpperArmR, [-140.0, 0.0, -15.0]);
            tools::shift(p, Hips, [0.0, 0.02 * pulse, 0.0]);
            tools::set(p, HandR, [0.0, 0.0, 20.0 * pulse]);
        }
        _ => {
            tools::set(p, Head, [-15.0 + 8.0 * pulse, 20.0 * pulse, 0.0]);
            tools::set(p, UpperArmR, [-90.0, 0.0, -10.0]);
            tools::shift(p, Hips, [0.0, -0.03 * pulse.abs(), 0.0]);
        }
    }
    tools::plant(p, &plan, tools::stance(&plan, 1.15, 0.02));
}
