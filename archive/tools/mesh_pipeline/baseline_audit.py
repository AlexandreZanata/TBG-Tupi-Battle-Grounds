#!/usr/bin/env python3
"""Baseline audit for Phase 04b — per-unit game GLB metrics and weight skew."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from mesh_pipeline.game_glb_validate import validate_game_glb  # noqa: E402
from mesh_pipeline.glb_inspect import inspect_glb  # noqa: E402
from mesh_pipeline.weight_quality import validate_weight_quality  # noqa: E402


@dataclass
class UnitAuditRow:
    id: str
    game_glb_exists: bool
    validate_ok: bool
    validate_errors: list[str]
    triangles: int
    bones: int
    animations: int
    file_bytes: int
    weight_skewed: bool
    dominant_bone: str | None
    dominant_share: float
    unweighted_vertices: int
    deform_review: str = "pending"


def _load_unit_ids(repo: Path, only: list[str] | None) -> list[str]:
    manifest = repo / "content/characters/fixed_test_imports.json"
    data = json.loads(manifest.read_text(encoding="utf-8"))
    order = data.get("showcaseOrder", [])
    if only:
        return [uid for uid in only if uid in order]
    return order


def audit_unit(repo: Path, unit_id: str) -> UnitAuditRow:
    game_path = repo / "content/characters/meshes/game" / f"{unit_id}.glb"
    if not game_path.is_file():
        return UnitAuditRow(
            id=unit_id,
            game_glb_exists=False,
            validate_ok=False,
            validate_errors=["missing game GLB"],
            triangles=0,
            bones=0,
            animations=0,
            file_bytes=0,
            weight_skewed=True,
            dominant_bone=None,
            dominant_share=0.0,
            unweighted_vertices=0,
        )

    stats = inspect_glb(game_path)
    validation = validate_game_glb(game_path)
    weight_result = validate_weight_quality(game_path)
    return UnitAuditRow(
        id=unit_id,
        game_glb_exists=True,
        validate_ok=validation.ok,
        validate_errors=validation.errors,
        triangles=stats.triangles,
        bones=stats.bones,
        animations=stats.animations,
        file_bytes=stats.file_bytes,
        weight_skewed=not weight_result.passes_gate,
        dominant_bone=weight_result.skew.dominant_bone,
        dominant_share=round(weight_result.skew.dominant_share, 4),
        unweighted_vertices=weight_result.skew.unweighted_vertices,
    )


def summarize(rows: list[UnitAuditRow]) -> dict:
    present = [r for r in rows if r.game_glb_exists]
    validate_pass = sum(1 for r in present if r.validate_ok)
    weight_pass = sum(1 for r in present if not r.weight_skewed and r.unweighted_vertices == 0)
    return {
        "total_units": len(rows),
        "game_glb_present": len(present),
        "validate_pass": validate_pass,
        "weight_gate_pass": weight_pass,
        "validate_pass_rate": round(validate_pass / max(1, len(present)), 3),
        "weight_pass_rate": round(weight_pass / max(1, len(present)), 3),
        "manual_review_pending": sum(1 for r in rows if r.deform_review == "pending"),
    }


def render_audit_md(repo: Path, rows: list[UnitAuditRow], summary: dict) -> str:
    today = date.today().isoformat()
    lines = [
        "# Phase 04b Baseline Audit",
        "",
        f"> Generated: {today} via `python3 tools/mesh_pipeline/baseline_audit.py --write`",
        "",
        "## Summary",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| Units in roster | {summary['total_units']} |",
        f"| Game GLB present | {summary['game_glb_present']} |",
        f"| `validate_game_glb` pass | {summary['validate_pass']} ({summary['validate_pass_rate']:.0%}) |",
        f"| Weight skew gate pass | {summary['weight_gate_pass']} ({summary['weight_pass_rate']:.0%}) |",
        f"| Est. needing manual weight fix | {summary['game_glb_present'] - summary['weight_gate_pass']} |",
        "",
        "## Per-unit metrics",
        "",
        "| id | validate | tris | bones | clips | size (KB) | weight skew | dominant bone | share | unweighted | deform review |",
        "|----|----------|------|-------|-------|-----------|-------------|---------------|-------|------------|---------------|",
    ]
    for row in rows:
        size_kb = round(row.file_bytes / 1024, 1) if row.file_bytes else "—"
        val = "PASS" if row.validate_ok else "FAIL"
        skew = "FAIL" if row.weight_skewed else "PASS"
        if not row.game_glb_exists:
            val = skew = "MISSING"
        lines.append(
            f"| {row.id} | {val} | {row.triangles or '—'} | {row.bones or '—'} | "
            f"{row.animations or '—'} | {size_kb} | {skew} | "
            f"{row.dominant_bone or '—'} | {row.dominant_share:.2f} | "
            f"{row.unweighted_vertices} | {row.deform_review} |"
        )

    failures = [r for r in rows if r.game_glb_exists and (not r.validate_ok or r.weight_skewed)]
    lines.extend(
        [
            "",
            "## Top failure modes (baseline)",
            "",
            "1. **Weight dominance skew** — proximity weights collapse to one bone (≥85% vertices share dominant bone).",
            "2. **Dense showcase mesh** — high-poly AI/sculpt input slows decimate; rare validate tri budget edge cases.",
            "3. **Proportion outliers** — armature hull misalignment on wide armor / capes (manual deform review pending).",
            "",
            "## Manual Animation Studio review",
            "",
            "Review shoulder / elbow / knee / hip in Animation Studio (Game tier). Update `deform_review` column above.",
            "",
            "### Units flagged for review",
            "",
        ]
    )
    if failures:
        for row in failures:
            notes = "; ".join(row.validate_errors) if row.validate_errors else "weight skew"
            lines.append(f"- `{row.id}` — {notes}")
    else:
        lines.append("- None flagged by automated gates.")

    lines.extend(
        [
            "",
            "## Commands",
            "",
            "```bash",
            "./tools/mesh-pipeline/batch_seed_to_game.sh",
            "python3 tools/mesh_pipeline/baseline_audit.py --write",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Phase 04b baseline audit for game GLBs.")
    parser.add_argument("--write", action="store_true", help="Write AUDIT.md under phase folder.")
    parser.add_argument("--json", type=Path, help="Write machine-readable report JSON.")
    parser.add_argument("unit_ids", nargs="*", help="Subset of unit ids (default: fixed_test_imports order).")
    args = parser.parse_args(argv)

    repo = Path(__file__).resolve().parents[2]
    unit_ids = _load_unit_ids(repo, args.unit_ids or None)
    rows = [audit_unit(repo, uid) for uid in unit_ids]
    summary = summarize(rows)

    print(json.dumps({"summary": summary, "units": [asdict(r) for r in rows]}, indent=2))

    if args.json:
        args.json.write_text(
            json.dumps({"summary": summary, "units": [asdict(r) for r in rows]}, indent=2),
            encoding="utf-8",
        )

    if args.write:
        out = repo / ".local/phases/04b-biped-game-ready-pipeline/AUDIT.md"
        out.write_text(render_audit_md(repo, rows, summary), encoding="utf-8")
        print(f"wrote {out}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
