"""Scan modular library manifests for Assembly Studio API."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from assembly_studio.library.schema import load_json, validate_base_manifest, validate_part_manifest

LIBRARY_REL = "content/characters/library"


def library_root(repo_root: Path) -> Path:
    return repo_root / "content" / "characters" / "library"


def load_library_catalog(repo_root: Path) -> dict[str, Any]:
    root = library_root(repo_root)
    return {
        "bases": _load_bases(root),
        "parts": _load_parts(root),
        "materials": _load_materials(root),
    }


def _load_bases(root: Path) -> list[dict[str, Any]]:
    items: list[dict[str, Any]] = []
    for manifest_path in sorted((root / "bases").glob("*/manifest.json")):
        data = load_json(manifest_path)
        errors = validate_base_manifest(data, base_dir=manifest_path.parent)
        if errors:
            continue
        base_id = str(data["id"])
        items.append(
            {
                "id": base_id,
                "label": data["label"],
                "heightM": data["heightM"],
                "bodyClass": data["bodyClass"],
                "meshUrl": f"/api/content/{LIBRARY_REL}/bases/{base_id}/body.glb",
            }
        )
    return items


def _load_parts(root: Path) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = {}
    for manifest_path in sorted(root.glob("parts/*/*.json")):
        part_dir = manifest_path.parent
        data = load_json(manifest_path)
        errors = validate_part_manifest(data, part_dir=part_dir)
        if errors:
            continue
        category = str(data["category"])
        part_id = str(data["id"])
        grouped.setdefault(category, []).append(
            {
                "id": part_id,
                "category": category,
                "socket": data["socket"],
                "baseCompat": list(data["baseCompat"]),
                "meshUrl": f"/api/content/{LIBRARY_REL}/parts/{category}/{data['mesh']}",
                "thumbnailUrl": f"/api/content/{LIBRARY_REL}/parts/{category}/{data['thumbnail']}",
            }
        )
    return grouped


def _load_materials(root: Path) -> list[dict[str, Any]]:
    items: list[dict[str, Any]] = []
    for path in sorted((root / "materials").glob("*.json")):
        data = load_json(path)
        items.append({"id": path.stem, "label": data.get("label", path.stem), "profile": data})
    return items
