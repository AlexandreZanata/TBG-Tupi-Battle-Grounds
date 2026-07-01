"""Run Mixamo Edit Blender build."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any


def dispatch_mixamo_edit(payload: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    unit_id = str(payload["unitId"])
    mixamo_fbx = _resolve_fbx(repo_root, str(payload["mixamoFbx"]))
    output = repo_root / "content/characters/imports/mixamo/edit" / f"{unit_id}.glb"
    output.parent.mkdir(parents=True, exist_ok=True)

    custom = payload.get("customMesh")
    custom_path = _resolve_mesh(repo_root, custom) if custom else None

    height_scale = float(payload.get("heightScale", 1.0))
    rig_scale = float(payload.get("rigScale", 1.0))
    manual = bool(payload.get("manualAlign"))
    rig_pos = payload.get("rigPosition") or [0.0, 0.0, 0.0]
    if len(rig_pos) != 3:
        rig_pos = [0.0, 0.0, 0.0]
    pos_str = ",".join(str(float(v)) for v in rig_pos)
    bone_props = payload.get("boneProportions") or {}
    blender = repo_root / "tools/assembly_studio/blender/blender_cmd.sh"
    script = repo_root / "tools/assembly_studio/blender/mixamo_edit_build.py"
    cmd = [
        str(blender),
        "--background",
        "--python",
        str(script),
        "--",
        "--mixamo-fbx",
        str(mixamo_fbx),
        "--output",
        str(output),
        "--height-scale",
        str(height_scale),
        "--rig-position",
        pos_str,
        "--rig-scale",
        str(rig_scale),
        "--report-json",
    ]
    if manual:
        cmd.append("--manual-align")
    if bone_props:
        cmd.extend(["--bone-proportions", json.dumps(bone_props)])
    if custom_path:
        cmd.extend(["--custom-mesh", str(custom_path)])

    proc = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        cwd=repo_root,
        timeout=600,
        check=False,
    )
    if proc.returncode != 0:
        tail = (proc.stderr or proc.stdout or "blender failed").strip()[-1200:]
        raise RuntimeError(tail)

    report = _parse_report(proc.stdout)
    report.update({
        "unitId": unit_id,
        "label": payload.get("label") or unit_id,
        "mixamoFbx": Path(mixamo_fbx).name,
        "customMesh": Path(custom_path).name if custom_path else None,
        "heightScale": height_scale,
        "meshUrl": f"/api/content/characters/imports/mixamo/edit/{unit_id}.glb",
    })
    return report


def _resolve_fbx(repo_root: Path, name: str) -> Path:
    path = repo_root / "content/characters/imports/mixamo/raw" / Path(name).name
    if not path.is_file():
        raise FileNotFoundError(f"Mixamo FBX not found: {path}")
    return path


def _resolve_mesh(repo_root: Path, name: str) -> Path:
    path = repo_root / "content/characters/imports/mixamo/meshes" / Path(name).name
    if not path.is_file():
        raise FileNotFoundError(f"custom mesh not found: {path}")
    return path


def _parse_report(stdout: str) -> dict[str, Any]:
    for line in reversed(stdout.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            return json.loads(line)
    raise RuntimeError("no JSON report from mixamo_edit_build.py")
