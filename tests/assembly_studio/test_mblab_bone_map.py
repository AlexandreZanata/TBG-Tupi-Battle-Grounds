"""Rename map contract — MB-Lab base skeleton → TBG_*."""

from __future__ import annotations

import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
MAP_PATH = REPO / "tools" / "assembly_studio" / "blender" / "mblab_bone_map.json"
REQUIRED_TBG = (
    "TBG_Root", "TBG_Spine", "TBG_Spine1", "TBG_Neck", "TBG_Head",
    "TBG_Shoulder.L", "TBG_UpperArm.L", "TBG_LowerArm.L", "TBG_Hand.L",
    "TBG_Shoulder.R", "TBG_UpperArm.R", "TBG_LowerArm.R", "TBG_Hand.R",
    "TBG_UpperLeg.L", "TBG_LowerLeg.L", "TBG_Foot.L",
    "TBG_UpperLeg.R", "TBG_LowerLeg.R", "TBG_Foot.R",
)


def test_mblab_map_targets_required_tbg_bones() -> None:
    data = json.loads(MAP_PATH.read_text(encoding="utf-8"))
    targets = set(data["mapping"].values())
    missing = [bone for bone in REQUIRED_TBG if bone not in targets]
    assert missing == [], f"missing TBG targets: {missing}"
    assert data["mblab_version"] == "1.8.1"
