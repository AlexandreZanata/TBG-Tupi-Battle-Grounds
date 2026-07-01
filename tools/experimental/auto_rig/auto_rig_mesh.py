#!/usr/bin/env python3
"""Experimental headless auto-rig: raw AI GLB → cleaned mesh + TBG skeleton + automatic weights."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("auto_rig_mesh requires Blender bpy") from exc

_REPO = Path(__file__).resolve().parents[3]
_BLENDER_SCRIPTS = _REPO / "tools/blender/scripts"
_EXPERIMENTAL = Path(__file__).resolve().parent
for path in (_REPO, _BLENDER_SCRIPTS, _EXPERIMENTAL):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from tools.blender.package_path import resolve_repo_path  # noqa: E402

from auto_rig_decimate import DEFAULT_MAX_FACES, maybe_decimate_for_rig  # noqa: E402
from auto_rig_ops import (  # noqa: E402
    BIND_AUTOMATIC,
    BIND_FALLBACK,
    BIND_PROXIMITY,
    bind_mesh,
    export_skinned_glb,
    load_template_armature,
    weight_metrics,
)
from prep_ai_mesh_ops import (  # noqa: E402
    clear_scene,
    count_loose_verts,
    import_mesh,
    join_and_clean,
    maybe_voxel_remesh,
    normalize_mesh,
    object_bounds,
    shade_smooth,
)

DEFAULT_HEIGHT = 1.8
DEFAULT_BODY_CLASS = "HumanoidLight"


@dataclass(frozen=True)
class AutoRigReport:
    body_class: str
    input_path: str
    output_path: str
    faces: int
    height: float
    loose_verts: int
    voxel_remeshed: bool
    decimated_for_rig: bool
    bind_mode: str
    bind_winner: str
    bone_count: int
    dominant_bone: str | None
    dominant_share: float
    unweighted_vertices: int
    bones_missing: tuple[str, ...]
    weight_gate_pass: bool


def auto_rig_mesh(
    input_path: Path,
    output_path: Path,
    *,
    body_class: str = DEFAULT_BODY_CLASS,
    target_height: float = DEFAULT_HEIGHT,
    voxel_remesh: bool = False,
    z_up: bool = False,
    max_faces: int = DEFAULT_MAX_FACES,
    bind_mode: str = BIND_FALLBACK,
) -> AutoRigReport:
    input_path = resolve_repo_path(input_path)
    output_path = resolve_repo_path(output_path)
    if output_path.suffix.lower() not in {".glb", ".gltf"}:
        output_path = output_path.with_suffix(".glb")

    clear_scene()
    arm_obj = load_template_armature(body_class)

    import_mesh(input_path)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError(f"no mesh geometry in {input_path}")

    mesh_obj = join_and_clean(meshes)
    if z_up:
        from prep_ai_mesh_ops import apply_z_up  # noqa: PLC0415

        apply_z_up(mesh_obj)
    remeshed = maybe_voxel_remesh(mesh_obj, force=voxel_remesh)
    normalize_mesh(mesh_obj, target_height)
    decimated = maybe_decimate_for_rig(mesh_obj, max_faces=max_faces)
    shade_smooth(mesh_obj)
    bind_winner = bind_mesh(mesh_obj, arm_obj, mode=bind_mode)
    metrics = weight_metrics(mesh_obj, arm_obj)
    export_skinned_glb(mesh_obj, arm_obj, output_path)

    _min_x, _max_x, min_y, max_y, _min_z, _max_z = object_bounds(mesh_obj)
    return AutoRigReport(
        body_class=body_class,
        input_path=str(input_path),
        output_path=str(output_path),
        faces=len(mesh_obj.data.polygons),
        height=max_y - min_y,
        loose_verts=count_loose_verts(mesh_obj),
        voxel_remeshed=remeshed,
        decimated_for_rig=decimated,
        bind_mode=bind_mode,
        bind_winner=bind_winner,
        bone_count=len(arm_obj.data.bones),
        dominant_bone=metrics.dominant_bone,
        dominant_share=metrics.dominant_share,
        unweighted_vertices=metrics.unweighted_vertices,
        bones_missing=metrics.bones_missing,
        weight_gate_pass=metrics.passes_gate,
    )


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(
        description="Experimental auto-rig: raw GLB/OBJ → skinned GLB via ARMATURE_AUTO.",
    )
    parser.add_argument("input", type=Path, help="Raw mesh GLB/OBJ (no skeleton)")
    parser.add_argument("output", type=Path, help="Output rigged GLB")
    parser.add_argument("--body-class", default=DEFAULT_BODY_CLASS)
    parser.add_argument("--height", type=float, default=DEFAULT_HEIGHT)
    parser.add_argument("--voxel-remesh", action="store_true")
    parser.add_argument("--z-up", action="store_true")
    parser.add_argument(
        "--bind-mode",
        choices=[BIND_AUTOMATIC, BIND_PROXIMITY, BIND_FALLBACK],
        default=BIND_FALLBACK,
        help="automatic=ARMATURE_AUTO only; fallback=proximity vs automatic (production)",
    )
    parser.add_argument("--max-faces", type=int, default=DEFAULT_MAX_FACES)
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)

    report = auto_rig_mesh(
        args.input,
        args.output,
        body_class=args.body_class,
        target_height=args.height,
        voxel_remesh=args.voxel_remesh,
        z_up=args.z_up,
        max_faces=args.max_faces,
        bind_mode=args.bind_mode,
    )
    if args.report_json:
        print(json.dumps(asdict(report)))
    return 0 if report.weight_gate_pass else 2


if __name__ == "__main__":
    raise SystemExit(_cli())
