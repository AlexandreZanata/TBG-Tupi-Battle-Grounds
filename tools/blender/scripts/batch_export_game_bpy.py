#!/usr/bin/env python3
"""Run inside Blender — export multiple game units in one session."""

from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    import bpy  # noqa: F401
except ImportError as exc:
    raise SystemExit("batch_export_game_bpy.py must run inside Blender") from exc

REPO_ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = REPO_ROOT / "tools"
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from mesh_pipeline.export_game_unit_blender import (  # noqa: E402
    DEFAULT_ANIMATION_SET,
    export_game_unit_blender,
)


def _parse_unit_ids(argv: list[str]) -> list[str]:
    if "--" in argv:
        return [arg for arg in argv[argv.index("--") + 1 :] if arg and not arg.startswith("-")]
    return [arg for arg in argv[1:] if arg and not arg.startswith("-")]


def _load_body_class(unit_id: str) -> str:
    roster_path = REPO_ROOT / "content/characters/pilot_roster.json"
    data = json.loads(roster_path.read_text(encoding="utf-8"))
    for entry in data.get("pilots", []):
        if entry.get("id") == unit_id:
            return str(entry["bodyClass"])
    raise ValueError(f"{unit_id}: not found in pilot_roster.json")


def _export_one(unit_id: str) -> None:
    definition_path = REPO_ROOT / "content/characters/definitions" / f"{unit_id}.json"
    definition = json.loads(definition_path.read_text(encoding="utf-8"))
    showcase = definition.get("meshShowcase") or definition.get("mesh")
    if not showcase:
        raise ValueError(f"{unit_id}: missing showcase mesh path")

    game_lod = definition.get("gameLod") or {}
    game_mesh = REPO_ROOT / "content/characters/meshes/game" / f"{unit_id}.glb"
    output = export_game_unit_blender(
        unit_id=unit_id,
        showcase_mesh=REPO_ROOT / showcase,
        game_mesh=game_mesh,
        animation_set_id=definition.get("animationSet", DEFAULT_ANIMATION_SET),
        body_class=_load_body_class(unit_id),
        repo=REPO_ROOT,
        tri_target=int(game_lod.get("triTarget", 8000)),
        texture_size=int(game_lod.get("textureSize", 1024)),
    )
    print(f"Exported game mesh (raw): {output}")


def main(argv: list[str] | None = None) -> int:
    unit_ids = _parse_unit_ids(argv if argv is not None else sys.argv)
    if not unit_ids:
        raise SystemExit("usage: batch_export_game_bpy.py <unit_id> [unit_id ...]")

    failed: list[str] = []
    for unit_id in unit_ids:
        print(f"==> blender export {unit_id}")
        try:
            _export_one(unit_id)
        except Exception as exc:  # noqa: BLE001 — batch reports per-unit failure
            print(f"FAILED {unit_id}: {exc}", file=sys.stderr)
            failed.append(unit_id)

    if failed:
        print(f"batch_export_game_bpy: failed: {' '.join(failed)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
