"""Pure validation rules — testable without bpy."""

from __future__ import annotations

MAX_TRIS_MVP = 8000


def tri_count_warning(tri_count: int, limit: int = MAX_TRIS_MVP) -> str | None:
    if tri_count > limit:
        return f"tri count {tri_count} exceeds MVP limit {limit}"
    return None


def require_armature(has_armature: bool) -> str | None:
    if not has_armature:
        return "armature required"
    return None


def require_uv(has_uv: bool, object_name: str) -> str | None:
    if not has_uv:
        return f"{object_name}: UV required"
    return None
