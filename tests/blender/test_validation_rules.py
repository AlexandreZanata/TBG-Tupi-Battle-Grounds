"""Tests for Blender export validation rules (no bpy required)."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "blender"))

from tbg_export.validation_rules import (  # noqa: E402
    MAX_TRIS_MVP,
    require_armature,
    require_uv,
    tri_count_warning,
)


def test_tri_count_under_limit_returns_none() -> None:
    assert tri_count_warning(4000) is None


def test_tri_count_over_limit_returns_warning() -> None:
    msg = tri_count_warning(MAX_TRIS_MVP + 1)
    assert msg is not None
    assert "exceeds MVP limit" in msg


def test_require_armature_when_missing() -> None:
    assert require_armature(False) == "armature required"


def test_require_armature_when_present() -> None:
    assert require_armature(True) is None


def test_require_uv_when_missing() -> None:
    msg = require_uv(False, "Body")
    assert msg is not None
    assert "Body" in msg
    assert "UV required" in msg


def test_require_uv_when_present() -> None:
    assert require_uv(True, "Body") is None
