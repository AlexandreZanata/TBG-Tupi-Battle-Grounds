"""glTF ↔ Blender TBG bone name normalization (no bpy)."""

from __future__ import annotations

import re
from typing import Any

_GLTF_SIDE = re.compile(r"^(TBG_.+)(L|R)$")


def gltf_to_blender_bone(name: str) -> str:
    if not name:
        return name
    if ".L" in name or ".R" in name:
        return name
    match = _GLTF_SIDE.match(name)
    return f"{match.group(1)}.{match.group(2)}" if match else name


def normalize_bones_payload(payload: dict[str, Any]) -> dict[str, Any]:
    bones = payload.get("bones") or {}
    normalized: dict[str, Any] = {}
    for name, info in bones.items():
        canon = gltf_to_blender_bone(name)
        parent = info.get("parent")
        normalized[canon] = {
            **info,
            "parent": gltf_to_blender_bone(parent) if parent else None,
        }
    return {**payload, "bones": normalized}
