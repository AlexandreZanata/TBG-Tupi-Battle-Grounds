"""Contract: open-source CMU walk import and cycle detection."""

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FIXTURE = REPO / "tests/blender/fixtures/mocap/35_01_walk_excerpt.bvh"


def test_detect_walk_cycle_on_fixture():
    from tools.blender.animation.retarget.curves_from_mocap import detect_walk_cycle

    start, period = detect_walk_cycle(FIXTURE)
    assert start >= 1
    assert 90 <= period <= 200


def test_import_cmu_walk_validates_missing_bvh():
    text = (REPO / "tools/blender/scripts/import_cmu_walk_bvh.py").read_text(encoding="utf-8")
    assert "CMU BVH not found" in text
    assert "download_reference_mocap" in text


def test_open_source_walk_doc_exists():
    path = REPO / "tools/blender/scripts/OPEN-SOURCE-WALK.md"
    body = path.read_text(encoding="utf-8")
    assert "Mixamo" in body and "Not used" in body
    assert "download_reference_mocap" in body
