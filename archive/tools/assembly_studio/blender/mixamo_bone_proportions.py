"""Rest-pose Mixamo bone proportion tweaks before skin bind."""

from __future__ import annotations

import bpy

_RULES = (
    ("hipWidth", ("LeftUpLeg", "RightUpLeg", "LeftLeg", "RightLeg"), "x"),
    ("shoulderWidth", ("LeftShoulder", "RightShoulder"), "x"),
    ("torsoDepth", ("Hips", "Spine", "Spine1", "Spine2"), "z"),
)


def _short(name: str) -> str:
    n = name
    if n.startswith("mixamorig"):
        n = n.removeprefix("mixamorig").lstrip("_:")
    return n


def _find_edit_bone(edit_bones, canonical: str):
    for eb in edit_bones:
        if _short(eb.name) == canonical:
            return eb
    return None


def _clamp(v: float) -> float:
    return max(0.5, min(2.0, float(v)))


def apply_bone_proportions(arm_obj: bpy.types.Object, props: dict | None) -> dict:
    props = props or {}
    applied = {key: _clamp(props.get(key, 1.0)) for key, _, _ in _RULES}
    if all(abs(v - 1.0) < 1e-4 for v in applied.values()):
        return applied

    bpy.ops.object.select_all(action="DESELECT")
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")
    edit = arm_obj.data.edit_bones
    for key, bones, axis in _RULES:
        factor = applied[key]
        if abs(factor - 1.0) < 1e-4:
            continue
        for canonical in bones:
            eb = _find_edit_bone(edit, canonical)
            if not eb:
                continue
            if axis == "x":
                eb.head.x *= factor
                eb.tail.x *= factor
            else:
                eb.head.z *= factor
                eb.tail.z *= factor
    bpy.ops.object.mode_set(mode="OBJECT")
    return applied
