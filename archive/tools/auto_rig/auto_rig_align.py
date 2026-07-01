"""Scale and translate a TBG template armature to match a normalized mesh."""

from __future__ import annotations

try:
    import bpy
except ImportError as exc:
    raise SystemExit("auto_rig_align requires Blender bpy") from exc

from mathutils import Vector


def mesh_world_bounds(mesh_obj: bpy.types.Object) -> tuple[float, float, float, float, float, float]:
    corners = [mesh_obj.matrix_world @ Vector(corner) for corner in mesh_obj.bound_box]
    xs = [corner.x for corner in corners]
    ys = [corner.y for corner in corners]
    zs = [corner.z for corner in corners]
    return min(xs), max(xs), min(ys), max(ys), min(zs), max(zs)


def armature_world_bounds(arm_obj: bpy.types.Object) -> tuple[float, float, float, float, float, float]:
    points: list[Vector] = []
    for bone in arm_obj.data.bones:
        points.append(arm_obj.matrix_world @ bone.head_local)
        points.append(arm_obj.matrix_world @ bone.tail_local)
    xs = [point.x for point in points]
    ys = [point.y for point in points]
    zs = [point.z for point in points]
    return min(xs), max(xs), min(ys), max(ys), min(zs), max(zs)


def apply_object_transforms(obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def align_armature_to_mesh(arm_obj: bpy.types.Object, mesh_obj: bpy.types.Object) -> None:
    min_x, max_x, min_y, max_y, min_z, max_z = mesh_world_bounds(mesh_obj)
    arm_obj.scale = (1.0, 1.0, 1.0)
    arm_obj.rotation_euler = (0.0, 0.0, 0.0)
    arm_obj.location = (0.0, 0.0, 0.0)
    bpy.context.view_layer.update()

    mesh_height = max(max_y - min_y, 0.01)
    _arm_min_x, _arm_max_x, arm_min_y, arm_max_y, _arm_min_z, _arm_max_z = armature_world_bounds(arm_obj)
    arm_height = max(arm_max_y - arm_min_y, 0.01)
    uniform = mesh_height / arm_height
    arm_obj.scale = (uniform, uniform, uniform)
    bpy.context.view_layer.update()

    arm_min_x, arm_max_x, arm_min_y, arm_max_y, arm_min_z, arm_max_z = armature_world_bounds(arm_obj)
    mesh_center_x = (min_x + max_x) * 0.5
    mesh_center_z = (min_z + max_z) * 0.5
    arm_center_x = (arm_min_x + arm_max_x) * 0.5
    arm_center_z = (arm_min_z + arm_max_z) * 0.5
    arm_obj.location = (
        mesh_center_x - arm_center_x,
        min_y - arm_min_y,
        mesh_center_z - arm_center_z,
    )
    bpy.context.view_layer.update()
    apply_object_transforms(arm_obj)


def widen_bone_envelopes(arm_obj: bpy.types.Object, mesh_obj: bpy.types.Object) -> None:
    _min_x, _max_x, min_y, max_y, _min_z, _max_z = mesh_world_bounds(mesh_obj)
    radius = max(max_y - min_y, 0.01) * 0.12
    for bone in arm_obj.data.bones:
        bone.envelope_distance = radius
