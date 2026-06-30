"""Package layout and backward-compatible script shims."""

import importlib


def test_walk_package_exports_public_api():
    mod = importlib.import_module("tools.blender.animation.walk")
    for name in (
        "WALK_FRAMES",
        "WALK_FPS",
        "WalkStyle",
        "sample_pose_at_frame",
        "resolve_walk_profile",
    ):
        assert hasattr(mod, name), f"missing export: {name}"


def test_legacy_walk_gait_shim_reexports():
    shim = importlib.import_module("tools.blender.scripts.walk_gait")
    assert shim.WALK_FRAMES == 40
    assert shim.resolve_walk_profile("demo_scout_01", "HumanoidLight") == "scout_pilot_v1"


def test_legacy_builder_shim_reexports_bake():
    shim = importlib.import_module("tools.blender.scripts.walk_cycle_builder")
    assert callable(shim.bake_walk_action)
