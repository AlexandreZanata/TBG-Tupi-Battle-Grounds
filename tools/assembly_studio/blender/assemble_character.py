#!/usr/bin/env python3
"""Assemble modular character from recipe JSON — Blender headless or GLB fallback."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
if str(REPO / "tools") not in sys.path:
    sys.path.insert(0, str(REPO / "tools"))

from assembly_studio.library.assemble import assemble_from_recipe  # noqa: E402


def main(argv: list[str] | None = None) -> int:
    args_list = argv if argv is not None else sys.argv
    if "--" in args_list:
        args_list = args_list[args_list.index("--") + 1 :]
    parser = argparse.ArgumentParser(description="Assemble library recipe → export GLB")
    parser.add_argument("--recipe", type=Path, required=True, help="Recipe JSON path")
    parser.add_argument("--repo", type=Path, default=REPO, help="Repository root")
    args = parser.parse_args(args_list)
    output = assemble_from_recipe(args.recipe.resolve(), repo_root=args.repo.resolve())
    print(json.dumps({"unitId": output.stem, "export": str(output)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
