"""Invoke experimental auto_rig_mesh.py via Blender headless."""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
BLENDER_CMD = REPO / "tools/blender/blender_cmd.sh"
AUTO_RIG_SCRIPT = REPO / "tools/experimental/auto_rig/auto_rig_mesh.py"


@dataclass(frozen=True)
class AutoRigRunResult:
    output: Path
    report: dict
    exit_code: int


def blender_available() -> bool:
    return BLENDER_CMD.is_file() and os.access(BLENDER_CMD, os.X_OK)


def run_auto_rig_mesh(
    input_path: Path,
    output_path: Path,
    *,
    body_class: str = "HumanoidLight",
    target_height: float = 1.8,
    voxel_remesh: bool = False,
    z_up: bool = False,
    max_faces: int = 100_000,
    bind_mode: str = "fallback",
) -> AutoRigRunResult:
    if not blender_available():
        raise RuntimeError(f"Blender not available: {BLENDER_CMD}")

    cmd = [
        str(BLENDER_CMD),
        "--background",
        "--python",
        str(AUTO_RIG_SCRIPT),
        "--",
        str(input_path),
        str(output_path),
        "--body-class",
        body_class,
        "--height",
        str(target_height),
        "--max-faces",
        str(max_faces),
        "--bind-mode",
        bind_mode,
        "--report-json",
    ]
    if voxel_remesh:
        cmd.append("--voxel-remesh")
    if z_up:
        cmd.append("--z-up")

    proc = subprocess.run(cmd, check=False, capture_output=True, text=True, cwd=REPO)
    report_line = next(
        (line for line in reversed(proc.stdout.splitlines()) if line.strip().startswith("{")),
        "{}",
    )
    return AutoRigRunResult(
        output=output_path,
        report=json.loads(report_line),
        exit_code=proc.returncode,
    )
