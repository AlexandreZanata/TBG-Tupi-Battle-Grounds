"""Contract: stride length, cadence, and root Z integration."""

from tools.blender.animation.root_motion.stride import (
    DEFAULT_STRIDE_LENGTH_M,
    root_forward_z,
    walk_velocity_mps,
)
from tools.blender.animation.walk import cycle_frames_for_profile


def test_walk_velocity_formula_110_spm_default_stride():
    velocity = walk_velocity_mps(1.4, 110.0)
    assert abs(velocity - 1.4 * 110.0 / 120.0) < 1e-9
    assert abs(velocity - 1.2833333333) < 1e-4


def test_root_forward_z_approaches_stride_before_loop_closure():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    start = root_forward_z(0, frames, DEFAULT_STRIDE_LENGTH_M)
    pre_loop = root_forward_z(frames - 1, frames, DEFAULT_STRIDE_LENGTH_M)
    loop = root_forward_z(frames, frames, DEFAULT_STRIDE_LENGTH_M)
    assert start == 0.0
    assert pre_loop > DEFAULT_STRIDE_LENGTH_M * 0.9
    assert loop == 0.0


def test_root_forward_z_loops_to_zero_at_cycle_boundary():
    frames = cycle_frames_for_profile("humanoid_light_v1")
    assert root_forward_z(frames, frames, 1.4) == 0.0
