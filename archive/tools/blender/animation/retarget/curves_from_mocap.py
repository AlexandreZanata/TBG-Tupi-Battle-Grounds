"""Extract gait %cycle curves from CMU BVH — no bpy."""

from __future__ import annotations

from pathlib import Path

from tools.blender.animation.retarget.bone_map import load_bone_map
from tools.blender.animation.retarget.bvh_parser import BvhMotion, parse_bvh
from tools.blender.animation.walk.curves import Keyframe

PERRY_HEEL_STRIKE = {"hip": 30.0, "knee": 0.0, "ankle": 0.0}
PERRY_TOE_OFF = {"hip": -20.0, "knee": 0.0, "ankle": 9.0}
PERRY_KNEE_SWING_PEAK = 58.0


def extract_curves_from_bvh(path: Path | str, bone_map: dict | None = None) -> dict[str, tuple[Keyframe, ...]]:
    data = parse_bvh(path)
    cfg = bone_map or load_bone_map()
    period = detect_gait_period(data, cfg)
    if period is None:
        raise ValueError(f"no walk cycle detected in {path}")
    start = _heel_strike_index(data, cfg, period)
    return _sample_leg_curves(data, cfg, start, period)


def detect_gait_period(motion: BvhMotion, bone_map: dict) -> int | None:
    detect = bone_map["cycle_detection"]
    skip = detect["skip_initial_frames"]
    joint = detect["leg_joint"]
    channel = detect["channel"]
    series = motion.joint_series(joint, channel)[skip:]
    min_period = detect["min_period_frames"]
    if len(series) < min_period + 10:
        return None
    centered = [value - sum(series) / len(series) for value in series]
    best_lag, best_corr = 0, -1.0
    for lag in range(detect["min_period_frames"], detect["max_period_frames"] + 1):
        corr = sum(centered[i] * centered[i + lag] for i in range(len(centered) - lag))
        if corr > best_corr:
            best_corr, best_lag = corr, lag
    return best_lag if best_corr > 0 else None


def average_curve_sets(curve_sets: list[dict[str, tuple[Keyframe, ...]]]) -> dict[str, tuple[Keyframe, ...]]:
    if not curve_sets:
        raise ValueError("no curve sets to average")
    keys = curve_sets[0].keys()
    merged: dict[str, tuple[Keyframe, ...]] = {}
    for key in keys:
        taus = [tau for tau, _ in curve_sets[0][key]]
        values = []
        for tau in taus:
            samples = [next(v for t, v in curves[key] if abs(t - tau) < 1e-9) for curves in curve_sets]
            values.append((tau, sum(samples) / len(samples)))
        merged[key] = tuple(values)
    return merged


def merge_with_perry(
    mocap: dict[str, tuple[Keyframe, ...]],
    perry: dict[str, tuple[Keyframe, ...]],
    blend: float = 0.3,
) -> dict[str, tuple[Keyframe, ...]]:
    out: dict[str, tuple[Keyframe, ...]] = {}
    for key, mocap_curve in mocap.items():
        perry_curve = perry.get(key, ())
        perry_by_tau = dict(perry_curve)
        merged: list[Keyframe] = []
        for tau, m_val in mocap_curve:
            p_val = perry_by_tau.get(tau, m_val)
            merged.append((tau, p_val * (1.0 - blend) + m_val * blend))
        out[key] = tuple(merged)
    return out


def detect_walk_cycle(path: Path | str, bone_map: dict | None = None) -> tuple[int, int]:
    """Return (heel_strike_frame, period_frames) for one gait cycle in a CMU BVH."""
    data = parse_bvh(path)
    cfg = bone_map or load_bone_map()
    period = detect_gait_period(data, cfg)
    if period is None:
        raise ValueError(f"no walk cycle detected in {path}")
    return _heel_strike_index(data, cfg, period), period


def calibrate_from_paths(paths: list[Path | str], blend: float = 0.25) -> dict[str, tuple[Keyframe, ...]]:
    from tools.blender.animation.walk import biomech_curves as perry

    perry_curves = {
        "hip": perry.HIP_CURVE_DEG,
        "knee": perry.KNEE_CURVE_DEG,
        "ankle": perry.ANKLE_CURVE_DEG,
    }
    extracted = [extract_curves_from_bvh(path) for path in paths]
    averaged = average_curve_sets(extracted)
    return merge_with_perry(averaged, perry_curves, blend=blend)


def perry_reference_angles() -> dict[str, dict[str, float]]:
    return {
        "heel_strike": dict(PERRY_HEEL_STRIKE),
        "toe_off": dict(PERRY_TOE_OFF),
        "knee_swing_peak_deg": PERRY_KNEE_SWING_PEAK,
    }


def _heel_strike_index(motion: BvhMotion, bone_map: dict, period: int) -> int:
    detect = bone_map["cycle_detection"]
    skip = detect["skip_initial_frames"]
    joint = detect["leg_joint"]
    channel = detect["channel"]
    series = motion.joint_series(joint, channel)[skip : skip + period]
    return skip + series.index(min(series))


def _sample_leg_curves(
    motion: BvhMotion,
    bone_map: dict,
    start: int,
    period: int,
) -> dict[str, tuple[Keyframe, ...]]:
    flex = bone_map["leg_flexion"]
    channel = flex["channel"]
    taus = bone_map["perry_key_taus"]
    hip = _transform_series(motion, "RightUpLeg", "RightLeg", channel, flex["right_scale"], start, period, taus)
    knee = _transform_series(motion, "RightLeg", "RightLeg", channel, flex["knee_scale"], start, period, taus)
    ankle = _transform_series(motion, "RightFoot", "RightFoot", channel, flex["ankle_scale"], start, period, taus)
    hip = _anchor_heel_strike(hip, PERRY_HEEL_STRIKE["hip"])
    knee = _anchor_heel_strike(knee, PERRY_HEEL_STRIKE["knee"])
    ankle = _anchor_heel_strike(ankle, PERRY_HEEL_STRIKE["ankle"])
    return {"hip": hip, "knee": knee, "ankle": ankle}


def _transform_series(
    motion: BvhMotion,
    joint: str,
    _joint_label: str,
    channel: str,
    scale: float,
    start: int,
    period: int,
    taus: list[float],
) -> tuple[Keyframe, ...]:
    raw = motion.joint_series(joint, channel)[start : start + period]
    keyframes: list[Keyframe] = []
    for tau in taus:
        idx = min(len(raw) - 1, int(round(tau * (len(raw) - 1))))
        keyframes.append((tau, scale * raw[idx]))
    return tuple(keyframes)


def _anchor_heel_strike(curve: tuple[Keyframe, ...], target: float) -> tuple[Keyframe, ...]:
    offset = target - curve[0][1]
    return tuple((tau, value + offset) for tau, value in curve)
