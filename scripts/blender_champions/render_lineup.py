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

STUDIO_ANGLES = {
    "shieldbearer": -44, "hexer": 19, "sawbot": 24, "bloomkeeper": 15,
    "siegewalker": -27, "rootcaller": 13, "nester": -44, "rampart": 29,
    "longaxe": 35, "farwatch": 27,
}
RIGHT_CARRY = {
    "siegewalker": (21, -133, -18),
    "longaxe": (5, -102, -8),
    "farwatch": (15, -110, -7),
}


def pitch(pivot, angle):
    point = TO_BLENDER @ pivot
    return Matrix.Translation(point) @ Matrix.Rotation(radians(angle), 4, "X") @ Matrix.Translation(-point)


def saw_guard(joint, rest, tier, branch):
    for side, tilt in (("R", -55), ("L", -38)):
        if joint not in {f"UpperArm{side}", f"Forearm{side}", f"Hand{side}"}:
            continue
        if side == "R" or (tier == 2 and branch == 0):
            return pitch(rest[f"UpperArm{side}"], tilt)
    return Matrix.Identity(4)


def studio_pose(unit, joint, rest, tier, branch):
    transform = Matrix.Translation(TO_BLENDER @ rest[joint])
    if unit == "sawbot":
        return saw_guard(joint, rest, tier, branch) @ transform
    if unit in RIGHT_CARRY:
        upper, fore, grip_pitch = RIGHT_CARRY[unit]
        shoulder = pitch(rest["UpperArmR"], upper)
        if joint == "UpperArmR":
            return shoulder @ transform
        if joint in {"ForearmR", "HandR", "WeaponR"}:
            transform = shoulder @ pitch(rest["ForearmR"], fore) @ transform
            if joint == "WeaponR":
                grip = Matrix.Translation(transform.translation) @ Matrix.Rotation(radians(grip_pitch), 4, "X")
                if unit == "longaxe":
                    grip = grip @ Matrix.Rotation(radians(-18), 4, "Y") @ Matrix.Translation((0,0,.66))
                return grip
            return transform
    if unit == "rootcaller":
        for side, grip in (("L","OffhandL"),("R","WeaponR")):
            if joint in {f"UpperArm{side}", f"Forearm{side}", f"Hand{side}", grip}:
                return pitch(rest[f"UpperArm{side}"], -27) @ transform
    return transform


def render(unit, angle):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    spacing = {"siegewalker": 1.46, "longaxe": 1.64}.get(unit,1.27)
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
            part.obj.matrix_basis = studio_pose(unit, part.joint, rest, tier, branch)
        anchor.location.x = (column - 2) * spacing
        anchor.rotation_euler.z = radians(angle)
        anchor.scale = (1 + .08 * tier,) * 3
    height, aim = (580,.76) if unit in {"shieldbearer","hexer"} else (610,.79)
    scale = {"siegewalker": 7.56, "longaxe": 9.4}.get(unit,6.58)
    if unit == "longaxe":
        height, aim = 850, 1.56
    studio((0,-7.8,2.25),(0,0,aim),scale,(1920,height),OUT / f"{unit}-lineup.png")
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    units = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    for unit in units or STUDIO_ANGLES:
        render(unit, STUDIO_ANGLES[unit])
