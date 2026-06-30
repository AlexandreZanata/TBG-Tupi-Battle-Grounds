#!/usr/bin/env python3
"""Import authored showcase sculpt GLBs into content/characters/meshes/showcase/."""

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
MANIFEST_PATH = ROOT / "content/characters/fixed_test_imports.json"


def load_import_mapping(manifest: Path | None = None) -> dict[str, str]:
    path = manifest or MANIFEST_PATH
    data = json.loads(path.read_text(encoding="utf-8"))
    return dict(data["imports"])


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
    return [(source_dir / filename, pilot_id) for filename, pilot_id in mapping.items()]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Import showcase sculpt GLBs for pilot roster units.")
    parser.add_argument(
        "--source-dir",
        type=Path,
        default=Path("/data/downloads"),
        help="Directory containing source .glb files",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=MANIFEST_PATH,
        help="JSON manifest with imports map (default: content/characters/fixed_test_imports.json)",
    )
    parser.add_argument("--force", action="store_true", help="Re-import even when byte-identical")
    parser.add_argument("--dry-run", action="store_true", help="Print mapping only")
    parser.add_argument(
        "--only",
        nargs="*",
        metavar="PILOT_ID",
        help="Import subset of pilot ids only",
    )
    args = parser.parse_args(argv)

    mapping = load_import_mapping(args.manifest)
    allowed = set(args.only) if args.only else None
    results: dict[str, str] = {}
    for source, pilot_id in resolve_sources(args.source_dir, mapping):
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
