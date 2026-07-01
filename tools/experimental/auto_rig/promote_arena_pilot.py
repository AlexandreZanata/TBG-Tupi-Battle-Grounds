#!/usr/bin/env python3
"""Promote one showcase pilot to game GLB via experimental auto-rig (Motion Arena dev)."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[2]
REPO = Path(__file__).resolve().parents[3]
if str(TOOLS) not in sys.path:
    sys.path.insert(0, str(TOOLS))

from mesh_pipeline.export_game_unit_core import (  # noqa: E402
    load_export_config,
    post_process_game_glb,
    repo_root,
)
from mesh_pipeline.prep_ai_mesh_run import blender_available  # noqa: E402

DEFAULT_PILOT = "demo_bruiser_01"
BLENDER_CMD = REPO / "tools/blender/blender_cmd.sh"
ARENA_BPY = REPO / "tools/experimental/auto_rig/arena_pilot_bpy.py"


def promote_arena_pilot(unit_id: str, *, skip_post: bool = False, fast: bool = False) -> dict:
    if not blender_available():
        raise RuntimeError(f"Blender not available: {BLENDER_CMD}")

    config = load_export_config(unit_id, repo_root())
    raw = config.game_mesh.with_suffix(".blender-raw.glb")
    cmd = [
        str(BLENDER_CMD),
        "--background",
        "--python",
        str(ARENA_BPY),
        "--",
        "--id",
        unit_id,
        "--showcase",
        str(config.showcase_mesh),
        "--raw-output",
        str(raw),
        "--body-class",
        config.body_class,
        "--animation-set",
        config.animation_set_id,
        "--report-json",
    ]
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=False)
    if proc.returncode not in (0, 2) or "Traceback" in proc.stderr:
        raise RuntimeError(proc.stderr or proc.stdout or f"Blender exit {proc.returncode}")

    report_line = next(
        (line for line in reversed(proc.stdout.splitlines()) if line.strip().startswith("{")),
        "{}",
    )
    report = json.loads(report_line)
    if skip_post:
        report["game_mesh"] = str(raw)
    elif fast:
        config.game_mesh.parent.mkdir(parents=True, exist_ok=True)
        raw.replace(config.game_mesh)
        report["game_mesh"] = str(config.game_mesh)
        report["fast"] = True
    else:
        post_process_game_glb(raw, config.game_mesh, config.texture_size)
        raw.unlink(missing_ok=True)
        report["game_mesh"] = str(config.game_mesh)
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description="Experimental auto-rig → game GLB for one arena pilot.")
    parser.add_argument("--id", default=DEFAULT_PILOT, help=f"Unit id (default: {DEFAULT_PILOT})")
    parser.add_argument("--skip-post", action="store_true", help="Keep .blender-raw.glb only")
    parser.add_argument(
        "--fast",
        action="store_true",
        help="Copy raw rigged GLB to game path (skip Draco post-process)",
    )
    args = parser.parse_args()
    report = promote_arena_pilot(args.id, skip_post=args.skip_post, fast=args.fast)
    print(json.dumps(report, indent=2))
    return 0 if report.get("weight_gate_pass") else 1


if __name__ == "__main__":
    raise SystemExit(main())
