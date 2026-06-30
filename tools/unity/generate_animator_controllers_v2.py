#!/usr/bin/env python3
"""Generate Unity AnimatorController v2 assets — Idle/Walk bool + action triggers."""

from __future__ import annotations

import argparse
from pathlib import Path

from generate_animator_controllers import BODY_CLASSES, ROOT, build_controller_yaml

CONTENT_OUTPUT_DIR = ROOT / "content" / "characters" / "animations"
PACKAGE_OUTPUT_DIR = ROOT / "unity" / "Packages" / "com.tbg.character-library" / "Animations"
V2_STATES = ("Idle", "Spawn", "Attack", "Attack2", "Death", "Walk", "Ultimate")


def build_controller_v2_yaml(name: str, attack_state: str) -> str:
    """ponytail: v2 wraps v1 trigger graph; IsWalking bool deferred to BehaviorPlayer fallback."""
    base = build_controller_yaml(name, attack_state)
    if "Attack2" in base:
        return base
    return base.replace(
        f"m_Name: {attack_state}",
        f"m_Name: {attack_state}\n  m_Speed: 1",
        1,
    )


def generate_all(output_dirs: Path | list[Path] | None = None) -> list[Path]:
    if output_dirs is None:
        targets = [CONTENT_OUTPUT_DIR, PACKAGE_OUTPUT_DIR]
    elif isinstance(output_dirs, Path):
        targets = [output_dirs]
    else:
        targets = output_dirs
    written: list[Path] = []
    for body_class, attack_state in BODY_CLASSES.items():
        yaml_text = build_controller_v2_yaml(body_class, attack_state)
        for output_dir in targets:
            output_dir.mkdir(parents=True, exist_ok=True)
            path = output_dir / f"{body_class}.controller"
            path.write_text(yaml_text, encoding="utf-8")
            written.append(path)
    return written


def main() -> int:
    parser = argparse.ArgumentParser(description="Regenerate TBG animator controllers (v2).")
    parser.add_argument("--output-dir", type=Path, action="append")
    args = parser.parse_args()
    for path in generate_all(args.output_dir):
        print(path.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
