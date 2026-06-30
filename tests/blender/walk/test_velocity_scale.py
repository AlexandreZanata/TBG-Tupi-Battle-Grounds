"""Contract: walk velocity scaling from lane move speed."""

from tools.blender.animation.root_motion.stride import walk_velocity_mps
from tools.blender.animation.walk.style import HUMANOID_LIGHT_STYLE
from tools.blender.animation.walk.velocity import scale_walk_style_for_speed, scaled_walk_velocity_mps


def test_double_speed_scales_velocity_linearly():
    base = scaled_walk_velocity_mps(HUMANOID_LIGHT_STYLE)
    scaled = scale_walk_style_for_speed(HUMANOID_LIGHT_STYLE, base * 2.0)
    assert scaled_walk_velocity_mps(scaled) == base * 2.0


def test_half_speed_scales_velocity_linearly():
    base = scaled_walk_velocity_mps(HUMANOID_LIGHT_STYLE)
    scaled = scale_walk_style_for_speed(HUMANOID_LIGHT_STYLE, base * 0.5)
    assert scaled_walk_velocity_mps(scaled) == base * 0.5


def test_velocity_formula_matches_stride_module():
    style = HUMANOID_LIGHT_STYLE
    assert scaled_walk_velocity_mps(style) == walk_velocity_mps(style.stride_length_m, style.cadence_spm)


def test_clamps_extreme_speed_requests():
    base = scaled_walk_velocity_mps(HUMANOID_LIGHT_STYLE)
    slow = scale_walk_style_for_speed(HUMANOID_LIGHT_STYLE, base * 0.01)
    fast = scale_walk_style_for_speed(HUMANOID_LIGHT_STYLE, base * 10.0)
    assert scaled_walk_velocity_mps(slow) >= base * 0.25
    assert scaled_walk_velocity_mps(fast) <= base * 2.0
