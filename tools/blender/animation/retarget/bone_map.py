"""Load and validate CMU → TBG bone mapping."""

from __future__ import annotations

import json
from pathlib import Path

MAP_PATH = Path(__file__).resolve().parent / "BONE_MAP_CMU_TO_TBG.json"

REQUIRED_TBG_BONES = (
    "TBG_Root",
    "TBG_Spine",
    "TBG_Spine1",
    "TBG_Neck",
    "TBG_Head",
    "TBG_Shoulder.L",
    "TBG_UpperArm.L",
    "TBG_LowerArm.L",
    "TBG_Hand.L",
    "TBG_Shoulder.R",
    "TBG_UpperArm.R",
    "TBG_LowerArm.R",
    "TBG_Hand.R",
    "TBG_UpperLeg.L",
    "TBG_LowerLeg.L",
    "TBG_Foot.L",
    "TBG_UpperLeg.R",
    "TBG_LowerLeg.R",
    "TBG_Foot.R",
)


def load_bone_map(path: Path | None = None) -> dict:
    data = json.loads((path or MAP_PATH).read_text(encoding="utf-8"))
    missing = [bone for bone in REQUIRED_TBG_BONES if bone not in data.get("mapping", {})]
    if missing:
        raise ValueError(f"bone map missing TBG bones: {missing}")
    return data


def cmu_joint_for_tbg(bone_map: dict, tbg_bone: str) -> str:
    return bone_map["mapping"][tbg_bone]


def required_tbg_bones() -> tuple[str, ...]:
    return REQUIRED_TBG_BONES
