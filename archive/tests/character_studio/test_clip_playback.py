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


def test_fk_upright_constraint_zeros_torso_yaw_roll():
    text = (REPO / "tools/character_studio/static/js/viewer/fkUprightConstraint.js").read_text(encoding="utf-8")
    assert "enforceFkUpright" in text
    assert "TBG_Spine" in text
    assert "applyWorldPitchOnly" in text
    assert "setFkUprightLocomotionPreview" in text


def test_fk_upright_skipped_during_locomotion_preview():
    text = (REPO / "tools/character_studio/static/js/viewer/fkUprightConstraint.js").read_text(encoding="utf-8")
    assert "skipDuringLocomotion" in text


def test_viewer_applies_upright_after_mixer_smoothing_off_by_default():
    text = (REPO / "tools/character_studio/static/js/viewer/viewer3d.js").read_text(encoding="utf-8")
    assert "enforceFkUpright(mixerRig)" in text
    assert "refreshSkinnedMeshes(mixerRig)" in text
    assert "setLocomotionPreview" in text
    assert "setClipTimeScale" in text
    assert "footIkDebug.setEnabled(false)" in text
    assert "motionSmoothing.setEnabled(false)" in text


def test_bone_spring_module_targets_spine_and_arms():
    text = (REPO / "tools/character_studio/static/js/viewer/boneSpring.js").read_text(encoding="utf-8")
    assert "TBG_Spine1" in text
    assert "afterUprightUpdate" in text
    assert "0.12" in text


def test_glb_animation_research_doc_exists():
    path = REPO / "docs/research/GLB-CHARACTER-ANIMATION-REFERENCES.md"
    assert path.is_file()
    body = path.read_text(encoding="utf-8")
    assert "ossos" in body
    assert "demo_scout_01" in body
