#!/usr/bin/env python3
"""Batch export all pilot roster units to content/characters/meshes/game/*.glb

One Blender session for all units; parallel gltf-transform post-process.
Override worker count: TBG_BATCH_WORKERS=4 (default).
"""

from __future__ import annotations

import sys
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from mesh_pipeline.export_game_batch import batch_export_game_units, roster_unit_ids  # noqa: E402
from mesh_pipeline.batch_weight_gate import audit_weight_gate, format_weight_gate_table  # noqa: E402
from mesh_pipeline.game_glb_validate import validate_game_glb  # noqa: E402
from mesh_pipeline.import_game_ready_core import weight_gate_enabled  # noqa: E402


def main(argv: list[str] | None = None) -> int:
    args = argv if argv is not None else sys.argv[1:]
    repo = Path(__file__).resolve().parents[2]
    unit_ids = [arg for arg in args if arg and not arg.startswith("-")]
    failed = batch_export_game_units(unit_ids or None)
    ids = unit_ids or roster_unit_ids(repo)

    validate_failed: list[str] = []
    check_weights = weight_gate_enabled()
    for unit_id in ids:
        if unit_id in failed:
            continue
        result = validate_game_glb(
            repo / "content/characters/meshes/game" / f"{unit_id}.glb",
            check_weights=check_weights,
        )
        if not result.ok:
            print(f"validation FAILED {unit_id}: {'; '.join(result.errors)}")
            validate_failed.append(unit_id)

    weight_rows = [audit_weight_gate(repo, unit_id) for unit_id in ids if unit_id not in failed]
    if weight_rows:
        print(format_weight_gate_table(weight_rows))
    weight_failed = [row.unit_id for row in weight_rows if not row.passes]

    all_failed = sorted(set(failed + validate_failed + weight_failed))
    if all_failed:
        print(f"batch_export_game_units: failed: {' '.join(all_failed)}", file=sys.stderr)
        return 1
    print(f"batch_export_game_units: OK ({len(ids) - len(all_failed)} units)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
