"""Contract: scout walk — segment gait + locked trident arm."""

from tools.blender.animation.walk import cycle_frames_for_profile, sample_pose_at_frame
from tools.blender.animation.shared.trident_guard import TRIDENT_GUARD_ARM


def test_spine_pitch_varies_with_segment_gait():
    frames = cycle_frames_for_profile("scout_pilot_v1")
    spine_vals = [sample_pose_at_frame("scout_pilot_v1", f)["TBG_Spine"][0] for f in range(frames + 1)]
    assert max(spine_vals) - min(spine_vals) > 0.02


def test_spine_has_no_yaw_or_roll():
    frames = cycle_frames_for_profile("scout_pilot_v1")
    for frame in (0, frames // 4, frames // 2):
        spine = sample_pose_at_frame("scout_pilot_v1", frame)["TBG_Spine"]
        assert abs(spine[1]) < 1e-6
        assert abs(spine[2]) < 1e-6


def test_legs_opposite_at_contact_frames():
    pose = sample_pose_at_frame("scout_pilot_v1", 0)
    r_hip = pose["TBG_UpperLeg.R"][0]
    l_hip = pose["TBG_UpperLeg.L"][0]
    assert r_hip > 0.08
    assert l_hip < 0.0


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
