"""Contract: repo-relative path resolution for Blender Docker cwd."""

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
PACKAGE = REPO / "tools" / "blender" / "package_path.py"


def test_resolve_repo_path_module_exists():
    text = PACKAGE.read_text(encoding="utf-8")
    assert "resolve_repo_path" in text


def test_import_mixamo_walk_validates_missing_fbx():
    text = (REPO / "tools/blender/scripts/import_mixamo_walk_fbx.py").read_text(encoding="utf-8")
    assert "Mixamo FBX not found" in text
    assert "resolve_repo_path" in text
