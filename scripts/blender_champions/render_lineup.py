"""Render the production Blender components for Base, A1, A2, B1 and B2."""

from math import radians
from pathlib import Path
import sys

import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from export_game import FORMS, GAME_UNITS, collection
from geometry import Model
from render_study import OUT, studio
from rig import TO_BLENDER, rest_positions


def render(unit, angle):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    for column, (label, tier, branch) in enumerate(FORMS):
        group = collection(label, scene.collection)
        anchor = bpy.data.objects.new(label, None)
        group.objects.link(anchor)
        model = Model(group)
        GAME_UNITS[unit](model, tier, branch, production=True)
        rest = rest_positions(unit, production=True, tier=tier, branch=branch)
        for part in model.parts:
            part.obj.parent = anchor
            part.obj.data.transform(TO_BLENDER)
            part.obj.location = TO_BLENDER @ rest[part.joint]
        anchor.location.x = (column - 2) * 1.27
        anchor.rotation_euler.z = radians(angle)
        anchor.scale = (1 + .08 * tier,) * 3
    studio((0,-7.8,2.25),(0,0,.76),6.58,(1920,580),OUT / f"{unit}-lineup.png")
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    render("shieldbearer", -44)
    render("hexer", 19)
