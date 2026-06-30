"""Blender mesh operations for prep_ai_mesh (bpy only)."""

from __future__ import annotations

import math
from pathlib import Path

import bpy
from mathutils import Vector

MERGE_DISTANCE = 0.0005
VOXEL_AUTO_THRESHOLD = 500_000


def clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def import_mesh(path: Path) -> None:
    suffix = path.suffix.lower()
    if suffix in {".glb", ".gltf"}:
        bpy.ops.import_scene.gltf(filepath=str(path))
        return
    if suffix == ".obj":
        if hasattr(bpy.ops.wm, "obj_import"):
            bpy.ops.wm.obj_import(filepath=str(path))
        else:
            bpy.ops.import_scene.obj(filepath=str(path))
        return
    raise ValueError(f"unsupported format: {path.suffix}")


def join_and_clean(meshes: list[bpy.types.Object]) -> bpy.types.Object:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    if len(meshes) > 1:
        bpy.ops.object.join()
    mesh_obj = bpy.context.object
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.remove_doubles(threshold=MERGE_DISTANCE)
    bpy.ops.mesh.delete_loose(use_verts=True, use_edges=True, use_faces=True)
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode="OBJECT")
    return mesh_obj


def apply_z_up(mesh_obj: bpy.types.Object) -> None:
    mesh_obj.rotation_euler = (math.radians(-90.0), 0.0, 0.0)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=False)


def maybe_voxel_remesh(mesh_obj: bpy.types.Object, *, force: bool) -> bool:
    faces = len(mesh_obj.data.polygons)
    if not force and faces <= VOXEL_AUTO_THRESHOLD:
        return False
    _min_x, _max_x, min_y, max_y, _min_z, _max_z = object_bounds(mesh_obj)
    height = max(max_y - min_y, 0.01)
    modifier = mesh_obj.modifiers.new(name="TBGVoxelRemesh", type="REMESH")
    modifier.mode = "VOXEL"
    modifier.voxel_size = height / 180.0
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    return True


def normalize_mesh(mesh_obj: bpy.types.Object, target_height: float) -> None:
    mesh_obj.location = (0.0, 0.0, 0.0)
    mesh_obj.scale = (1.0, 1.0, 1.0)
    bpy.context.view_layer.update()
    _min_x, _max_x, min_y, max_y, _min_z, _max_z = object_bounds(mesh_obj)
    uniform = target_height / max(max_y - min_y, 1e-6)
    mesh_obj.scale = (uniform, uniform, uniform)
    bpy.context.view_layer.update()
    min_x, max_x, min_y, max_y, min_z, max_z = object_bounds(mesh_obj)
    mesh_obj.location = (-(min_x + max_x) * 0.5, -min_y, -(min_z + max_z) * 0.5)
    bpy.context.view_layer.update()
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def shade_smooth(mesh_obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.shade_smooth()


def count_loose_verts(mesh_obj: bpy.types.Object) -> int:
    import bmesh

    bm = bmesh.new()
    bm.from_mesh(mesh_obj.data)
    loose = sum(1 for vert in bm.verts if not vert.link_edges)
    bm.free()
    return loose


def export_glb(mesh_obj: bpy.types.Object, output_path: Path) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_animations=False,
        export_yup=True,
    )


def object_bounds(mesh_obj: bpy.types.Object) -> tuple[float, float, float, float, float, float]:
    corners = [mesh_obj.matrix_world @ Vector(corner) for corner in mesh_obj.bound_box]
    xs = [corner.x for corner in corners]
    ys = [corner.y for corner in corners]
    zs = [corner.z for corner in corners]
    return min(xs), max(xs), min(ys), max(ys), min(zs), max(zs)
