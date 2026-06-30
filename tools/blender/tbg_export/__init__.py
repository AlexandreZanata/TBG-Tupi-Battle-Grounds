"""TBG Export — Blender add-on for glTF unit export (GPL, authoring only)."""

from __future__ import annotations

bl_info = {
    "name": "TBG Export",
    "author": "TBG",
    "version": (0, 1, 0),
    "blender": (3, 6, 0),
    "location": "File > Export > TBG Unit (.glb)",
    "description": "Export TBG character unit as glTF binary with validation",
    "category": "Import-Export",
}


def register() -> None:
    import bpy

    from .addon_ops import classes, menu_export

    for cls in classes:
        bpy.utils.register_class(cls)
    bpy.types.TOPBAR_MT_file_export.append(menu_export)


def unregister() -> None:
    import bpy

    from .addon_ops import classes, menu_export

    bpy.types.TOPBAR_MT_file_export.remove(menu_export)
    for cls in reversed(classes):
        bpy.utils.unregister_class(cls)
