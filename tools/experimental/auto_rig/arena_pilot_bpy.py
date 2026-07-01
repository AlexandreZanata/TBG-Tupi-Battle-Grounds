#!/usr/bin/env python3
"""Blender: experimental auto-rig + clip retarget for one Motion Arena pilot."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("arena_pilot_bpy requires Blender bpy") from exc

_REPO = Path(__file__).resolve().parents[3]
for path in (_REPO, _REPO / "tools/blender/scripts", Path(__file__).resolve().parent):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from auto_rig_mesh import auto_rig_mesh  # noqa: E402
from retarget_clip_library import retarget_clip_library  # noqa: E402
from tools.blender.package_path import resolve_repo_path  # noqa: E402


def export_arena_pilot(
    unit_id: str,
    showcase_path: Path,
    raw_output: Path,
    *,
    body_class: str,
    animation_set_id: str,
    voxel_remesh: bool = True,
) -> dict:
    repo = _REPO
    tmp_dir = repo / "content/characters/meshes/.tmp"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    skinned = tmp_dir / f"{unit_id}_arena_skinned.glb"

    report = auto_rig_mesh(
        showcase_path,
        skinned,
        body_class=body_class,
        voxel_remesh=voxel_remesh,
        bind_mode="fallback",
    )
    retarget_clip_library(skinned, animation_set_id, raw_output, repo, unit_id=unit_id)
    skinned.unlink(missing_ok=True)
    return {
        "unit_id": unit_id,
        "raw_output": str(raw_output),
        "weight_gate_pass": report.weight_gate_pass,
        "bind_winner": report.bind_winner,
        "faces": report.faces,
    }


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Auto-rig one pilot and embed combat clips.")
    parser.add_argument("--id", dest="unit_id", required=True)
    parser.add_argument("--showcase", type=Path, required=True)
    parser.add_argument("--raw-output", type=Path, required=True)
    parser.add_argument("--body-class", default="HumanoidLight")
    parser.add_argument("--animation-set", default="humanoid_light_combat_v1")
    parser.add_argument("--no-voxel-remesh", action="store_true")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)

    payload = export_arena_pilot(
        args.unit_id,
        resolve_repo_path(args.showcase),
        resolve_repo_path(args.raw_output),
        body_class=args.body_class,
        animation_set_id=args.animation_set,
        voxel_remesh=not args.no_voxel_remesh,
    )
    if args.report_json:
        print(json.dumps(payload))
    return 0 if payload["weight_gate_pass"] else 2


if __name__ == "__main__":
    raise SystemExit(_cli())
