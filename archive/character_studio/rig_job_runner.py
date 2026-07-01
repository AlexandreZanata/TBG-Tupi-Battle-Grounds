"""Rig Studio Blender subprocess dispatch."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

from mesh_pipeline.export_game_unit_core import load_export_config, post_process_game_glb
from mesh_pipeline.weight_quality import validate_weight_quality

from .rig_jobs import (
    BLENDER_CMD,
    PROMOTE_SCRIPT,
    PROMOTE_WALK_SCRIPT,
    REPO,
    RIG_STUDIO_BPY,
    session_blend_path,
    session_bones_path,
    session_meta,
    session_preview_path,
)

def dispatch_rig_job(unit_id: str, kind: str, *, fast: bool) -> dict[str, Any]:
    if kind == "auto":
        return _run_auto(unit_id, fast=fast)
    if kind == "prep":
        return _run_prep(unit_id)
    if kind == "bake_walk":
        return _run_bake_walk(unit_id)
    return _run_finalize(unit_id, fast=fast)


def _run_blender(args: list[str]) -> dict[str, Any]:
    cmd = [str(BLENDER_CMD), "--background", "--python", str(RIG_STUDIO_BPY), "--", *args, "--report-json"]
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=False)
    if proc.returncode != 0 or "Traceback" in proc.stderr:
        raise RuntimeError(proc.stderr or proc.stdout or f"Blender exit {proc.returncode}")
    line = next(
        (row for row in reversed(proc.stdout.splitlines()) if row.strip().startswith("{")),
        "{}",
    )
    return json.loads(line)


def _run_prep(unit_id: str) -> dict[str, Any]:
    config = load_export_config(unit_id, REPO)
    if not config.showcase_mesh.is_file():
        raise FileNotFoundError(f"showcase mesh missing: {config.showcase_mesh}")
    blend = session_blend_path(unit_id)
    preview = session_preview_path(unit_id)
    bones = session_bones_path(unit_id)
    report = _run_blender(
        [
            "prep",
            "--showcase",
            str(config.showcase_mesh),
            "--body-class",
            config.body_class,
            "--session-blend",
            str(blend),
            "--preview-glb",
            str(preview),
            "--bones-json",
            str(bones),
        ]
    )
    report["session"] = session_meta(unit_id)
    return report


def _run_finalize(unit_id: str, *, fast: bool) -> dict[str, Any]:
    blend = session_blend_path(unit_id)
    if not blend.is_file():
        raise FileNotFoundError(f"session blend missing — run prep first: {blend}")
    config = load_export_config(unit_id, REPO)
    raw = config.game_mesh.with_suffix(".blender-raw.glb")
    report = _run_blender(
        [
            "finalize",
            "--session-blend",
            str(blend),
            "--raw-output",
            str(raw),
            "--id",
            unit_id,
            "--animation-set",
            config.animation_set_id,
            "--bones-json",
            str(session_bones_path(unit_id)),
        ]
    )
    if fast:
        config.game_mesh.parent.mkdir(parents=True, exist_ok=True)
        raw.replace(config.game_mesh)
        report["game_mesh"] = str(config.game_mesh)
        report["fast"] = True
    else:
        post_process_game_glb(raw, config.game_mesh, config.texture_size)
        raw.unlink(missing_ok=True)
        report["game_mesh"] = str(config.game_mesh)
    report["weight"] = _weight_report(config.game_mesh)
    report["weight_gate_pass"] = report["weight"]["passesGate"]
    return report


def _run_bake_walk(unit_id: str) -> dict[str, Any]:
    cmd = ["python3", str(PROMOTE_WALK_SCRIPT), "--id", unit_id, "--report-json"]
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=False)
    if proc.returncode != 0 or "Traceback" in proc.stderr:
        raise RuntimeError(proc.stderr or proc.stdout or f"bake_walk exit {proc.returncode}")
    line = next((row for row in reversed(proc.stdout.splitlines()) if row.strip().startswith("{")), "{}")
    return json.loads(line)


def _run_auto(unit_id: str, *, fast: bool) -> dict[str, Any]:
    cmd = ["python3", str(PROMOTE_SCRIPT), "--id", unit_id]
    if fast:
        cmd.append("--fast")
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=False)
    if proc.returncode not in (0, 1, 2) or "Traceback" in proc.stderr:
        raise RuntimeError(proc.stderr or proc.stdout or f"promote exit {proc.returncode}")
    report = json.loads(proc.stdout)
    game = Path(report.get("game_mesh", ""))
    if game.is_file():
        report["weight"] = _weight_report(game)
    return report


def _weight_report(game_mesh: Path) -> dict[str, Any]:
    result = validate_weight_quality(game_mesh)
    skew = result.skew
    return {
        "passesGate": result.passes_gate,
        "dominantBone": skew.dominant_bone,
        "dominantShare": round(skew.dominant_share, 4),
        "unweightedVertices": skew.unweighted_vertices,
        "bonesMissing": list(result.bones_missing_coverage),
        "errors": list(result.errors),
    }
