"""Bake tbg_{slider}_min/_max morph targets for MB-Lab studio sliders."""

from __future__ import annotations

from typing import Any

try:
    import bpy
except ImportError:
    bpy = None  # type: ignore[assignment]

from mblab_morph import apply_morphs


def neutral_morphs(sliders: list[dict[str, Any]]) -> dict[str, float]:
    return {s["id"]: 0.5 for s in sliders}


def ensure_basis(mesh_obj: Any) -> None:
    if mesh_obj.data.shape_keys is None:
        mesh_obj.shape_key_add(name="Basis", from_mix=False)


def restore_mesh_positions(mesh_obj: Any, coords: list[Any]) -> None:
    for idx, vert in enumerate(mesh_obj.data.vertices):
        vert.co = coords[idx].copy()


def prune_shape_keys(mesh_obj: Any, keep: set[str]) -> None:
    keys = mesh_obj.data.shape_keys
    if keys is None:
        return
    bpy.context.view_layer.objects.active = mesh_obj
    mesh_obj.select_set(True)
    for name in list(keys.key_blocks.keys()):
        if name in keep:
            continue
        keys.key_blocks[name].value = 0.0
        bpy.ops.object.shape_key_remove(all=False)


def bake_body_morph_targets(mesh_obj: Any, sliders: list[dict[str, Any]]) -> list[str]:
    ensure_basis(mesh_obj)
    apply_morphs(neutral_morphs(sliders))
    basis = mesh_obj.data.shape_keys.key_blocks[0]
    for idx, vert in enumerate(mesh_obj.data.vertices):
        basis.data[idx].co = vert.co.copy()
    created: list[str] = []
    for entry in sliders:
        morph_id = entry["id"]
        for suffix, val in (("_max", 1.0), ("_min", 0.0)):
            morphs = neutral_morphs(sliders)
            morphs[morph_id] = val
            apply_morphs(morphs)
            name = f"tbg_{morph_id}{suffix}"
            key = mesh_obj.shape_key_add(name=name, from_mix=False)
            for idx, vert in enumerate(mesh_obj.data.vertices):
                key.data[idx].co = vert.co.copy()
            created.append(name)
    prune_shape_keys(mesh_obj, {"Basis", *created})
    apply_morphs(neutral_morphs(sliders))
    return created


def fit_hair_to_head(hair_obj: Any, body_obj: Any) -> None:
    if hair_obj.data.shape_keys:
        raise RuntimeError("fit_hair_to_head requires mesh without shape keys")
    mod = hair_obj.modifiers.new("TBG_HairShrink", "SHRINKWRAP")
    mod.target = body_obj
    mod.wrap_method = "NEAREST_SURFACEPOINT"
    mod.offset = 0.006
    bpy.context.view_layer.objects.active = hair_obj
    hair_obj.select_set(True)
    bpy.ops.object.modifier_apply(modifier=mod.name)


def bake_hair_morph_targets(hair_obj: Any, body_obj: Any, sliders: list[dict[str, Any]]) -> list[str]:
    """Sample hair verts per morph extreme; add shape keys after all shrinkwraps."""
    neutral = neutral_morphs(sliders)
    pos: dict[str, list[Any]] = {}
    apply_morphs(neutral)
    fit_hair_to_head(hair_obj, body_obj)
    pos["Basis"] = [v.co.copy() for v in hair_obj.data.vertices]
    created: list[str] = []
    for entry in sliders:
        morph_id = entry["id"]
        for suffix, val in (("_max", 1.0), ("_min", 0.0)):
            morphs = {**neutral, morph_id: val}
            apply_morphs(morphs)
            restore_mesh_positions(hair_obj, pos["Basis"])
            fit_hair_to_head(hair_obj, body_obj)
            name = f"tbg_{morph_id}{suffix}"
            pos[name] = [v.co.copy() for v in hair_obj.data.vertices]
            created.append(name)
    apply_morphs(neutral)
    restore_mesh_positions(hair_obj, pos["Basis"])
    ensure_basis(hair_obj)
    basis = hair_obj.data.shape_keys.key_blocks[0]
    for idx, co in enumerate(pos["Basis"]):
        basis.data[idx].co = co.copy()
    for name in created:
        key = hair_obj.shape_key_add(name=name, from_mix=False)
        for idx, co in enumerate(pos[name]):
            key.data[idx].co = co.copy()
    prune_shape_keys(hair_obj, {"Basis", *created})
    return created
