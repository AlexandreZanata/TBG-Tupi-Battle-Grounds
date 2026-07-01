#!/usr/bin/env python3
"""CLI: CMU BVH → TBG gait curves and optional bpy pose sample."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))


def _resolve_bvh(path: str) -> Path:
    candidate = Path(path)
    if candidate.is_file():
        return candidate.resolve()
    rooted = REPO_ROOT / path
    if rooted.is_file():
        return rooted
    raise FileNotFoundError(path)

from tools.blender.animation.retarget.curves_from_mocap import (  # noqa: E402
    calibrate_from_paths,
    extract_curves_from_bvh,
)
from tools.blender.animation.walk import biomech_curves  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser(description="Retarget CMU walk BVH to TBG gait curves")
    parser.add_argument("bvhs", nargs="*", help="BVH file paths")
    parser.add_argument("--calibrate", action="store_true", help="Print merged Perry+mocap curves")
    parser.add_argument("--extract", action="store_true", help="Print raw extracted curves per file")
    parser.add_argument("--write-curves", action="store_true", help="Patch biomech_curves.py from BVHs")
    parser.add_argument("--sample", action="store_true", help="Sample GaitPose via Blender bpy")
    parser.add_argument("--frame", type=int, default=60, help="Frame for --sample")
    parser.add_argument("--blend", type=float, default=0.25, help="Mocap blend weight for calibration")
    args = parser.parse_args()

    if args.sample:
        return _sample_pose(args.bvhs[0], args.frame)
    if not args.bvhs:
        parser.error("provide at least one BVH path")
    if args.extract:
        for path in args.bvhs:
            curves = extract_curves_from_bvh(_resolve_bvh(path))
            print(json.dumps({"file": path, "curves": _serialize(curves)}, indent=2))
        return 0
    curves = calibrate_from_paths([_resolve_bvh(p) for p in args.bvhs], blend=args.blend)
    if args.write_curves:
        _write_biomech_curves(curves)
        print(f"updated {biomech_curves.__file__} from {len(args.bvhs)} BVH(s)")
        return 0
    print(json.dumps(_serialize(curves), indent=2))
    return 0


def _serialize(curves: dict) -> dict:
    return {key: [[tau, round(val, 2)] for tau, val in items] for key, items in curves.items()}


def _write_biomech_curves(curves: dict) -> None:
    path = Path(biomech_curves.__file__)
    lines = path.read_text(encoding="utf-8").splitlines()
    targets = {
        "HIP_CURVE_DEG": curves["hip"],
        "KNEE_CURVE_DEG": curves["knee"],
        "ANKLE_CURVE_DEG": curves["ankle"],
    }
    out: list[str] = []
    idx = 0
    while idx < len(lines):
        line = lines[idx]
        matched = next((name for name in targets if line.startswith(f"{name}:")), None)
        if matched is None:
            out.append(line)
            idx += 1
            continue
        out.extend(_format_tuple(matched, targets[matched]).splitlines())
        idx += 1
        while idx < len(lines) and not lines[idx].startswith("#") and not lines[idx].endswith(")"):
            idx += 1
        if idx < len(lines) and lines[idx].strip() == ")":
            idx += 1
    path.write_text("\n".join(out) + "\n", encoding="utf-8")


def _format_tuple(name: str, keyframes: tuple) -> str:
    lines = [f"{name}: tuple[Keyframe, ...] = ("]
    for tau, value in keyframes:
        lines.append(f"    ({tau:.2f}, {value:.1f}),")
    lines.append(")")
    return "\n".join(lines)


def _sample_pose(bvh_path: str, frame: int) -> int:
    from tools.blender.animation.retarget.import_bvh import sample_gait_pose_from_bvh

    pose = sample_gait_pose_from_bvh(_resolve_bvh(bvh_path), frame)
    serial = {bone: [round(v, 4) for v in vals] for bone, vals in pose.items()}
    print(json.dumps({"frame": frame, "pose": serial}, indent=2))
    return 0


if __name__ == "__main__":
    if "--" in sys.argv:
        sys.argv = [sys.argv[0], *sys.argv[sys.argv.index("--") + 1 :]]
    raise SystemExit(main())
