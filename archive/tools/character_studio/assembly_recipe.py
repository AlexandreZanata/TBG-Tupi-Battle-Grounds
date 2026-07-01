"""Optional assembly recipe on character definitions."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class AssemblyRecipe:
    unitId: str
    baseId: str
    parts: list[str] = field(default_factory=list)
    materialId: str = "default"

    def to_dict(self) -> dict[str, Any]:
        return {
            "unitId": self.unitId,
            "baseId": self.baseId,
            "parts": list(self.parts),
            "materialId": self.materialId,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> AssemblyRecipe:
        return cls(
            unitId=str(data["unitId"]),
            baseId=str(data["baseId"]),
            parts=[str(p) for p in data.get("parts", [])],
            materialId=str(data.get("materialId", "default")),
        )
