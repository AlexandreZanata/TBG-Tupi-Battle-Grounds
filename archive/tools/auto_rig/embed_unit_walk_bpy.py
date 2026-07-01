#!/usr/bin/env python3
"""Re-embed combat clip set into game GLB (unit Walk.glb overrides procedural Walk)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]
_TOOLS = _REPO / "tools"
for path in (_REPO, _TOOLS):
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

from tools.blender.package_path import resolve_repo_path  # noqa: E402


def _unit_paths(unit_id: str) -> tuple[Path, str, Path]:
    defn_path = _REPO / "content/characters/definitions" / f"{unit_id}.json"
    if not defn_path.is_file():
        raise FileNotFoundError(f"missing definition: {defn_path}")
    defn = json.loads(defn_path.read_text(encoding="utf-8"))
    animation_set = str(defn.get("animationSet") or "humanoid_light_combat_v1")
    game_mesh = _REPO / "content/characters/meshes/game" / f"{unit_id}.glb"
    return game_mesh, animation_set, defn_path


def embed_unit_walk_clips(
    unit_id: str,
    *,
    input_glb: Path | None = None,
    output_glb: Path | None = None,
) -> dict:
    try:
        import bpy
    except ImportError as exc:
        raise SystemExit("embed_unit_walk_bpy requires Blender bpy") from exc

    from tools.blender.scripts.retarget_clip_library import retarget_clip_library  # noqa: PLC0415

    game_mesh, animation_set, _ = _unit_paths(unit_id)
    src = resolve_repo_path(input_glb) if input_glb else game_mesh
    out = resolve_repo_path(output_glb) if output_glb else game_mesh
    if not src.is_file():
        raise FileNotFoundError(f"input game mesh missing: {src}")

    retarget_clip_library(src, animation_set, out, _REPO, unit_id=unit_id)
    unit_walk = _REPO / "content/characters/animations/clips/units" / unit_id / "Walk.glb"
    return {
        "kind": "embed_clips",
        "unitId": unit_id,
        "input": str(src),
        "output": str(out),
        "unitWalkClip": str(unit_walk) if unit_walk.is_file() else None,
        "animationSet": animation_set,
    }


def _cli(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Embed clips into game GLB.")
    parser.add_argument("--id", dest="unit_id", required=True)
    parser.add_argument("--input", type=Path, default=None)
    parser.add_argument("--output", type=Path, default=None)
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(args_list)
    report = embed_unit_walk_clips(args.unit_id, input_glb=args.input, output_glb=args.output)
    if args.report_json:
        print(json.dumps(report))
    else:
        print(f"Embedded clips → {report['output']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
