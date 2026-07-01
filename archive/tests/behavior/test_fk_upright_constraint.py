"""Contract: Unity FkUprightConstraint mirrors web fkUprightConstraint.js bone sets."""

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
JS = REPO / "tools/character_studio/static/js/viewer/fkUprightConstraint.js"
CS = (
    REPO
    / "unity/Packages/com.tbg.character-library/Runtime/Locomotion/FkUprightConstraint.cs"
)

TORSO = ("TBG_Spine", "TBG_Spine1", "TBG_Neck", "TBG_Head")
WEAPON_ARM = ("TBG_Shoulder.R", "TBG_UpperArm.R", "TBG_LowerArm.R", "TBG_Hand.R")


def test_csharp_fk_upright_exports_attach_and_late_update():
    text = CS.read_text(encoding="utf-8")
    assert "FkUprightConstraint" in text
    assert "LateUpdate" in text
    assert "Attach(GameObject root)" in text


def test_csharp_fk_upright_bone_sets_match_js():
    js = JS.read_text(encoding="utf-8")
    cs = CS.read_text(encoding="utf-8")
    for bone in (*TORSO, "TBG_Root", *WEAPON_ARM):
        assert bone in js
        assert bone in cs


def test_factory_attaches_fk_upright_before_foot_ik():
    text = (
        REPO / "unity/Packages/com.tbg.character-library/Runtime/Mesh3DCharacterFactory.cs"
    ).read_text(encoding="utf-8")
    upright = text.index("FkUprightConstraint.Attach")
    spring = text.index("BoneSpring.Attach")
    foot_ik = text.index("LocomotionFootIk.Attach")
    assert upright < spring < foot_ik


def test_bone_spring_csharp_spike_exists():
    path = REPO / "unity/Packages/com.tbg.character-library/Runtime/Locomotion/BoneSpring.cs"
    text = path.read_text(encoding="utf-8")
    assert "TBG_Spine1" in text
    assert "Strength = 0.12f" in text
