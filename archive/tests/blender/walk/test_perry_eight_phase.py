"""Contract: Perry 8-phase gait spec (Winter/Perry table)."""

import math

from tools.blender.animation.walk.eight_phase_gait import sample_eight_phase_pose
from tools.blender.animation.walk.perry_eight_phase import (
    get_leg_angles,
    leg_phase_tau,
)
from tools.blender.animation.walk.style import BLOCKY_STUDY_STYLE


def test_initial_contact_hip_flexion():
    hip, knee, ankle = get_leg_angles(0.0, "R")
    assert hip == 20.0
    assert knee == 0.0
    assert ankle == 0.0


def test_left_leg_offset_fifty_percent():
    assert leg_phase_tau(0.0, "L") == 0.5
    l_hip, _, _ = get_leg_angles(0.0, "L")
    r_hip, _, _ = get_leg_angles(0.5, "R")
    assert l_hip == r_hip


def test_swing_knee_peak_above_forty_degrees():
    _, knee, _ = get_leg_angles(0.65, "R")
    assert knee >= 40.0


def test_legs_opposite_at_mid_cycle():
    r_hip, _, _ = get_leg_angles(0.25, "R")
    l_hip, _, _ = get_leg_angles(0.25, "L")
    assert math.copysign(1, r_hip) != math.copysign(1, l_hip) or abs(r_hip - l_hip) > 5.0


def test_eight_phase_pose_loops():
    frames = 33
    start = sample_eight_phase_pose(0, frames, BLOCKY_STUDY_STYLE)
    end = sample_eight_phase_pose(frames, frames, BLOCKY_STUDY_STYLE)
    assert start["TBG_UpperLeg.R"][0] == end["TBG_UpperLeg.R"][0]
