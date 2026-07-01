#!/usr/bin/env python3
"""Generate placeholder clip GLBs for TBG animation libraries."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parents[1]
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from animation_sets.placeholder_glb import write_placeholder_clip_glb  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
SETS_DIR = ROOT / "content" / "characters" / "animation_sets"
DEFAULT_SET_ID = "humanoid_heavy_combat_v1"


def _load_set(set_id: str) -> dict:
    path = SETS_DIR / f"{set_id}.json"
    if not path.is_file():
        raise FileNotFoundError(path)
    return json.loads(path.read_text(encoding="utf-8"))


def generate_for_set(set_id: str) -> list[Path]:
    data = _load_set(set_id)
    if data.get("clipSource") != "library":
        raise ValueError(f"{set_id}: clipSource must be library for placeholder export")

    body_class = data["bodyClass"]
    clips = data["clips"]
    output_dir = ROOT / "content" / "characters" / "animations" / "clips" / body_class
    written: list[Path] = []
    seen: set[str] = set()
    for clip_name in clips.values():
        if clip_name in seen:
            continue
        seen.add(clip_name)
        path = output_dir / f"{clip_name}.glb"
        write_placeholder_clip_glb(path, clip_name)
        written.append(path)
    return written


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate placeholder animation clip GLBs.")
    parser.add_argument("--set", default=DEFAULT_SET_ID, help="Animation set id")
    args = parser.parse_args()
    for path in generate_for_set(args.set):
        print(path.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
