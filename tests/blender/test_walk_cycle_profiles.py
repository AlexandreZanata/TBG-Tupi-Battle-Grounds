"""Contract: walk cycle pose tables loop cleanly at frame 40."""

from tools.blender.scripts.walk_cycle_profiles import (
    WALK_FRAMES,
    WALK_PROFILES,
    UNIT_WALK_PROFILE,
    resolve_walk_profile,
    root_bob_y,
)


def test_scout_pilot_profile_registered():
    assert UNIT_WALK_PROFILE["demo_scout_01"] == "scout_pilot_v1"


def test_walk_profiles_loop_at_frame_zero_and_end():
    for profile_id, poses in WALK_PROFILES.items():
        assert 0 in poses
        assert root_bob_y(0) == root_bob_y(WALK_FRAMES)


def test_resolve_scout_profile():
    assert resolve_walk_profile("demo_scout_01", "HumanoidLight") == "scout_pilot_v1"
    assert resolve_walk_profile(None, "HumanoidLight") == "humanoid_light_v1"
