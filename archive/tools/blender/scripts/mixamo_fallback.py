#!/usr/bin/env python3
"""Export OBJ for Mixamo upload and print the manual checklist."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("mixamo_fallback requires Blender bpy") from exc

_SCRIPTS_DIR = Path(__file__).resolve().parent
_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from tools.blender.package_path import ensure_repo_paths  # noqa: E402

ensure_repo_paths()

from prep_ai_mesh_ops import clear_scene, import_mesh  # noqa: E402
from tools.mesh_pipeline.mixamo_checklist import print_mixamo_checklist  # noqa: E402


def export_obj_for_mixamo(source: Path, output: Path) -> Path:
    source = source.resolve()
    output = output.resolve()
    clear_scene()
    import_mesh(source)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError(f"no mesh in {source}")
    mesh_obj = meshes[0] if len(meshes) == 1 else _join_meshes(meshes)
    output.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    if hasattr(bpy.ops.wm, "obj_export"):
        bpy.ops.wm.obj_export(filepath=str(output), export_selected_objects=True)
    else:
        bpy.ops.export_scene.obj(filepath=str(output), use_selection=True)
    return output


def _join_meshes(meshes: list[bpy.types.Object]) -> bpy.types.Object:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    return bpy.context.object


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Export OBJ for Mixamo + print checklist.")
    parser.add_argument("--source", type=Path, required=True, help="GLB/OBJ mesh to upload")
    parser.add_argument("--out", type=Path, required=True, help="Output OBJ path")
    parser.add_argument("--checklist-only", action="store_true", help="Skip export; print steps")
    args = parser.parse_args(args_list)
    obj_path = args.out.resolve()
    if not args.checklist_only:
        export_obj_for_mixamo(args.source, obj_path)
    print_mixamo_checklist(obj_path if not args.checklist_only else None)
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
