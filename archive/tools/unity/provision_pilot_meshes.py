#!/usr/bin/env python3
"""Copy placeholder glb meshes for Phase 02 pilot roster (ponytail: until authored assets land)."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ROSTER_PATH = ROOT / "content" / "characters" / "pilot_roster.json"
FIXTURE = ROOT / "content" / "characters" / "meshes" / "_test_fixture.glb"
MESHES_DIR = ROOT / "content" / "characters" / "meshes" / "showcase"


def load_pilot_ids() -> list[str]:
    data = json.loads(ROSTER_PATH.read_text(encoding="utf-8"))
    return [str(entry["id"]) for entry in data["pilots"]]


def provision(force: bool = False) -> list[Path]:
    if not FIXTURE.is_file():
        raise FileNotFoundError(f"missing fixture glb: {FIXTURE}")

    MESHES_DIR.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []
    for pilot_id in load_pilot_ids():
        target = MESHES_DIR / f"{pilot_id}.glb"
        if target.exists() and not force:
            continue
        shutil.copy2(FIXTURE, target)
        written.append(target)
    return written


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Provision pilot placeholder .glb meshes.")
    parser.add_argument("--force", action="store_true", help="Overwrite existing pilot glbs")
    args = parser.parse_args(argv)

    for path in provision(force=args.force):
        print(path.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
