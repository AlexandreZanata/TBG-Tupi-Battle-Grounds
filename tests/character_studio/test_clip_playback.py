"""Contract: Animation Studio clip playback helpers."""

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]


def test_clip_playback_module_exports_crossfade():
    text = (REPO / "tools/character_studio/static/js/viewer/clipPlayback.js").read_text(encoding="utf-8")
    assert "crossFadeTo" in text
    assert "prepareClipsForPlayback" in text


def test_motion_smoothing_module_targets_spine_and_legs():
    text = (REPO / "tools/character_studio/static/js/viewer/motionSmoothing.js").read_text(encoding="utf-8")
    assert "TBG_Spine" in text
    assert "afterMixerUpdate" in text


def test_glb_animation_research_doc_exists():
    path = REPO / "docs/research/GLB-CHARACTER-ANIMATION-REFERENCES.md"
    assert path.is_file()
    body = path.read_text(encoding="utf-8")
    assert "ossos" in body
    assert "demo_scout_01" in body
