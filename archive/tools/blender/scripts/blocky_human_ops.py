"""Minecraft-style block human aligned to anatomical TBG bones (walk study)."""

from __future__ import annotations

import math
from dataclasses import dataclass

try:
    import bpy
except ImportError as exc:
    raise SystemExit("blocky_human_ops requires Blender bpy") from exc

try:
    from mathutils import Vector
except ImportError as exc:
    raise SystemExit("blocky_human_ops requires mathutils") from exc

BODY_CLASS = "HumanoidLight"
TOTAL_HEIGHT_M = 1.75

# head, tail in armature space — Y-up, feet at Y=0, +Z forward
ANATOMICAL_BONES: tuple[tuple[str, str | None, tuple[float, float, float], tuple[float, float, float]], ...] = (
    ("TBG_Root", None, (0.0, 0.0, 0.0), (0.0, 0.72, 0.0)),
    ("TBG_Spine", "TBG_Root", (0.0, 0.72, 0.0), (0.0, 0.92, 0.0)),
    ("TBG_Spine1", "TBG_Spine", (0.0, 0.92, 0.0), (0.0, 1.32, 0.0)),
    ("TBG_Neck", "TBG_Spine1", (0.0, 1.32, 0.0), (0.0, 1.42, 0.0)),
    ("TBG_Head", "TBG_Neck", (0.0, 1.42, 0.0), (0.0, 1.75, 0.0)),
    ("TBG_Shoulder.L", "TBG_Spine1", (-0.08, 1.26, 0.0), (-0.22, 1.26, 0.0)),
    ("TBG_UpperArm.L", "TBG_Shoulder.L", (-0.22, 1.26, 0.0), (-0.22, 0.98, 0.0)),
    ("TBG_LowerArm.L", "TBG_UpperArm.L", (-0.22, 0.98, 0.0), (-0.22, 0.72, 0.0)),
    ("TBG_Hand.L", "TBG_LowerArm.L", (-0.22, 0.72, 0.0), (-0.22, 0.60, 0.0)),
    ("TBG_Shoulder.R", "TBG_Spine1", (0.08, 1.26, 0.0), (0.22, 1.26, 0.0)),
    ("TBG_UpperArm.R", "TBG_Shoulder.R", (0.22, 1.26, 0.0), (0.22, 0.98, 0.0)),
    ("TBG_LowerArm.R", "TBG_UpperArm.R", (0.22, 0.98, 0.0), (0.22, 0.72, 0.0)),
    ("TBG_Hand.R", "TBG_LowerArm.R", (0.22, 0.72, 0.0), (0.22, 0.60, 0.0)),
    ("TBG_UpperLeg.L", "TBG_Spine", (-0.11, 0.72, 0.0), (-0.11, 0.52, 0.0)),
    ("TBG_LowerLeg.L", "TBG_UpperLeg.L", (-0.11, 0.52, 0.0), (-0.11, 0.12, 0.0)),
    ("TBG_Foot.L", "TBG_LowerLeg.L", (-0.11, 0.12, 0.0), (-0.11, 0.04, 0.12)),
    ("TBG_Toe.L", "TBG_Foot.L", (-0.11, 0.04, 0.12), (-0.11, 0.04, 0.20)),
    ("TBG_UpperLeg.R", "TBG_Spine", (0.11, 0.72, 0.0), (0.11, 0.52, 0.0)),
    ("TBG_LowerLeg.R", "TBG_UpperLeg.R", (0.11, 0.52, 0.0), (0.11, 0.12, 0.0)),
    ("TBG_Foot.R", "TBG_LowerLeg.R", (0.11, 0.12, 0.0), (0.11, 0.04, 0.12)),
    ("TBG_Toe.R", "TBG_Foot.R", (0.11, 0.04, 0.12), (0.11, 0.04, 0.20)),
)

SKIP_CUBE_BONES = frozenset({"TBG_Root", "TBG_Toe.L", "TBG_Toe.R", "TBG_Shoulder.L", "TBG_Shoulder.R"})


@dataclass(frozen=True)
class BlockColor:
    bone: str
    rgb: tuple[float, float, float]


