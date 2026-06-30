"""Batch weight-gate summary rows for game mesh exports."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from mesh_pipeline.weight_quality import validate_weight_quality


@dataclass(frozen=True)
class WeightGateRow:
    unit_id: str
    passes: bool
    dominant_bone: str | None
    dominant_share: float
    errors: tuple[str, ...]


def audit_weight_gate(repo: Path, unit_id: str) -> WeightGateRow:
    game_path = repo / "content/characters/meshes/game" / f"{unit_id}.glb"
    if not game_path.is_file():
        return WeightGateRow(unit_id, False, None, 0.0, ("missing game GLB",))
    result = validate_weight_quality(game_path)
    skew = result.skew
    return WeightGateRow(
        unit_id=unit_id,
        passes=result.passes_gate,
        dominant_bone=skew.dominant_bone,
        dominant_share=skew.dominant_share,
        errors=result.errors,
    )


def format_weight_gate_table(rows: list[WeightGateRow]) -> str:
    header = "| unit_id | weight_gate | dominant_bone | dominant_share |"
    sep = "|---------|-------------|---------------|----------------|"
    lines = [header, sep]
    for row in rows:
        bone = row.dominant_bone or "—"
        share = f"{row.dominant_share:.0%}" if row.dominant_share else "—"
        status = "PASS" if row.passes else "FAIL"
        lines.append(f"| {row.unit_id} | {status} | {bone} | {share} |")
    passed = sum(1 for row in rows if row.passes)
    lines.append("")
    lines.append(f"weight gate: {passed}/{len(rows)} passed")
    return "\n".join(lines)
