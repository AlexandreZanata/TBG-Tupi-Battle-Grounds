"""Contract: parametric walk alternates legs and loops."""

import math

from tools.blender.animation.walk import (
    WALK_FRAMES,
    UNIT_WALK_PROFILE,
    cycle_frames_for_profile,
    resolve_walk_profile,
    sample_pose_at_frame,
)


def test_scout_pilot_style_registered():
    from tools.blender.animation.walk.profiles import STYLE_PROFILES

    assert "scout_pilot_v1" in STYLE_PROFILES


def test_resolve_body_class_profile():
    assert resolve_walk_profile(None, "HumanoidLight") == "humanoid_light_v1"
    assert resolve_walk_profile(None, "HumanoidHeavy") == "humanoid_heavy_v1"


def test_gait_alternates_legs_at_contact_points():
    half = cycle_frames_for_profile("scout_pilot_v1") // 2
    r_contact = sample_pose_at_frame("scout_pilot_v1", 0)
    l_contact = sample_pose_at_frame("scout_pilot_v1", half)
    assert r_contact["TBG_UpperLeg.R"][0] > r_contact["TBG_UpperLeg.L"][0]
    assert l_contact["TBG_UpperLeg.L"][0] > l_contact["TBG_UpperLeg.R"][0]


def test_gait_loops_at_frame_zero_and_end():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    start = sample_pose_at_frame("humanoid_light_v1", 0)
    end = sample_pose_at_frame("humanoid_light_v1", frames)
    assert start["TBG_UpperLeg.R"] == end["TBG_UpperLeg.R"]

def test_humanoid_light_cadence_115_spm():
    assert cycle_frames_for_profile("humanoid_light_v1") == 31
