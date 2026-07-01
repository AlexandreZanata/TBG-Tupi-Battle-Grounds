"""Pytest suite for Character Studio core — contract-first."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from character_studio.defaults import (
    ARCHETYPE_PROPORTIONS,
    FACTION_COLORS,
    SHAPE_VOCABULARY,
    create_default_definition,
)
from character_studio.models import CharacterDefinition
from character_studio.repository import CharacterRepository
from character_studio.validation import ValidationError, validate_definition


@pytest.fixture
def repo(tmp_path: Path) -> CharacterRepository:
    (tmp_path / "content" / "characters" / "definitions").mkdir(parents=True)
    (tmp_path / "content" / "characters" / "cards").mkdir(parents=True)
    (tmp_path / "content" / "characters" / "previews").mkdir(parents=True)
    return CharacterRepository(tmp_path)


def test_archetype_proportions_cover_all_archetypes() -> None:
    expected = {"Tank", "DPS", "Support", "Ranged", "Control"}
    assert set(ARCHETYPE_PROPORTIONS) == expected


def test_faction_colors_cover_all_factions() -> None:
    expected = {"Humans", "Undead", "Machines", "Plants"}
    assert set(FACTION_COLORS) == expected


def test_shape_vocabulary_locked() -> None:
    assert SHAPE_VOCABULARY["head"] == "Cube"
    assert SHAPE_VOCABULARY["limbs"] == "Capsule"
    assert "Shield" in SHAPE_VOCABULARY["accessories"]


def test_create_default_definition_uses_faction_palette() -> None:
    defn = create_default_definition("tank_human_01", "Tank", "Humans")
    assert defn.primaryColor == FACTION_COLORS["Humans"][0]
    assert defn.heightScale == ARCHETYPE_PROPORTIONS["Tank"].heightScale


def test_save_and_load_round_trip(repo: CharacterRepository) -> None:
    defn = create_default_definition("dps_undead_01", "DPS", "Undead", "Bone Striker")
    repo.create(defn)
    loaded = repo.load("dps_undead_01")
    assert loaded.characterName == "Bone Striker"
    assert loaded.archetype == "DPS"


def test_reject_duplicate_id(repo: CharacterRepository) -> None:
    repo.create(create_default_definition("tank_human_01", "Tank", "Humans"))
    duplicate = create_default_definition("tank_human_01", "DPS", "Machines")
    with pytest.raises(ValidationError):
        repo.create(duplicate)


def test_delete_removes_json_and_card(repo: CharacterRepository, tmp_path: Path) -> None:
    from character_studio.card_renderer import render_card

    defn = create_default_definition("support_machine_01", "Support", "Machines")
    repo.create(defn)
    render_card(defn, repo.card_path(defn.id))
    repo.delete(defn.id)
    assert not (repo._definitions / "support_machine_01.json").exists()
    assert not repo.card_path("support_machine_01").exists()


def test_invalid_archetype_raises(repo: CharacterRepository) -> None:
    defn = create_default_definition("bad_unit", "Tank", "Humans")
    bad = CharacterDefinition.from_dict({**defn.to_dict(), "archetype": "Flyer"})
    with pytest.raises(ValidationError):
        repo.save(bad)


def test_pilot_roster_contract(repo: CharacterRepository) -> None:
    pilots = [
        ("tank_human_01", "Tank", "Humans", "Iron Guard"),
        ("dps_undead_01", "DPS", "Undead", "Bone Striker"),
        ("support_machine_01", "Support", "Machines", "Gear Medic"),
        ("ranged_plant_01", "Ranged", "Plants", "Thorn Sniper"),
    ]
    for unit_id, archetype, faction, name in pilots:
        repo.create(create_default_definition(unit_id, archetype, faction, name))
    assert repo.list_ids() == sorted(p[0] for p in pilots)
