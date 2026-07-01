"""Rig session archive contract tests."""

from __future__ import annotations

import json
from pathlib import Path

from tools.experimental.auto_rig.archive_rig_session import ARCHIVE_ROOT

CANONICAL_BONES = {
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
    "TBG_Toe.L",
    "TBG_UpperLeg.R",
    "TBG_LowerLeg.R",
    "TBG_Foot.R",
    "TBG_Toe.R",
}


def test_demo_guard_01_archive_has_canonical_bones() -> None:
    bones_path = ARCHIVE_ROOT / "demo_guard_01" / "bones.json"
    assert bones_path.is_file(), "run archive_rig_session.py for demo_guard_01"
    payload = json.loads(bones_path.read_text(encoding="utf-8"))
    names = set(payload["bones"])
    assert names == CANONICAL_BONES
    assert payload["version"] == 1


def test_demo_guard_01_manifest() -> None:
    manifest_path = ARCHIVE_ROOT / "demo_guard_01" / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    assert manifest["unitId"] == "demo_guard_01"
    assert manifest["bodyClass"] == "HumanoidHeavy"
    assert manifest["boneCount"] == 21
