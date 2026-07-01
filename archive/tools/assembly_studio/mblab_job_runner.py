"""MB-Lab Blender job runner."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any


def dispatch_mblab(recipe_data: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    recipe_path = _write_temp_recipe(recipe_data, repo_root)
    blender = repo_root / "tools" / "assembly_studio" / "blender" / "blender_cmd.sh"
    script = repo_root / "tools" / "assembly_studio" / "blender" / "generate_mblab_base.py"
    try:
        proc = subprocess.run(
            [
                str(blender),
                "--background",
                "--python",
                str(script),
                "--",
                "--recipe",
                str(recipe_path),
            ],
            capture_output=True,
            text=True,
            cwd=repo_root,
            timeout=600,
            check=False,
        )
    finally:
        recipe_path.unlink(missing_ok=True)
    if proc.returncode != 0:
        tail = (proc.stderr or proc.stdout or "blender failed").strip()[-1200:]
        raise RuntimeError(tail)
    return _parse_report(proc.stdout)


def _write_temp_recipe(data: dict[str, Any], repo_root: Path) -> Path:
    recipes = repo_root / "content" / "characters" / "mblab" / "recipes"
    recipes.mkdir(parents=True, exist_ok=True)
    path = recipes / f".job_{data['unitId']}.json"
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return path


def _parse_report(stdout: str) -> dict[str, Any]:
    for line in reversed(stdout.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            data = json.loads(line)
            if data.get("error"):
                raise RuntimeError(str(data["error"]))
            return data
    raise RuntimeError("no JSON report from generate_mblab_base.py")
