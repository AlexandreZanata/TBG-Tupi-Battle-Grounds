//! Authored action, life and signature celebration channels for the completion champions.
use super::super::{
    common::moves::{self, *},
    tools, Moment,
};
mod attacks;
use attacks::*;
mod signatures;
mod victory;
use super::{forms, plan};
use crate::art::{
    anim::{Clip, Pose},
    ik,
    rig::{Joint::*, Node::Secondary, Spring},
};
use bevy::prelude::*;
use tbg_core::UnitKind::{self, *};
pub fn pose(kind: UnitKind, m: &Moment, p: &mut Pose) {
    if kind == Hushfang {
        super::hushfang::pose(m, p);
        return;
    }
    let body = plan(kind, m.tier);
    match m.clip {
        Clip::Idle => moves::idle(m, p, &body, weight(kind)),
        Clip::Walk => {
            moves::walk(m, p, 3.0, 2.0);
            tools::add(p, Head, [0.0, 0.0, 0.0]);
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => attack(kind, m, p, &body),
        Clip::Hit => {
            moves::hit(m, p, weight(kind));
            tools::add(p, Head, [-4.0, 0.0, 0.0]);
        }
        Clip::Death => death(kind, m, p, &body),
        Clip::Spawn => moves::spawn(
            m,
            p,
            &body,
            if matches!(kind, Gravecutter | LastlightCantor) {
                Entry::Rise
            } else {
                Entry::March
            },
        ),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            tools::add(
                p,
                Head,
                [-8.0 * tools::hold(m.t, [0.0, 0.2, 0.4, 0.9]), 0.0, 0.0],
            );
        }
        Clip::Victory => victory(kind, m, p, &body),
    }
    if !matches!(
        m.clip,
        Clip::Death | Clip::AttackMelee | Clip::AttackRanged | Clip::Cast | Clip::Victory
    ) {
        rest_grip(kind, m, p, &body);
    }
    if matches!(kind, Starbreaker | Sunlancer) && m.clip != Clip::Death && m.clip != Clip::Victory {
        shaft(kind, m, p, &body);
    }
    signatures::apply(kind, m, p);
    special(kind, m, p);
    if super::is_insect(kind) && m.clip != Clip::Death {
        aux_stance(p, &body);
    }
}
fn weight(k: UnitKind) -> f32 {
    match k {
        Starbreaker | Cragclaw => 0.85,
        MireTender | Blastbeetle => 0.65,
        _ => 0.3,
    }
}
fn rest_grip(k: UnitKind, m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    match k {
        MireTender | NectarNurse => {
            tools::reach(
                p,
                true,
                Vec3::new(0.14, 0.13, 0.22),
                Some(Quat::IDENTITY),
                (b, Vec3::new(0.2, -0.8, 0.25)),
            );
            tools::reach(
                p,
                false,
                Vec3::new(-0.13, 0.20, 0.23),
                Some(Quat::from_rotation_x(-0.3)),
                (b, Vec3::new(-0.2, -0.8, 0.25)),
            );
        }
        Dawnmender => lantern(m, p, b, 0.17, 0.25),
        Starbreaker | Sunlancer => shaft(k, m, p, b),
        Spinehurler => {
            tools::reach(
                p,
                false,
                Vec3::new(-0.15, 0.15, 0.12),
                Some(Quat::from_rotation_x(-1.25)),
                (b, Vec3::new(-0.2, -0.8, 0.25)),
            );
        }
        Prismancer | Needlewing => {
            tools::reach(
                p,
                false,
                Vec3::new(-0.13, 0.19, 0.18),
                Some(Quat::IDENTITY),
                (b, Vec3::new(-0.2, -0.8, 0.25)),
            );
            tools::reach(
                p,
                true,
                Vec3::new(0.04, 0.19, 0.23),
                Some(Quat::IDENTITY),
                (b, Vec3::new(0.2, -0.8, 0.25)),
            );
        }
        Gravecutter => {
            tools::reach(
                p,
                false,
                Vec3::new(-0.12, 0.16, 0.24),
                Some(Quat::from_rotation_y(70.0f32.to_radians())),
                (b, Vec3::new(-0.2, -0.8, 0.25)),
            );
            tools::reach(
                p,
                true,
                Vec3::new(0.12, 0.13, 0.12),
                Some(Quat::IDENTITY),
                (b, Vec3::new(0.2, -0.8, 0.25)),
            );
        }
        Leafcutter => tools::reach(
            p,
            false,
            Vec3::new(-0.12, 0.13, 0.14),
            Some(Quat::IDENTITY),
            (b, Vec3::new(-0.2, -0.8, 0.25)),
        ),
        LastlightCantor => {
            tools::reach(
                p,
                false,
                Vec3::new(-0.12, 0.12, 0.18),
                Some(Quat::IDENTITY),
                (b, Vec3::new(-0.2, -0.8, 0.25)),
            );
            tools::reach(
                p,
                true,
                Vec3::new(0.12, 0.15, 0.13),
                Some(Quat::IDENTITY),
                (b, Vec3::new(0.2, -0.8, 0.25)),
            );
        }
        Cragclaw => {
            tools::reach(
                p,
                false,
                Vec3::new(-0.10, 0.10, 0.20),
                Some(Quat::IDENTITY),
                (b, Vec3::NEG_Y),
            );
            tools::set(p, UpperArmL, [-15.0, 0.0, 4.0]);
        }
        EclipseDancer => {
            tools::reach(
                p,
                false,
                Vec3::new(-0.10, 0.15, 0.16),
                Some(Quat::from_rotation_x(-0.70)),
                (b, Vec3::new(-0.2, -0.8, 0.25)),
            );
            tools::reach(
                p,
                true,
                Vec3::new(0.10, 0.15, 0.18),
                Some(Quat::from_rotation_x(-0.65)),
                (b, Vec3::new(0.2, -0.8, 0.25)),
            );
        }
        _ => {}
    }
}
fn attack(k: UnitKind, m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    let table = match k {
        Cragclaw | Starbreaker => CHOP,
        Gravecutter | EclipseDancer | Leafcutter => SLASH,
        Sunlancer | Blastbeetle => THRUST,
        Spinehurler => THROW,
        Prismancer | Needlewing => POINT,
        _ => CAST,
    };
    moves::strike(m, p, b, table, 1.12);
    if k == EclipseDancer && m.variant & 1 == 0 {
        for (l, r) in [(UpperArmL, UpperArmR), (ForearmL, ForearmR), (HandL, HandR)] {
            p.swap(l.index(), r.index());
            p[l.index()].0.z *= -1.0;
            p[r.index()].0.z *= -1.0;
        }
    }

    let contact = tools::hold(m.t, [0.16, 0.30, 0.38, 0.62]);
    match k {
        Gravecutter => {
            tools::add(p, ForearmR, [0.0, -18.0 * contact, 0.0]);
            tools::add(p, HandR, [0.0, 35.0 * contact, 0.0]);
        }
        LastlightCantor => {
            tools::set(p, UpperArmL, [-30.0, 0.0, 20.0]);
            tools::set(p, ForearmL, [-50.0, 0.0, 0.0]);
            tools::add(p, HandR, [12.0 * contact, 0.0, 0.0]);
        }
        Cragclaw => {
            tools::set(p, UpperArmL, [-20.0, 0.0, 15.0]);
            tools::set(p, ForearmL, [-20.0, 0.0, 0.0]);
        }
        EclipseDancer => {
            tools::add(p, UpperArmL, [-75.0 * contact, 0.0, 20.0 * contact]);
            tools::add(p, HandL, [0.0, -35.0 * contact, 0.0]);
        }
        Starbreaker => tools::add(p, Chest, [12.0 * contact, 0.0, 0.0]),
        MireTender | NectarNurse => {
            tools::reach(
                p,
                true,
                Vec3::new(0.14, 0.13, 0.22),
                Some(Quat::IDENTITY),
                (b, Vec3::X),
            );
        }
        Dawnmender => lantern(m, p, b, 0.19, 0.22 + 0.09 * contact),
        Gorgehowl | Stridulator => {
            tools::add(p, Chest, [-8.0 * contact, 0.0, 0.0]);
            tools::set(p, UpperArmL, [-55.0 * contact, 0.0, 15.0]);
        }
        _ => {}
    }
}
fn special(k: UnitKind, m: &Moment, p: &mut Pose) {
    let Some(t) = m.special else {
        return;
    };
    let wind = if k == Gorgehowl { 0.10 } else { 0.30 };
    let s = tools::hold(t, [0.0, wind * 0.8, wind + 0.04, wind + 0.3]);
    match k {
        Gorgehowl => {
            p[Secondary(Spring::HeadGear).index()].0.x = -18.0 * s;
            tools::add(p, Chest, [-12.0 * s, 0.0, 0.0]);
            tools::add(p, Head, [-8.0 * s, 0.0, 0.0]);
        }
        Prismancer => {
            tools::set(p, UpperArmR, [-82.0 * s, 0.0, -5.0]);
            tools::set(p, ForearmR, [-15.0 * s, 0.0, 0.0]);
        }
        Stridulator => {
            tools::add(p, UpperArmL, [-20.0 * s, 0.0, 35.0 * s]);
            tools::add(p, UpperArmR, [-20.0 * s, 0.0, -35.0 * s]);
            tools::add(p, Chest, [-6.0 * s, 0.0, 0.0]);
        }
        _ => {}
    }
}
fn death(k: UnitKind, m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    let fall = match k {
        Gravecutter | Starbreaker => Fall::Kneel,
        Cragclaw | Leafcutter | Needlewing => Fall::Forward,
        Blastbeetle => Fall::Back,
        _ => Fall::Kneel,
    };
    moves::death(m, p, b, fall);
    let d = tools::window(m.t, 0.12, 0.8);
    match k {
        Gravecutter => tools::set(p, HandR, [0.0, 0.0, -65.0 * d]),
        LastlightCantor => tools::set(p, HandR, [50.0 * d, 0.0, 0.0]),
        Cragclaw => tools::set(p, ForearmR, [-35.0 * d, 0.0, 0.0]),
        Spinehurler => tools::set(p, Head, [24.0 * d, 0.0, 15.0 * d]),
        MireTender => tools::set(p, ForearmL, [-70.0 * d, 0.0, 0.0]),
        Gorgehowl => {
            p[Secondary(Spring::HeadGear).index()].0.x = 25.0 * d;
            tools::set(p, Head, [35.0 * d, 0.0, 0.0]);
        }
        Sunlancer => tools::set(p, HandR, [0.0, 35.0 * d, 0.0]),
        EclipseDancer => {
            tools::set(p, UpperArmL, [15.0 * d, 0.0, 20.0 * d]);
            tools::set(p, HandR, [0.0, 0.0, -40.0 * d]);
        }
        Starbreaker => tools::set(p, ForearmR, [-20.0 * d, 0.0, 0.0]),
        Dawnmender => tools::set(p, ForearmL, [-55.0 * d, 0.0, 0.0]),
        Prismancer => tools::set(p, UpperArmR, [-20.0 * d, 0.0, -10.0 * d]),
        Leafcutter => tools::set(p, Head, [10.0 * d, -25.0 * d, 0.0]),
        Blastbeetle => tools::set(p, Head, [-25.0 * d, 0.0, 0.0]),
        Needlewing => tools::set(p, Chest, [20.0 * d, 0.0, 12.0 * d]),
        NectarNurse => tools::set(p, ForearmL, [-60.0 * d, 0.0, 0.0]),
        Stridulator => tools::set(p, UpperArmR, [0.0, 0.0, -25.0 * d]),
        _ => {}
    }
    if super::is_insect(k) {
        for n in [Secondary(Spring::Back), Secondary(Spring::Tabard)] {
            p[n.index()].0.x = -70.0 * d;
        }
    }
}
fn victory(k: UnitKind, m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    p.fill((Vec3::ZERO, Vec3::ZERO));
    rest_grip(k, m, p, b);
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    tools::shift(p, Hips, [0.012 * s, -0.015, 0.0]);
    tools::plant(p, b, tools::stance(b, 1.1, 0.0));
    victory::apply(k, m, p, b);
}

