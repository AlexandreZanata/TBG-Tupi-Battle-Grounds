"""Contract: post-gate movement deferrals are documented (ADR-005 + roadmap)."""

from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
ADR = REPO_ROOT / "docs/adr/ADR-005-deferred-movement-features.md"
ROADMAP = REPO_ROOT / "tools/blender/animation/DEFERRED-ROADMAP.md"
MOVEMENT = REPO_ROOT / "tools/blender/animation/MOVEMENT-MODULE.md"

DEFERRED_IDS = ("D1", "D2", "D3", "D4", "D5")
DEFERRED_KEYWORDS = (
    "VRIK",
    "Motion matching",
    "ML locomotion",
    "Quadruped",
    "Nakama",
)


def test_adr_and_roadmap_exist():
    assert ADR.is_file(), f"missing {ADR}"
    assert ROADMAP.is_file(), f"missing {ROADMAP}"


def test_roadmap_lists_all_deferred_ids():
    text = ROADMAP.read_text(encoding="utf-8")
    for item_id in DEFERRED_IDS:
        assert item_id in text, f"roadmap missing {item_id}"


def test_adr_covers_deferred_themes():
    text = ADR.read_text(encoding="utf-8")
    for keyword in DEFERRED_KEYWORDS:
        assert keyword.lower() in text.lower(), f"ADR missing theme: {keyword}"


def test_movement_module_links_deferred_roadmap():
    text = MOVEMENT.read_text(encoding="utf-8")
    assert "DEFERRED-ROADMAP.md" in text
    assert "ADR-005" in text


def test_roadmap_declares_not_implemented():
    text = ROADMAP.read_text(encoding="utf-8")
    assert "Not implemented" in text or "not implemented" in text.lower()
