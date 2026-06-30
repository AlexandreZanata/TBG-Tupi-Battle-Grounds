"""Character definition data model — mirrors Unity CharacterDefinition JSON contract."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


ARCHETYPES = frozenset({"Tank", "DPS", "Support", "Ranged", "Control"})
FACTIONS = frozenset({"Humans", "Undead", "Machines", "Plants"})
ACCESSORIES = frozenset({"Shield", "Weapon", "Helmet", "ArmorBlock"})
ASSET_MODES = frozenset({"procedural", "mesh3d"})
DEFAULT_ASSET_MODE = "procedural"
PROPORTION_MIN = 0.1
PROPORTION_MAX = 3.0
EVOLUTION_MIN = 0
EVOLUTION_MAX = 2


@dataclass
class CharacterTextures:
    albedo: str
    normal: str
    orm: str
    emissive: str | None = None

    def to_dict(self) -> dict[str, str]:
        data: dict[str, str] = {
            "albedo": self.albedo,
            "normal": self.normal,
            "orm": self.orm,
        }
        if self.emissive:
            data["emissive"] = self.emissive
        return data

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> CharacterTextures:
        return cls(
            albedo=str(data["albedo"]),
            normal=str(data["normal"]),
            orm=str(data["orm"]),
            emissive=str(data["emissive"]) if data.get("emissive") else None,
        )


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
    assetMode: str = DEFAULT_ASSET_MODE
    mesh: str | None = None
    textures: CharacterTextures | None = None
    materialProfile: str | None = None
    behaviorProfile: str | None = None
    animatorController: str | None = None

    def to_dict(self) -> dict[str, Any]:
        data: dict[str, Any] = {
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
            "assetMode": self.assetMode,
        }
        if self.mesh:
            data["mesh"] = self.mesh
        if self.textures:
            data["textures"] = self.textures.to_dict()
        if self.materialProfile:
            data["materialProfile"] = self.materialProfile
        if self.behaviorProfile:
            data["behaviorProfile"] = self.behaviorProfile
        if self.animatorController:
            data["animatorController"] = self.animatorController
        return data

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> CharacterDefinition:
        textures = (
            CharacterTextures.from_dict(data["textures"])
            if isinstance(data.get("textures"), dict)
            else None
        )
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
            assetMode=str(data.get("assetMode", DEFAULT_ASSET_MODE)),
            mesh=str(data["mesh"]) if data.get("mesh") else None,
            textures=textures,
            materialProfile=str(data["materialProfile"]) if data.get("materialProfile") else None,
            behaviorProfile=str(data["behaviorProfile"]) if data.get("behaviorProfile") else None,
            animatorController=(
                str(data["animatorController"]) if data.get("animatorController") else None
            ),
        )
