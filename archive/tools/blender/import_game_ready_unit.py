#!/usr/bin/env python3
"""Unified CLI: raw AI GLB → showcase → game export → weight gate."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from mesh_pipeline.import_game_ready_core import (  # noqa: E402
    ImportGameReadyConfig,
    gate_failure_lines,
    import_game_ready_unit,
)


def _print_report(report) -> None:
    print(f"import_game_ready_unit: {report.unit_id}")
    print(f"  steps: {', '.join(report.steps)}")
    if report.showcase_path:
        print(f"  showcase: {report.showcase_path}")
    if report.game_path:
        print(f"  game: {report.game_path}")
    print(f"  weight_gate: {'PASS' if report.weight_gate_pass else 'FAIL'}")
    print(f"  validate: {'PASS' if report.validate_ok else 'FAIL'}")
    if report.validate_errors:
        for error in report.validate_errors:
            print(f"    error: {error}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Import AI mesh and export validated game-ready unit GLB."
    )
    parser.add_argument("--source", type=Path, required=True, help="Raw AI/sculpt GLB or OBJ")
    parser.add_argument("--id", dest="unit_id", required=True, help="Unit id (definition must exist)")
    parser.add_argument("--body-class", default=None, help="Override pilot_roster bodyClass")
    parser.add_argument("--animation-set", default=None, help="Override definition animationSet")
    parser.add_argument(
        "--mixamo-fallback",
        action="store_true",
        help="On weight gate fail, print Mixamo upload checklist",
    )
    parser.add_argument("--skip-prep", action="store_true", help="Skip prep_ai_mesh normalization")
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(argv)

    config = ImportGameReadyConfig(
        source=args.source,
        unit_id=args.unit_id,
        body_class=args.body_class,
        animation_set=args.animation_set,
        mixamo_fallback=args.mixamo_fallback,
        skip_prep=args.skip_prep,
    )
    report = import_game_ready_unit(config)

    if report.blocked:
        print(report.block_reason, file=sys.stderr)
        return 1

    _print_report(report)
    if args.report_json:
        payload = {
            "unit_id": report.unit_id,
            "ok": report.ok,
            "steps": report.steps,
            "weight_gate_pass": report.weight_gate_pass,
            "weight_errors": list(report.weight_errors),
            "validate_ok": report.validate_ok,
            "validate_errors": report.validate_errors,
        }
        print(json.dumps(payload))

    if not report.ok:
        for line in gate_failure_lines(
            report.unit_id,
            report.weight_errors,
            mixamo_fallback=args.mixamo_fallback,
            showcase_path=report.showcase_path,
        ):
            print(line, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
