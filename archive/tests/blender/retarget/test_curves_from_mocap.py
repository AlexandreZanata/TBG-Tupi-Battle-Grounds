"""Contract: mocap curve extraction detects walk cycle on CMU fixture."""

from pathlib import Path

from tools.blender.animation.retarget.bone_map import load_bone_map
from tools.blender.animation.retarget.curves_from_mocap import (
    average_curve_sets,
    detect_gait_period,
    extract_curves_from_bvh,
)
from tools.blender.animation.retarget.bvh_parser import parse_bvh

FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "mocap" / "35_01_walk_excerpt.bvh"


def test_detect_gait_period_on_cmu_walk_fixture():
    motion = parse_bvh(FIXTURE)
    period = detect_gait_period(motion, load_bone_map())
    assert period is not None
    assert 90 <= period <= 200


def test_extract_hip_knee_ankle_curves():
    curves = extract_curves_from_bvh(FIXTURE)
    for key in ("hip", "knee", "ankle"):
        assert key in curves
        assert curves[key][0][0] == 0.0
        assert len(curves[key]) >= 10


def test_average_curve_sets_matches_tau_grid():
    curves = extract_curves_from_bvh(FIXTURE)
    merged = average_curve_sets([curves, curves])
    assert merged["hip"][0] == curves["hip"][0]
    assert merged["knee"][5][1] == curves["knee"][5][1]
