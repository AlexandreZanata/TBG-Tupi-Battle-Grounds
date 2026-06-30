"""Skin weight quality gate for game-ready GLBs."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from mesh_pipeline.weight_skew import SINGLE_BONE_DOMINANCE, WeightSkewReport, analyze_weight_skew

MIN_BONE_WEIGHT = 0.01


@dataclass(frozen=True)
class WeightQualityResult:
    skew: WeightSkewReport
    bones_missing_coverage: tuple[str, ...]
    errors: tuple[str, ...]

    @property
    def passes_gate(self) -> bool:
        return not self.errors


def validate_weight_quality(path: Path, *, dominance_threshold: float = SINGLE_BONE_DOMINANCE) -> WeightQualityResult:
    skew = analyze_weight_skew(path, threshold=dominance_threshold)
    errors = _collect_errors(skew, dominance_threshold)
    missing = _bones_missing_coverage(skew)
    if missing:
        errors = [*errors, f"bones without weight > {MIN_BONE_WEIGHT}: {', '.join(missing)}"]
    return WeightQualityResult(skew=skew, bones_missing_coverage=missing, errors=tuple(errors))


def _bones_missing_coverage(skew: WeightSkewReport) -> tuple[str, ...]:
    if not skew.joint_names:
        return ()
    max_map = dict(skew.bone_max_weight)
    missing = [
        name
        for name in skew.joint_names
        if name.startswith("TBG_") and max_map.get(name, 0.0) < MIN_BONE_WEIGHT
    ]
    return tuple(missing)


def _collect_errors(skew: WeightSkewReport, threshold: float) -> list[str]:
    errors: list[str] = []
    if skew.vertex_count == 0:
        errors.append("no skinned vertices")
    if skew.unweighted_vertices > 0:
        errors.append(f"{skew.unweighted_vertices} vertices with zero weight sum")
    if skew.skewed:
        bone = skew.dominant_bone or "?"
        errors.append(
            f"weight skew: {bone} dominates {skew.dominant_share:.0%} of vertices (max {threshold:.0%})"
        )
    return errors
