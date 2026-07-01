"""Rig Studio bone JSON sync for browser ↔ Blender."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

try:
    import bpy
except ImportError as exc:
    raise SystemExit("rig_studio_bones requires Blender bpy") from exc

from mathutils import Vector

from rig_bone_names import normalize_bones_payload


def export_bones_payload(arm_obj: bpy.types.Object) -> dict[str, Any]:
    bones: dict[str, Any] = {}
    for bone in arm_obj.data.bones:
        if not bone.name.startswith("TBG_"):
            continue
        bones[bone.name] = {
            "head": [round(v, 6) for v in bone.head_local],
            "tail": [round(v, 6) for v in bone.tail_local],
            "parent": bone.parent.name if bone.parent else None,
        }
    return {"version": 1, "armature": arm_obj.name, "bones": bones}


def apply_bones_payload(arm_obj: bpy.types.Object, payload: dict[str, Any]) -> int:
    data = normalize_bones_payload(payload).get("bones") or {}
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")
    edit_bones = arm_obj.data.edit_bones
    updated = 0
    for name, info in data.items():
        if name not in edit_bones:
            continue
        bone = edit_bones[name]
        bone.head = Vector(info["head"])
        bone.tail = Vector(info["tail"])
        updated += 1
    bpy.ops.object.mode_set(mode="OBJECT")
    return updated


def write_bones_json(arm_obj: bpy.types.Object, path: Path) -> None:
    path.write_text(json.dumps(export_bones_payload(arm_obj), indent=2), encoding="utf-8")


def read_bones_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))
