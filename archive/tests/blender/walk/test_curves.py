"""Contract: cyclic Catmull-Rom closes loop and interpolates keyframes."""

from tools.blender.animation.walk.curves import sample_cycle_curve


def test_curve_closes_at_tau_one():
    kf = ((0.0, 10.0), (0.5, 30.0), (1.0, 10.0))
    assert abs(sample_cycle_curve(kf, 0.0) - sample_cycle_curve(kf, 1.0)) < 1e-6


def test_curve_hits_mid_keyframe():
    kf = ((0.0, 0.0), (0.5, 20.0), (1.0, 0.0))
    assert abs(sample_cycle_curve(kf, 0.5) - 20.0) < 0.5
