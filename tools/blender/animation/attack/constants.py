"""Trident lunge thrust timing — matches ranged_shot_v1 behavior contract."""

from __future__ import annotations

from typing import Final

ATTACK_FPS: Final[int] = 30
WINDUP_MS: Final[int] = 300
HIT_MS: Final[int] = 450
RECOVERY_MS: Final[int] = 300
TOTAL_MS: Final[int] = WINDUP_MS + HIT_MS + RECOVERY_MS
HIT_IMPACT_RATIO: Final[float] = 0.85


def ms_to_frame(ms: float, fps: int = ATTACK_FPS) -> int:
    return round(ms / 1000.0 * fps)


def total_frames(fps: int = ATTACK_FPS) -> int:
    return ms_to_frame(TOTAL_MS, fps)


def impact_time_ms() -> int:
    return WINDUP_MS + round(HIT_MS * HIT_IMPACT_RATIO)
