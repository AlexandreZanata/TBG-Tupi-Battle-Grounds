#!/usr/bin/env python3
"""Bake Perry walk on Rig Studio skeleton and embed into game GLB."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[2]
REPO = Path(__file__).resolve().parents[3]
for path in (str(REPO), str(TOOLS)):
    if path not in sys.path:
        sys.path.insert(0, path)

from mesh_pipeline.export_game_unit_core import load_export_config, repo_root  # noqa: E402
from mesh_pipeline.prep_ai_mesh_run import blender_available  # noqa: E402

from tools.experimental.auto_rig.run_session_walk_test import run_session_walk_test  # noqa: E402

BLENDER_CMD = REPO / "tools/blender/blender_cmd.sh"
EMBED_BPY = REPO / "tools/experimental/auto_rig/embed_unit_walk_bpy.py"


def _run_embed(unit_id: str) -> dict:
    cmd = [
        str(BLENDER_CMD),
        "--background",
        "--python",
        str(EMBED_BPY),
        "--",
        "--id",
        unit_id,
        "--report-json",
    ]
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=False)
    if proc.returncode != 0 or "Traceback" in proc.stderr:
        raise RuntimeError(proc.stderr or proc.stdout or f"embed exit {proc.returncode}")
    line = next((row for row in reversed(proc.stdout.splitlines()) if row.strip().startswith("{")), "{}")
    return json.loads(line)


def promote_rig_session_walk(unit_id: str, *, root_motion: bool = True) -> dict:
    if not blender_available():
        raise RuntimeError(f"Blender not available: {BLENDER_CMD}")

    config = load_export_config(unit_id, repo_root())
    if not config.game_mesh.is_file():
        raise FileNotFoundError(
            f"game mesh missing: {config.game_mesh} — run Rig Studio Bind & export first",
        )

    walk = run_session_walk_test(unit_id, root_motion=False)
    walk_root = None
    if root_motion:
        walk_root = run_session_walk_test(unit_id, root_motion=True)
    embed = _run_embed(unit_id)

    return {
        "unitId": unit_id,
        "bodyClass": config.body_class,
        "walkProfile": walk.get("walkProfile"),
        "walkClip": walk.get("output"),
        "walkRootClip": walk_root.get("output") if walk_root else None,
        "gameMesh": embed.get("output"),
        "bonesJson": walk.get("bonesJson"),
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Bake realistic Walk from Rig Studio bones.json and embed in game GLB.",
    )
    parser.add_argument("--id", dest="unit_id", required=True)
    parser.add_argument("--no-walk-root", action="store_true")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args()
    report = promote_rig_session_walk(args.unit_id, root_motion=not args.no_walk_root)
    if args.report_json:
        print(json.dumps(report, indent=2))
    else:
        print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
