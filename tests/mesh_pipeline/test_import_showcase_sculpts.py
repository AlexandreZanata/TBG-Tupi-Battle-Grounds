"""Tests for showcase sculpt import script."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "tools/mesh-pipeline/import_showcase_sculpts.py"


def test_import_mapping_covers_six_pilots() -> None:
    result = subprocess.run(
        [sys.executable, str(SCRIPT), "--dry-run"],
        capture_output=True,
        text=True,
        check=True,
    )
    mapping = json.loads(result.stdout)
    assert set(mapping) == {
        "demo_hunter_01",
        "demo_bruiser_01",
        "demo_striker_01",
        "demo_vanguard_01",
        "demo_sentinel_01",
        "demo_guard_01",
        "demo_scout_01",
        "demo_reaver_01",
    }


def test_only_flag_limits_import_subset() -> None:
    result = subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            "--dry-run",
            "--only",
            "demo_scout_01",
            "demo_reaver_01",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    mapping = json.loads(result.stdout)
    assert set(mapping) == {"demo_scout_01", "demo_reaver_01"}


def test_imported_sculpts_exist_after_import() -> None:
    meshes = REPO_ROOT / "content/characters/meshes/showcase"
    for pilot_id in (
        "demo_scout_01",
        "demo_reaver_01",
        "demo_hunter_01",
        "demo_bruiser_01",
        "demo_striker_01",
        "demo_vanguard_01",
        "demo_sentinel_01",
        "demo_guard_01",
    ):
        path = meshes / f"{pilot_id}.glb"
        assert path.is_file(), f"missing imported sculpt: {path}"
        assert path.stat().st_size > 1_000_000
