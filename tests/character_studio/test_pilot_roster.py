"""Contract tests for Phase 02 mesh3d pilot roster."""

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
TEXTURES_DIR = REPO_ROOT / "content/characters/textures"
ROSTER_PATH = REPO_ROOT / "content/characters/pilot_roster.json"
REQUIRED_TEXTURES = ("albedo", "normal", "orm")


@pytest.fixture(scope="module")
def pilot_roster() -> list[dict]:
    data = json.loads(ROSTER_PATH.read_text(encoding="utf-8"))
    return list(data["pilots"])


def test_pilot_roster_manifest_has_four_factions(pilot_roster: list[dict]) -> None:
    assert len(pilot_roster) == 4
    factions = {entry["faction"] for entry in pilot_roster}
    assert factions == {"Humans", "Undead", "Machines", "Plants"}


@pytest.mark.parametrize(
    "pilot_id",
    ["tank_human_01", "dps_undead_01", "support_machine_01", "ranged_plant_01"],
)
def test_pilot_definition_is_mesh3d_with_assets(pilot_id: str) -> None:
    path = DEFINITIONS_DIR / f"{pilot_id}.json"
    defn = CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))
    validate_definition(defn)
    assert defn.assetMode == "mesh3d"
    assert defn.mesh == f"content/characters/meshes/{pilot_id}.glb"
    assert defn.textures is not None
    for slot in REQUIRED_TEXTURES:
        assert getattr(defn.textures, slot)
        texture_path = REPO_ROOT / getattr(defn.textures, slot)
        assert texture_path.is_file(), f"missing texture: {texture_path}"
    glb_path = REPO_ROOT / defn.mesh
    assert glb_path.is_file(), f"missing glb: {glb_path}"


def test_pilot_definitions_match_roster_manifest(pilot_roster: list[dict]) -> None:
    for entry in pilot_roster:
        pilot_id = entry["id"]
        path = DEFINITIONS_DIR / f"{pilot_id}.json"
        defn = CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))
        assert defn.behaviorProfile == entry["behaviorProfile"]
        assert defn.materialProfile == entry["materialProfile"]
        assert defn.animatorController == entry["animatorController"]


def test_provision_pilot_meshes_script() -> None:
    script = REPO_ROOT / "tools/unity/provision_pilot_meshes.py"
    result = subprocess.run(
        [sys.executable, str(script)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    for pilot_id in (
        "tank_human_01",
        "dps_undead_01",
        "support_machine_01",
        "ranged_plant_01",
    ):
        assert (MESHES_DIR / f"{pilot_id}.glb").is_file()
