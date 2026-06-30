"""Contract: calibrated mocap curves stay within Perry tolerance at gait events."""

from pathlib import Path

from tools.blender.animation.retarget.curves_from_mocap import (
    calibrate_from_paths,
    perry_reference_angles,
)
from tools.blender.animation.walk.curves import sample_cycle_curve
from tools.blender.animation.walk import biomech_curves

FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "mocap" / "35_01_walk_excerpt.bvh"
TOLERANCE_DEG = 15.0


def _value_at(curve: tuple, tau: float) -> float:
    return sample_cycle_curve(curve, tau)


def test_calibrated_heel_strike_within_perry_tolerance():
    curves = calibrate_from_paths([FIXTURE], blend=0.25)
    refs = perry_reference_angles()["heel_strike"]
    assert abs(_value_at(curves["hip"], 0.0) - refs["hip"]) <= TOLERANCE_DEG
    assert abs(_value_at(curves["knee"], 0.0) - refs["knee"]) <= TOLERANCE_DEG
    assert abs(_value_at(curves["ankle"], 0.0) - refs["ankle"]) <= TOLERANCE_DEG


def test_calibrated_toe_off_hip_within_perry_tolerance():
    curves = calibrate_from_paths([FIXTURE], blend=0.25)
    refs = perry_reference_angles()["toe_off"]
    assert abs(_value_at(curves["hip"], 0.5) - refs["hip"]) <= TOLERANCE_DEG


def test_calibrated_knee_swing_peak_reasonable():
    curves = calibrate_from_paths([FIXTURE], blend=0.25)
    peak = perry_reference_angles()["knee_swing_peak_deg"]
    swing = _value_at(curves["knee"], 0.73)
    assert swing > 35.0
    assert abs(swing - peak) <= 25.0


def test_biomech_curves_still_sample_at_heel_strike():
    assert _value_at(biomech_curves.HIP_CURVE_DEG, 0.0) == 30.0
    assert _value_at(biomech_curves.KNEE_CURVE_DEG, 0.0) == 0.0
