#!/usr/bin/env python3
"""Decimate showcase GLB to mobile tri budget (Blender bpy)."""

from __future__ import annotations

from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("sculpt_to_game_mesh requires Blender bpy") from exc

TRI_TARGET = 8000


def sculpt_to_game_mesh(showcase_path: Path, output_path: Path, tri_target: int = TRI_TARGET) -> Path:
    _clear_scene()
    bpy.ops.import_scene.gltf(filepath=str(showcase_path))
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError("showcase import produced no meshes")

    for obj in meshes:
        bpy.context.view_layer.objects.active = obj
        modifier = obj.modifiers.new(name="TBGDecimate", type="DECIMATE")
        ratio = min(1.0, tri_target / max(1, _mesh_tri_count(obj)))
        modifier.ratio = ratio
        bpy.ops.object.modifier_apply(modifier=modifier.name)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_animations=False,
        export_yup=True,
    )
    return output_path


def _clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def _mesh_tri_count(obj: bpy.types.Object) -> int:
    mesh = obj.data
    return sum(len(poly.vertices) for poly in mesh.polygons)
