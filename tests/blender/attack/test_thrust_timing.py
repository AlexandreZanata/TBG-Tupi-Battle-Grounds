"""Contract: trident thrust timing and kinetic-chain ordering."""

from tools.blender.animation.attack import (
    frame_count_for_profile,
    impact_time_ms_for_profile,
    sample_pose_at_frame,
    sample_pose_at_ms,
)
from tools.blender.animation.attack.constants import HIT_MS, RECOVERY_MS, TOTAL_MS, WINDUP_MS

PROFILE = "scout_trident_thrust_v1"


def test_total_duration_matches_behavior_contract():
    assert WINDUP_MS == 300
    assert HIT_MS == 450
    assert RECOVERY_MS == 300
    assert TOTAL_MS == 1050
    assert frame_count_for_profile(PROFILE) == 32


def test_impact_near_end_of_hit_phase():
    assert impact_time_ms_for_profile(PROFILE) == 300 + round(450 * 0.85)
    impact = sample_pose_at_ms(PROFILE, impact_time_ms_for_profile(PROFILE))
    windup = sample_pose_at_ms(PROFILE, WINDUP_MS - 1)
    assert impact["TBG_LowerArm.R"][0] < windup["TBG_LowerArm.R"][0] - 0.15


def test_elbow_near_extension_at_impact():
    impact = sample_pose_at_ms(PROFILE, impact_time_ms_for_profile(PROFILE))
    windup = sample_pose_at_ms(PROFILE, WINDUP_MS - 1)
    assert impact["TBG_LowerArm.R"][0] < 0.35
    assert impact["TBG_LowerArm.R"][0] < windup["TBG_LowerArm.R"][0] - 0.7

def test_windup_coils_before_forward_drive():
    guard = sample_pose_at_ms(PROFILE, 0)
    windup = sample_pose_at_ms(PROFILE, WINDUP_MS - 1)
    assert windup["TBG_LowerArm.R"][0] > guard["TBG_LowerArm.R"][0]
    assert windup["TBG_Spine"][0] < guard["TBG_Spine"][0]


def test_lunge_advances_root_forward():
    peak = sample_pose_at_ms(PROFILE, WINDUP_MS + HIT_MS - 1)
    guard = sample_pose_at_ms(PROFILE, 0)
    assert peak["TBG_Root"][2] > guard["TBG_Root"][2] + 0.02


def test_spine_has_no_yaw():
    frames = frame_count_for_profile(PROFILE)
    for frame in (0, frames // 4, frames // 2, frames):
        spine = sample_pose_at_frame(PROFILE, frame)["TBG_Spine"]
        assert abs(spine[1]) < 1e-6
        assert abs(spine[2]) < 1e-6


def test_hand_extends_on_thrust():
    impact = sample_pose_at_ms(PROFILE, impact_time_ms_for_profile(PROFILE))
    guard = sample_pose_at_ms(PROFILE, 0)
    assert impact["TBG_LowerArm.R"][0] < guard["TBG_LowerArm.R"][0] - 0.5
    assert impact["TBG_Hand.R"][0] < guard["TBG_Hand.R"][0] - 0.05


def test_recovery_relaxes_from_peak():
    peak = sample_pose_at_frame(PROFILE, frame_count_for_profile(PROFILE) // 2)
    end = sample_pose_at_frame(PROFILE, frame_count_for_profile(PROFILE))
    guard = sample_pose_at_frame(PROFILE, 0)
    peak_elbow = abs(peak["TBG_LowerArm.R"][0] - guard["TBG_LowerArm.R"][0])
    end_elbow = abs(end["TBG_LowerArm.R"][0] - guard["TBG_LowerArm.R"][0])
    assert end_elbow < peak_elbow


def test_spine_has_no_roll():
    frames = frame_count_for_profile(PROFILE)
    for frame in (0, frames // 4, frames // 2, frames):
        spine = sample_pose_at_frame(PROFILE, frame)["TBG_Spine"]
        assert abs(spine[2]) < 1e-6
