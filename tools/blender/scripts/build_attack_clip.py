#!/usr/bin/env python3
"""CLI entry — export Attack / Attack2 clip GLB."""

from __future__ import annotations

import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

from tools.blender.package_path import ensure_repo_paths  # noqa: E402

ensure_repo_paths()

from tools.blender.animation.attack.export import (  # noqa: E402
    default_output_path,
    export_attack_clip,
)


def _cli(argv: list[str]) -> int:
    if len(argv) < 2:
        raise SystemExit("usage: build_attack_clip.py <bodyClass> <clipName> [unit_id]")
    body_class = argv[0]
    clip_name = argv[1]
    unit_id = argv[2] if len(argv) > 2 else None
    out = default_output_path(body_class, clip_name, unit_id)
    export_attack_clip(body_class, clip_name, out, unit_id=unit_id)
    print(f"Exported attack clip: {out}")
    return 0


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else sys.argv[1:]
    raise SystemExit(_cli(args))
