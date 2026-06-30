"""Blender operators — imported only when add-on registers."""

from __future__ import annotations

import bpy

from .export import export_unit_glb
from .validation import validate_scene


class TBG_OT_export_unit(bpy.types.Operator):
    bl_idname = "export_scene.tbg_unit_glb"
    bl_label = "TBG Unit (.glb)"
    bl_options = {"REGISTER"}

    unit_id: bpy.props.StringProperty(name="Unit Id", default="unit_01")

    def execute(self, context):
        for warning in validate_scene(context.scene):
            self.report({"WARNING"}, warning)
        path = export_unit_glb(self.unit_id)
        self.report({"INFO"}, f"Exported {path}")
        return {"FINISHED"}


def menu_export(self, context):
    self.layout.operator(TBG_OT_export_unit.bl_idname, text="TBG Unit (.glb)")


classes = (TBG_OT_export_unit,)
