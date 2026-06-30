"""Safe resolution of versioned character content files for browser preview."""

from __future__ import annotations

from pathlib import Path

from fastapi import HTTPException

from .models import CharacterDefinition

MAX_ASSET_BYTES = 150 * 1024 * 1024


def resolve_character_asset(repo_root: Path, relative_path: str) -> Path:
    if not relative_path or relative_path.startswith("/"):
        raise HTTPException(status_code=400, detail="invalid asset path")

    normalized = relative_path.replace("\\", "/").lstrip("/")
    if ".." in normalized.split("/"):
        raise HTTPException(status_code=400, detail="invalid asset path")

    if normalized.startswith("content/"):
        normalized = normalized.removeprefix("content/")
    if normalized.startswith("characters/"):
        normalized = normalized.removeprefix("characters/")

    content_root = (repo_root / "content" / "characters").resolve()
    absolute = (content_root / normalized).resolve()

    if content_root not in absolute.parents and absolute != content_root:
        raise HTTPException(status_code=403, detail="asset path outside character content")

    if not absolute.is_file():
        raise HTTPException(status_code=404, detail=f"asset not found: {relative_path}")

    if absolute.stat().st_size > MAX_ASSET_BYTES:
        raise HTTPException(status_code=413, detail="asset exceeds preview size limit")

    return absolute


def preview_mesh_path(defn: CharacterDefinition) -> str | None:
    """Return showcase mesh for web preview when dual-tier paths are configured."""
    if defn.meshShowcase:
        return defn.meshShowcase
    return defn.mesh


MESH_TIERS = frozenset({"showcase", "game"})


def resolve_mesh_path(
    defn: CharacterDefinition,
    tier: str = "showcase",
    repo_root: Path | None = None,
) -> str | None:
    """Return relative mesh path for showcase (default) or game tier."""
    if tier not in MESH_TIERS:
        raise HTTPException(status_code=400, detail=f"invalid mesh tier: {tier}")
    if tier == "game":
        return defn.mesh

    if repo_root is None:
        return preview_mesh_path(defn)

    return resolve_showcase_mesh_path(defn, repo_root)


def resolve_showcase_mesh_path(defn: CharacterDefinition, repo_root: Path) -> str | None:
    """Pick first existing showcase candidate (dual-tier, legacy flat, or id-based)."""
    candidates: list[str] = []
    if defn.meshShowcase:
        candidates.append(defn.meshShowcase)
    if defn.mesh and "/meshes/game/" not in defn.mesh.replace("\\", "/"):
        candidates.append(defn.mesh)
    if defn.id:
        candidates.append(f"content/characters/meshes/showcase/{defn.id}.glb")
        candidates.append(f"content/characters/meshes/{defn.id}.glb")

    seen: set[str] = set()
    for relative in candidates:
        if not relative or relative in seen:
            continue
        seen.add(relative)
        try:
            resolve_character_asset(repo_root, relative)
        except HTTPException:
            continue
        return relative

    return candidates[0] if candidates else None


def build_mesh_info(defn: CharacterDefinition, repo_root: Path, tier: str = "showcase") -> dict:
    """Build mesh-info payload for showcase or game tier."""
    relative = resolve_mesh_path(defn, tier, repo_root)
    if not relative:
        raise HTTPException(status_code=404, detail=f"unit has no {tier} mesh")

    path = resolve_character_asset(repo_root, relative)
    info: dict[str, int | str | bool | list[str]] = {
        "tier": tier,
        "path": relative,
        "size": path.stat().st_size,
        "contentType": mesh_media_type(path),
    }

    if tier != "game":
        return info

    from mesh_pipeline.game_glb_validate import validate_game_glb
    from mesh_pipeline.glb_inspect import inspect_glb, load_glb

    stats = inspect_glb(path)
    validation = validate_game_glb(path)
    gltf, _ = load_glb(path)
    clip_names = [anim.get("name", f"clip_{index}") for index, anim in enumerate(gltf.get("animations", []))]

    info.update(
        {
            "triangles": stats.triangles,
            "bones": stats.bones,
            "animations": stats.animations,
            "materials": stats.materials,
            "textures": stats.textures,
            "maxTextureSize": stats.max_texture_size,
            "clipNames": clip_names,
            "budgetOk": validation.ok,
            "warnings": validation.warnings,
        }
    )
    return info


def mesh_media_type(path: Path) -> str:
    suffix = path.suffix.lower()
    if suffix == ".glb":
        return "model/gltf-binary"
    if suffix == ".gltf":
        return "model/gltf+json"
    return "application/octet-stream"


def texture_media_type(path: Path) -> str:
    suffix = path.suffix.lower()
    if suffix == ".png":
        return "image/png"
    if suffix in {".jpg", ".jpeg"}:
        return "image/jpeg"
    return "application/octet-stream"
