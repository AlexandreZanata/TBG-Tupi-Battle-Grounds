"""Contract: spine stays upright; legs mirror for sagittal plane."""

import math

from tools.blender.animation.walk import cycle_frames_for_profile, sample_pose_at_frame


def test_spine_has_no_yaw_or_roll():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    for frame in (0, frames // 4, frames // 2):
        spine = sample_pose_at_frame("humanoid_light_v1", frame)["TBG_Spine"]
        assert abs(spine[1]) < 1e-6, "yaw tilts character diagonally"
        assert abs(spine[2]) < 1e-6, "roll tilts character sideways"


def test_legs_mirror_at_opposite_contacts():
    pose = sample_pose_at_frame("humanoid_light_v1", 0)
    r_hip = pose["TBG_UpperLeg.R"][0]
    l_hip = pose["TBG_UpperLeg.L"][0]
    assert r_hip > 0.12
    assert l_hip < -0.12
    assert abs(r_hip + l_hip) < 0.05


def test_opposite_contact_legs_alternate():
    half = cycle_frames_for_profile("humanoid_light_v1") // 2
    mid = sample_pose_at_frame("humanoid_light_v1", half)
    assert mid["TBG_UpperLeg.L"][0] > 0.12
    assert mid["TBG_UpperLeg.R"][0] < -0.12


def test_feet_height_balanced_at_double_support():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    pose = sample_pose_at_frame("humanoid_light_v1", 0)
    r_knee = pose["TBG_LowerLeg.R"][0]
    l_knee = pose["TBG_LowerLeg.L"][0]
    assert abs(r_knee - l_knee) < 0.15
