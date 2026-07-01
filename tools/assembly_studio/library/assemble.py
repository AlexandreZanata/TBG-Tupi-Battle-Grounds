"""Dispatch assembly to Blender bpy or pure-Python GLB merge."""

from __future__ import annotations

from pathlib import Path

from assembly_studio.library.assemble_gltf import assemble_gltf
from assembly_studio.library.recipe import AssemblyRecipe, resolve_recipe


def assemble_from_recipe(recipe_path: Path, *, repo_root: Path | None = None) -> Path:
    recipe = resolve_recipe(recipe_path, repo_root)
    try:
        import bpy  # noqa: F401

        from assembly_studio.blender.assemble_ops import assemble_in_blender

        return assemble_in_blender(recipe)
    except ImportError:
        return assemble_gltf(recipe)


def default_recipe_path(repo_root: Path, unit_id: str) -> Path:
    return repo_root / "content" / "characters" / "library" / "exports" / f"{unit_id}.recipe.json"
