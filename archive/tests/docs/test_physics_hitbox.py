"""Contract tests for physics hitbox spec and C# stubs (active)."""

from __future__ import annotations

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

PHYSICS_SPEC = REPO / "docs/design/physics-hitbox-spec.md"
HITBOX_PROFILE = REPO / "unity/Packages/com.tbg.character-library/Runtime/Physics/IHitboxProfile.cs"
BONE_COLLIDER = REPO / "unity/Packages/com.tbg.character-library/Runtime/Physics/BoneColliderDefinition.cs"
DEFAULT_PROFILE = (
    REPO / "unity/Packages/com.tbg.character-library/Runtime/Physics/DefaultBipedHitboxProfile.cs"
)


def test_physics_hitbox_spec_lists_tbg_bones():
    text = PHYSICS_SPEC.read_text(encoding="utf-8")
    assert "TBG_Head" in text
    assert "TBG_Spine1" in text
    assert "TBG_Hand.R" in text
    assert "IHitboxProfile" in text


def test_hitbox_csharp_stubs_exist():
    assert HITBOX_PROFILE.is_file()
    assert BONE_COLLIDER.is_file()
    assert DEFAULT_PROFILE.is_file()
    assert "DefaultBipedHitboxProfile" in DEFAULT_PROFILE.read_text(encoding="utf-8")
