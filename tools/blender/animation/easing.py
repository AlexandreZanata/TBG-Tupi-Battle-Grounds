"""Shared easing for procedural attack clips."""

from __future__ import annotations

import math


def clamp01(value: float) -> float:
    return max(0.0, min(1.0, value))


def ease_out(value: float) -> float:
    t = clamp01(value)
    return 1.0 - (1.0 - t) * (1.0 - t)


def ease_in_expo(value: float) -> float:
    t = clamp01(value)
    if t <= 0.0:
        return 0.0
    return math.pow(2.0, 10.0 * (t - 1.0))
