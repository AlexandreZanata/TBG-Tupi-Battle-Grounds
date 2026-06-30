"""Contract: two-bone IK reach clamp matches Unity/web solvers."""

from __future__ import annotations

import math


def clamp_reach(root: tuple[float, float, float], target: tuple[float, float, float], max_reach: float):
    ox = target[0] - root[0]
    oy = target[1] - root[1]
    oz = target[2] - root[2]
    dist_sq = ox * ox + oy * oy + oz * oz
    if dist_sq <= max_reach * max_reach:
        return target
    dist = math.sqrt(dist_sq)
    scale = max_reach / dist
    return (root[0] + ox * scale, root[1] + oy * scale, root[2] + oz * scale)


def _length(vec: tuple[float, float, float]) -> float:
    return math.sqrt(vec[0] ** 2 + vec[1] ** 2 + vec[2] ** 2)


def test_clamp_reach_shortens_far_targets():
    clamped = clamp_reach((0.0, 0.0, 0.0), (0.0, 0.0, 5.0), 2.0)
    assert math.isclose(_length(clamped), 2.0, rel_tol=1e-6)


def test_clamp_reach_preserves_near_targets():
    target = (0.2, 0.0, 0.3)
    assert clamp_reach((0.0, 0.0, 0.0), target, 2.0) == target
