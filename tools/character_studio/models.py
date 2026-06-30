"""Character definition data model — mirrors Unity CharacterDefinition JSON contract."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


ARCHETYPES = frozenset({"Tank", "DPS", "Support", "Ranged", "Control"})
FACTIONS = frozenset({"Humans", "Undead", "Machines", "Plants"})
ACCESSORIES = frozenset({"Shield", "Weapon", "Helmet", "ArmorBlock"})
PROPORTION_MIN = 0.1
PROPORTION_MAX = 3.0
EVOLUTION_MIN = 0
EVOLUTION_MAX = 2


@dataclass
class CharacterDefinition:
    id: str
    characterName: str
    archetype: str
    faction: str
    heightScale: float
    bodyWidthScale: float
    headScale: float
    limbLengthScale: float
    primaryColor: str
    secondaryColor: str
    accessories: list[str] = field(default_factory=list)
    evolutionStage: int = 0

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "characterName": self.characterName,
            "archetype": self.archetype,
            "faction": self.faction,
            "heightScale": self.heightScale,
            "bodyWidthScale": self.bodyWidthScale,
            "headScale": self.headScale,
            "limbLengthScale": self.limbLengthScale,
            "primaryColor": self.primaryColor,
            "secondaryColor": self.secondaryColor,
            "accessories": list(self.accessories),
            "evolutionStage": self.evolutionStage,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> CharacterDefinition:
        return cls(
            id=str(data["id"]),
            characterName=str(data["characterName"]),
            archetype=str(data["archetype"]),
            faction=str(data["faction"]),
            heightScale=float(data["heightScale"]),
            bodyWidthScale=float(data["bodyWidthScale"]),
            headScale=float(data["headScale"]),
            limbLengthScale=float(data["limbLengthScale"]),
            primaryColor=str(data["primaryColor"]),
            secondaryColor=str(data["secondaryColor"]),
            accessories=[str(a) for a in data.get("accessories", [])],
            evolutionStage=int(data.get("evolutionStage", 0)),
        )
