"""Walk gait type aliases."""

from __future__ import annotations

BoneEuler = tuple[float, float, float]
GaitPose = dict[str, BoneEuler]
