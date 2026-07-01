#!/usr/bin/env python3
"""Normalize AI/sculpt GLB or OBJ for TBG showcase pipeline (Blender bpy)."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("prep_ai_mesh requires Blender bpy") from exc

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from prep_ai_mesh_ops import (  # noqa: E402
    VOXEL_AUTO_THRESHOLD,
    apply_z_up,
    clear_scene,
    count_loose_verts,
    export_glb,
    import_mesh,
    join_and_clean,
    maybe_voxel_remesh,
    normalize_mesh,
    object_bounds,
    shade_smooth,
)

DEFAULT_HEIGHT = 1.8


@dataclass(frozen=True)
class PrepReport:
    faces: int
    height: float
    min_y: float
    center_x: float
    center_z: float
    voxel_remeshed: bool
    loose_verts: int


def prep_ai_mesh(
    input_path: Path,
    output_path: Path,
    *,
    target_height: float = DEFAULT_HEIGHT,
    voxel_remesh: bool = False,
    z_up: bool = False,
) -> PrepReport:
    input_path = input_path.resolve()
    output_path = output_path.resolve()
    clear_scene()
    import_mesh(input_path)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError(f"no mesh geometry in {input_path}")

    mesh_obj = join_and_clean(meshes)
    if z_up:
        apply_z_up(mesh_obj)
    remeshed = maybe_voxel_remesh(mesh_obj, force=voxel_remesh)
    normalize_mesh(mesh_obj, target_height)
    shade_smooth(mesh_obj)

    report = _build_report(mesh_obj, remeshed)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    export_glb(mesh_obj, output_path)
    return report


def _build_report(mesh_obj: bpy.types.Object, voxel_remeshed: bool) -> PrepReport:
    min_x, max_x, min_y, max_y, min_z, max_z = object_bounds(mesh_obj)
    return PrepReport(
        faces=len(mesh_obj.data.polygons),
        height=max_y - min_y,
        min_y=min_y,
        center_x=(min_x + max_x) * 0.5,
        center_z=(min_z + max_z) * 0.5,
        voxel_remeshed=voxel_remeshed,
        loose_verts=count_loose_verts(mesh_obj),
    )


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Normalize AI/sculpt mesh for TBG showcase import.")
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--height", type=float, default=DEFAULT_HEIGHT)
    parser.add_argument("--voxel-remesh", action="store_true")
    parser.add_argument("--z-up", action="store_true")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)
    report = prep_ai_mesh(
        args.input,
        args.output,
        target_height=args.height,
        voxel_remesh=args.voxel_remesh,
        z_up=args.z_up,
    )
    if args.report_json:
        print(json.dumps(asdict(report)))
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
