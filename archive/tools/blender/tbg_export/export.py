"""glTF binary export preset for TBG units."""

from __future__ import annotations

import os

try:
    import bpy
except ImportError:  # pragma: no cover
    bpy = None


def repo_root_from_blend() -> str:
    if bpy is None:
        raise RuntimeError("export requires Blender bpy module")
    blend_path = bpy.data.filepath
    if blend_path:
        # tools/blender/templates/*.blend -> repo root is 3 levels up
        return os.path.abspath(os.path.join(os.path.dirname(blend_path), "..", "..", ".."))
    return os.path.abspath(os.path.join(os.getcwd(), "..", ".."))


def export_unit_glb(unit_id: str, output_dir: str | None = None) -> str:
    if bpy is None:
        raise RuntimeError("export_unit_glb requires Blender bpy module")

    repo_root = repo_root_from_blend()
    meshes_dir = output_dir or os.path.join(repo_root, "content", "characters", "meshes")
    os.makedirs(meshes_dir, exist_ok=True)
    filepath = os.path.join(meshes_dir, f"{unit_id}.glb")

    export_objects = _collect_export_objects()
    if not export_objects:
        raise RuntimeError("no mesh or armature objects to export")

    _select_objects(export_objects)
    bpy.ops.export_scene.gltf(
        filepath=filepath,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_animations=True,
        export_yup=True,
    )
    return filepath


def _collect_export_objects() -> list:
    allowed = {"MESH", "ARMATURE"}
    return [obj for obj in bpy.context.scene.objects if obj.type in allowed]


def _select_objects(objects: list) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
