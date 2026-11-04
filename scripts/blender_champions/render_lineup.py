"""Render the production Blender components for Base, A1, A2, B1 and B2."""

from math import radians, sqrt
from pathlib import Path
import sys

import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from export_game import FORMS, GAME_UNITS, collection
from geometry import Model
from render_study import OUT, studio
from rig import TO_BLENDER, rest_positions

STUDIO_ANGLES = {
    "shieldbearer": -44, "hexer": 19, "sawbot": 24, "bloomkeeper": 15,
    "siegewalker": -27, "rootcaller": 13, "nester": -44, "rampart": 29,
    "longaxe": 35, "farwatch": 27, "bombard": 32, "rotcap": 24,
    "oldfinn": -28, "antidotebrewer": 24,
}
RIGHT_CARRY = {
    "siegewalker": (21, -133, -18),
    "longaxe": (5, -102, -8),
    "farwatch": (15, -110, -7),
    "oldfinn": (-17, -72, -26),
    "antidotebrewer": (-19, -64, -9),
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


def mortar_carry(joint, rest):
    turn = Matrix.Rotation(radians(-20), 4, "X")
    grip = TO_BLENDER @ (rest["Chest"] + Vector((-.06,.11,.14)))
    if joint == "WeaponR":
        return Matrix.Translation(grip) @ turn
    for side, sign in (("R",-1),("L",1)):
        upper, fore, hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
        if joint not in (upper, fore, hand):
            continue
        target = grip if side == "R" else grip + turn.to_3x3() @ (TO_BLENDER @ Vector((0,-.03,.24)))
        if joint == hand:
            return Matrix.Translation(target) @ turn
        shoulder = TO_BLENDER @ rest[upper]
        delta = target - shoulder
        axis = delta.normalized()
        hint = TO_BLENDER @ Vector((sign,-.75,-.40))
        outward = (hint-axis*hint.dot(axis)).normalized()
        a, b = (rest[fore]-rest[upper]).length, (rest[hand]-rest[fore]).length
        along = (a*a - b*b + delta.length_squared) / (2*delta.length)
        elbow = shoulder + axis*along + outward*sqrt(max(0,a*a-along*along))
        start, end = (shoulder, elbow) if joint == upper else (elbow, target)
        rotate = Vector((0,0,-1)).rotation_difference(end-start).to_matrix().to_4x4()
        return Matrix.Translation(start) @ rotate
    return Matrix.Translation(TO_BLENDER @ rest[joint])


def studio_pose(unit, joint, rest, tier, branch):
    transform = Matrix.Translation(TO_BLENDER @ rest[joint])
    if unit == "sawbot":
        return saw_guard(joint, rest, tier, branch) @ transform
    if unit == "bombard":
        return mortar_carry(joint, rest)
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
    if unit == "oldfinn":
        height, aim = 690, .92
    studio((0,-7.8,2.25),(0,0,aim),scale,(1920,height),OUT / f"{unit}-lineup.png")
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    units = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    for unit in units or STUDIO_ANGLES:
        render(unit, STUDIO_ANGLES[unit])
