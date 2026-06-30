"""Contract tests for Phase 02 mesh3d schema extension."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from character_studio.defaults import create_default_definition
from character_studio.models import CharacterDefinition, CharacterTextures
from character_studio.repository import CharacterRepository
from character_studio.validation import ValidationError, validate_definition

REPO_ROOT = Path(__file__).resolve().parents[2]
MESH3D_EXAMPLE = REPO_ROOT / "content/characters/definitions/_schema.mesh3d.example.json"
PILOT_IDS = {"tank_human_01", "dps_undead_01", "support_machine_01", "ranged_plant_01"}


@pytest.fixture
def repo(tmp_path: Path) -> CharacterRepository:
    (tmp_path / "content" / "characters" / "definitions").mkdir(parents=True)
    (tmp_path / "content" / "characters" / "cards").mkdir(parents=True)
    (tmp_path / "content" / "characters" / "previews").mkdir(parents=True)
    return CharacterRepository(tmp_path)


def _mesh3d_definition(unit_id: str = "tank_human_01") -> CharacterDefinition:
    base = create_default_definition(unit_id, "Tank", "Humans")
    return CharacterDefinition(
        **{
            **base.to_dict(),
            "assetMode": "mesh3d",
            "mesh": f"content/characters/meshes/{unit_id}.glb",
            "textures": CharacterTextures(
                albedo=f"content/characters/textures/{unit_id}_albedo.png",
                normal=f"content/characters/textures/{unit_id}_normal.png",
                orm=f"content/characters/textures/{unit_id}_orm.png",
            ),
            "materialProfile": "humans_pbr_v1",
            "behaviorProfile": "tank_melee_v1",
        }
    )


def test_procedural_definition_without_asset_mode_still_valid(repo: CharacterRepository) -> None:
    defn = create_default_definition("tank_human_01", "Tank", "Humans")
    repo.create(defn)
    loaded = repo.load("tank_human_01")
    assert loaded.assetMode == "procedural"
    assert loaded.mesh is None


def test_existing_pilot_json_files_default_to_procedural() -> None:
    definitions = REPO_ROOT / "content" / "characters" / "definitions"
    for path in sorted(definitions.glob("*.json")):
        if path.stem not in PILOT_IDS:
            continue
        defn = CharacterDefinition.from_dict(json.loads(path.read_text(encoding="utf-8")))
        validate_definition(defn)
        assert defn.assetMode == "procedural"


def test_mesh3d_requires_mesh_path() -> None:
    defn = _mesh3d_definition()
    defn.mesh = None
    with pytest.raises(ValidationError, match="requires mesh path"):
        validate_definition(defn)


def test_mesh3d_rejects_non_glb_mesh_path() -> None:
    defn = _mesh3d_definition()
    defn.mesh = "content/characters/meshes/tank_human_01.fbx"
    with pytest.raises(ValidationError, match="\\.glb"):
        validate_definition(defn)


def test_mesh3d_valid_with_mesh_and_textures(repo: CharacterRepository) -> None:
    defn = _mesh3d_definition("dps_undead_01")
    repo.create(defn)
    loaded = repo.load("dps_undead_01")
    assert loaded.assetMode == "mesh3d"
    assert loaded.mesh.endswith(".glb")
    assert loaded.textures is not None
    assert loaded.textures.albedo.endswith("_albedo.png")


def test_mesh3d_example_json_matches_contract() -> None:
    data = json.loads(MESH3D_EXAMPLE.read_text(encoding="utf-8"))
    defn = CharacterDefinition.from_dict(data)
    validate_definition(defn)
    assert defn.assetMode == "mesh3d"


def test_invalid_asset_mode_rejected() -> None:
    defn = create_default_definition("bad_mode", "Tank", "Humans")
    bad = CharacterDefinition.from_dict({**defn.to_dict(), "assetMode": "fbx"})
    with pytest.raises(ValidationError, match="invalid assetMode"):
        validate_definition(bad)


def test_textures_object_requires_core_maps() -> None:
    defn = _mesh3d_definition()
    defn.textures = CharacterTextures(albedo="", normal="n.png", orm="o.png")
    with pytest.raises(ValidationError, match="textures.albedo"):
        validate_definition(defn)
