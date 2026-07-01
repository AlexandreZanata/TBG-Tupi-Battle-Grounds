"""Contract: BVH parser reads hierarchy and motion channels."""

from pathlib import Path

from tools.blender.animation.retarget.bvh_parser import parse_bvh

FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "mocap" / "35_01_walk_excerpt.bvh"


def test_parse_fixture_bvh_frame_count():
    motion = parse_bvh(FIXTURE)
    assert motion.frame_count == 140
    assert abs(motion.fps - 120.0) < 0.1


def test_parse_fixture_has_right_leg_channels():
    motion = parse_bvh(FIXTURE)
    motion.channel_index("RightUpLeg", "Xrotation")
    motion.channel_index("RightLeg", "Xrotation")
    motion.channel_index("RightFoot", "Xrotation")


def test_joint_series_length_matches_frames():
    motion = parse_bvh(FIXTURE)
    series = motion.joint_series("RightUpLeg", "Xrotation")
    assert len(series) == motion.frame_count
