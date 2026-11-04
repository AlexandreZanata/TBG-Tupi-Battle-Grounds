"""Render the production Blender components for Base, A1, A2, B1 and B2."""

from math import radians
from pathlib import Path
import sys

import bpy
from mathutils import Matrix

sys.path.insert(0, str(Path(__file__).resolve().parent))
from export_game import FORMS, GAME_UNITS, collection
from geometry import Model
from render_study import OUT, studio
from rig import TO_BLENDER, rest_positions

STUDIO_ANGLES = {"shieldbearer": -44, "hexer": 19, "sawbot": 24, "bloomkeeper": 15}


def saw_guard(joint, rest, tier, branch):
    for side, tilt in (("R", -55), ("L", -38)):
        if joint not in {f"UpperArm{side}", f"Forearm{side}", f"Hand{side}"}:
            continue
        if side == "R" or (tier == 2 and branch == 0):
            pivot = TO_BLENDER @ rest[f"UpperArm{side}"]
            return Matrix.Translation(pivot) @ Matrix.Rotation(radians(tilt), 4, "X") @ Matrix.Translation(-pivot)
    return Matrix.Identity(4)


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
            transform = Matrix.Translation(TO_BLENDER @ rest[part.joint])
            if unit == "sawbot":
                transform = saw_guard(part.joint, rest, tier, branch) @ transform
            part.obj.matrix_basis = transform
        anchor.location.x = (column - 2) * 1.27
        anchor.rotation_euler.z = radians(angle)
        anchor.scale = (1 + .08 * tier,) * 3
    height, aim = (580,.76) if unit in {"shieldbearer","hexer"} else (610,.79)
    studio((0,-7.8,2.25),(0,0,aim),6.58,(1920,height),OUT / f"{unit}-lineup.png")
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    units = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    for unit in units or STUDIO_ANGLES:
        render(unit, STUDIO_ANGLES[unit])
