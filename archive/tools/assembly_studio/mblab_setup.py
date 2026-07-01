"""Headless MB-Lab vendor, bootstrap, and library export."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

from assembly_studio.library.sockets import BASE_IDS

# Blocky seed fixtures from seed_library.sh are ~74KB; MB-Lab exports are multi-MB.
MBLAB_BODY_MIN_BYTES = 500_000


def _is_mblab_body(path: Path) -> bool:
    return path.is_file() and path.stat().st_size >= MBLAB_BODY_MIN_BYTES


ARCHETYPE_EXPORT: dict[str, dict[str, Any]] = {
    "male": {"heightM": 1.8, "bodyClass": "HumanoidLight", "label": "Male body"},
    "female": {"heightM": 1.65, "bodyClass": "HumanoidLight", "label": "Female body"},
    "small": {"heightM": 1.2, "bodyClass": "HumanoidLight", "label": "Small body"},
    "large": {"heightM": 2.1, "bodyClass": "HumanoidHeavy", "label": "Large body"},
}


def _run(cmd: list[str], *, cwd: Path, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, capture_output=True, text=True, cwd=cwd, timeout=timeout, check=False)


def _blender(repo_root: Path) -> str:
    return str(repo_root / "tools" / "assembly_studio" / "blender" / "blender_cmd.sh")


def _parse_json_line(stdout: str) -> dict[str, Any] | None:
    for line in reversed(stdout.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            return json.loads(line)
    return None


def _ensure_vendor(repo_root: Path) -> dict[str, Any]:
    vendor = repo_root / "third_party" / "MB-Lab" / "__init__.py"
    if vendor.is_file():
        return {"vendored": True, "skipped": True}
    script = repo_root / "scripts" / "vendor-mblab.sh"
    proc = _run(["bash", str(script)], cwd=repo_root, timeout=900)
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "vendor failed")[-800:])
    return {"vendored": True, "skipped": False}


def _ensure_install(repo_root: Path) -> dict[str, Any]:
    script = repo_root / "scripts" / "install-mblab.sh"
    proc = _run(["bash", str(script)], cwd=repo_root)
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "install failed")[-800:])
    return {"installed": True}


def bootstrap_template(base_id: str, repo_root: Path) -> dict[str, Any]:
    blend = repo_root / "content" / "characters" / "mblab" / "projects" / f"{base_id}.blend"
    if blend.is_file():
        return {"baseId": base_id, "blend": str(blend), "skipped": True}
    script = repo_root / "tools" / "assembly_studio" / "blender" / "bootstrap_mblab_template.py"
    proc = _run(
        [_blender(repo_root), "--background", "--python", str(script), "--", base_id],
        cwd=repo_root,
    )
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "bootstrap failed")[-1200:])
    report = _parse_json_line(proc.stdout) or {"baseId": base_id, "blend": str(blend)}
    return report


def _export_preview_only(base_id: str, repo_root: Path, blend: Path) -> dict[str, Any]:
    script = repo_root / "tools/assembly_studio/blender/export_mblab_studio_preview.py"
    proc = _run(
        [_blender(repo_root), "--background", str(blend), "--python", str(script), "--", "--base-id", base_id],
        cwd=repo_root,
        timeout=900,
    )
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "preview bake failed")[-1200:])
    report = _parse_json_line(proc.stdout) or {}
    report["previewOnly"] = True
    report["previewUrl"] = f"/api/assembly/bases/{base_id}/preview"
    return report


def _export_hair_part(repo_root: Path, blend: Path) -> dict[str, Any]:
    if blend.name != "male.blend":
        return {"skipped": True, "reason": "hair baked from male reference"}
    hair_glb = repo_root / "content/characters/library/parts/hair/hair_mblab_01.glb"
    if hair_glb.is_file() and hair_glb.stat().st_size > 50_000:
        return {"skipped": True, "hair": str(hair_glb)}
    script = repo_root / "tools/assembly_studio/blender/bake_mblab_hair.py"
    proc = _run(
        [_blender(repo_root), "--background", str(blend), "--python", str(script)],
        cwd=repo_root,
        timeout=900,
    )
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "hair bake failed")[-1200:])
    return _parse_json_line(proc.stdout) or {"hair": str(hair_glb)}


def export_library_base(base_id: str, repo_root: Path, *, force: bool = False) -> dict[str, Any]:
    profile = ARCHETYPE_EXPORT[base_id]
    blend = repo_root / "content" / "characters" / "mblab" / "projects" / f"{base_id}.blend"
    body = repo_root / "content/characters/library/bases" / base_id / "body.glb"
    preview = repo_root / "content/characters/library/bases" / base_id / "preview.glb"
    if body.is_file() and _is_mblab_body(body) and preview.is_file() and not force:
        return {
            "baseId": base_id,
            "previewUrl": f"/api/assembly/bases/{base_id}/preview",
            "skipped": True,
        }
    if not blend.is_file():
        bootstrap_template(base_id, repo_root)
    if body.is_file() and _is_mblab_body(body) and not preview.is_file() and not force:
        return _export_preview_only(base_id, repo_root, blend)
    script = repo_root / "tools" / "assembly_studio" / "blender" / "export_mblab_project.py"
    proc = _run(
        [
            _blender(repo_root),
            "--background",
            str(blend),
            "--python",
            str(script),
            "--",
            "--base-id",
            base_id,
            "--height-m",
            str(profile["heightM"]),
            "--body-class",
            str(profile["bodyClass"]),
            "--label",
            str(profile["label"]),
        ],
        cwd=repo_root,
    )
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "export failed")[-1200:])
    report = _parse_json_line(proc.stdout) or {}
    report["previewUrl"] = f"/api/assembly/bases/{base_id}/preview"
    if base_id == "male":
        report["hair"] = _export_hair_part(repo_root, blend)
    return report


def run_full_setup(repo_root: Path, *, export_library: bool = True) -> dict[str, Any]:
    report: dict[str, Any] = {"vendor": _ensure_vendor(repo_root), "install": _ensure_install(repo_root)}
    bootstrapped: list[dict[str, Any]] = []
    exported: list[dict[str, Any]] = []
    for base_id in BASE_IDS:
        bootstrapped.append(bootstrap_template(base_id, repo_root))
        if export_library:
            exported.append(export_library_base(base_id, repo_root))
    report["bootstrapped"] = bootstrapped
    report["exported"] = exported
    report["templatesReady"] = all(
        (repo_root / "content/characters/mblab/projects" / f"{bid}.blend").is_file() for bid in BASE_IDS
    )
    report["libraryBasesReady"] = all(
        _is_mblab_body(repo_root / "content/characters/library/bases" / bid / "body.glb")
        for bid in BASE_IDS
    )
    return report
