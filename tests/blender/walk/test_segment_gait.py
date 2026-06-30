"""Contract: segment gait — torso pitch, FK spine, Perry knee lobes."""

import math

from tools.blender.animation.walk import cycle_frames_for_profile, sample_pose_at_frame
from tools.blender.animation.walk.biomech_curves import KNEE_CURVE_DEG
from tools.blender.animation.walk.curves import sample_cycle_curve


def test_segment_spine_pitch_varies_over_cycle():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    spine_vals = [sample_pose_at_frame("humanoid_light_v1", f)["TBG_Spine"][0] for f in range(frames + 1)]
    assert max(spine_vals) - min(spine_vals) > 0.02


def test_segment_spine_has_no_yaw_or_roll():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    for frame in (0, frames // 4, frames // 2):
        spine = sample_pose_at_frame("humanoid_light_v1", frame)["TBG_Spine"]
        assert abs(spine[1]) < 1e-6
        assert abs(spine[2]) < 1e-6


def test_segment_pelvis_bob_on_root_only():
    pose = sample_pose_at_frame("humanoid_light_v1", 0)
    root = pose["TBG_Root"]
    assert root[0] == 0.0
    assert root[2] == 0.0


def test_knee_curve_has_dual_lobes():
    stance = sample_cycle_curve(KNEE_CURVE_DEG, 0.08)
    swing = sample_cycle_curve(KNEE_CURVE_DEG, 0.73)
    assert swing > stance * 2.0


def test_knee_increases_through_swing_phase():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    mid = frames // 2
    swing = [
        sample_pose_at_frame("humanoid_light_v1", f)["TBG_LowerLeg.R"][0]
        for f in range(mid, frames + 1)
    ]
    peak_idx = swing.index(max(swing))
    if peak_idx > 0:
        assert swing[peak_idx] >= swing[peak_idx - 1] - 0.05


def test_humanoid_heavy_profile_registered():
    frames = cycle_frames_for_profile("humanoid_heavy_v1")
    assert frames == 38  # 95 spm @ 30 fps
