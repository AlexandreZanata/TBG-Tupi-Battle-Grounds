"""Contract: decimate ratio must use face count, not corner-sum (3x bug)."""

from mesh_pipeline.decimate_ratio import decimate_keep_ratio


def test_decimate_ratio_targets_full_budget_not_one_third():
    faces = 47_498
    target = 8_000
    ratio = decimate_keep_ratio(faces, target)
    kept = int(faces * ratio)
    assert kept >= 7_500, f"expected ~8k tris, got {kept}"
    wrong_ratio = decimate_keep_ratio(faces * 3, target)
    assert int(faces * wrong_ratio) < 3_000
