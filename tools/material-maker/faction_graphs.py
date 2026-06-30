"""Faction graph metadata and pilot roster for texture export."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

FACTION_GRAPHS: dict[str, str] = {
    "Humans": "humans_fabric.mm",
    "Undead": "undead_decay.mm",
    "Machines": "machines_metal.mm",
    "Plants": "plants_organic.mm",
}

PILOT_UNITS: dict[str, str] = {
    "tank_human_01": "Humans",
    "dps_undead_01": "Undead",
    "support_machine_01": "Machines",
    "ranged_plant_01": "Plants",
}


@dataclass(frozen=True)
class GraphDescriptor:
    name: str
    faction: str
    material_profile: str
    outputs: tuple[str, ...]
    primary_rgb: tuple[int, int, int]
    secondary_rgb: tuple[int, int, int]
    roughness_base: float
    metallic_base: float
    ao_strength: float
    pattern_scale: float
    emissive_strength: float


def graph_dir() -> Path:
    return Path(__file__).resolve().parent


def load_graph(faction: str) -> GraphDescriptor:
    filename = FACTION_GRAPHS.get(faction)
    if not filename:
        raise ValueError(f"unknown faction: {faction}")

    path = graph_dir() / filename
    data = json.loads(path.read_text(encoding="utf-8"))
    primary = _hex_to_rgb(data["baseColors"]["primary"])
    secondary = _hex_to_rgb(data["baseColors"]["secondary"])
    params = data["parameters"]
    scale_key = next(
        (k for k in ("weaveScale", "brushScale", "veinScale") if k in params),
        "weaveScale",
    )
    return GraphDescriptor(
        name=str(data["name"]),
        faction=str(data["faction"]),
        material_profile=str(data["materialProfile"]),
        outputs=tuple(str(o) for o in data["outputs"]),
        primary_rgb=primary,
        secondary_rgb=secondary,
        roughness_base=float(params["roughnessBase"]),
        metallic_base=float(params["metallicBase"]),
        ao_strength=float(params["aoStrength"]),
        pattern_scale=float(params[scale_key]),
        emissive_strength=float(params.get("emissiveStrength", 0.0)),
    )


def _hex_to_rgb(value: str) -> tuple[int, int, int]:
    cleaned = value.lstrip("#")
    if len(cleaned) != 6:
        raise ValueError(f"invalid hex color: {value}")
    return (
        int(cleaned[0:2], 16),
        int(cleaned[2:4], 16),
        int(cleaned[4:6], 16),
    )
