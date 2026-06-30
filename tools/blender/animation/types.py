"""Shared pose types for procedural clips."""

from __future__ import annotations

BoneEuler = tuple[float, float, float]
BonePose = dict[str, BoneEuler]
