"""Contract: FK legs use X-axis only; frame deltas stay smooth."""

from tools.blender.animation.walk import WALK_FRAMES, sample_pose_at_frame

LEG_BONES = (
    "TBG_UpperLeg.R",
    "TBG_LowerLeg.R",
    "TBG_Foot.R",
    "TBG_UpperLeg.L",
    "TBG_LowerLeg.L",
    "TBG_Foot.L",
)


def test_legs_rotate_on_x_axis_only():
    pose = sample_pose_at_frame("humanoid_light_v1", 11)
    for bone in LEG_BONES:
        _, y, z = pose[bone]
        assert abs(y) < 1e-6
        assert abs(z) < 1e-6


def test_frame_to_frame_leg_motion_is_smooth():
    prev = sample_pose_at_frame("humanoid_light_v1", 0)["TBG_UpperLeg.R"][0]
    for frame in range(1, WALK_FRAMES + 1):
        cur = sample_pose_at_frame("humanoid_light_v1", frame)["TBG_UpperLeg.R"][0]
        assert abs(cur - prev) < 0.07
        prev = cur
