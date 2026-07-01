"""Load and validate Mixamo → TBG bone mapping."""

from __future__ import annotations

import json
from pathlib import Path

MAP_PATH = Path(__file__).resolve().parent / "BONE_MAP_MIXAMO_TO_TBG.json"
OPTIONAL_TBG_BONES = ("TBG_Toe.L", "TBG_Toe.R")


def load_mixamo_bone_map(path: Path | None = None) -> dict:
    data = json.loads((path or MAP_PATH).read_text(encoding="utf-8"))
    required = list(data.get("mapping", {}).keys())
    if data.get("root_policy") == "inject_from_spine":
        required = [b for b in required if b != "TBG_Root"]
    required.extend(b for b in OPTIONAL_TBG_BONES if b in data.get("mapping", {}))
    missing = [b for b in required if b not in data.get("mapping", {})]
    if missing:
        raise ValueError(f"mixamo bone map missing TBG bones: {missing}")
    return data


def normalize_mixamo_bone_name(name: str) -> str:
    if ":" in name:
        name = name.split(":")[-1]
    if name.startswith("mixamorig"):
        name = name.removeprefix("mixamorig").lstrip("_:")
    return name


def mixamo_lookup_names(bone_map: dict, mixamo_canonical: str) -> set[str]:
    names = {mixamo_canonical, normalize_mixamo_bone_name(mixamo_canonical)}
    for alias in bone_map.get("name_aliases", {}).get(mixamo_canonical, []):
        names.add(alias)
        names.add(normalize_mixamo_bone_name(alias))
    return names


def tbg_bone_for_mixamo(bone_map: dict, mixamo_bone_name: str) -> str | None:
    normalized = normalize_mixamo_bone_name(mixamo_bone_name)
    for tbg_bone, canonical in bone_map["mapping"].items():
        if normalized == canonical or mixamo_bone_name in mixamo_lookup_names(bone_map, canonical):
            return tbg_bone
    return None
