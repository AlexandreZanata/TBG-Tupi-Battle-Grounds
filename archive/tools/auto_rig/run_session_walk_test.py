"""Subprocess wrapper — bake Walk from archived or live Rig Studio session."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

from mesh_pipeline.export_game_unit_core import repo_root

REPO = repo_root()
BLENDER_CMD = REPO / "tools" / "blender" / "blender_cmd.sh"
BAKE_BPY = REPO / "tools" / "experimental" / "auto_rig" / "bake_session_walk_bpy.py"
SESSIONS = REPO / ".local" / "rig-studio" / "sessions"
ARCHIVE = REPO / "content" / "characters" / "rig-sessions"


def resolve_session_paths(unit_id: str) -> tuple[Path, Path, str]:
    manifest_path = ARCHIVE / unit_id / "manifest.json"
    bones_archived = ARCHIVE / unit_id / "bones.json"
    bones_live = SESSIONS / unit_id / "bones.json"
    blend = SESSIONS / unit_id / "session.blend"

    if not blend.is_file():
        raise FileNotFoundError(f"session blend missing: {blend} — run Rig Studio prep first")

    bones = bones_archived if bones_archived.is_file() else bones_live
    if not bones.is_file():
        raise FileNotFoundError(f"bones.json missing for {unit_id}")

    body_class = "HumanoidHeavy"
    if manifest_path.is_file():
        body_class = json.loads(manifest_path.read_text(encoding="utf-8")).get("bodyClass", body_class)
    return blend, bones, body_class


def run_session_walk_test(unit_id: str, *, root_motion: bool = False) -> dict:
    blend, bones, body_class = resolve_session_paths(unit_id)
    cmd = [
        str(BLENDER_CMD),
        "--background",
        "--python",
        str(BAKE_BPY),
        "--",
        "--session-blend",
        str(blend),
        "--bones-json",
        str(bones),
        "--id",
        unit_id,
        "--body-class",
        body_class,
        "--report-json",
    ]
    if root_motion:
        cmd.append("--root-motion")
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=False)
    if proc.returncode != 0 or "Traceback" in proc.stderr:
        raise RuntimeError(proc.stderr or proc.stdout or f"exit {proc.returncode}")
    line = next((row for row in reversed(proc.stdout.splitlines()) if row.strip().startswith("{")), "{}")
    return json.loads(line)
