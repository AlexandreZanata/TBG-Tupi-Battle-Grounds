#!/usr/bin/env python3
"""Build body-class rig template .blend files (run inside Blender).

Usage:
  blender --background --python tools/blender/scripts/build_rig_template.py
"""

from __future__ import annotations

import os
import sys

try:
    import bpy
except ImportError as exc:
    raise SystemExit("Run inside Blender") from exc

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
DEFAULT_BODY_CLASSES = ("HumanoidHeavy", "HumanoidLight")


def _templates_dir() -> str:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    path = os.path.abspath(os.path.join(script_dir, "..", "templates"))
    os.makedirs(path, exist_ok=True)
    return path


def _clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def _create_armature(body_class: str) -> bpy.types.Object:
    bpy.ops.object.armature_add(enter_editmode=True)
    arm_obj = bpy.context.object
    arm_obj.name = f"{body_class}_Armature"
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


def _create_proxy_mesh(body_class: str, arm_obj: bpy.types.Object) -> None:
    scale = 0.9 if body_class == "HumanoidLight" else 1.0
    bpy.ops.mesh.primitive_cube_add(size=0.5 * scale, location=(0.0, 1.0, 0.0))
    mesh_obj = bpy.context.object
    mesh_obj.name = f"{body_class}_Body"
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.uv.unwrap()
    bpy.ops.object.mode_set(mode="OBJECT")
    mesh_obj.parent = arm_obj
    mesh_obj.parent_type = "ARMATURE"
    modifier = mesh_obj.modifiers.new(name="Armature", type="ARMATURE")
    modifier.object = arm_obj


def build_template(body_class: str) -> str:
    _clear_scene()
    arm_obj = _create_armature(body_class)
    _create_proxy_mesh(body_class, arm_obj)
    output = os.path.join(_templates_dir(), f"{body_class}_template.blend")
    bpy.ops.wm.save_as_mainfile(filepath=output)
    return output


def _parse_body_classes(argv: list[str]) -> tuple[str, ...]:
    if "--" in argv:
        args = argv[argv.index("--") + 1 :]
        if args:
            return tuple(args)
    return DEFAULT_BODY_CLASSES


def main(argv: list[str] | None = None) -> None:
    for body_class in _parse_body_classes(argv if argv is not None else sys.argv):
        output = build_template(body_class)
        print(f"Wrote template: {output}")


if __name__ == "__main__":
    main()
