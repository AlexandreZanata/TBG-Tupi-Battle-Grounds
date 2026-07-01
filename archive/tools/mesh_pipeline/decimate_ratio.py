"""Pure helpers for mobile mesh decimation ratio (no Blender dependency)."""


def decimate_keep_ratio(face_count: int, tri_target: int, *, max_step: float = 0.75) -> float:
    """Blender DECIMATE ratio: fraction of faces to keep per pass."""
    return min(max_step, tri_target / max(1, face_count))
