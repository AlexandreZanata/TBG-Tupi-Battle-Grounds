#!/usr/bin/env python3
"""Export MB-Lab base body + armature GLB for library/bases/{id}/."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("export_base_body requires Blender bpy") from exc

REPO = Path(__file__).resolve().parents[3]
LIBRARY = REPO / "content" / "characters" / "library"


def _export_selection(path: Path, objects: list[bpy.types.Object]) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_skins=True,
        export_animations=False,
        export_yup=True,
    )


def export_base(base_id: str, *, body_class: str, height_m: float, label: str) -> dict:
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    arms = [o for o in bpy.context.scene.objects if o.type == "ARMATURE"]
    if not meshes or not arms:
        raise RuntimeError("scene needs skinned MB-Lab mesh + armature")
    body_path = LIBRARY / "bases" / base_id / "body.glb"
    rig_path = LIBRARY / "rigs" / base_id / "armature.glb"
    _export_selection(body_path, meshes + arms)
    _export_selection(rig_path, arms)
    manifest = {
        "id": base_id,
        "label": label,
        "heightM": height_m,
        "bodyClass": body_class,
        "mesh": "body.glb",
        "rig": f"../../rigs/{base_id}/armature.glb",
    }
    manifest_path = LIBRARY / "bases" / base_id / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return {"body": str(body_path), "rig": str(rig_path), "manifest": str(manifest_path)}


def main(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Export MB-Lab base to library/")
    parser.add_argument("--base-id", required=True, choices=["male", "female", "small", "large"])
    parser.add_argument("--height-m", type=float, required=True)
    parser.add_argument("--body-class", default="HumanoidLight")
    parser.add_argument("--label", required=True)
    args = parser.parse_args(args_list)
    report = export_base(
        args.base_id,
        body_class=args.body_class,
        height_m=args.height_m,
        label=args.label,
    )
    print(json.dumps(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
