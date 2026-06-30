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
