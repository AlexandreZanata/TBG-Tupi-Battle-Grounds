"""Strip glTF extensions that break web preview."""

from __future__ import annotations

from pathlib import Path

from mesh_pipeline.game_glb_texture_pack import _write_glb
from mesh_pipeline.glb_inspect import load_glb

STRIP_EXTENSIONS = ("KHR_materials_specular",)


def strip_preview_extensions(path: Path) -> Path:
    """Remove glTF extensions that break Three.js web preview (e.g. specular)."""
    gltf, bin_chunk = load_glb(path)
    changed = False
    for material in gltf.get("materials", []):
        extensions = material.get("extensions")
        if not extensions:
            continue
        for key in STRIP_EXTENSIONS:
            if extensions.pop(key, None) is not None:
                changed = True
        if not extensions:
            material.pop("extensions", None)
            changed = True

    used = gltf.get("extensionsUsed") or []
    filtered = [ext for ext in used if ext not in STRIP_EXTENSIONS]
    if filtered != used:
        gltf["extensionsUsed"] = filtered
        changed = True
    if not gltf.get("extensionsUsed"):
        gltf.pop("extensionsUsed", None)

    if not changed:
        return path
    _write_glb(path, gltf, bin_chunk or b"")
    return path
