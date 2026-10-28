//! Archer base motion tables: braced bow arm, draw, anchor, 180 ms release, recovery.

use super::{
    anim::{Clip, Easing::*, Pose},
    clips::Key,
    curves::sample_track,
    rig::{
        Joint::*,
        Node,
        Node::{Body, Prop},
        Socket,
    },
};
type Track = (Node, &'static [Key]);
const Z: [f32; 3] = [0.0; 3];

#[rustfmt::skip]
const IDLE: &[Track] = &[
    (Body(UpperArmL), &[(0.0, [-25.0,0.0,-8.0],Z,CubicInOut),(1.0,[-25.0,0.0,-8.0],Z,Hold)]),
    (Body(ForearmL), &[(0.0, [-65.0,0.0,0.0],Z,Hold)]),
    (Prop(Socket::OffhandL), &[(0.0,[90.0,0.0,8.0],Z,Hold)]),
    (Body(UpperArmR), &[(0.0,[-8.0,0.0,-8.0],Z,CubicInOut),(0.5,[-12.0,0.0,-8.0],Z,CubicInOut),(1.0,[-8.0,0.0,-8.0],Z,Hold)]),
];
#[rustfmt::skip]
const WALK: &[Track] = &[
    (Body(UpperArmL), &[(0.0,[-25.0,0.0,-8.0],Z,CubicInOut),(0.5,[-30.0,0.0,-8.0],Z,CubicInOut),(1.0,[-25.0,0.0,-8.0],Z,Hold)]),
    (Body(ForearmL), &[(0.0,[-65.0,0.0,0.0],Z,Hold)]),
    (Prop(Socket::OffhandL), &[(0.0,[90.0,0.0,8.0],Z,Hold)]),
    (Body(UpperArmR), &[(0.0,[18.0,0.0,-8.0],Z,CubicInOut),(0.5,[-22.0,0.0,-8.0],Z,CubicInOut),(1.0,[18.0,0.0,-8.0],Z,Hold)]),
];
#[rustfmt::skip]
const RANGED: &[Track] = &[
    (Body(UpperArmL), &[(0.0,[-25.0,0.0,-8.0],Z,CubicOut),(0.24,[-87.0,0.0,-8.0],Z,Hold),
        (0.36,[-87.0,0.0,-8.0],Z,CubicOut),(0.48,[-82.0,0.0,-10.0],Z,CubicInOut),(1.0,[-25.0,0.0,-8.0],Z,Hold)]),
    (Body(ForearmL), &[(0.0,[-65.0,0.0,0.0],Z,CubicOut),(0.24,Z,Z,Hold),(0.64,Z,Z,CubicInOut),(1.0,[-65.0,0.0,0.0],Z,Hold)]),
    (Prop(Socket::OffhandL), &[(0.0,[90.0,0.0,8.0],Z,CubicOut),(0.24,[87.0,0.0,8.0],Z,Hold),
        (0.48,[82.0,0.0,10.0],Z,CubicInOut),(1.0,[90.0,0.0,8.0],Z,Hold)]),
    (Body(UpperArmR), &[(0.0,[-8.0,0.0,-8.0],Z,CubicIn),(0.24,[-80.0,-38.0,-12.0],Z,Hold),
        (0.36,[-80.0,-38.0,-12.0],Z,CubicOut),(0.48,[-64.0,-60.0,-28.0],Z,CubicInOut),(1.0,[-8.0,0.0,-8.0],Z,Hold)]),
    (Body(ForearmR), &[(0.0,Z,Z,CubicOut),(0.24,[-78.0,0.0,0.0],Z,Hold),(0.36,[-78.0,0.0,0.0],Z,CubicOut),
        (0.48,[-110.0,0.0,-15.0],Z,CubicInOut),(1.0,Z,Z,Hold)]),
    (Body(Chest), &[(0.0,Z,Z,CubicIn),(0.24,[0.0,-12.0,0.0],Z,Hold),(0.36,[0.0,-12.0,0.0],Z,CubicOut),
        (0.48,[-4.0,-17.0,0.0],Z,CubicInOut),(1.0,Z,Z,Hold)]),
    (Body(Head), &[(0.0,Z,Z,CubicInOut),(0.24,[0.0,12.0,0.0],Z,Hold),(0.64,[0.0,12.0,0.0],Z,CubicInOut),(1.0,Z,Z,Hold)]),
    (Body(HandR), &[(0.0,Z,Z,Hold),(0.36,Z,Z,Linear),(0.48,[0.0,0.0,-12.0],Z,CubicOut),(1.0,Z,Z,Hold)]),
];
pub fn apply(clip: Clip, seconds: f32, pose: &mut Pose) {
    let tracks = match clip {
        Clip::Idle => IDLE,
        Clip::Walk => WALK,
        Clip::AttackRanged => RANGED,
        _ => return,
    };
    let raw = seconds / clip.duration();
    let k = if clip.looping() {
        raw.rem_euclid(1.0)
    } else {
        raw.clamp(0.0, 1.0)
    };
    for (node, keys) in tracks {
        pose[node.index()] = sample_track(keys, k);
    }
    if clip == Clip::AttackRanged {
        super::archer_aim::apply(seconds, pose);
    }
}
