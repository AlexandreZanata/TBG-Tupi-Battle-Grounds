#!/usr/bin/env python3
"""Archive Rig Studio bones.json from .local into versioned content/."""

from __future__ import annotations

import argparse
import json
from datetime import UTC, datetime
from pathlib import Path

from mesh_pipeline.export_game_unit_core import repo_root

from tools.experimental.auto_rig.rig_bone_names import normalize_bones_payload

REPO = repo_root()
SESSIONS = REPO / ".local" / "rig-studio" / "sessions"
ARCHIVE_ROOT = REPO / "content" / "characters" / "rig-sessions"


def archive_session(unit_id: str, *, body_class: str | None = None) -> dict:
    src = SESSIONS / unit_id / "bones.json"
    if not src.is_file():
        raise FileNotFoundError(f"missing session bones: {src} — save in Rig Studio first")

    payload = normalize_bones_payload(json.loads(src.read_text(encoding="utf-8")))
    out_dir = ARCHIVE_ROOT / unit_id
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "bones.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    if body_class is None:
        body_class = _body_class_from_definition(unit_id)

    manifest = {
        "unitId": unit_id,
        "bodyClass": body_class,
        "boneCount": len(payload.get("bones") or {}),
        "archivedAt": datetime.now(UTC).isoformat(),
        "source": str(src.relative_to(REPO)),
        "workflow": "rig-studio-manual-placement",
    }
    (out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def _body_class_from_definition(unit_id: str) -> str:
    path = REPO / "content" / "characters" / "definitions" / f"{unit_id}.json"
    if not path.is_file():
        return "HumanoidLight"
    data = json.loads(path.read_text(encoding="utf-8"))
    return str(data.get("bodyClass") or "HumanoidLight")


def _cli(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Archive Rig Studio bones into content/characters/rig-sessions/")
    parser.add_argument("--id", dest="unit_id", required=True)
    parser.add_argument("--body-class", default=None)
    parser.add_argument("--report-json", action="store_true")
    args = parser.parse_args(argv)
    report = archive_session(args.unit_id, body_class=args.body_class)
    if args.report_json:
        print(json.dumps(report))
    else:
        print(f"Archived {report['boneCount']} bones → {ARCHIVE_ROOT / args.unit_id}")
    return 0


if __name__ == "__main__":
    raise SystemExit(_cli())
