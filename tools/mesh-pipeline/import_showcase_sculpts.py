#!/usr/bin/env python3
"""Import authored showcase sculpt GLBs into content/characters/meshes/."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MESHES_DIR = ROOT / "content" / "characters" / "meshes" / "showcase"
OPTIMIZE = ROOT / "tools" / "mesh-pipeline" / "optimize_glb.sh"

# Bbox-matched imports (source filename → pilot id). Skip ids already byte-identical.
DEFAULT_IMPORTS: dict[str, str] = {
    "120d200084b373275ca28c80e71d0f02.glb": "demo_hunter_01",
    "07dd3dc310083f0449890fa9c9776aa8.glb": "demo_bruiser_01",
    "5ccf65ec1612d2a429ca34b79b930df5.glb": "demo_striker_01",
    "e76641810d5acb8c6c6b82f36417276f.glb": "demo_vanguard_01",
    "1b13ac9bf5116441b95170cacce2d34f.glb": "demo_sentinel_01",
    "70e087ad8a7b59c0bdadd6c019325cfd.glb": "demo_guard_01",
    "25d6783c7ee5ec383f940edf0973879c.glb": "demo_scout_01",
    "27fbc655169662b90769705919e039f8.glb": "demo_reaver_01",
}


def file_md5(path: Path) -> str:
    digest = hashlib.md5()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def import_sculpt(source: Path, pilot_id: str, *, force: bool = False) -> str:
    target = MESHES_DIR / f"{pilot_id}.glb"
    if not source.is_file():
        raise FileNotFoundError(f"missing source glb: {source}")

    if target.is_file() and not force and file_md5(source) == file_md5(target):
        return "skipped"

    target.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(suffix=".glb", delete=False, dir=target.parent) as handle:
        tmp = Path(handle.name)
    try:
        subprocess.run([str(OPTIMIZE), "--mode", "visual", str(source), str(tmp)], check=True)
        tmp.replace(target)
    finally:
        if tmp.exists() and not target.exists():
            tmp.unlink(missing_ok=True)
    return "imported"


def resolve_sources(source_dir: Path, mapping: dict[str, str]) -> list[tuple[Path, str]]:
    pairs: list[tuple[Path, str]] = []
    for filename, pilot_id in mapping.items():
        pairs.append((source_dir / filename, pilot_id))
    return pairs


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Import showcase sculpt GLBs for pilot roster units.")
    parser.add_argument(
        "--source-dir",
        type=Path,
        default=Path("/data/downloads"),
        help="Directory containing source .glb files",
    )
    parser.add_argument("--force", action="store_true", help="Re-import even when byte-identical")
    parser.add_argument("--dry-run", action="store_true", help="Print mapping only")
    parser.add_argument(
        "--only",
        nargs="*",
        metavar="PILOT_ID",
        help="Import subset of pilot ids only (e.g. demo_scout_01 demo_reaver_01)",
    )
    args = parser.parse_args(argv)

    allowed = set(args.only) if args.only else None
    results: dict[str, str] = {}
    for source, pilot_id in resolve_sources(args.source_dir, DEFAULT_IMPORTS):
        if allowed is not None and pilot_id not in allowed:
            continue
        if args.dry_run:
            results[pilot_id] = "dry-run"
            continue
        results[pilot_id] = import_sculpt(source, pilot_id, force=args.force)

    print(json.dumps(results, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
