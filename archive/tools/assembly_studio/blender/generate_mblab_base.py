#!/usr/bin/env python3
"""Generate GLB from MB-Lab .blend template + morph recipe."""

from __future__ import annotations

import argparse
import json
import sys
import traceback
from pathlib import Path
from typing import Any

try:
    import bpy
except ImportError as exc:
    raise SystemExit("generate_mblab_base requires Blender bpy") from exc

REPO = Path(__file__).resolve().parents[3]
PROJECTS = REPO / "content" / "characters" / "mblab"
LIBRARY = REPO / "content" / "characters" / "library"
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from export_base_body import export_base, export_glb_objects  # noqa: E402
from mblab_enable import ensure_mblab_addon  # noqa: E402
from mblab_morph import apply_morphs  # noqa: E402
from rename_mblab_bones import rename_armature_bones  # noqa: E402

ARCHETYPES: dict[str, dict[str, Any]] = {
    "male": {"heightM": 1.8, "bodyClass": "HumanoidLight", "label": "Male body"},
    "female": {"heightM": 1.65, "bodyClass": "HumanoidLight", "label": "Female body"},
    "small": {"heightM": 1.2, "bodyClass": "HumanoidLight", "label": "Small body"},
    "large": {"heightM": 2.1, "bodyClass": "HumanoidHeavy", "label": "Large body"},
}


def _open_template(base_id: str) -> Path:
    path = PROJECTS / "projects" / f"{base_id}.blend"
    if not path.is_file():
        raise FileNotFoundError(f"Missing {path}")
    ensure_mblab_addon()
    bpy.ops.wm.open_mainfile(filepath=str(path))
    return path


def _active_armature() -> bpy.types.Object:
    arm = next((o for o in bpy.context.scene.objects if o.type == "ARMATURE"), None)
    if arm is None:
        raise RuntimeError("no armature after loading MB-Lab template")
    return arm


def _export_unit_glb(unit_id: str) -> Path:
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    arms = [o for o in bpy.context.scene.objects if o.type == "ARMATURE"]
    out = LIBRARY / "exports" / f"{unit_id}.glb"
    export_glb_objects(out, meshes + arms)
    return out


def generate(recipe: dict[str, Any]) -> dict[str, Any]:
    base_id = str(recipe["baseId"])
    unit_id = str(recipe["unitId"])
    profile = ARCHETYPES[base_id]
    template = _open_template(base_id)
    morph_report = apply_morphs(dict(recipe.get("morphs") or {}))
    renamed = rename_armature_bones(_active_armature())
    base_report: dict[str, Any] = {}
    if recipe.get("promoteToLibrary"):
        base_report = export_base(
            base_id,
            body_class=str(profile["bodyClass"]),
            height_m=float(profile["heightM"]),
            label=str(recipe.get("label") or profile["label"]),
        )
    export_path = _export_unit_glb(unit_id)
    return {
        "template": str(template),
        "exportUrl": f"/api/assembly/exports/{unit_id}/mesh",
        "exportPath": str(export_path),
        "renamedBones": len(renamed),
        **morph_report,
        **base_report,
    }


def main(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="MB-Lab template → GLB")
    parser.add_argument("--recipe", type=Path, required=True)
    args = parser.parse_args(args_list)
    try:
        report = generate(json.loads(args.recipe.read_text(encoding="utf-8")))
        print(json.dumps(report))
        return 0
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"error": str(exc), "trace": traceback.format_exc()[-800:]}))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
