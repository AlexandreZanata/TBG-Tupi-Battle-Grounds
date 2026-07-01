"""glTF ↔ Blender TBG bone name normalization."""

from __future__ import annotations

from tools.experimental.auto_rig.rig_bone_names import gltf_to_blender_bone, normalize_bones_payload


def test_gltf_to_blender_side_bones() -> None:
    assert gltf_to_blender_bone("TBG_ShoulderL") == "TBG_Shoulder.L"
    assert gltf_to_blender_bone("TBG_UpperArm.R") == "TBG_UpperArm.R"
    assert gltf_to_blender_bone("TBG_Root") == "TBG_Root"


def test_normalize_bones_payload_rewrites_keys_and_parents() -> None:
    payload = {
        "version": 1,
        "bones": {
            "TBG_UpperArmL": {
                "head": [0, 0, 0],
                "tail": [0, 0.1, 0],
                "parent": "TBG_ShoulderL",
            }
        },
    }
    out = normalize_bones_payload(payload)
    assert "TBG_UpperArm.L" in out["bones"]
    assert out["bones"]["TBG_UpperArm.L"]["parent"] == "TBG_Shoulder.L"
