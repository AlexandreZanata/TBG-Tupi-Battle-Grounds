"""Gait contact timing from Perry stance/swing split."""

from __future__ import annotations

from tools.blender.animation.walk.biomech_curves import KNEE_CURVE_DEG
from tools.blender.animation.walk.curves import sample_cycle_curve

STANCE_END_TAU: float = 0.62


def heel_strike_tau() -> float:
    return 0.0


def toe_off_tau() -> float:
    return STANCE_END_TAU


def stance_phase_frames(cycle_frames: int) -> tuple[int, int]:
    """Inclusive frame range [heel, toe_off] for right-leg stance."""
    if cycle_frames < 2:
        return 0, 0
    toe_off = max(1, round(STANCE_END_TAU * cycle_frames))
    return 0, min(toe_off, cycle_frames - 1)


def knee_flex_at_tau(tau: float) -> float:
    return sample_cycle_curve(KNEE_CURVE_DEG, tau % 1.0)
