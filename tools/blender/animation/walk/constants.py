"""Walk cycle timing — cadence-driven frame count (§4 spec)."""

from __future__ import annotations

from typing import Final

WALK_FPS: Final[int] = 30
DEFAULT_CADENCE_SPM: Final[float] = 110.0


def cycle_frames(cadence_spm: float = DEFAULT_CADENCE_SPM) -> int:
    """Frames per full gait cycle (2 steps) at given cadence."""
    return max(24, min(48, round(120.0 / cadence_spm * WALK_FPS)))


WALK_FRAMES: Final[int] = cycle_frames()
