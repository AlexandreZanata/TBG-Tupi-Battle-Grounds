"""Integration tests against real content/characters paths in the repository."""

from __future__ import annotations

import pytest

from character_studio.card_renderer import render_card
from character_studio.paths import find_repo_root
from character_studio.repository import CharacterRepository

STUDIO_UNITS = [
    ("demo_scout_01", "Ranged", "Humans", "Scout Pilot"),
    ("demo_striker_01", "DPS", "Humans", "Striker Pilot"),
    ("demo_guard_01", "Tank", "Humans", "Guard Pilot"),
    ("demo_vanguard_01", "Control", "Humans", "Vanguard Pilot"),
    ("demo_reaver_01", "DPS", "Humans", "Reaver Pilot"),
    ("demo_hunter_01", "Ranged", "Humans", "Hunter Pilot"),
    ("demo_bruiser_01", "DPS", "Humans", "Bruiser Pilot"),
    ("demo_sentinel_01", "Support", "Humans", "Sentinel Pilot"),
    ("test_procedural_spawn", "Tank", "Humans", "Test Procedural Fixture"),
]

MESH3D_IDS = {
    "demo_scout_01",
    "demo_striker_01",
    "demo_guard_01",
    "demo_vanguard_01",
    "demo_reaver_01",
    "demo_hunter_01",
    "demo_bruiser_01",
    "demo_sentinel_01",
}


@pytest.fixture(scope="module")
def repo() -> CharacterRepository:
    return CharacterRepository(find_repo_root())


def test_repo_root_has_character_content() -> None:
    root = find_repo_root()
    assert (root / "content" / "characters" / "definitions").is_dir()


@pytest.mark.parametrize("unit_id,archetype,faction,name", STUDIO_UNITS)
def test_studio_definition_exists_with_card(
    repo: CharacterRepository, unit_id: str, archetype: str, faction: str, name: str
) -> None:
    defn = repo.load(unit_id)
    assert defn.archetype == archetype
    assert defn.faction == faction
    assert defn.characterName == name
    card = repo.card_path(unit_id)
    assert card.is_file()
    assert card.stat().st_size > 500
    if unit_id in MESH3D_IDS:
        assert defn.assetMode == "mesh3d"


def test_export_all_cards_idempotent(repo: CharacterRepository) -> None:
    for unit_id, *_ in STUDIO_UNITS:
        defn = repo.load(unit_id)
        path = repo.card_path(unit_id)
        render_card(defn, path)
        first = path.read_bytes()
        render_card(defn, path)
        second = path.read_bytes()
        assert len(second) > 500
        assert first == second
