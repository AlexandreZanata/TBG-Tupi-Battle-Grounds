"""Contract: trident thrust timing and behavior hit alignment."""

from __future__ import annotations

import json
from pathlib import Path

from tools.blender.animation.attack import (
    frame_count_for_profile,
    impact_time_ms_for_profile,
    sample_pose_at_frame,
    sample_pose_at_ms,
)
from tools.blender.animation.attack.constants import HIT_MS, RECOVERY_MS, TOTAL_MS, WINDUP_MS
from tools.blender.animation.attack.timing import ms_to_frame

REPO_ROOT = Path(__file__).resolve().parents[3]
RANGED_BEHAVIOR = json.loads(
    (REPO_ROOT / "content/characters/behaviors/ranged_shot_v1.json").read_text(encoding="utf-8")
)
PROFILE = "scout_trident_thrust_v1"


def _peak_impact_frame(profile_id: str) -> int:
    guard = sample_pose_at_ms(profile_id, 0)
    best_ms = 0
    best_delta = 0.0
    for ms in range(0, TOTAL_MS + 50, 5):
        pose = sample_pose_at_ms(profile_id, ms)
        delta = abs(pose["TBG_LowerArm.R"][0] - guard["TBG_LowerArm.R"][0])
        if delta > best_delta:
            best_delta = delta
            best_ms = ms
    return ms_to_frame(best_ms)


def test_total_duration_matches_behavior_contract():
    assert WINDUP_MS == RANGED_BEHAVIOR["timings"]["attackWindupMs"]
    assert HIT_MS == 450
    assert RECOVERY_MS == 300
    assert TOTAL_MS == 1050
    assert frame_count_for_profile(PROFILE) == 32


def test_impact_aligns_with_behavior_attack_hit_ms():
    behavior_hit = RANGED_BEHAVIOR["timings"]["attackHitMs"]
    assert impact_time_ms_for_profile(PROFILE) == behavior_hit
    peak_frame = _peak_impact_frame(PROFILE)
    assert abs(peak_frame - ms_to_frame(behavior_hit)) <= 1


def test_elbow_near_extension_at_impact():
    impact = sample_pose_at_ms(PROFILE, impact_time_ms_for_profile(PROFILE))
    windup = sample_pose_at_ms(PROFILE, WINDUP_MS - 1)
    assert impact["TBG_LowerArm.R"][0] < windup["TBG_LowerArm.R"][0] - 0.5


def test_windup_coils_before_forward_drive():
    guard = sample_pose_at_ms(PROFILE, 0)
    windup = sample_pose_at_ms(PROFILE, WINDUP_MS - 1)
    assert windup["TBG_LowerArm.R"][0] > guard["TBG_LowerArm.R"][0]
    assert windup["TBG_Spine"][0] < guard["TBG_Spine"][0]


def test_lunge_advances_root_forward():
    peak = sample_pose_at_ms(PROFILE, impact_time_ms_for_profile(PROFILE))
    guard = sample_pose_at_ms(PROFILE, 0)
    assert peak["TBG_Root"][2] > guard["TBG_Root"][2]


def test_spine_has_no_yaw():
    frames = frame_count_for_profile(PROFILE)
    for frame in (0, frames // 4, frames // 2, frames):
        spine = sample_pose_at_frame(PROFILE, frame)["TBG_Spine"]
        assert abs(spine[1]) < 1e-6
        assert abs(spine[2]) < 1e-6


def test_hand_extends_on_thrust():
    impact = sample_pose_at_ms(PROFILE, impact_time_ms_for_profile(PROFILE))
    guard = sample_pose_at_ms(PROFILE, 0)
    assert impact["TBG_LowerArm.R"][0] < guard["TBG_LowerArm.R"][0] - 0.3


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