pub fn aux_stance(p: &mut Pose, b: &crate::art::rig::Proportions) {
    let hips = tools::fk(p, b, &[Root, Hips]);
    for (u, l, foot) in [
        (
            Secondary(Spring::Back),
            crate::art::rig::Node::Prop(crate::art::rig::Socket::StringTop),
            crate::art::rig::Node::Prop(crate::art::rig::Socket::Back),
        ),
        (
            Secondary(Spring::Tabard),
            crate::art::rig::Node::Prop(crate::art::rig::Socket::StringBottom),
            crate::art::rig::Node::Prop(crate::art::rig::Socket::Nock),
        ),
    ] {
        let start = forms::insect_rest(u, b);
        let target = hips
            .inverse()
            .transform_point3(Vec3::new(start.x * 1.4, 0.05, -0.10));
        let (a, c) = ik::two_bone(start, target, b.leg * 0.55, b.leg * 0.55, Vec3::NEG_Z);
        p[u.index()].0 = ik::degrees(a);
        p[l.index()].0 = ik::degrees(c);
        let hiprot = ik::quat(p[Hips.index()].0);
        p[foot.index()].0 = ik::degrees((hiprot * a * c).inverse());
    }
}

pub fn channel_pose(m: &Moment, p: &mut Pose) {
    let b = plan(NectarNurse, m.tier);
    let pulse = (m.clock * 5.0).sin() * 0.007;
    tools::reach(
        p,
        true,
        Vec3::new(0.14, 0.13, 0.22),
        Some(Quat::IDENTITY),
        (&b, Vec3::new(0.2, -0.8, 0.25)),
    );
    tools::reach(
        p,
        false,
        Vec3::new(-0.08, 0.20 + pulse, 0.29),
        Some(Quat::IDENTITY),
        (&b, Vec3::new(-0.2, -0.8, 0.25)),
    );
}

