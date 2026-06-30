#!/usr/bin/env python3
"""CLI entry — export Walk clip GLB (delegates to animation.walk.export)."""

from __future__ import annotations

import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

from tools.blender.package_path import ensure_repo_paths  # noqa: E402

ensure_repo_paths()

from tools.blender.animation.walk.export import default_output_path, export_walk_clip  # noqa: E402


def _cli(argv: list[str]) -> int:
    if not argv:
        raise SystemExit("usage: build_walk_clip.py <bodyClass> [unit_id]")
    body_class = argv[0]
    unit_id = argv[1] if len(argv) > 1 else None
    out = default_output_path(body_class, unit_id)
    export_walk_clip(body_class, out, unit_id=unit_id)
    print(f"Exported walk clip: {out}")
    return 0


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else sys.argv[1:]
    raise SystemExit(_cli(args))
