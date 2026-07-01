#!/usr/bin/env python3
"""Create MB-Lab archetype .blend templates (headless)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

try:
    import bpy
except ImportError as exc:
    raise SystemExit("bootstrap_mblab_template requires Blender bpy") from exc

REPO = Path(__file__).resolve().parents[3]
PROJECTS = REPO / "content" / "characters" / "mblab" / "projects"
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

from mblab_enable import ensure_mblab_addon  # noqa: E402

ARCHETYPES: dict[str, dict[str, Any]] = {
    "male": {"characterId": "m_ca01", "age": 0.3, "mass": 0.0, "tone": 0.2},
    "female": {"characterId": "f_ca01", "age": 0.3, "mass": 0.0, "tone": 0.1},
    "small": {"characterId": "m_ca01", "age": 0.15, "mass": -0.25, "tone": 0.0},
    "large": {"characterId": "m_ca01", "age": 0.35, "mass": 0.55, "tone": 0.45},
}


def _humanoid_mesh() -> bpy.types.Object | None:
    for obj in bpy.context.scene.objects:
        if obj.type == "MESH" and obj.get("manuellab_id"):
            return obj
    return next((o for o in bpy.context.scene.objects if o.type == "MESH"), None)


def _apply_meta(mesh: bpy.types.Object, profile: dict[str, Any]) -> None:
    mesh.character_age = float(profile["age"])
    mesh.character_mass = float(profile["mass"])
    mesh.character_tone = float(profile["tone"])


def _update_humanoid() -> None:
    import importlib

    mod = importlib.import_module("MB-Lab")
    humanoid = getattr(mod, "mblab_humanoid", None)
    if humanoid is None:
        return
    humanoid.calculate_transformation("AGE")
    humanoid.calculate_transformation("FAT")
    humanoid.calculate_transformation("MUSCLE")
    humanoid.update_character(mode="update_all")


def bootstrap(base_id: str) -> dict[str, Any]:
    if base_id not in ARCHETYPES:
        raise ValueError(f"unknown base: {base_id}")
    if not ensure_mblab_addon():
        raise RuntimeError("MB-Lab add-on not available — run ./scripts/install-mblab.sh")

    profile = ARCHETYPES[base_id]
    bpy.ops.wm.read_homefile(use_empty=True)
    scn = bpy.context.scene
    scn.mblab_use_ik = False
    scn.mblab_use_muscle = False
    scn.mblab_character_name = profile["characterId"]
    result = bpy.ops.mbast.init_character()
    if "CANCELLED" in result:
        raise RuntimeError("mbast.init_character cancelled — check MB-Lab log")

    mesh = _humanoid_mesh()
    if mesh is None:
        raise RuntimeError("no MB-Lab mesh after init_character")
    _apply_meta(mesh, profile)
    _update_humanoid()

    out = PROJECTS / f"{base_id}.blend"
    out.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(out))
    return {"baseId": base_id, "blend": str(out), "characterId": profile["characterId"]}


def main(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Bootstrap MB-Lab project template")
    parser.add_argument("base_id", choices=list(ARCHETYPES))
    args = parser.parse_args(args_list)
    print(json.dumps(bootstrap(args.base_id)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
