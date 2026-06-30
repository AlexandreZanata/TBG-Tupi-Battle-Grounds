"""Contract: root +Z monotonic during stance — no backward slide."""

from tools.blender.animation.root_motion.contacts import stance_phase_frames
from tools.blender.animation.root_motion.stride import root_forward_z
from tools.blender.animation.walk import cycle_frames_for_profile, sample_pose_at_frame


def test_root_z_monotonic_during_stance_phase():
    profile = "humanoid_light_v1"
    frames = cycle_frames_for_profile(profile)
    style_stride = 1.4
    heel, toe_off = stance_phase_frames(frames)
    z_vals = [root_forward_z(f, frames, style_stride) for f in range(heel, toe_off + 1)]
    for idx in range(1, len(z_vals)):
        assert z_vals[idx] >= z_vals[idx - 1] - 1e-9


def test_root_motion_pose_advances_forward_z():
    pose = sample_pose_at_frame("humanoid_light_v1", 0, root_motion=False)
    pose_rm = sample_pose_at_frame("humanoid_light_v1", 0, root_motion=True)
    assert pose["TBG_Root"][2] == 0.0
    assert pose_rm["TBG_Root"][1] == pose["TBG_Root"][1]
    assert pose_rm["TBG_Root"][2] == 0.0

    mid = cycle_frames_for_profile("humanoid_light_v1") // 2
    forward = sample_pose_at_frame("humanoid_light_v1", mid, root_motion=True)
    assert forward["TBG_Root"][2] > 0.5
