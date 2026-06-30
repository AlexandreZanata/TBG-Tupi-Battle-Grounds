"""Scene validation using bpy — authoring only."""

from __future__ import annotations

from .validation_rules import MAX_TRIS_MVP, require_armature, require_uv, tri_count_warning

try:
    import bpy
except ImportError:  # pragma: no cover - bpy only inside Blender
    bpy = None


def validate_scene(scene) -> list[str]:
    if bpy is None:
        raise RuntimeError("validate_scene requires Blender bpy module")

    warnings: list[str] = []
    mesh_objects = [obj for obj in scene.objects if obj.type == "MESH"]
    armatures = [obj for obj in scene.objects if obj.type == "ARMATURE"]

    armature_warning = require_armature(len(armatures) > 0)
    if armature_warning:
        warnings.append(armature_warning)

    total_tris = 0
    for obj in mesh_objects:
        mesh = obj.data
        has_uv = bool(mesh.uv_layers)
        uv_warning = require_uv(has_uv, obj.name)
        if uv_warning:
            warnings.append(uv_warning)
        total_tris += _mesh_tri_count(mesh)

    tri_warning = tri_count_warning(total_tris, MAX_TRIS_MVP)
    if tri_warning:
        warnings.append(tri_warning)

    return warnings


def _mesh_tri_count(mesh) -> int:
    if hasattr(mesh, "loop_triangles"):
        return len(mesh.loop_triangles)
    return sum(max(0, len(face.vertices) - 2) for face in mesh.polygons)