fn lantern_grip(m: &Moment) -> f32 {
    if m.tier > 0 && m.branch == 0 {
        0.17
    } else {
        0.14
    }
}
fn lantern(m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions, y: f32, z: f32) {
    let x = lantern_grip(m);
    tools::reach(
        p,
        false,
        Vec3::new(-x, y, z),
        Some(Quat::IDENTITY),
        (b, Vec3::new(-0.2, -0.8, 0.25)),
    );
    tools::reach(
        p,
        true,
        Vec3::new(x, y, z),
        Some(Quat::IDENTITY),
        (b, Vec3::new(0.2, -0.8, 0.25)),
    );
}
fn shaft(k: UnitKind, m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    let active = matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast);
    let load = if active {
        tools::hold(m.t, [0.0, 0.18, 0.20, 0.30])
    } else {
        0.0
    };
    let hit = if active {
        tools::hold(m.t, [0.20, 0.30, 0.38, 0.62])
    } else {
        0.0
    };
    let angle = if k == Starbreaker {
        -2.20 * load - 0.15 * hit
    } else {
        -1.05 + 0.90 * hit
    };
    let rotation = Quat::from_rotation_x(angle);
    let hand = if k == Starbreaker {
        Vec3::new(
            0.0,
            0.17 + 0.10 * load + 0.02 * hit,
            0.10 - 0.03 * load + 0.05 * hit,
        )
    } else {
        Vec3::new(
            0.0,
            0.12 + 0.15 * load + 0.02 * hit,
            0.13 - 0.03 * load + 0.02 * hit,
        )
    };
    let grip = if k == Starbreaker {
        0.13 - 0.03 * hit
    } else {
        0.20 - 0.10 * hit
    };
    shaft_pose(p, b, hand, rotation, grip);
}
fn shaft_pose(
    p: &mut Pose,
    b: &crate::art::rig::Proportions,
    hand: Vec3,
    rotation: Quat,
    grip: f32,
) {
    tools::reach(
        p,
        false,
        hand,
        Some(rotation),
        (b, Vec3::new(-0.2, -0.8, 0.25)),
    );
    tools::reach(
        p,
        true,
        hand + rotation * Vec3::Z * grip,
        Some(rotation),
        (b, Vec3::new(0.2, -0.8, 0.25)),
    );
}
