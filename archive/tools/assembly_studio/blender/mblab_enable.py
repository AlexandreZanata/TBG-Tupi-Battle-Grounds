"""Enable vendored MB-Lab add-on inside a Blender Python session."""

from __future__ import annotations

import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
VENDOR = REPO / "third_party" / "MB-Lab"


def ensure_mblab_addon() -> bool:
    """Add vendored MB-Lab to sys.path and enable if present."""
    if not (VENDOR / "__init__.py").is_file():
        return False
    vendor_parent = str(VENDOR.parent)
    if vendor_parent not in sys.path:
        sys.path.insert(0, vendor_parent)
    try:
        import bpy  # noqa: PLC0415
    except ImportError:
        return False
    prefs = bpy.context.preferences.addons
    if "MB-Lab" not in prefs:
        try:
            bpy.ops.preferences.addon_enable(module="MB-Lab")
        except Exception:  # noqa: BLE001
            try:
                import addon_utils

                addon_utils.enable("MB-Lab", default_set=True, persistent=True)
            except Exception:  # noqa: BLE001
                return False
    return "MB-Lab" in bpy.context.preferences.addons
