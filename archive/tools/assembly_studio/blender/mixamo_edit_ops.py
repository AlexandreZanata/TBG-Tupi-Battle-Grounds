"""Mixamo Edit — align Mixamo rig inside custom skin, TBG bones, export clips."""

from __future__ import annotations

from pathlib import Path

import bpy
from mathutils import Vector

from import_mixamo_ops import (
    clear_scene,
    export_skinned_glb,
    import_mixamo_fbx,
    import_mixamo_fbx_incremental,
    retarget_mixamo_to_tbg,
)
from mixamo_skin_bind import skin_custom_mesh
from mixamo_bone_proportions import apply_bone_proportions
from mixamo_edit_export import prepare_skinned_export


def build_mixamo_edit(
    mixamo_fbx: Path,
    output_glb: Path,
    *,
    custom_mesh: Path | None = None,
    height_scale: float = 1.0,
    rig_position: tuple[float, float, float] = (0.0, 0.0, 0.0),
    rig_scale: float = 1.0,
    manual_align: bool = False,
    bone_proportions: dict | None = None,
) -> dict:
    if custom_mesh and custom_mesh.is_file() and manual_align:
        return _build_manual_align(
            mixamo_fbx, output_glb, custom_mesh, rig_position, rig_scale, height_scale, bone_proportions
        )
    mesh_obj, arm_obj = import_mixamo_fbx(mixamo_fbx)
    renamed = retarget_mixamo_to_tbg(arm_obj, mesh_obj)
    if custom_mesh and custom_mesh.is_file():
        custom = _import_glb_mesh(custom_mesh)
        _align_mesh_to_reference(custom, mesh_obj, height_scale)
        skin_custom_mesh(custom, mesh_obj, arm_obj)
        bpy.data.objects.remove(mesh_obj, do_unlink=True)
        mesh_obj = custom
    export_skinned_glb(mesh_obj, arm_obj, output_glb, export_animations=True)
    return _report(output_glb, renamed, arm_obj, custom_mesh, height_scale, False)


def _build_manual_align(
    mixamo_fbx: Path,
    output_glb: Path,
    custom_mesh: Path,
    rig_position: tuple[float, float, float],
    rig_scale: float,
    height_scale: float,
    bone_proportions: dict | None,
) -> dict:
    clear_scene()
    custom = _import_glb_mesh(custom_mesh)
    _normalize_height(custom)
    _apply_transforms(custom)
    _ground_object(custom)
    mixamo_mesh, arm_obj = import_mixamo_fbx_incremental(mixamo_fbx)
    props = apply_bone_proportions(arm_obj, bone_proportions)
    renamed = retarget_mixamo_to_tbg(arm_obj, mixamo_mesh)
    _normalize_rig(mixamo_mesh, arm_obj)
    _ground_object(mixamo_mesh)
    _align_mesh_to_reference(custom, mixamo_mesh, 1.0)
    skin_custom_mesh(custom, mixamo_mesh, arm_obj)
    bpy.data.objects.remove(mixamo_mesh, do_unlink=True)
    scale = max(0.25, min(3.0, rig_scale * height_scale))
    _apply_rig_transform(arm_obj, custom, rig_position, scale)
    prepare_skinned_export(arm_obj, custom)
    export_skinned_glb(custom, arm_obj, output_glb, export_animations=True)
    return _report(
        output_glb, renamed, arm_obj, custom_mesh, height_scale, True, rig_position, scale, props
    )


def _report(output, renamed, arm_obj, custom_mesh, height_scale, manual, rig_pos=(0, 0, 0), rig_scale=1.0, bone_props=None):
    clips = []
    if arm_obj.animation_data and arm_obj.animation_data.action:
        clips = [arm_obj.animation_data.action.name]
    return {
        "output": str(output),
        "bonesRenamed": len(renamed),
        "clipNames": clips,
        "customMesh": bool(custom_mesh),
        "heightScale": height_scale,
        "manualAlign": manual,
        "rigPosition": list(rig_pos),
        "rigScale": rig_scale,
        "boneProportions": bone_props or {},
    }


