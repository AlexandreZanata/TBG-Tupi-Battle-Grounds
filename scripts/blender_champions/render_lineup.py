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
from shatterback import plate as shatter_plate

STUDIO_ANGLES = {
    "shieldbearer": -44, "hexer": 19, "sawbot": 24, "bloomkeeper": 15,
    "siegewalker": -27, "rootcaller": 13, "nester": -44, "rampart": 29,
    "longaxe": 35, "farwatch": 27, "bombard": 32, "rotcap": 24,
    "oldfinn": -28, "antidotebrewer": 24,
    "shatterback": 28, "bloodfurnace": 26,
    "bloodweaver": 24, "oilpress": -26,
    "drumguard": 23, "leechroot": -25,
    "cargocrab": 32, "puppetmaster": 17, "smuggler": -33,
    "bonehorn": 42, "ravager": 34, "acidmaw": 24,
}
RIGHT_CARRY = {
    "siegewalker": (21, -133, -18),
    "longaxe": (5, -102, -8),
    "farwatch": (15, -110, -7),
    "oldfinn": (-17, -72, -26),
    "antidotebrewer": (-19, -64, -9),
    "bloodweaver": (-10, -75, -14),
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


def two_hand_carry(joint, rest, at, fore_grip, angle):
    turn = Matrix.Rotation(radians(angle), 4, "X")
    grip = TO_BLENDER @ (rest["Chest"] + Vector(at))
    if joint in arm_nodes("R"):
        return reach_arm(joint,rest,"R",grip,turn)
    if joint in arm_nodes("L"):
        target = grip + turn.to_3x3() @ (TO_BLENDER @ Vector(fore_grip))
        return reach_arm(joint,rest,"L",target,turn)
    return Matrix.Translation(TO_BLENDER @ rest[joint])


def arm_nodes(side):
    return (f"UpperArm{side}",f"Forearm{side}",f"Hand{side}","WeaponR" if side=="R" else "OffhandL")


def reach_arm(joint,rest,side,target,turn):
    upper,fore,hand,prop = arm_nodes(side)
    if joint in (hand,prop):
        return Matrix.Translation(target) @ turn
    shoulder = TO_BLENDER @ rest[upper]
    delta = target-shoulder
    axis = delta.normalized()
    hint = TO_BLENDER @ Vector((-1 if side=="R" else 1,-.75,-.40))
    outward = (hint-axis*hint.dot(axis)).normalized()
    a,b = (rest[fore]-rest[upper]).length,(rest[hand]-rest[fore]).length
    along = (a*a-b*b+delta.length_squared)/(2*delta.length)
    elbow = shoulder+axis*along+outward*sqrt(max(0,a*a-along*along))
    start,end = (shoulder,elbow) if joint==upper else (elbow,target)
    rotate = Vector((0,0,-1)).rotation_difference(end-start).to_matrix().to_4x4()
    return Matrix.Translation(start) @ rotate


def drum_pose(joint,rest,tier,branch):
    depth = (.15 if tier==1 else .18) if tier and branch==0 else .13
    for side,x,lift in (("R",-.025,0),("L",.13,.71)):
        if joint not in arm_nodes(side):
            continue
        turn = Matrix.Rotation(radians(60+25*lift),4,"X")
        tip = TO_BLENDER @ (rest["Hips"]+Vector((x,.0275+depth*.5+.075*lift,.27)))
        target = tip-turn.to_3x3() @ (TO_BLENDER @ Vector((0,0,.23)))
        return reach_arm(joint,rest,side,target,turn)
    return Matrix.Translation(TO_BLENDER @ rest[joint])


def studio_pose(unit, joint, rest, tier, branch):
    transform = Matrix.Translation(TO_BLENDER @ rest[joint])
    if unit == "cargocrab":
        for side in ("L","R"):
            if joint in arm_nodes(side):
                return folded_arm(joint,rest,side,-42,-55)
    if unit == "puppetmaster":
        for side, fore in (("L",-63),("R",-72)):
            if joint in arm_nodes(side) and (side=="R" or branch==1):
                return folded_arm(joint,rest,side,-24,fore,0)
    if unit == "smuggler":
        if joint in arm_nodes("R"):
            return folded_arm(joint,rest,"R",-17,-61,12)
        if joint in arm_nodes("L") and (tier,branch)==(2,1):
            return folded_arm(joint,rest,"L",-10,-43,0)
    if unit == "acidmaw":
        petal_turns = {"WeaponR": ("Z",14), "OffhandL": ("Z",-14),
                       "Back": ("X",-14), "StringTop": ("X",14)}
        if joint in petal_turns:
            axis, angle = petal_turns[joint]
            return transform @ Matrix.Rotation(radians(angle),4,axis)
    if unit == "sawbot":
        return saw_guard(joint, rest, tier, branch) @ transform
    if unit == "bombard":
        return two_hand_carry(joint,rest,(-.06,.11,.14),(0,-.03,.24),-20)
    if unit == "bloodfurnace":
        return two_hand_carry(joint,rest,(.02,.25,.25),(0,0,.14),35)
    if unit == "bloodweaver" and joint in ("UpperArmL","ForearmL","HandL","OffhandL"):
        return folded_arm(joint,rest,"L",-21,-67,0)
    if unit == "oilpress":
        for side in ("L","R"):
            if joint in (f"UpperArm{side}",f"Forearm{side}",f"Hand{side}"):
                return folded_arm(joint,rest,side,-16,-34)
    if unit == "drumguard":
        return drum_pose(joint,rest,tier,branch)
    if unit == "leechroot" and joint not in ("Root","Hips") and not joint.startswith(("Thigh","Shin","Foot")):
        if joint in ("Head","HeadGear"):
            transform = pitch(rest["Head"],-14) @ transform
        for side in ("L","R"):
            if joint in arm_nodes(side):
                transform = folded_arm(joint,rest,side,-18,-23)
        return pitch(rest["Spine"],20) @ transform
    if unit == "shatterback":
        for side in ("L","R"):
            if joint in (f"UpperArm{side}",f"Forearm{side}",f"Hand{side}"):
                forward = pitch(rest[f"UpperArm{side}"],-38.5) @ transform
                return Matrix.Translation(forward.translation) if joint.startswith("Hand") else forward
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


def folded_arm(joint,rest,side,upper,fore,prop_pitch=None):
    transform = Matrix.Translation(TO_BLENDER @ rest[joint])
    shoulder = pitch(rest[f"UpperArm{side}"],upper)
    if joint == f"UpperArm{side}":
        return shoulder @ transform
    transform = shoulder @ pitch(rest[f"Forearm{side}"],fore) @ transform
    if prop_pitch is not None and joint in ("WeaponR","OffhandL"):
        return Matrix.Translation(transform.translation) @ Matrix.Rotation(radians(prop_pitch),4,"X")
    return transform


def render(unit, angle):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    spacing = {
        "siegewalker": 1.46, "longaxe": 1.64, "shatterback": 1.46, "oilpress": 1.46,
        "cargocrab": 1.66, "bonehorn": 2.06, "ravager": 1.80, "acidmaw": 1.15,
    }.get(unit,1.27)
    for column, (label, tier, branch) in enumerate(FORMS):
        group = collection(label, scene.collection)
        anchor = bpy.data.objects.new(label, None)
        group.objects.link(anchor)
        model = Model(group)
        GAME_UNITS[unit](model, tier, branch, production=True)
        if unit == "shatterback":
            for index in range(4):
                shatter_plate(model,tier,branch,index)
        rest = rest_positions(unit, production=True, tier=tier, branch=branch)
        for part in model.parts:
            part.obj.parent = anchor
            part.obj.data.transform(TO_BLENDER)
            part.obj.matrix_basis = studio_pose(unit, part.joint, rest, tier, branch)
        anchor.location.x = (column - 2) * spacing - (.20 if unit=="bonehorn" else 0)
        # Rear-mounted Evolution equipment needs a wider profile to stay legible.
        rear_angle = {"oilpress": -23, "cargocrab": 23, "smuggler": -15}.get(unit,0)
        anchor.rotation_euler.z = radians(angle+rear_angle if tier and branch==1 else angle)
        anchor.scale = (1 + .08 * tier,) * 3
    height, aim = (580,.76) if unit in {"shieldbearer","hexer"} else (610,.79)
    scale = {"siegewalker": 7.56, "longaxe": 9.4}.get(unit,6.58)
    if unit == "longaxe":
        height, aim = 850, 1.56
    if unit == "oldfinn":
        height, aim = 690, .92
    if unit == "shatterback":
        scale, height, aim = 7.55, 650, .51
    if unit == "bloodweaver":
        height, aim = 710, .91
    if unit == "oilpress":
        scale, height = 7.56, 665
    if unit == "drumguard":
        height, aim = 690, .89
    if unit in {"cargocrab","puppetmaster","smuggler","bonehorn","ravager","acidmaw"}:
        scale,height,aim = {
            "cargocrab": (8.75,475,.39), "puppetmaster": (6.68,665,.79),
            "smuggler": (6.71,675,.78), "bonehorn": (11.15,600,.67),
            "ravager": (9.70,515,.45), "acidmaw": (6.25,720,.80),
        }[unit]
    studio((0,-7.8,2.25),(0,0,aim),scale,(1920,height),OUT / f"{unit}-lineup.png")
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    units = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    for unit in units or STUDIO_ANGLES:
        render(unit, STUDIO_ANGLES[unit])
