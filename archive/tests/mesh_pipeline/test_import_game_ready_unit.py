"""Contract tests for unified import_game_ready_unit orchestrator."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
if str(REPO / "tools") not in sys.path:
    sys.path.insert(0, str(REPO / "tools"))

from character_studio.models import DEFAULT_RIG_PROFILE, CharacterDefinition  # noqa: E402
from mesh_pipeline.batch_weight_gate import audit_weight_gate, format_weight_gate_table  # noqa: E402
from mesh_pipeline.import_game_ready_core import (  # noqa: E402
    ImportGameReadyConfig,
    EXCEPTION_WORKFLOW_DOC,
    import_game_ready_unit,
    rig_exception_block_message,
)


def test_rig_profile_defaults_on_parse():
    defn = CharacterDefinition.from_dict(
        {
            "id": "demo_test",
            "characterName": "Test",
            "archetype": "Tank",
            "faction": "Humans",
            "heightScale": 1.0,
            "bodyWidthScale": 1.0,
            "headScale": 1.0,
            "limbLengthScale": 1.0,
            "primaryColor": "#FFFFFF",
            "secondaryColor": "#CCCCCC",
        }
    )
    assert defn.rigProfile == DEFAULT_RIG_PROFILE
    assert defn.rigException is False


def test_rig_exception_block_message_points_at_workflow_doc():
    defn = CharacterDefinition.from_dict(
        {
            "id": "organic_blob",
            "characterName": "Blob",
            "archetype": "Tank",
            "faction": "Plants",
            "heightScale": 1.0,
            "bodyWidthScale": 1.0,
            "headScale": 1.0,
            "limbLengthScale": 1.0,
            "primaryColor": "#FFFFFF",
            "secondaryColor": "#CCCCCC",
            "rigException": True,
            "rigExceptionReason": "non-biped",
        }
    )
    message = rig_exception_block_message(defn)
    assert message is not None
    assert EXCEPTION_WORKFLOW_DOC in message
    assert "non-biped" in message


def test_import_blocked_when_rig_exception(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    repo = tmp_path
    definitions = repo / "content/characters/definitions"
    definitions.mkdir(parents=True)
    (definitions / "blocked_unit.json").write_text(
        json.dumps(
            {
                "id": "blocked_unit",
                "characterName": "Blocked",
                "archetype": "Tank",
                "faction": "Humans",
                "heightScale": 1.0,
                "bodyWidthScale": 1.0,
                "headScale": 1.0,
                "limbLengthScale": 1.0,
                "primaryColor": "#FFFFFF",
                "secondaryColor": "#CCCCCC",
                "rigException": True,
            }
        ),
        encoding="utf-8",
    )
    source = tmp_path / "raw.glb"
    source.write_bytes(b"glb")

    monkeypatch.setattr(
        "mesh_pipeline.import_game_ready_core.repo_root",
        lambda: repo,
    )
    report = import_game_ready_unit(
        ImportGameReadyConfig(source=source, unit_id="blocked_unit"),
        repo=repo,
    )
    assert report.blocked
    assert report.block_reason
    assert not report.steps


def test_weight_gate_table_includes_pass_fail_columns():
    rows = [
        audit_weight_gate(REPO, "demo_scout_01"),
    ]
    table = format_weight_gate_table(rows)
    assert "unit_id" in table
    assert "weight_gate" in table
    assert "demo_scout_01" in table


def test_import_cli_exists():
    cli = REPO / "tools/blender/import_game_ready_unit.py"
    assert cli.is_file()


def test_verify_sh_exports_weight_gate_env():
    text = (REPO / "scripts/verify.sh").read_text(encoding="utf-8")
    assert "TBG_WEIGHT_GATE=1" in text


def test_batch_script_exports_weight_gate_default():
    text = (REPO / "tools/mesh-pipeline/batch_seed_to_game.sh").read_text(encoding="utf-8")
    assert "TBG_WEIGHT_GATE" in text
