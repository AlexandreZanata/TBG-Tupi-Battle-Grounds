"""Assembly export job runner."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from assembly_studio.library.assemble import assemble_from_recipe
from assembly_studio.library.recipe import build_recipe, recipe_to_dict


def dispatch_assembly(recipe_data: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    recipe = build_recipe(recipe_data, repo_root=repo_root)
    recipe_path = _write_temp_recipe(recipe_data, repo_root)
    try:
        output = assemble_from_recipe(recipe_path, repo_root=repo_root)
    finally:
        recipe_path.unlink(missing_ok=True)
    rel = f"content/characters/library/exports/{recipe.unit_id}.glb"
    return {
        "recipe": recipe_to_dict(recipe),
        "export": str(output),
        "exportUrl": f"/api/content/{rel}",
    }


def _write_temp_recipe(data: dict[str, Any], repo_root: Path) -> Path:
    exports = repo_root / "content" / "characters" / "library" / "exports"
    exports.mkdir(parents=True, exist_ok=True)
    path = exports / f".job_{data['unitId']}.recipe.json"
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return path
