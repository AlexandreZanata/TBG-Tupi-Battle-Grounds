"""Manual part transforms for socket placement."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class PartTransform:
    translation: tuple[float, float, float]
    rotation: tuple[float, float, float, float]
    scale: tuple[float, float, float]


def parse_attachments(data: dict[str, Any]) -> dict[str, PartTransform]:
    out: dict[str, PartTransform] = {}
    for item in data.get("attachments", []) or []:
        part_id = str(item.get("partId", ""))
        if not part_id:
            continue
        out[part_id] = PartTransform(
            translation=_vec3(item.get("translation"), (0.0, 0.0, 0.0)),
            rotation=_vec4(item.get("rotation"), (0.0, 0.0, 0.0, 1.0)),
            scale=_vec3(item.get("scale"), (1.0, 1.0, 1.0)),
        )
    return out


def transform_node(node: dict, transform: PartTransform | None) -> None:
    if transform is None:
        return
    node["translation"] = list(transform.translation)
    node["rotation"] = list(transform.rotation)
    node["scale"] = list(transform.scale)


def _vec3(value: Any, default: tuple[float, float, float]) -> tuple[float, float, float]:
    if not isinstance(value, (list, tuple)) or len(value) != 3:
        return default
    return (float(value[0]), float(value[1]), float(value[2]))


def _vec4(value: Any, default: tuple[float, float, float, float]) -> tuple[float, float, float, float]:
    if not isinstance(value, (list, tuple)) or len(value) != 4:
        return default
    return (float(value[0]), float(value[1]), float(value[2]), float(value[3]))
