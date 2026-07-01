#!/usr/bin/env python3
"""Headless export CLI.

Usage:
  blender --background template.blend --python export_unit.py -- <unit_id>
"""

from __future__ import annotations

import os
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

try:
    import bpy
except ImportError as exc:
    raise SystemExit("Run inside Blender: blender --background ... --python export_unit.py -- <id>") from exc

from tbg_export.export import export_unit_glb
from tbg_export.validation import validate_scene


def _parse_unit_id() -> str:
    if "--" in sys.argv:
        args = sys.argv[sys.argv.index("--") + 1 :]
        if args:
            return args[0]
    return "unit_01"


def main() -> None:
    unit_id = _parse_unit_id()
    for warning in validate_scene(bpy.context.scene):
        print(f"WARNING: {warning}")
    path = export_unit_glb(unit_id)
    print(f"Exported: {path}")


if __name__ == "__main__":
    main()
