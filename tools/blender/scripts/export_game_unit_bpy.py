#!/usr/bin/env python3
"""Run inside Blender only — no host Python dependencies (PIL, etc.)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    import bpy  # noqa: F401
except ImportError as exc:
    raise SystemExit("export_game_unit_bpy.py must run inside Blender") from exc

REPO_ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = REPO_ROOT / "tools"
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from mesh_pipeline.export_game_unit_blender import (  # noqa: E402
    DEFAULT_ANIMATION_SET,
    export_game_unit_blender,
)

PILOT_UNIT_ID = "demo_sentinel_01"


def _parse_unit_id(argv: list[str]) -> str:
    if "--" in argv:
        args = argv[argv.index("--") + 1 :]
        if args:
            return args[0]
    if len(argv) >= 2 and not argv[1].startswith("-"):
        return argv[1]
    raise SystemExit("usage: export_game_unit_bpy.py <unit_id>")


def main(argv: list[str] | None = None) -> int:
    unit_id = _parse_unit_id(argv if argv is not None else sys.argv)
    if unit_id != PILOT_UNIT_ID:
        raise SystemExit(f"Step C pilot only: {PILOT_UNIT_ID}")

    definition_path = REPO_ROOT / "content/characters/definitions" / f"{unit_id}.json"
    definition = json.loads(definition_path.read_text(encoding="utf-8"))
    showcase = definition.get("meshShowcase") or definition.get("mesh")
    if not showcase:
        raise SystemExit(f"{unit_id}: missing showcase mesh path")

    game_mesh = REPO_ROOT / "content/characters/meshes/game" / f"{unit_id}.glb"
    output = export_game_unit_blender(
        unit_id=unit_id,
        showcase_mesh=REPO_ROOT / showcase,
        game_mesh=game_mesh,
        animation_set_id=definition.get("animationSet", DEFAULT_ANIMATION_SET),
        body_class="HumanoidHeavy",
        repo=REPO_ROOT,
    )
    print(f"Exported game mesh (raw): {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
