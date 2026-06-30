"""Integration tests against real content/characters paths in the repository."""

from __future__ import annotations

from pathlib import Path

import pytest

from character_studio.card_renderer import render_card
from character_studio.defaults import create_default_definition
from character_studio.paths import find_repo_root
from character_studio.repository import CharacterRepository

PILOTS = [
    ("tank_human_01", "Tank", "Humans", "Iron Guard"),
    ("dps_undead_01", "DPS", "Undead", "Bone Striker"),
    ("support_machine_01", "Support", "Machines", "Gear Medic"),
    ("ranged_plant_01", "Ranged", "Plants", "Thorn Sniper"),
]


PILOT_IDS = {unit_id for unit_id, *_ in PILOTS}


@pytest.fixture(scope="module")
def repo() -> CharacterRepository:
    return CharacterRepository(find_repo_root())


def test_repo_root_has_character_content() -> None:
    root = find_repo_root()
    assert (root / "content" / "characters" / "definitions").is_dir()


@pytest.mark.parametrize("unit_id,archetype,faction,name", PILOTS)
def test_pilot_definition_exists_with_card(
    repo: CharacterRepository, unit_id: str, archetype: str, faction: str, name: str
) -> None:
    defn = repo.load(unit_id)
    assert defn.assetMode == "mesh3d"
    assert defn.archetype == archetype
    assert defn.faction == faction
    assert defn.characterName == name
    card = repo.card_path(unit_id)
    assert card.is_file()
    assert card.stat().st_size > 500


def test_export_all_cards_idempotent(repo: CharacterRepository) -> None:
    for unit_id in PILOT_IDS:
        defn = repo.load(unit_id)
        before = repo.card_path(unit_id).read_bytes()
        render_card(defn, repo.card_path(unit_id))
        after = repo.card_path(unit_id).read_bytes()
        assert len(after) > 500
        assert before == after
