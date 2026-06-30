"""Root motion — forward stride math (no bpy)."""

from __future__ import annotations

DEFAULT_STRIDE_LENGTH_M: float = 1.4


def walk_velocity_mps(stride_length_m: float, cadence_spm: float) -> float:
    """Meters per second from stride length and cadence (2 steps per cycle)."""
    return stride_length_m * cadence_spm / 120.0


def root_forward_z(frame: int, cycle_frames: int, stride_length_m: float) -> float:
    """Analytic root +Z for frame within a cyclic walk (τ integration)."""
    if cycle_frames <= 0:
        return 0.0
    if frame > 0 and frame % cycle_frames == 0:
        return 0.0
    tau = (frame % cycle_frames) / cycle_frames
    return stride_length_m * tau


def root_location_yz(
    frame: int,
    cycle_frames: int,
    pelvis_y: float,
    stride_length_m: float,
) -> tuple[float, float]:
    """Root Y bob + forward Z for glTF Y-up rig."""
    return pelvis_y, root_forward_z(frame, cycle_frames, stride_length_m)
