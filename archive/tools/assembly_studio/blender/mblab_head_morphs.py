"""MB-Lab head / face structural morphs (native mesh, not socket parts)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

CONFIG_PATH = (
    Path(__file__).resolve().parents[3] / "content" / "characters" / "library" / "mblab_head_morphs.json"
)


def load_head_morph_config() -> dict[str, Any]:
    return json.loads(CONFIG_PATH.read_text(encoding="utf-8"))


def ui_to_mblab_value(slider_0_100: float) -> float:
    """Browser 0..1 maps directly to MB-Lab structural 0..1 (0.5 = neutral)."""
    return max(0.0, min(1.0, float(slider_0_100)))


def apply_head_morphs(humanoid: Any, morphs: dict[str, float]) -> list[str]:
    cfg = load_head_morph_config()
    id_to_prop = {s["id"]: s["mblab"] for s in cfg.get("sliders", [])}
    applied: list[str] = []
    data = getattr(humanoid, "character_data", None)
    if not data:
        return applied
    for ui_id, raw in morphs.items():
        prop = id_to_prop.get(ui_id)
        if not prop or prop not in data:
            continue
        data[prop] = ui_to_mblab_value(raw)
        applied.append(ui_id)
    return applied


def head_morph_ids() -> frozenset[str]:
    cfg = load_head_morph_config()
    return frozenset(s["id"] for s in cfg.get("sliders", []))
