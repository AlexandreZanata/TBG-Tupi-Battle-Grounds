"""Contract tests for Phase 04b §7 public docs and quality gate documentation."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
BIPED_DOC = REPO / "docs/design/biped-game-ready-pipeline.md"
ADR_006 = REPO / "docs/adr/ADR-006-biped-auto-rig-pipeline.md"
REAL_3D = REPO / "docs/design/real-3d-character-pipeline.md"
IMPORT_CLI = REPO / "tools/blender/import_game_ready_unit.py"

if str(REPO / "tools") not in sys.path:
    sys.path.insert(0, str(REPO / "tools"))

from mesh_pipeline.prep_ai_mesh_run import blender_available  # noqa: E402


def test_biped_game_ready_pipeline_doc_exists_and_links():
    text = BIPED_DOC.read_text(encoding="utf-8")
    assert "import_game_ready_unit.py" in text
    assert "Quality gate status" in text
    assert "mesh-pipeline/README.md" in text
    assert ADR_006.name in text


def test_adr_006_records_auto_rig_decision():
    text = ADR_006.read_text(encoding="utf-8")
    assert "Accepted" in text
    assert "TBG_WEIGHT_GATE" in text
    assert "rigException" in text


def test_real_3d_pipeline_links_biped_path():
    text = REAL_3D.read_text(encoding="utf-8")
    assert "biped-game-ready-pipeline.md" in text
    assert "ADR-006" in text


THREE_PILOT_IMPORTS = ("demo_scout_01", "demo_warden_01", "demo_shade_01")


@pytest.mark.skipif(not blender_available(), reason="Blender not available")
@pytest.mark.parametrize("unit_id", THREE_PILOT_IMPORTS)
def test_import_game_ready_unit_runs_pipeline_steps(unit_id: str):
    showcase = REPO / "content/characters/meshes/showcase" / f"{unit_id}.glb"
    if not showcase.is_file():
        pytest.skip(f"missing showcase: {showcase}")
    proc = subprocess.run(
        [
            sys.executable,
            str(IMPORT_CLI),
            "--source",
            str(showcase),
            "--id",
            unit_id,
            "--skip-prep",
            "--report-json",
        ],
        cwd=REPO,
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode in (0, 1), proc.stderr
    report_line = next(
        (line for line in reversed(proc.stdout.splitlines()) if line.strip().startswith("{")),
        "",
    )
    assert report_line, proc.stdout
    report = json.loads(report_line)
    assert report["unit_id"] == unit_id
    for step in ("showcase_import", "export_game_unit", "weight_gate", "validate_game_glb"):
        assert step in report["steps"]
