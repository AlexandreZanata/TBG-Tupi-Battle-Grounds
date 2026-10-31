//! Archer bow checks on the posed skeleton: every limb meets its string pivot,
//! the string joins the tips at rest and reaches the nock at full draw, the
//! bow stands upright while idle and shoots level, and the arrow is nocked
//! only until Release.

use bevy::prelude::*;

use super::{model, pose, KIT, RELEASE};
use crate::art::{
    anim::{Clip, Pose},
    ik,
    kits::{
        common::{gear, Form},
        tools, Attack, Moment,
    },
    rig::{Joint, Node, Socket, NODE_COUNT},
};

/// The five bow forms: base, Longbow, Marksman, Volley, Arrow Rain.
const FORMS: [(u8, u8); 5] = [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)];
/// How far a limb may stop short of its string pivot, or the string of its nock.
const GAP: f32 = 0.03;

fn posed(clip: Clip, t: f32, tier: u8, branch: u8) -> Pose {
    let mut p = [(Vec3::ZERO, Vec3::ZERO); NODE_COUNT];
    let m = Moment {
        clip,
        t,
        tier,
        branch,
        aim: None,
        since_attack: 0.0,
        variant: 0,
        clock: 0.0,
        walk: None,
        gesture: 1e6,
        growth: 0.0,
    };
    pose(&m, &mut p);
    p
}

/// Where a string half's far end sits: its pivot plus its length along the
/// turned axis (`down` -1 for the top half, which hangs below its pivot).
fn far_end(p: &Pose, node: Socket, half: f32, down: f32) -> Vec3 {
    let (rot, pivot) = p[Node::Prop(node).index()];
    pivot + ik::quat(rot) * Vec3::Y * down * gear::string_half(half)
}

#[test]
fn every_limb_meets_its_string_pivot() {
    for (tier, branch) in FORMS {
        let half = model::half_len(Form::of(tier, branch));
        let p = posed(Clip::Idle, 0.0, tier, branch);
        for (sign, node) in [(1.0, Socket::StringTop), (-1.0, Socket::StringBottom)] {
            let limb = gear::limb_points(half, sign);
            let end = limb[limb.len() - 1];
            let pivot = p[Node::Prop(node).index()].1;
            assert!(
                end.distance(pivot) <= GAP,
                "form {tier}/{branch}: limb ends at {end:?}, string pivot at {pivot:?}"
            );
        }
    }
}

#[test]
fn the_string_joins_the_tips_at_rest_and_reaches_the_nock_at_full_draw() {
    for (tier, branch) in FORMS {
        let half = model::half_len(Form::of(tier, branch));
        let rest = posed(Clip::Idle, 0.0, tier, branch);
        // Both halves cover the middle of the bow, so the string is one line.
        let top = far_end(&rest, Socket::StringTop, half, -1.0);
        let bottom = far_end(&rest, Socket::StringBottom, half, 1.0);
        assert!(
            top.y <= 0.0 && bottom.y >= 0.0,
            "form {tier}/{branch} rest: {top:?} {bottom:?}"
        );

        let drawn = posed(Clip::AttackRanged, 0.12, tier, branch);
        let nock = drawn[Node::Prop(Socket::Nock).index()].1;
        for end in [
            far_end(&drawn, Socket::StringTop, half, -1.0),
            far_end(&drawn, Socket::StringBottom, half, 1.0),
        ] {
            assert!(
                end.distance(nock) <= GAP,
                "form {tier}/{branch} draw: string ends at {end:?}, nock at {nock:?}"
            );
        }
    }
}

#[test]
fn the_bow_stands_upright_when_idle_and_shoots_level() {
    for (tier, branch) in FORMS {
        let p = model::plan(tier);
        let chest_to_hand = [
            Joint::Hips,
            Joint::Spine,
            Joint::Chest,
            Joint::UpperArmL,
            Joint::ForearmL,
            Joint::HandL,
        ];
        let hand = tools::fk(&posed(Clip::Idle, 0.0, tier, branch), &p, &chest_to_hand);
        let bow_of = |pose: &Pose| {
            let (rot, _) = pose[Node::Prop(Socket::OffhandL).index()];
            hand * Mat4::from_quat(ik::quat(rot))
        };
        let idle = posed(Clip::Idle, 0.0, tier, branch);
        let long = bow_of(&idle).transform_vector3(Vec3::Y).normalize();
        assert!(
            long.dot(Vec3::Y) > 0.99,
            "form {tier}/{branch} idle: long axis {long:?}"
        );

        // No target: the shot leaves level in the figure's own frame.
        let shot = posed(Clip::AttackRanged, 0.12, tier, branch);
        let hand = tools::fk(&shot, &p, &chest_to_hand);
        let aim = (hand * Mat4::from_quat(ik::quat(shot[Node::Prop(Socket::OffhandL).index()].0)))
            .transform_vector3(Vec3::Z)
            .normalize();
        assert!(aim.y.abs() < 0.01, "form {tier}/{branch} shot: {aim:?}");
    }
}

#[test]
fn the_arrow_is_nocked_only_until_release() {
    let Attack::Shot {
        loaded: Some((node, shown)),
        ..
    } = KIT.attack
    else {
        panic!("the archer shoots from a nocked arrow");
    };
    assert_eq!(node.index(), Node::Prop(Socket::Nock).index());
    assert!(shown(Clip::AttackRanged, 0.05));
    assert!(!shown(Clip::AttackRanged, RELEASE));
    assert!(!shown(Clip::Idle, 0.0));
}
