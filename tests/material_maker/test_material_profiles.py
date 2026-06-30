"""Contract tests for material profile JSON files."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
MATERIALS_DIR = REPO_ROOT / "content" / "characters" / "materials"
REQUIRED_SLOTS = frozenset({"albedo", "normal", "orm"})
FACTION_PROFILES = {
    "Humans": "humans_pbr_v1",
    "Undead": "undead_pbr_v1",
    "Machines": "machines_pbr_v1",
    "Plants": "plants_pbr_v1",
}


def _load_profile(profile_id: str) -> dict:
    path = MATERIALS_DIR / f"{profile_id}.json"
    assert path.is_file(), f"missing profile: {path}"
    return json.loads(path.read_text(encoding="utf-8"))


@pytest.mark.parametrize("faction,profile_id", list(FACTION_PROFILES.items()))
def test_faction_material_profile_exists(faction: str, profile_id: str) -> None:
    data = _load_profile(profile_id)
    assert data["id"] == profile_id
    assert data["faction"] == faction
    assert data["shader"] == "TBG/CharacterPBR"
    assert REQUIRED_SLOTS.issubset(set(data["textureSlots"]))


def test_all_profiles_reference_existing_graphs() -> None:
    for profile_id in FACTION_PROFILES.values():
        data = _load_profile(profile_id)
        graph_path = REPO_ROOT / data["graph"]
        assert graph_path.is_file(), f"missing graph for {profile_id}: {graph_path}"


def test_profile_properties_are_numeric() -> None:
    for profile_id in FACTION_PROFILES.values():
        props = _load_profile(profile_id)["properties"]
        for key in ("_Metallic", "_Smoothness", "_OcclusionStrength", "_EmissionStrength"):
            assert key in props
            assert isinstance(props[key], (int, float))
