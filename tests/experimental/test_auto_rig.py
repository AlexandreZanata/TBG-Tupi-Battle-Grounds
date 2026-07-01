"""Contract and integration tests for experimental auto-rig pipeline."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
AUTO_RIG_DIR = REPO / "tools/experimental/auto_rig"
TOOLS = REPO / "tools"
for path in (TOOLS, AUTO_RIG_DIR):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from mesh_pipeline.glb_inspect import inspect_glb, skinned_mesh_has_weights  # noqa: E402
from run_auto_rig import blender_available, run_auto_rig_mesh  # noqa: E402

AUTO_RIG_SCRIPT = AUTO_RIG_DIR / "auto_rig_mesh.py"
SHOWCASE = REPO / "content/characters/meshes/showcase/demo_bruiser_01.glb"
TMP_DIR = REPO / ".local/tmp/pytest_auto_rig"


@pytest.fixture
def rigged_output() -> Path:
    TMP_DIR.mkdir(parents=True, exist_ok=True)
    path = TMP_DIR / "rigged.glb"
    if path.exists():
        path.unlink()
    yield path
    path.unlink(missing_ok=True)


def test_auto_rig_source_contract():
    mesh_text = AUTO_RIG_SCRIPT.read_text(encoding="utf-8")
    ops_text = (AUTO_RIG_DIR / "auto_rig_ops.py").read_text(encoding="utf-8")
    assert "auto_rig_mesh" in mesh_text
    assert "bind_mesh" in ops_text
    assert "BIND_FALLBACK" in ops_text
    assert "load_template_armature" in ops_text


def test_experimental_readme_documents_cli():
    readme = (AUTO_RIG_DIR / "README.md").read_text(encoding="utf-8")
    assert "auto_rig_mesh.py" in readme
    assert "bind-mode" in readme
    assert "ARMATURE_AUTO" in readme


@pytest.mark.skipif(not blender_available(), reason="Blender not available")
@pytest.mark.skipif(not SHOWCASE.is_file(), reason="showcase pilot missing")
def test_auto_rig_produces_skinned_glb(rigged_output: Path) -> None:
    input_stats = inspect_glb(SHOWCASE)
    result = run_auto_rig_mesh(
        SHOWCASE,
        rigged_output,
        body_class="HumanoidLight",
        voxel_remesh=True,
        bind_mode="fallback",
    )
    assert rigged_output.is_file()
    assert result.report["bone_count"] > 0
    assert result.report["height"] >= 1.6
    assert result.report["loose_verts"] == 0

    output_stats = inspect_glb(rigged_output)
    assert output_stats.skins >= 1
    assert output_stats.bones >= result.report["bone_count"]
    assert skinned_mesh_has_weights(rigged_output)
    assert output_stats.animations == 0

    if input_stats.skins == 0:
        assert output_stats.skins > input_stats.skins
