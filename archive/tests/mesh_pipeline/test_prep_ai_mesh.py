"""Contract and integration tests for prep_ai_mesh normalization."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
TOOLS = REPO / "tools"
if str(TOOLS) not in sys.path:
    sys.path.insert(0, str(TOOLS))

from mesh_pipeline.prep_ai_mesh_run import blender_available, run_prep_ai_mesh  # noqa: E402

PREP_SCRIPT = REPO / "tools/blender/scripts/prep_ai_mesh.py"
SHOWCASE = REPO / "content/characters/meshes/showcase/demo_titan_01.glb"
HEIGHT_MIN = 1.6
HEIGHT_MAX = 2.0
GROUND_EPS = 0.02
CENTER_EPS = 0.05
PREP_TMP_DIR = REPO / ".local/tmp/pytest_prep"


@pytest.fixture
def prep_output() -> Path:
    PREP_TMP_DIR.mkdir(parents=True, exist_ok=True)
    path = PREP_TMP_DIR / "prepped.glb"
    if path.exists():
        path.unlink()
    yield path
    path.unlink(missing_ok=True)


def test_prep_ai_mesh_source_contract():
    text = PREP_SCRIPT.read_text(encoding="utf-8")
    ops = (REPO / "tools/blender/scripts/prep_ai_mesh_ops.py").read_text(encoding="utf-8")
    assert "delete_loose" in ops
    assert "remove_doubles" in ops
    assert "loose_verts" in text
    assert "VOXEL_AUTO_THRESHOLD = 500_000" in ops


def test_import_showcase_exposes_prep_flag():
    text = (REPO / "tools/mesh-pipeline/import_showcase_sculpts.py").read_text(encoding="utf-8")
    assert "--prep" in text
    assert "run_prep_ai_mesh" in text


@pytest.mark.skipif(not blender_available(), reason="Blender not available")
@pytest.mark.skipif(not SHOWCASE.is_file(), reason="showcase pilot missing")
def test_prep_ai_mesh_normalizes_height_and_ground(prep_output: Path) -> None:
    result = run_prep_ai_mesh(SHOWCASE, prep_output)
    assert prep_output.is_file()
    assert result.report["loose_verts"] == 0
    assert HEIGHT_MIN <= result.report["height"] <= HEIGHT_MAX
    assert result.report["min_y"] >= -GROUND_EPS
    assert abs(result.report["center_x"]) <= CENTER_EPS
    assert abs(result.report["center_z"]) <= CENTER_EPS


@pytest.mark.skipif(not blender_available(), reason="Blender not available")
@pytest.mark.skipif(not SHOWCASE.is_file(), reason="showcase pilot missing")
def test_prep_ai_mesh_cli_report_json(prep_output: Path) -> None:
    cmd = [
        str(REPO / "tools/blender/blender_cmd.sh"),
        "--background",
        "--python",
        str(PREP_SCRIPT),
        "--",
        str(SHOWCASE.resolve()),
        str(prep_output.resolve()),
        "--report-json",
    ]
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=True)
    assert '"loose_verts": 0' in proc.stdout
    assert prep_output.is_file()
