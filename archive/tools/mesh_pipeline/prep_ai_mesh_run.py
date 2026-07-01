"""Invoke prep_ai_mesh.py via Blender headless."""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

BLENDER_CMD = Path(__file__).resolve().parents[1] / "blender" / "blender_cmd.sh"
PREP_SCRIPT = Path(__file__).resolve().parents[1] / "blender" / "scripts" / "prep_ai_mesh.py"


@dataclass(frozen=True)
class PrepRunResult:
    output: Path
    report: dict


def blender_available() -> bool:
    return BLENDER_CMD.is_file() and os.access(BLENDER_CMD, os.X_OK)


def run_prep_ai_mesh(
    input_path: Path,
    output_path: Path,
    *,
    target_height: float = 1.8,
    voxel_remesh: bool = False,
    z_up: bool = False,
) -> PrepRunResult:
    if not blender_available():
        raise RuntimeError(f"Blender not available: {BLENDER_CMD}")

    cmd = [
        str(BLENDER_CMD),
        "--background",
        "--python",
        str(PREP_SCRIPT),
        "--",
        str(input_path),
        str(output_path),
        "--height",
        str(target_height),
        "--report-json",
    ]
    if voxel_remesh:
        cmd.append("--voxel-remesh")
    if z_up:
        cmd.append("--z-up")

    proc = subprocess.run(cmd, check=True, capture_output=True, text=True)
    report_line = next(
        (line for line in reversed(proc.stdout.splitlines()) if line.strip().startswith("{")),
        "{}",
    )
    return PrepRunResult(output=output_path, report=json.loads(report_line))
