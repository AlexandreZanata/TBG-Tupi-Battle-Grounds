"""Mixamo upload checklist (no bpy)."""

from __future__ import annotations

from pathlib import Path

MIXAMO_UPLOAD_URL = "https://www.mixamo.com/#/"


def mixamo_checklist_lines(obj_path: Path | None = None) -> list[str]:
    lines = [
        "Mixamo fallback checklist (dev-only rig path):",
        f"1. Open {MIXAMO_UPLOAD_URL}",
        "2. Sign in with a free Adobe account",
        "3. Upload Character → choose the exported OBJ (T-pose, Y-up, centered)",
        "4. Confirm auto-rig markers on wrists, elbows, knees, groin",
        "5. Download FBX for Unity (.fbx 7.4 binary) WITH skin, WITHOUT animation",
        "6. Import to TBG:",
        "   ./tools/blender/blender_cmd.sh --background --python tools/blender/scripts/import_mixamo_fbx.py -- \\",
        "     path/to/mixamo.fbx path/to/tbg_rigged.glb --report-json",
        "7. Ship animations from body-class clip library — NOT Mixamo mocap clips",
    ]
    if obj_path is not None:
        lines.insert(3, f"   OBJ file: {obj_path.resolve()}")
    return lines


def print_mixamo_checklist(obj_path: Path | None = None) -> None:
    for line in mixamo_checklist_lines(obj_path):
        print(line)
