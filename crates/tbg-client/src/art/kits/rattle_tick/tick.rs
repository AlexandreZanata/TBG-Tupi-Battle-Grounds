//! Tick aims and recoils on his shoulder rig without steering Rattle's Root.
use super::*;
use crate::art::{ik, presentation::pose_world, rig::Proportions};

pub(super) fn pose(m: &Moment, p: &mut Pose, body: &Proportions) {
    if m.clip == Clip::Death {
        return;
    }
    p[Prop(Socket::Back).index()].0 = Vec3::new(-65.0, 0.0, 0.0);
    p[Prop(Socket::StringTop).index()].0 = Vec3::new(-82.0, 0.0, 0.0);
    p[Secondary(Spring::HeadGear).index()].0 = Vec3::ZERO;
    let recoil = m
        .special
        .map_or(0.0, |t| tools::hold(t, [0.29, 0.30, 0.35, 0.47]));
    p[Prop(Socket::Nock).index()].1.z = -0.035 * recoil;
    if let Some(target) = m.special_aim {
        let world = pose_world(tbg_core::UnitKind::RattleTick, body, p);
        let local = world[Chest.index()].inverse().transform_point3(target)
            - model::rest(Secondary(Spring::Back), body);
        let yaw = local.x.atan2(local.z);
        let pitch = local.y.atan2(local.xz().length());
        let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(-pitch);
        p[Secondary(Spring::Back).index()].0 = ik::degrees(rotation);
    }
}
