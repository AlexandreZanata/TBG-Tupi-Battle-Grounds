"""Visual grammar defaults — single source for studio archetype and faction baselines."""

from __future__ import annotations

from dataclasses import dataclass

from .models import CharacterDefinition


@dataclass(frozen=True)
class ProportionSet:
    heightScale: float
    bodyWidthScale: float
    headScale: float
    limbLengthScale: float


ARCHETYPE_PROPORTIONS: dict[str, ProportionSet] = {
    "Tank": ProportionSet(1.2, 1.8, 0.8, 0.7),
    "DPS": ProportionSet(1.0, 0.8, 0.9, 1.1),
    "Support": ProportionSet(0.9, 0.7, 1.2, 0.8),
    "Ranged": ProportionSet(1.1, 0.6, 0.8, 1.3),
    "Control": ProportionSet(1.0, 0.7, 1.1, 0.9),
}

FACTION_COLORS: dict[str, tuple[str, str]] = {
    "Humans": ("#8B7355", "#C4A882"),
    "Undead": ("#4A5D23", "#7D8B55"),
    "Machines": ("#4A6FA5", "#B0C4DE"),
    "Plants": ("#2D5A27", "#6B8E23"),
}

SHAPE_VOCABULARY = {
    "head": "Cube",
    "torso": "Cube",
    "limbs": "Capsule",
    "feet": "Cube (flattened)",
    "accessories": ["Shield", "Weapon", "Helmet", "ArmorBlock"],
}


@dataclass(frozen=True)
class EvolutionVisualModifiers:
    stage: int
    scale_multiplier: float
    add_accessory: str | None
    saturate_colors: bool


EVOLUTION_MODIFIERS: dict[int, EvolutionVisualModifiers] = {
    0: EvolutionVisualModifiers(0, 1.0, None, False),
    1: EvolutionVisualModifiers(1, 1.1, "ArmorBlock", False),
    2: EvolutionVisualModifiers(2, 1.2, None, True),
}


def default_accessories_for_archetype(archetype: str) -> list[str]:
    mapping = {
        "Tank": ["Shield"],
        "DPS": ["Weapon"],
        "Support": ["Helmet"],
        "Ranged": ["Weapon"],
        "Control": ["Helmet"],
    }
    return list(mapping.get(archetype, []))


def create_default_definition(
    unit_id: str,
    archetype: str,
    faction: str,
    character_name: str | None = None,
) -> CharacterDefinition:
    props = ARCHETYPE_PROPORTIONS[archetype]
    primary, secondary = FACTION_COLORS[faction]
    return CharacterDefinition(
        id=unit_id,
        characterName=character_name or unit_id.replace("_", " ").title(),
        archetype=archetype,
        faction=faction,
        heightScale=props.heightScale,
        bodyWidthScale=props.bodyWidthScale,
        headScale=props.headScale,
        limbLengthScale=props.limbLengthScale,
        primaryColor=primary,
        secondaryColor=secondary,
        accessories=default_accessories_for_archetype(archetype),
        evolutionStage=0,
        assetMode="procedural",
    )
