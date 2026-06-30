#!/usr/bin/env python3
"""Build HumanoidHeavy rig template .blend (run inside Blender).

Usage:
  blender --background --python tools/blender/scripts/build_rig_template.py
"""

from __future__ import annotations

import os

try:
    import bpy
except ImportError as exc:
    raise SystemExit("Run inside Blender") from exc

BODY_CLASS = "HumanoidHeavy"
BONE_CHAIN = (
    ("TBG_Root", None, (0.0, 0.0, 0.0)),
    ("TBG_Spine", "TBG_Root", (0.0, 0.5, 0.0)),
    ("TBG_Spine1", "TBG_Spine", (0.0, 0.4, 0.0)),
    ("TBG_Neck", "TBG_Spine1", (0.0, 0.35, 0.0)),
    ("TBG_Head", "TBG_Neck", (0.0, 0.2, 0.0)),
    ("TBG_Shoulder.L", "TBG_Spine1", (-0.25, 0.3, 0.0)),
    ("TBG_UpperArm.L", "TBG_Shoulder.L", (-0.2, 0.0, 0.0)),
    ("TBG_LowerArm.L", "TBG_UpperArm.L", (-0.25, 0.0, 0.0)),
    ("TBG_Hand.L", "TBG_LowerArm.L", (-0.2, 0.0, 0.0)),
    ("TBG_Shoulder.R", "TBG_Spine1", (0.25, 0.3, 0.0)),
    ("TBG_UpperArm.R", "TBG_Shoulder.R", (0.2, 0.0, 0.0)),
    ("TBG_LowerArm.R", "TBG_UpperArm.R", (0.25, 0.0, 0.0)),
    ("TBG_Hand.R", "TBG_LowerArm.R", (0.2, 0.0, 0.0)),
    ("TBG_UpperLeg.L", "TBG_Spine", (-0.12, -0.1, 0.0)),
    ("TBG_LowerLeg.L", "TBG_UpperLeg.L", (0.0, -0.45, 0.0)),
    ("TBG_Foot.L", "TBG_LowerLeg.L", (0.0, -0.45, 0.0)),
    ("TBG_UpperLeg.R", "TBG_Spine", (0.12, -0.1, 0.0)),
    ("TBG_LowerLeg.R", "TBG_UpperLeg.R", (0.0, -0.45, 0.0)),
    ("TBG_Foot.R", "TBG_LowerLeg.R", (0.0, -0.45, 0.0)),
)


def _script_output_path() -> str:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    templates_dir = os.path.join(script_dir, "..", "templates")
    os.makedirs(templates_dir, exist_ok=True)
    return os.path.abspath(os.path.join(templates_dir, f"{BODY_CLASS}_template.blend"))


def _clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def _create_armature() -> bpy.types.Object:
    bpy.ops.object.armature_add(enter_editmode=True)
    arm_obj = bpy.context.object
    arm_obj.name = f"{BODY_CLASS}_Armature"
    edit_bones = arm_obj.data.edit_bones
    edit_bones.remove(edit_bones[0])

    bone_map: dict[str, bpy.types.EditBone] = {}
    for name, parent_name, head in BONE_CHAIN:
        bone = edit_bones.new(name)
        bone.head = head
        bone.tail = (head[0], head[1] + 0.1, head[2])
        if parent_name and parent_name in bone_map:
            bone.parent = bone_map[parent_name]
        bone_map[name] = bone

    bpy.ops.object.mode_set(mode="OBJECT")
    return arm_obj


def _create_proxy_mesh(arm_obj: bpy.types.Object) -> None:
    bpy.ops.mesh.primitive_cube_add(size=0.5, location=(0.0, 1.0, 0.0))
    mesh_obj = bpy.context.object
    mesh_obj.name = f"{BODY_CLASS}_Body"
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.uv.unwrap()
    bpy.ops.object.mode_set(mode="OBJECT")
    mesh_obj.parent = arm_obj
    mesh_obj.parent_type = "ARMATURE"
    modifier = mesh_obj.modifiers.new(name="Armature", type="ARMATURE")
    modifier.object = arm_obj


def main() -> None:
    _clear_scene()
    arm_obj = _create_armature()
    _create_proxy_mesh(arm_obj)
    output = _script_output_path()
    bpy.ops.wm.save_as_mainfile(filepath=output)
    print(f"Wrote template: {output}")


if __name__ == "__main__":
    main()
