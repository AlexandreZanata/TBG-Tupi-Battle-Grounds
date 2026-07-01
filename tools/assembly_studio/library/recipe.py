"""Assembly recipe validation and library path resolution."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from assembly_studio.library.schema import load_json, validate_base_manifest
from assembly_studio.library.sockets import BASE_IDS

RECIPE_REQUIRED = frozenset({"unitId", "baseId", "parts", "materialId"})


@dataclass(frozen=True)
class PartRef:
    part_id: str
    category: str
    socket: str
    mesh_path: Path
    manifest_path: Path


@dataclass(frozen=True)
class AssemblyRecipe:
    unit_id: str
    base_id: str
    part_ids: tuple[str, ...]
    material_id: str
    base_body_path: Path
    export_path: Path
    parts: tuple[PartRef, ...]
    material_path: Path


def validate_recipe(data: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    missing = RECIPE_REQUIRED - data.keys()
    if missing:
        errors.append(f"missing keys: {sorted(missing)}")
    if data.get("baseId") not in BASE_IDS:
        errors.append(f"baseId must be one of {BASE_IDS}")
    parts = data.get("parts", [])
    if not isinstance(parts, list):
        errors.append("parts must be an array")
    return errors


def resolve_recipe(recipe_path: Path, repo_root: Path | None = None) -> AssemblyRecipe:
    repo = repo_root or Path(__file__).resolve().parents[3]
    return build_recipe(load_json(recipe_path), repo_root=repo)


def build_recipe(data: dict[str, Any], *, repo_root: Path) -> AssemblyRecipe:
    library = repo_root / "content" / "characters" / "library"
    errors = validate_recipe(data)
    if errors:
        raise ValueError(f"invalid recipe: {errors}")

    base_id = str(data["baseId"])
    base_dir = library / "bases" / base_id
    base_manifest = load_json(base_dir / "manifest.json")
    base_errors = validate_base_manifest(base_manifest, base_dir=base_dir)
    if base_errors:
        raise ValueError(f"invalid base manifest: {base_errors}")

    part_refs: list[PartRef] = []
    for part_id in data["parts"]:
        manifest_path = _find_part_manifest(library, part_id)
        part_data = load_json(manifest_path)
        if base_id not in part_data.get("baseCompat", []):
            raise ValueError(f"part {part_id} incompatible with base {base_id}")
        part_dir = manifest_path.parent
        part_refs.append(
            PartRef(
                part_id=part_id,
                category=str(part_data["category"]),
                socket=str(part_data["socket"]),
                mesh_path=part_dir / str(part_data["mesh"]),
                manifest_path=manifest_path,
            )
        )

    material_id = str(data["materialId"])
    material_path = library / "materials" / f"{material_id}.json"
    if not material_path.is_file():
        raise FileNotFoundError(f"missing material profile: {material_path}")

    unit_id = str(data["unitId"])
    return AssemblyRecipe(
        unit_id=unit_id,
        base_id=base_id,
        part_ids=tuple(data["parts"]),
        material_id=material_id,
        base_body_path=base_dir / "body.glb",
        export_path=library / "exports" / f"{unit_id}.glb",
        parts=tuple(part_refs),
        material_path=material_path,
    )


def _find_part_manifest(library: Path, part_id: str) -> Path:
    matches = list(library.glob(f"parts/*/{part_id}.json"))
    if not matches:
        raise FileNotFoundError(f"part manifest not found: {part_id}")
    return matches[0]


def recipe_to_dict(recipe: AssemblyRecipe) -> dict[str, Any]:
    return {
        "unitId": recipe.unit_id,
        "baseId": recipe.base_id,
        "parts": list(recipe.part_ids),
        "materialId": recipe.material_id,
    }
