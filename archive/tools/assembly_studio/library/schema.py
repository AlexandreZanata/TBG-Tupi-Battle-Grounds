"""Validate base and part manifest JSON contracts."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from assembly_studio.library.sockets import BASE_IDS, CATEGORY_SOCKETS

BASE_REQUIRED = frozenset({"id", "label", "heightM", "bodyClass", "mesh", "rig"})
PART_REQUIRED = frozenset({"id", "category", "baseCompat", "socket", "mesh", "thumbnail"})
BODY_CLASSES = frozenset({"HumanoidLight", "HumanoidHeavy"})


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_base_manifest(data: dict[str, Any], *, base_dir: Path) -> list[str]:
    errors: list[str] = []
    missing = BASE_REQUIRED - data.keys()
    if missing:
        errors.append(f"missing keys: {sorted(missing)}")
    if data.get("id") not in BASE_IDS:
        errors.append(f"id must be one of {BASE_IDS}")
    if data.get("bodyClass") not in BODY_CLASSES:
        errors.append("bodyClass must be HumanoidLight or HumanoidHeavy")
    mesh = base_dir / str(data.get("mesh", ""))
    if not mesh.is_file():
        errors.append(f"missing mesh: {mesh}")
    rig = (base_dir / str(data.get("rig", ""))).resolve()
    if not rig.is_file():
        errors.append(f"missing rig: {rig}")
    return errors


def validate_part_manifest(data: dict[str, Any], *, part_dir: Path) -> list[str]:
    errors: list[str] = []
    missing = PART_REQUIRED - data.keys()
    if missing:
        errors.append(f"missing keys: {sorted(missing)}")
    category = str(data.get("category", ""))
    expected_socket = CATEGORY_SOCKETS.get(category)
    if expected_socket is None:
        errors.append(f"unknown category: {category}")
    elif data.get("socket") != expected_socket:
        errors.append(f"socket must be {expected_socket} for category {category}")
    compat = data.get("baseCompat", [])
    if not isinstance(compat, list) or not compat:
        errors.append("baseCompat must be a non-empty list")
    elif invalid := [b for b in compat if b not in BASE_IDS]:
        errors.append(f"invalid baseCompat entries: {invalid}")
    mesh = part_dir / str(data.get("mesh", ""))
    if not mesh.is_file():
        errors.append(f"missing mesh: {mesh}")
    thumb = part_dir / str(data.get("thumbnail", ""))
    if not thumb.is_file():
        errors.append(f"missing thumbnail: {thumb}")
    return errors
