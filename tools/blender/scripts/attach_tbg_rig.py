#!/usr/bin/env python3
"""Attach TBG body-class armature to a low-poly mesh (Blender bpy)."""

from __future__ import annotations

from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("attach_tbg_rig requires Blender bpy") from exc

try:
    from mathutils import Vector
except ImportError as exc:
    raise SystemExit("attach_tbg_rig requires mathutils") from exc

TEMPLATE_DIR = Path(__file__).resolve().parents[1] / "templates"


def attach_tbg_rig(low_mesh_path: Path, body_class: str, output_path: Path) -> Path:
    template = TEMPLATE_DIR / f"{body_class}_template.blend"
    if not template.is_file():
        raise FileNotFoundError(
            f"missing rig template: {template} — run tools/blender/setup.sh"
        )

    bpy.ops.wm.open_mainfile(filepath=str(template))
    template_body = f"{body_class}_Body"
    template_mesh = bpy.data.objects.get(template_body)
    if template_mesh is not None:
        bpy.data.objects.remove(template_mesh, do_unlink=True)

    bpy.ops.import_scene.gltf(filepath=str(low_mesh_path))

    imported = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    armatures = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    if not imported or not armatures:
        raise RuntimeError("expected imported mesh and template armature")

    mesh_obj = _join_meshes(imported)
    arm_obj = armatures[0]
    _align_armature_to_mesh(arm_obj, mesh_obj)
    _bind_mesh_to_armature(mesh_obj, arm_obj)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.export_scene.gltf(
        filepath=str(output_path),
        export_format="GLB",
        use_selection=True,
        export_apply=False,
        export_skins=True,
        export_animations=False,
        export_yup=True,
    )
    return output_path


def _join_meshes(meshes: list[bpy.types.Object]) -> bpy.types.Object:
    if len(meshes) == 1:
        return meshes[0]
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    return bpy.context.object


def _mesh_world_bounds(mesh_obj: bpy.types.Object) -> tuple[float, float, float, float, float, float]:
    corners = [mesh_obj.matrix_world @ Vector(corner) for corner in mesh_obj.bound_box]
    xs = [corner.x for corner in corners]
    ys = [corner.y for corner in corners]
    zs = [corner.z for corner in corners]
    return min(xs), max(xs), min(ys), max(ys), min(zs), max(zs)


def _align_armature_to_mesh(arm_obj: bpy.types.Object, mesh_obj: bpy.types.Object) -> None:
    """Scale and position armature to overlap the decimated mesh bounds."""
    min_x, max_x, min_y, max_y, min_z, max_z = _mesh_world_bounds(mesh_obj)
    mesh_height = max(max_y - min_y, 0.01)
    mesh_center_x = (min_x + max_x) * 0.5
    mesh_center_z = (min_z + max_z) * 0.5

    arm_obj.scale = (1.0, 1.0, 1.0)
    arm_obj.rotation_euler = (0.0, 0.0, 0.0)
    arm_height = max(arm_obj.dimensions.y, 0.01)
    scale = mesh_height / arm_height
    arm_obj.scale = (scale, scale, scale)
    bpy.context.view_layer.update()

    arm_obj.location = (mesh_center_x, min_y, mesh_center_z)
    bpy.context.view_layer.update()
    _apply_object_transforms(arm_obj)


def _apply_object_transforms(obj: bpy.types.Object) -> None:
    """Apply object transforms so glTF skin bind matrices stay consistent."""
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def _bind_mesh_to_armature(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object) -> None:
    if mesh_obj.parent is not None:
        mesh_obj.parent = None

    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_obj

    try:
        bpy.ops.object.parent_set(type="ARMATURE_AUTO", keep_transform=True)
    except RuntimeError:
        bpy.ops.object.parent_set(type="ARMATURE", keep_transform=True)

    if mesh_obj.vertex_groups:
        mesh_obj.parent = arm_obj
        return

    _assign_envelope_weights(mesh_obj, arm_obj)
    mesh_obj.parent = arm_obj


def _assign_envelope_weights(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object) -> None:
    """Fallback when ARMATURE_AUTO fails — bind all verts to nearest spine chain bone."""
    mesh_obj.modifiers.new(name="Armature", type="ARMATURE").object = arm_obj
    bone_names = [bone.name for bone in arm_obj.data.bones]
    spine = next((name for name in bone_names if "Spine" in name), bone_names[0])
    group = mesh_obj.vertex_groups.new(name=spine)
    group.add(list(range(len(mesh_obj.data.vertices))), 1.0, "REPLACE")
