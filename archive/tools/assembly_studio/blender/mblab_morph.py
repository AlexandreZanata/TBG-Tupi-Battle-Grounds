"""Apply MB-Lab body meta + head structural morphs on an open character scene."""

from __future__ import annotations

from typing import Any

try:
    import bpy
except ImportError:
    bpy = None  # type: ignore[assignment]

from mblab_enable import ensure_mblab_addon
from mblab_head_morphs import apply_head_morphs, head_morph_ids

BODY_META = frozenset({"age", "body_mass", "muscles", "stature"})


def _humanoid_mesh() -> Any | None:
    if bpy is None:
        return None
    for obj in bpy.context.scene.objects:
        if obj.type == "MESH" and obj.get("manuellab_id"):
            return obj
    return next((o for o in bpy.context.scene.objects if o.type == "MESH"), None)


def _humanoid_engine() -> Any | None:
    import importlib

    if not ensure_mblab_addon():
        return None
    mblab = importlib.import_module("MB-Lab")
    ho = getattr(mblab, "mblab_humanoid", None)
    if ho is not None and getattr(ho, "has_data", False):
        return ho
    if hasattr(mblab, "start_lab_session"):
        mblab.start_lab_session()
    return getattr(mblab, "mblab_humanoid", None)


def _ui_to_meta(value: float) -> float:
    return max(-1.0, min(1.0, float(value) * 2.0 - 1.0))


def apply_morphs(morphs: dict[str, float]) -> dict[str, Any]:
    if not morphs:
        return {"applied": [], "skipped": []}
    mesh = _humanoid_mesh()
    ho = _humanoid_engine()
    if mesh is None or ho is None or not getattr(ho, "has_data", False):
        return {"applied": [], "skipped": list(morphs.keys()), "error": "no MB-Lab session"}

    applied: list[str] = []
    head_ids = head_morph_ids()
    head_morphs = {k: v for k, v in morphs.items() if k in head_ids}
    body_morphs = {k: v for k, v in morphs.items() if k in BODY_META}

    if "age" in body_morphs:
        mesh.character_age = _ui_to_meta(body_morphs["age"])
        applied.append("age")
    if "body_mass" in body_morphs:
        mesh.character_mass = _ui_to_meta(body_morphs["body_mass"])
        applied.append("body_mass")
    if "muscles" in body_morphs:
        mesh.character_tone = _ui_to_meta(body_morphs["muscles"])
        applied.append("muscles")

    if any(k in body_morphs for k in ("age", "body_mass", "muscles")):
        ho.calculate_transformation("AGE")
        ho.calculate_transformation("FAT")
        ho.calculate_transformation("MUSCLE")

    applied.extend(apply_head_morphs(ho, head_morphs))
    ho.update_character(mode="update_all")

    if "stature" in body_morphs:
        scale = 0.82 + float(body_morphs["stature"]) * 0.36
        root = _humanoid_mesh()
        if root is not None:
            root.scale = (scale, scale, scale)
            applied.append("stature")

    skipped = sorted(set(morphs) - set(applied))
    return {"applied": applied, "skipped": skipped, "headMorphs": list(head_morphs.keys())}
