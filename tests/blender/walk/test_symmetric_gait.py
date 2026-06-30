"""Contract: held-arm walk — upright torso, mirrored legs, frozen weapon arm."""

from tools.blender.animation.walk import cycle_frames_for_profile, sample_pose_at_frame
from tools.blender.animation.shared.trident_guard import TRIDENT_GUARD_ARM


def test_spine_has_no_yaw_or_roll():
    frames = cycle_frames_for_profile("scout_pilot_v1")
    for frame in (0, frames // 4, frames // 2):
        spine = sample_pose_at_frame("scout_pilot_v1", frame)["TBG_Spine"]
        assert abs(spine[1]) < 1e-6
        assert abs(spine[2]) < 1e-6


def test_legs_mirror_at_same_instant():
    pose = sample_pose_at_frame("scout_pilot_v1", 0)
    r_hip = pose["TBG_UpperLeg.R"][0]
    l_hip = pose["TBG_UpperLeg.L"][0]
    assert r_hip > 0.08
    assert l_hip < -0.08
    assert abs(r_hip + l_hip) < 0.05


def test_opposite_contact_legs_alternate():
    half = cycle_frames_for_profile("scout_pilot_v1") // 2
    mid = sample_pose_at_frame("scout_pilot_v1", half)
    assert mid["TBG_UpperLeg.L"][0] > 0.08
    assert mid["TBG_UpperLeg.R"][0] < -0.08


def test_weapon_arm_locked_during_walk():
    frames = cycle_frames_for_profile("scout_pilot_v1")
    guard = TRIDENT_GUARD_ARM["TBG_UpperArm.R"]
    for frame in (0, frames // 4, frames // 2):
        pose = sample_pose_at_frame("scout_pilot_v1", frame)
        assert pose["TBG_UpperArm.R"] == guard
        assert pose["TBG_LowerArm.R"] == TRIDENT_GUARD_ARM["TBG_LowerArm.R"]
