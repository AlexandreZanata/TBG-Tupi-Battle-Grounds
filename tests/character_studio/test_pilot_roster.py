"""Contract tests for studio demo sculpt roster."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

from character_studio.models import CharacterDefinition
from character_studio.validation import validate_definition

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFINITIONS_DIR = REPO_ROOT / "content" / "characters" / "definitions"
MESHES_DIR = REPO_ROOT / "content/characters/meshes"
ROSTER_PATH = REPO_ROOT / "content/characters/pilot_roster.json"
DEMO_IDS = (
    "demo_scout_01",
    "demo_striker_01",
    "demo_guard_01",
    "demo_vanguard_01",
    "demo_reaver_01",
)


@pytest.fixture(scope="module")
def pilot_roster() -> list[dict]:
    data = json.loads(ROSTER_PATH.read_text(encoding="utf-8"))
    return list(data["pilots"])


def test_pilot_roster_manifest_has_five_demo_sculpts(pilot_roster: list[dict]) -> None:
    assert len(pilot_roster) == 5
    assert {entry["id"] for entry in pilot_roster} == set(DEMO_IDS)


@pytest.mark.parametrize("pilot_id", DEMO_IDS)
def test_demo_sculpt_definition_is_mesh3d_with_glb(pilot_id: str) -> None:
    path = DEFINITIONS_DIR / f"{pilot_id}.json"
    defn = CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))
    validate_definition(defn)
    assert defn.assetMode == "mesh3d"
    assert defn.mesh == f"content/characters/meshes/{pilot_id}.glb"
    glb_path = REPO_ROOT / defn.mesh
    assert glb_path.is_file(), f"missing glb: {glb_path}"
    assert glb_path.stat().st_size > 1_000_000


def test_pilot_definitions_match_roster_manifest(pilot_roster: list[dict]) -> None:
    for entry in pilot_roster:
        pilot_id = entry["id"]
        path = DEFINITIONS_DIR / f"{pilot_id}.json"
        defn = CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))
        assert defn.behaviorProfile == entry["behaviorProfile"]
        assert defn.materialProfile == entry["materialProfile"]
        assert defn.animatorController == entry["animatorController"]


def test_provision_pilot_meshes_script_skips_existing_demo_glb() -> None:
    script = REPO_ROOT / "tools/unity/provision_pilot_meshes.py"
    result = subprocess.run(
        [sys.executable, str(script)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    for pilot_id in DEMO_IDS:
        assert (MESHES_DIR / f"{pilot_id}.glb").is_file()
