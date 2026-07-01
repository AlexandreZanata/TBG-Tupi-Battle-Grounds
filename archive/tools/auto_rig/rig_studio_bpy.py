#!/usr/bin/env python3
"""Blender: Rig Studio — prep, browser bone sync, finalize bind + export."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError as exc:
    raise SystemExit("rig_studio_bpy requires Blender bpy") from exc

_REPO = Path(__file__).resolve().parents[3]
_SCRIPTS = _REPO / "tools/blender/scripts"
for path in (_REPO, _SCRIPTS, Path(__file__).resolve().parent):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from attach_tbg_rig import (  # noqa: E402
    TEMPLATE_DIR,
    _align_armature_to_mesh,
    _join_meshes,
    _widen_bone_envelopes,
)
from rig_studio_bones import apply_bones_payload, read_bones_json, write_bones_json  # noqa: E402
from rig_weight_bind import bind_mesh_with_weight_fallback  # noqa: E402
from retarget_clip_library import retarget_clip_library  # noqa: E402
from tools.blender.package_path import resolve_repo_path  # noqa: E402


def _remove_template_body(body_class: str) -> None:
    mesh = bpy.data.objects.get(f"{body_class}_Body")
    if mesh is not None:
        bpy.data.objects.remove(mesh, do_unlink=True)


def _find_mesh_and_armature() -> tuple[bpy.types.Object, bpy.types.Object]:
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    arms = [obj for obj in bpy.context.scene.objects if obj.type == "ARMATURE"]
    if not meshes or not arms:
        raise RuntimeError("session needs one mesh and one armature")
    return _join_meshes(meshes), arms[0]


def prep_rig_session(
    showcase_path: Path,
    body_class: str,
    session_blend: Path,
    *,
    preview_glb: Path | None = None,
    bones_json: Path | None = None,
) -> dict:
    template = TEMPLATE_DIR / f"{body_class}_template.blend"
    if not template.is_file():
        raise FileNotFoundError(f"missing template {template} — run tools/blender/setup.sh")

    bpy.ops.wm.open_mainfile(filepath=str(template))
    _remove_template_body(body_class)
    bpy.ops.import_scene.gltf(filepath=str(showcase_path))
    mesh_obj, arm_obj = _find_mesh_and_armature()
    _align_armature_to_mesh(arm_obj, mesh_obj)
    mesh_obj.name = f"{body_class}_Mesh"

    session_blend.parent.mkdir(parents=True, exist_ok=True)
    preview_path = preview_glb or session_blend.parent / "preview.glb"
    bones_path = bones_json or session_blend.parent / "bones.json"
    _export_gltf(mesh_obj, arm_obj, preview_path, skins=False)
    write_bones_json(arm_obj, bones_path)
    bpy.ops.wm.save_as_mainfile(filepath=str(session_blend))

    return {
        "kind": "prep",
        "body_class": body_class,
        "session_blend": str(session_blend),
        "preview_glb": str(preview_path),
        "bones_json": str(bones_path),
        "bone_count": len(arm_obj.data.bones),
    }


def sync_bones_to_session(session_blend: Path, bones_json: Path) -> dict:
    bpy.ops.wm.open_mainfile(filepath=str(session_blend))
    _mesh, arm_obj = _find_mesh_and_armature()
    count = apply_bones_payload(arm_obj, read_bones_json(bones_json))
    bpy.ops.wm.save_as_mainfile(filepath=str(session_blend))
    return {"kind": "sync", "bones_updated": count, "session_blend": str(session_blend)}


def finalize_rig_session(
    session_blend: Path,
    raw_output: Path,
    *,
    unit_id: str,
    animation_set_id: str,
    bones_json: Path | None = None,
) -> dict:
    bpy.ops.wm.open_mainfile(filepath=str(session_blend))
    mesh_obj, arm_obj = _find_mesh_and_armature()
    bones_path = bones_json or session_blend.parent / "bones.json"
    if bones_path.is_file():
        apply_bones_payload(arm_obj, read_bones_json(bones_path))

    bind_winner = bind_mesh_with_weight_fallback(
        mesh_obj, arm_obj, widen_envelopes=_widen_bone_envelopes
    )
    mesh_obj.parent = arm_obj
    tmp = raw_output.with_suffix(".skinned.glb")
    tmp.parent.mkdir(parents=True, exist_ok=True)
    _export_gltf(mesh_obj, arm_obj, tmp, skins=True)
    retarget_clip_library(tmp, animation_set_id, raw_output, _REPO, unit_id=unit_id)
    tmp.unlink(missing_ok=True)
    return {
        "kind": "finalize",
        "unit_id": unit_id,
        "raw_output": str(raw_output),
        "bind_winner": bind_winner,
    }


def _export_gltf(mesh_obj: bpy.types.Object, arm_obj: bpy.types.Object, path: Path, *, skins: bool) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=False,
        export_skins=skins,
        export_animations=False,
        export_yup=True,
    )


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Rig Studio Blender ops.")
    sub = parser.add_subparsers(dest="command", required=True)

    prep = sub.add_parser("prep")
    prep.add_argument("--showcase", type=Path, required=True)
    prep.add_argument("--body-class", default="HumanoidLight")
    prep.add_argument("--session-blend", type=Path, required=True)
    prep.add_argument("--preview-glb", type=Path, default=None)
    prep.add_argument("--bones-json", type=Path, default=None)

    sync = sub.add_parser("sync")
    sync.add_argument("--session-blend", type=Path, required=True)
    sync.add_argument("--bones-json", type=Path, required=True)

    fin = sub.add_parser("finalize")
    fin.add_argument("--session-blend", type=Path, required=True)
    fin.add_argument("--raw-output", type=Path, required=True)
    fin.add_argument("--id", dest="unit_id", required=True)
    fin.add_argument("--animation-set", default="humanoid_light_combat_v1")
    fin.add_argument("--bones-json", type=Path, default=None)

    for sub_parser in (prep, sync, fin):
        sub_parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)

    if args.command == "prep":
        payload = prep_rig_session(
            resolve_repo_path(args.showcase),
            args.body_class,
            resolve_repo_path(args.session_blend),
            preview_glb=resolve_repo_path(args.preview_glb) if args.preview_glb else None,
            bones_json=resolve_repo_path(args.bones_json) if args.bones_json else None,
        )
    elif args.command == "sync":
        payload = sync_bones_to_session(
            resolve_repo_path(args.session_blend),
            resolve_repo_path(args.bones_json),
        )
    else:
        payload = finalize_rig_session(
            resolve_repo_path(args.session_blend),
            resolve_repo_path(args.raw_output),
            unit_id=args.unit_id,
            animation_set_id=args.animation_set,
            bones_json=resolve_repo_path(args.bones_json) if args.bones_json else None,
        )
    if args.report_json:
        print(json.dumps(payload))
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
