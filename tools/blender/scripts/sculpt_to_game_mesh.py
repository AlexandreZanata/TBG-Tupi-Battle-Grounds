#!/usr/bin/env python3
"""Decimate showcase GLB to mobile tri budget (Blender bpy)."""

from __future__ import annotations

from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("sculpt_to_game_mesh requires Blender bpy") from exc

TRI_TARGET = 8000
TEXTURE_SIZE = 1024
MAX_DECIMATE_PASSES = 12


def sculpt_to_game_mesh(
    showcase_path: Path,
    output_path: Path,
    tri_target: int = TRI_TARGET,
    texture_size: int = TEXTURE_SIZE,
) -> Path:
    _clear_scene()
    bpy.ops.import_scene.gltf(filepath=str(showcase_path))
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError("showcase import produced no meshes")

    for _ in range(MAX_DECIMATE_PASSES):
        if _total_tri_count(meshes) <= tri_target:
            break
        ratio = min(0.5, tri_target / max(1, _total_tri_count(meshes)))
        for obj in meshes:
            bpy.context.view_layer.objects.active = obj
            modifier = obj.modifiers.new(name="TBGDecimate", type="DECIMATE")
            modifier.ratio = ratio
            bpy.ops.object.modifier_apply(modifier=modifier.name)

    _resize_embedded_textures(texture_size)
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


def _resize_embedded_textures(texture_size: int) -> None:
    for image in bpy.data.images:
        if image.size[0] <= texture_size and image.size[1] <= texture_size:
            continue
        image.scale(texture_size, texture_size)


def _clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def _total_tri_count(meshes: list) -> int:
    total = 0
    for obj in meshes:
        total += sum(len(poly.vertices) for poly in obj.data.polygons)
    return total