BONE_COLORS: dict[str, tuple[float, float, float]] = {
    "TBG_Head": (0.82, 0.62, 0.45),
    "TBG_Spine1": (0.0, 0.55, 0.78),
    "TBG_Spine": (0.0, 0.45, 0.68),
    "TBG_UpperLeg.L": (0.12, 0.12, 0.55),
    "TBG_LowerLeg.L": (0.12, 0.12, 0.55),
    "TBG_Foot.L": (0.18, 0.18, 0.22),
    "TBG_UpperLeg.R": (0.12, 0.12, 0.55),
    "TBG_LowerLeg.R": (0.12, 0.12, 0.55),
    "TBG_Foot.R": (0.18, 0.18, 0.22),
    "TBG_UpperArm.L": (0.82, 0.62, 0.45),
    "TBG_LowerArm.L": (0.82, 0.62, 0.45),
    "TBG_Hand.L": (0.82, 0.62, 0.45),
    "TBG_UpperArm.R": (0.82, 0.62, 0.45),
    "TBG_LowerArm.R": (0.82, 0.62, 0.45),
    "TBG_Hand.R": (0.82, 0.62, 0.45),
    "TBG_Neck": (0.82, 0.62, 0.45),
}


def rebuild_anatomical_armature(arm_obj: bpy.types.Object) -> None:
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")
    edit = arm_obj.data.edit_bones
    for bone in list(edit):
        edit.remove(bone)
    bone_map: dict[str, bpy.types.EditBone] = {}
    for name, parent, head, tail in ANATOMICAL_BONES:
        bone = edit.new(name)
        bone.head = head
        bone.tail = tail
        if parent and parent in bone_map:
            bone.parent = bone_map[parent]
        bone_map[name] = bone
    bpy.ops.object.mode_set(mode="OBJECT")


def create_blocky_human(arm_obj: bpy.types.Object) -> bpy.types.Object:
    rebuild_anatomical_armature(arm_obj)
    parts: list[bpy.types.Object] = []
    for bone_name in BONE_COLORS:
        if bone_name in SKIP_CUBE_BONES:
            continue
        parts.append(_cube_on_bone(arm_obj, bone_name, BONE_COLORS[bone_name]))
    mesh_obj = _join_meshes(parts)
    mesh_obj.name = "BlockyHuman_Body"
    mod = mesh_obj.modifiers.new("Armature", "ARMATURE")
    mod.object = arm_obj
    return mesh_obj


def prepare_blocky_for_gltf_export(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object) -> None:
    """Parent mesh to armature and rotate bind pose for TBG Y-up game GLB (matches attach_tbg_rig)."""
    mesh_obj.parent = arm_obj
    mesh_obj.rotation_euler = (math.radians(-90.0), 0.0, 0.0)
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    bpy.ops.object.transform_apply(rotation=True)


def _cube_on_bone(arm_obj: bpy.types.Object, bone_name: str, color: tuple[float, float, float]) -> bpy.types.Object:
    bone = arm_obj.data.bones[bone_name]
    head = Vector(bone.head_local)
    tail = Vector(bone.tail_local)
    direction = tail - head
    length = max(direction.length, 0.05)
    center = head + direction * 0.5
    width = length * (0.72 if "Head" in bone_name else 0.58)

    bpy.ops.mesh.primitive_cube_add(location=center)
    obj = bpy.context.active_object
    obj.name = bone_name.replace("TBG_", "")
    if direction.length_squared > 1e-8:
        obj.rotation_euler = direction.to_track_quat("Y", "Z").to_euler()
    obj.dimensions = (width, length, width * 0.85)

    mat = bpy.data.materials.new(name=f"{obj.name}_Mat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (*color, 1.0)
        bsdf.inputs["Roughness"].default_value = 0.85
    obj.data.materials.append(mat)

    group = obj.vertex_groups.new(name=bone_name)
    group.add(list(range(len(obj.data.vertices))), 1.0, "REPLACE")
    return obj


def _join_meshes(meshes: list[bpy.types.Object]) -> bpy.types.Object:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    return bpy.context.active_object
