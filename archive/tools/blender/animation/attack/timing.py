"""Attack phase timing — aligns with behavior profile `timings` fields."""

from __future__ import annotations

from dataclasses import dataclass

from tools.blender.animation.attack.constants import ATTACK_FPS, HIT_IMPACT_RATIO

FRAME_MS: float = 1000.0 / ATTACK_FPS


@dataclass(frozen=True)
class AttackTiming:
    windup_ms: int
    hit_ms: int
    recovery_ms: int

    @property
    def total_ms(self) -> int:
        return self.windup_ms + self.hit_ms + self.recovery_ms


def ms_to_frame(ms: float, fps: int = ATTACK_FPS) -> int:
    return round(ms / 1000.0 * fps)


def frame_count(timing: AttackTiming, fps: int = ATTACK_FPS) -> int:
    return ms_to_frame(timing.total_ms, fps)


def impact_ms(timing: AttackTiming, ratio: float = HIT_IMPACT_RATIO) -> int:
    return timing.windup_ms + round(timing.hit_ms * ratio)


def within_frame_tolerance(expected_ms: int, actual_ms: int, frames: int = 1) -> bool:
    return abs(expected_ms - actual_ms) <= frames * FRAME_MS + 1e-6
