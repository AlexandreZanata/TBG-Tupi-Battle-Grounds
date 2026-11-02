//! Grounded marine locomotion, event-timed weapon actions and individual celebrations.
use super::super::{
    common::moves::{self, Entry, Fall, Power},
    tools, Moment,
};
use super::common::*;
use crate::art::{
    anim::{Clip, Pose},
    rig::Node,
};

pub fn pose(k: UnitKind, m: &Moment, p: &mut Pose) {
    let plan = plan(k, m.tier);
    match m.clip {
        Clip::Idle => moves::idle(m, p, &plan, if k == TrenchFist { 0.8 } else { 0.45 }),
        Clip::Walk => {
            moves::walk(m, p, if k == NacreDuelist { 9.0 } else { 5.0 }, 2.0);
            let sway = m.walk.map_or(0.0, |x| (x * std::f32::consts::TAU).sin());
            tools::add(p, Chest, [2.0, 3.0 * sway, 1.5 * sway]);
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let hit = tools::hold(m.t, [0.05, 0.18, 0.30, 0.55]);
            tools::face(p, m.aim, Vec3::ZERO);
            tools::plant(p, &plan, tools::stance(&plan, 1.18, 0.04));
            let (pitch, yaw) = match k {
                NacreDuelist => (
                    5.0,
                    if m.variant.is_multiple_of(2) {
                        25.0
                    } else {
                        -25.0
                    },
                ),
                ReefLancer => (10.0, -12.0),
                TrenchFist => (18.0, 15.0),
                PearlSlinger => (-8.0, 12.0),
                AbyssHarpooner => (-5.0, -6.0),
                UndertowHerald => (-10.0, 0.0),
                TrenchLamplighter => (-8.0, 5.0),
                FoamWeaver => (-5.0, -4.0),
                _ => (8.0, 6.0),
            };
            tools::add(p, Chest, [pitch * hit, yaw * hit, 0.0]);
            if k == TrenchFist {
                tools::shift(p, Hips, [0.0, -0.025 * hit, 0.055 * hit]);
            }
        }
        Clip::Hit => moves::hit(m, p, 0.6),
        Clip::Spawn => moves::spawn(
            m,
            p,
            &plan,
            match k {
                TrenchFist => Entry::Drop,
                PearlSlinger | PodSnapper => Entry::Pop,
                TrenchLamplighter | FoamWeaver => Entry::Rise,
                _ => Entry::March,
            },
        ),
        Clip::Evolve => moves::evolve(
            m,
            p,
            &plan,
            if matches!(k, FoamWeaver | TrenchLamplighter | UndertowHerald) {
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
                NacreDuelist | ReefLancer => Fall::Forward,
                FoamWeaver | TrenchLamplighter => Fall::Kneel,
                PodSnapper => Fall::Sink,
                _ => Fall::Back,
            },
        ),
        Clip::Victory => {
            victory(k, m, p, &plan);
            return;
        }
    }
    if !matches!(m.clip, Clip::Spawn | Clip::Evolve | Clip::Death) {
        hold(k, m, p, &plan);
    }
    if let Some(t) = m.special {
        let cue = tools::hold(t, [0.0, 0.12, 0.30, 0.65]);
        if k == TrenchLamplighter {
            tools::add(p, UpperArmR, [-30.0 * cue, 0.0, 0.0]);
            tools::add(p, Head, [-12.0 * cue, 0.0, 0.0]);
        }
        if k == UndertowHerald {
            tools::add(p, Chest, [-8.0 * cue, 0.0, 0.0]);
            tools::add(p, ForearmL, [-20.0 * cue, 0.0, 0.0]);
        }
    }
}
fn hold(k: UnitKind, m: &Moment, p: &mut Pose, plan: &Proportions) {
    let hit = if matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast) {
        tools::hold(m.t, [0.05, 0.18, 0.30, 0.54])
    } else {
        0.0
    };
    let (r, l) = match k {
        NacreDuelist => (
            [-0.24, -0.05, 0.12 + 0.22 * hit],
            [0.24, -0.03, 0.12 + 0.14 * hit],
        ),
        ReefLancer => ([-0.21, 0.10, 0.18 + 0.17 * hit], [0.15, 0.03, 0.18]),
        TrenchFist => ([-0.25, 0.04, 0.14 + 0.20 * hit], [0.25, 0.08, 0.12]),
        PearlSlinger => ([-0.18, 0.15, 0.17 + 0.10 * hit], [0.14, 0.08, 0.13]),
        AbyssHarpooner => ([-0.12, 0.15, 0.23], [0.12, 0.14, 0.34]),
        UndertowHerald => ([-0.13, 0.18, 0.20], [0.14, 0.18, 0.25]),
        TrenchLamplighter => ([-0.20, 0.11 + 0.10 * hit, 0.18], [0.14, 0.07, 0.16]),
        FoamWeaver => ([-0.09, 0.13 + 0.04 * hit, 0.26], [0.12, 0.12, 0.27]),
        _ => ([-0.20, 0.11, 0.23 + 0.08 * hit], [0.16, 0.08, 0.14]),
    };
    tools::reach(
        p,
        false,
        Vec3::from(r),
        Some(Quat::IDENTITY),
        (plan, Vec3::new(-0.6, -0.8, 0.5)),
    );
    tools::reach(
        p,
        true,
        Vec3::from(l),
        Some(Quat::IDENTITY),
        (plan, Vec3::new(0.6, -0.8, 0.5)),
    );
    if k == PodSnapper {
        p[Node::Prop(Socket::StringTop).index()].0 = Vec3::new(0.0, -18.0 * hit, 10.0 * hit);
        p[Node::Prop(Socket::StringBottom).index()].0 = Vec3::new(0.0, 18.0 * hit, -10.0 * hit);
    }
}
fn victory(k: UnitKind, m: &Moment, p: &mut Pose, plan: &Proportions) {
    moves::idle(&Moment { aim: None, ..*m }, p, plan, 0.4);
    let pulse = (m.t * std::f32::consts::TAU * 1.5).sin() * (1.0 - tools::window(m.t, 2.4, 3.0));
    match k {
        NacreDuelist => {
            tools::set(p, ForearmL, [-65.0, 0.0, 0.0]);
            tools::set(p, ForearmR, [-65.0, 0.0, 0.0]);
            tools::add(p, Chest, [0.0, 15.0 * pulse, 0.0]);
        }
        ReefLancer => {
            tools::set(p, UpperArmR, [-80.0, 0.0, -12.0]);
            tools::add(p, Head, [-10.0 * pulse.abs(), 0.0, 0.0]);
        }
        TrenchFist => {
            tools::set(p, UpperArmR, [-105.0, 0.0, -20.0]);
            tools::shift(p, Hips, [0.0, -0.03 * pulse.abs(), 0.0]);
        }
        PearlSlinger => {
            tools::set(p, UpperArmR, [-120.0, 0.0, -12.0]);
            tools::add(p, HandR, [0.0, 20.0 * pulse, 0.0]);
        }
        AbyssHarpooner => {
            tools::set(p, ForearmR, [-70.0, 0.0, 0.0]);
            tools::add(p, Chest, [0.0, 0.0, 5.0 * pulse]);
        }
        UndertowHerald => {
            hold(k, m, p, plan);
            tools::add(p, Head, [-8.0 * pulse, 0.0, 0.0]);
        }
        TrenchLamplighter => {
            tools::set(p, UpperArmR, [-100.0, 0.0, -10.0]);
            tools::set(p, Head, [-15.0, 10.0 * pulse, 0.0]);
        }
        FoamWeaver => {
            tools::set(p, UpperArmL, [-70.0, 0.0, 20.0]);
            tools::set(p, UpperArmR, [-70.0, 0.0, -20.0]);
            tools::add(p, Chest, [0.0, 8.0 * pulse, 0.0]);
        }
        _ => {
            tools::set(p, UpperArmR, [-90.0 + 20.0 * pulse, 0.0, -15.0]);
            tools::set(p, Head, [0.0, 0.0, 10.0 * pulse]);
        }
    }
    tools::plant(p, plan, tools::stance(plan, 1.15, 0.02));
}
