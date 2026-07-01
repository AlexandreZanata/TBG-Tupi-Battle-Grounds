#!/usr/bin/env python3
"""Rename MB-Lab base skeleton bones to TBG_* (run inside Blender)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("rename_mblab_bones requires Blender bpy") from exc

MAP_PATH = Path(__file__).resolve().parent / "mblab_bone_map.json"


def _load_map() -> dict[str, str]:
    data = json.loads(MAP_PATH.read_text(encoding="utf-8"))
    return {str(k): str(v) for k, v in data.get("mapping", {}).items()}


def rename_armature_bones(arm_obj: bpy.types.Object) -> list[str]:
    mapping = _load_map()
    renamed: list[str] = []
    for bone in arm_obj.data.bones:
        target = mapping.get(bone.name)
        if target and bone.name != target:
            bone.name = target
            renamed.append(target)
    return renamed


def main(argv: list[str] | None = None) -> int:
    args = argv if argv is not None else sys.argv
    if "--" in args:
        args = args[args.index("--") + 1 :]
    arm_name = args[0] if args else None
    arm_obj = bpy.data.objects.get(arm_name) if arm_name else None
    if arm_obj is None:
        arm_obj = next((o for o in bpy.context.scene.objects if o.type == "ARMATURE"), None)
    if arm_obj is None:
        raise SystemExit("no armature found — select MB-Lab character first")
    renamed = rename_armature_bones(arm_obj)
    print(json.dumps({"armature": arm_obj.name, "renamed": len(renamed)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
