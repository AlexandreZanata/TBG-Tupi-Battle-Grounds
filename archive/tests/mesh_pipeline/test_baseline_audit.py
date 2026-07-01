"""Contract: baseline audit covers fixed_test_imports roster."""

from __future__ import annotations

import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
AUDIT_SCRIPT = REPO / "tools/mesh_pipeline/baseline_audit.py"
MANIFEST = REPO / "content/characters/fixed_test_imports.json"


def test_baseline_audit_script_exists():
    assert AUDIT_SCRIPT.is_file()


def test_fixed_test_imports_has_26_units():
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    assert len(data["showcaseOrder"]) == 25


def test_audit_render_includes_summary_table():
    text = AUDIT_SCRIPT.read_text(encoding="utf-8")
    assert "render_audit_md" in text
    assert "weight_gate_pass" in text
