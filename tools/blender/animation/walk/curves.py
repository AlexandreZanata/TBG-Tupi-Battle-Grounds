"""Cyclic Catmull-Rom interpolation for gait joint curves (τ ∈ [0, 1))."""

from __future__ import annotations

Keyframe = tuple[float, float]


def catmull_rom(p0: float, p1: float, p2: float, p3: float, t: float) -> float:
    t2 = t * t
    t3 = t2 * t
    return 0.5 * (
        (2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
    )


def sample_cycle_curve(keyframes: tuple[Keyframe, ...], tau: float) -> float:
    """Sample a cyclic joint curve at normalized gait phase τ."""
    if not keyframes:
        return 0.0
    tau = tau % 1.0
    pts = sorted(keyframes, key=lambda item: item[0])
    if pts[0][0] > 0.0:
        pts = [(0.0, pts[-1][1])] + pts
    if pts[-1][0] < 1.0:
        pts = pts + [(1.0, pts[0][1])]

    times = [p[0] for p in pts]
    values = [p[1] for p in pts]
    core = values[:-1]
    n = len(core)

    seg = 0
    for idx in range(len(times) - 1):
        if times[idx] <= tau <= times[idx + 1]:
            seg = idx
            break
    span = max(1e-9, times[seg + 1] - times[seg])
    local = (tau - times[seg]) / span

    def core_at(offset: int) -> float:
        return core[(seg + offset) % n]

    return catmull_rom(core_at(-1), core_at(0), core_at(1), core_at(2), local)