def _import_glb_mesh(path: Path) -> bpy.types.Object:
    before = {obj.name for obj in bpy.context.scene.objects}
    bpy.ops.import_scene.gltf(filepath=str(path))
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH" and o.name not in before]
    if not meshes:
        raise RuntimeError(f"no mesh in GLB: {path}")
    if len(meshes) == 1:
        return meshes[0]
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    return bpy.context.object


TARGET_HEIGHT = 1.8


def _bbox_height(obj: bpy.types.Object) -> float:
    corners = [obj.matrix_world @ Vector(c) for c in obj.bound_box]
    ys = [p.y for p in corners]
    return max(ys) - min(ys)


def _scale_to_height(obj: bpy.types.Object, target: float = TARGET_HEIGHT) -> None:
    h = _bbox_height(obj)
    if h < 1e-4:
        return
    s = target / h
    if abs(s - 1.0) < 0.05:
        return
    obj.scale = tuple(v * s for v in obj.scale)
    bpy.context.view_layer.update()


def _normalize_height(obj: bpy.types.Object, target: float = TARGET_HEIGHT) -> None:
    _scale_to_height(obj, target)


def _normalize_rig(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object, target: float = TARGET_HEIGHT) -> None:
    if mesh_obj.parent == arm_obj:
        mw = mesh_obj.matrix_world.copy()
        mesh_obj.parent = None
        mesh_obj.matrix_world = mw
    h = _bbox_height(mesh_obj)
    if h < 1e-4:
        return
    s = target / h
    if abs(s - 1.0) < 0.05:
        return
    for obj in (mesh_obj, arm_obj):
        obj.scale = tuple(v * s for v in obj.scale)
    bpy.context.view_layer.update()
    _apply_transforms(mesh_obj)
    _apply_transforms(arm_obj)


def _apply_transforms(obj: bpy.types.Object) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def _ground_object(obj: bpy.types.Object) -> None:
    corners = [obj.matrix_world @ Vector(c) for c in obj.bound_box]
    obj.location.y -= min(p.y for p in corners)


def _apply_rig_transform(arm_obj, mesh_obj, offset, scale):
    bpy.ops.object.empty_add(type="PLAIN_AXES", location=Vector(offset))
    pivot = bpy.context.object
    pivot.name = "TBG_RigPivot"
    pivot.scale = (scale, scale, scale)
    for obj in (arm_obj, mesh_obj):
        mw = obj.matrix_world.copy()
        obj.parent = pivot
        obj.matrix_world = mw


def _align_mesh_to_reference(custom, reference, height_scale):
    def bbox(o):
        cs = [o.matrix_world @ Vector(c) for c in o.bound_box]
        xs, ys, zs = [p.x for p in cs], [p.y for p in cs], [p.z for p in cs]
        return Vector((min(xs), min(ys), min(zs))), Vector((max(xs), max(ys), max(zs)))

    ref_min, ref_max = bbox(reference)
    cur_min, cur_max = bbox(custom)
    ref_h = max(ref_max.y - ref_min.y, 1e-6)
    cur_h = max(cur_max.y - cur_min.y, 1e-6)
    s = (ref_h / cur_h) * max(0.25, min(3.0, height_scale))
    custom.scale = (s, s, s)
    bpy.context.view_layer.update()
    cur_min, cur_max = bbox(custom)
    rc, cc = (ref_min + ref_max) * 0.5, (cur_min + cur_max) * 0.5
    custom.location += Vector((rc.x - cc.x, ref_min.y - cur_min.y, rc.z - cc.z))
    bpy.ops.object.select_all(action="DESELECT")
    custom.select_set(True)
    bpy.context.view_layer.objects.active = custom
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
