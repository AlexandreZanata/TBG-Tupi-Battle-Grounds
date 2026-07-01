"""Contract: motion arena locomotion and playback."""

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]


def test_arena_locomotion_uses_viewer_play_clip():
    text = (REPO / "tools/character_studio/static/js/motion-arena/arenaLocomotion.js").read_text(encoding="utf-8")
    assert "viewer.playClip" in text
    assert "setClipTimeScale" in text
    assert "WALK_ARM_DELAY_SEC" in text
    assert "isWalkEngaged" in text


def test_arena_controller_caps_speed_for_walk_cycle():
    text = (REPO / "tools/character_studio/static/js/motion-arena/arenaController.js").read_text(encoding="utf-8")
    assert "MAX_SPEED = 1.25" in text
    assert "groundY" in text
    assert "allowMove" in text
    assert "displayRoot.position.y = 0" not in text


def test_arena_app_wires_arena_locomotion():
    text = (REPO / "tools/character_studio/static/js/motion-arena/arenaApp.js").read_text(encoding="utf-8")
    assert "createArenaLocomotion" in text
    assert "skipInitialClip: true" in text
    assert "filterArenaPilots" in text
    assert "MOTION_ARENA_PILOT_IDS" in (
        REPO / "tools/character_studio/static/js/motion-arena/arenaPilotConfig.js"
    ).read_text(encoding="utf-8")


def test_viewer_mesh3d_skips_display_root_scale():
    text = (REPO / "tools/character_studio/static/js/viewer/viewer3d.js").read_text(encoding="utf-8")
    assert "SkinnedMesh breaks" in text or "breaks GPU skinning" in text
    block = text.split("async function buildMesh3dRoot")[1].split("async function buildProceduralRoot")[0]
    assert "displayRoot.scale.set" not in block


def test_motion_arena_page_exists():
    html = (REPO / "tools/character_studio/static/motion-arena.html").read_text(encoding="utf-8")
    assert "Motion Arena" in html
    assert "mixamo.com" not in html.lower()
    assert "OPEN-SOURCE-WALK" in html
