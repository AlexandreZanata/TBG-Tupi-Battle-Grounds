"""Contract tests for MB-Lab head morph map."""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools" / "assembly_studio" / "blender"))

from mblab_head_morphs import (  # noqa: E402
    CONFIG_PATH,
    head_morph_ids,
    load_head_morph_config,
    ui_to_mblab_value,
)


def test_head_morph_config_exists() -> None:
    assert CONFIG_PATH.is_file()
    cfg = load_head_morph_config()
    assert cfg["neutral"] == 0.5
    ids = [s["id"] for s in cfg["sliders"]]
    assert "head_width" in ids
    assert len(ids) == len(head_morph_ids())


def test_ui_maps_to_mblab_neutral_at_half() -> None:
    assert ui_to_mblab_value(0.5) == 0.5


def test_head_morph_mblab_keys_are_strings() -> None:
    cfg = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    for slider in cfg["sliders"]:
        assert slider["mblab"].startswith(("Head_", "Jaw_", "Cheeks_"))
