"""Simplify showcase GLBs — gltf-transform coarse pass + Blender final decimate."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from mesh_pipeline.export_game_unit_blender import DEFAULT_TEXTURE_SIZE, DEFAULT_TRI_TARGET, LOD1_TRI_RATIO
from mesh_pipeline.game_glb_transform import run_gltf_transform
from mesh_pipeline.glb_inspect import inspect_glb

COARSE_TRI_TARGET = 50_000
SIMPLIFY_ERROR = 0.008
TRIM_ERROR = 0.003
MAX_TRIM_PASSES = 5
MAX_COARSE_PASSES = 6
BLENDER_CMD = Path(__file__).resolve().parents[1] / "blender" / "blender_cmd.sh"
SCULPT_SCRIPT = Path(__file__).resolve().parents[1] / "blender" / "scripts" / "sculpt_to_game_mesh.py"


def _weld_simplify(source: Path, output: Path, ratio: float, error: float) -> None:
    """Weld split vertices then simplify — glTF Transform recommended order."""
    welded = output.with_suffix(".weld.glb")
    run_gltf_transform(["weld", str(source), str(welded)])
    run_gltf_transform(["simplify", str(welded), str(output), "--ratio", f"{ratio:.6f}", "--error", str(error)])
    welded.unlink(missing_ok=True)


def coarsen_glb(source: Path, output: Path, coarse_target: int = COARSE_TRI_TARGET) -> Path:
    """Reduce heavy sculpts (~1M tris) toward ~50k before Blender decimate."""
    output.parent.mkdir(parents=True, exist_ok=True)
    if inspect_glb(source).triangles <= coarse_target:
        shutil.copy2(source, output)
        return output

    current = source
    buf_a = output.with_suffix(".coarse-a.glb")
    buf_b = output.with_suffix(".coarse-b.glb")
    for pass_idx in range(MAX_COARSE_PASSES):
        current_tris = inspect_glb(current).triangles
        if current_tris <= coarse_target:
            break
        ratio = min(0.5, (coarse_target / max(1, current_tris)) * 0.95)
        nxt = buf_a if pass_idx % 2 == 0 else buf_b
        _weld_simplify(current, nxt, ratio, SIMPLIFY_ERROR)
        if current not in (source, output):
            Path(current).unlink(missing_ok=True)
        current = nxt

    if current != output:
        shutil.copy2(current, output)
    for buf in (buf_a, buf_b):
        if buf.is_file() and buf != output:
            buf.unlink(missing_ok=True)
    return output


def trim_to_tri_budget(mesh: Path, tri_target: int) -> Path:
    """gltf-transform trim when Blender stalls above tri budget."""
    tmp = mesh.with_suffix(".trim.glb")
    for _ in range(MAX_TRIM_PASSES):
        current = inspect_glb(mesh).triangles
        if current <= tri_target:
            return mesh
        ratio = min(1.0, (tri_target / max(1, current)) * 0.95)
        _weld_simplify(mesh, tmp, ratio, TRIM_ERROR)
        tmp.replace(mesh)
    return mesh


def _refine_to_budget(mesh: Path, tri_target: int, texture_size: int) -> Path:
    """Alternate trim + Blender until tri budget is met (max 3 rounds)."""
    for _ in range(3):
        if inspect_glb(mesh).triangles <= tri_target:
            return mesh
        trim_to_tri_budget(mesh, tri_target)
        if inspect_glb(mesh).triangles <= tri_target:
            return mesh
        fin = mesh.with_suffix(".fin.glb")
        _run_blender_sculpt(mesh, fin, tri_target, texture_size)
        fin.replace(mesh)
    return mesh


def _run_blender_sculpt(source: Path, output: Path, tri_target: int, texture_size: int) -> None:
    if not BLENDER_CMD.is_file():
        raise RuntimeError("Blender not available for final decimate")
    repo = Path(__file__).resolve().parents[2]
    proc = subprocess.run(
        [
            str(BLENDER_CMD.resolve()),
            "--background",
            "--python",
            str(SCULPT_SCRIPT.resolve()),
            "--",
            str(source.resolve()),
            str(output.resolve()),
            str(tri_target),
            str(texture_size),
        ],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    combined = f"{proc.stdout}\n{proc.stderr}".strip()
    if combined:
        print(combined)
    if proc.returncode != 0 or "Traceback" in combined:
        raise RuntimeError("Blender sculpt_to_game_mesh failed")


def prepare_unit_low_meshes(
    unit_id: str,
    showcase_mesh: Path,
    repo_root: Path,
    tri_target: int = DEFAULT_TRI_TARGET,
    texture_size: int = DEFAULT_TEXTURE_SIZE,
) -> tuple[Path, Path]:
    """Coarse simplify + Blender decimate → .tmp/{id}_low.glb and lod1."""
    tmp_dir = repo_root / "content/characters/meshes/.tmp"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    coarse_mesh = tmp_dir / f"{unit_id}_coarse.glb"
    low_mesh = tmp_dir / f"{unit_id}_low.glb"
    lod1_dir = repo_root / "content/characters/meshes/game/lod1"
    lod1_dir.mkdir(parents=True, exist_ok=True)
    lod1_mesh = lod1_dir / f"{unit_id}.glb"
    lod1_target = max(500, int(tri_target * LOD1_TRI_RATIO))
    lod1_tex = max(256, texture_size // 2)

    coarsen_glb(showcase_mesh, coarse_mesh)
    _run_blender_sculpt(coarse_mesh, low_mesh, tri_target, texture_size)
    _refine_to_budget(low_mesh, tri_target, texture_size)
    _run_blender_sculpt(low_mesh, lod1_mesh, lod1_target, lod1_tex)
    _refine_to_budget(lod1_mesh, lod1_target, lod1_tex)
    coarse_mesh.unlink(missing_ok=True)
    return low_mesh, lod1_mesh
