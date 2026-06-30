"""Contract: heavy melee swing timing and behavior hit alignment."""

from __future__ import annotations

import json
from pathlib import Path

from tools.blender.animation.attack import (
    frame_count_for_profile,
    impact_time_ms_for_profile,
    sample_pose_at_frame,
    sample_pose_at_ms,
)
from tools.blender.animation.attack.profiles import TANK_MELEE_TIMING
from tools.blender.animation.attack.timing import ms_to_frame, within_frame_tolerance

REPO_ROOT = Path(__file__).resolve().parents[3]
TANK_BEHAVIOR = json.loads(
    (REPO_ROOT / "content/characters/behaviors/tank_melee_v1.json").read_text(encoding="utf-8")
)
PRIMARY = "heavy_melee_swing_v1"
ALTERNATE = "heavy_melee_swing_v2"


def _peak_impact_frame(profile_id: str) -> int:
    guard = sample_pose_at_ms(profile_id, 0)
    best_ms = 0
    best_delta = 0.0
    for ms in range(0, TANK_MELEE_TIMING.total_ms + 50, 5):
        pose = sample_pose_at_ms(profile_id, ms)
        delta = abs(pose["TBG_UpperArm.R"][0] - guard["TBG_UpperArm.R"][0])
        if delta > best_delta:
            best_delta = delta
            best_ms = ms
    return ms_to_frame(best_ms)


def test_total_duration_matches_tank_timing():
    assert TANK_MELEE_TIMING.windup_ms == TANK_BEHAVIOR["timings"]["attackWindupMs"]
    assert frame_count_for_profile(PRIMARY) == ms_to_frame(TANK_MELEE_TIMING.total_ms)
    assert frame_count_for_profile(ALTERNATE) == frame_count_for_profile(PRIMARY)


def test_impact_aligns_with_behavior_attack_hit_ms():
    behavior_hit = TANK_BEHAVIOR["timings"]["attackHitMs"]
    assert impact_time_ms_for_profile(PRIMARY) == behavior_hit
    peak_frame = _peak_impact_frame(PRIMARY)
    expected = ms_to_frame(behavior_hit)
    assert abs(peak_frame - expected) <= 1


def test_attack2_impact_within_tolerance():
    behavior_hit = TANK_BEHAVIOR["timings"]["attackHitMs"]
    peak_frame = _peak_impact_frame(ALTERNATE)
    expected = ms_to_frame(behavior_hit)
    assert abs(peak_frame - expected) <= 1


def test_attack2_differs_from_primary_at_mid_windup():
    mid = TANK_MELEE_TIMING.windup_ms // 2
    primary = sample_pose_at_ms(PRIMARY, mid)
    alternate = sample_pose_at_ms(ALTERNATE, mid)
    assert alternate["TBG_UpperArm.R"][0] != primary["TBG_UpperArm.R"][0]


def test_windup_coils_weapon_arm():
    guard = sample_pose_at_ms(PRIMARY, 0)
    windup = sample_pose_at_ms(PRIMARY, TANK_MELEE_TIMING.windup_ms - 1)
    assert windup["TBG_UpperArm.R"][0] < guard["TBG_UpperArm.R"][0] - 0.4


def test_downswing_extends_through_hit():
    windup = sample_pose_at_ms(PRIMARY, TANK_MELEE_TIMING.windup_ms - 1)
    impact = sample_pose_at_ms(PRIMARY, impact_time_ms_for_profile(PRIMARY))
    assert impact["TBG_UpperArm.R"][0] > windup["TBG_UpperArm.R"][0] + 0.8


def test_spine_has_no_yaw_or_roll():
    frames = frame_count_for_profile(PRIMARY)
    for frame in (0, frames // 4, frames // 2, frames):
        spine = sample_pose_at_frame(PRIMARY, frame)["TBG_Spine"]
        assert abs(spine[1]) < 1e-6
        assert abs(spine[2]) < 1e-6


def test_within_frame_tolerance_helper():
    assert within_frame_tolerance(520, 533, frames=1)
    assert not within_frame_tolerance(520, 560, frames=1)
