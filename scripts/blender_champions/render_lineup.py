"""Render the production Blender components for Base, A1, A2, B1 and B2."""

from math import atan2, radians, sqrt
from pathlib import Path
import sys

import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from abyssharpooner import SUPPORT as HARPOON_SUPPORT
from foamweaver import SUPPORT as CHALICE_SUPPORT
from undertowherald import SUPPORT as CONCH_SUPPORT
from celestialarcher import HALF as CELESTIAL_BOW_HALF, TIP_Z as BOW_TIP_Z
from export_game import FORMS, GAME_UNITS, collection
from geometry import Model
from hornet import HOSE, HOSE_START, HOSE_END
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
    "whisperer": 16, "bulwark": -22, "demolisher": 27, "crowcaller": 24,
    "trigger": -24, "rattletick": 20,
    "capybara": 23, "trident": 15, "solarsentinel": 19,
    "luminousoracle": 18, "celestialarcher": 27, "justiciar": -24,
    "jawguard": 27, "mantis": 35, "hornet": 24, "weaver": 34,
    "pyrewarden": -38, "drillnest": 36, "gravecutter": -24,
    "lastlightcantor": 24, "cragclaw": -22, "hushfang": 47,
    "spinehurler": -28, "miretender": -19, "gorgehowl": 17, "sunlancer": -20,
    "eclipsedancer": 20, "starbreaker": 23, "dawnmender": 18, "prismancer": -23,
    "leafcutter": 36, "blastbeetle": -43,
    "needlewing": 33, "nectarnurse": -29, "stridulator": 26,
    "basaltbulwark": -22, "cindermaul": 26, "hookbrawler": 27,
    "flintrunner": -26, "claybinder": 33, "gustdrummer": -17, "hidemender": 29,
    "pebbleslinger": 14, "rivetballista": 29, "nacreduelist": -23, "reeflancer": 21,
    "trenchfist": -19, "pearlslinger": 22, "abyssharpooner": -28, "undertowherald": 17,
    "trenchlamplighter": 27, "foamweaver": -24,
}
INSECT_AND_PREVIEW_FRAMING = {
    "jawguard": (1.44,7.58,595,.62), "mantis": (1.32,7.10,665,.74),
    "hornet": (1.56,8.35,635,.75), "weaver": (1.70,9.05,530,.47),
    "pyrewarden": (1.46,7.86,665,.76), "drillnest": (1.52,8.05,540,.52),
    "gravecutter": (1.29,6.84,640,.71), "lastlightcantor": (1.32,7.05,695,.75),
    "cragclaw": (1.35,7.15,625,.71), "hushfang": (1.62,8.80,520,.42),
}
COMPLETION_FRAMING = {
    "spinehurler": (1.38,7.30,685,.77), "miretender": (1.34,7.10,595,.64),
    "gorgehowl": (1.37,7.24,665,.74), "sunlancer": (1.53,8.00,730,.89),
    "eclipsedancer": (1.43,7.55,665,.74), "starbreaker": (1.51,7.90,675,.77),
    "dawnmender": (1.43,7.52,655,.75), "prismancer": (1.38,7.35,665,.74),
    "leafcutter": (1.44,7.55,630,.72), "blastbeetle": (1.48,7.81,630,.73),
    "needlewing": (1.48,7.79,655,.74), "nectarnurse": (1.40,7.39,640,.73),
    "stridulator": (1.39,7.30,630,.73), "basaltbulwark": (1.57,8.19,655,.74),
    "cindermaul": (1.52,7.99,645,.74), "hookbrawler": (1.48,7.77,675,.80),
    "flintrunner": (1.42,7.45,675,.82), "claybinder": (1.44,7.63,670,.77),
    "gustdrummer": (1.48,7.78,645,.76), "hidemender": (1.42,7.50,665,.78),
    "pebbleslinger": (1.35,7.12,650,.69), "rivetballista": (1.43,7.52,690,.81),
    "nacreduelist": (1.39,7.34,670,.80), "reeflancer": (1.39,7.31,725,.86),
    "trenchfist": (1.44,7.59,625,.71), "pearlslinger": (1.37,7.23,645,.70),
    "abyssharpooner": (1.51,7.95,675,.80), "undertowherald": (1.44,7.57,665,.75),
    "trenchlamplighter": (1.42,7.51,710,.86), "foamweaver": (1.42,7.49,665,.78),
}
COMPLETION_CARRY = {
    "miretender": (("R",(-.154,.192,.211),-12,-9),("L",(.165,.147,.194),0,0)),
    "eclipsedancer": (("R",(-.155,.180,.202),-48,-17),("L",(.160,.154,.214),-33,21)),
    "prismancer": (("R",(-.165,.173,.201),-8,-8),("L",(.122,.159,.196),0,0)),
    "leafcutter": (("R",(-.169,.167,.210),-29,24),),
    "blastbeetle": (("R",(-.151,.177,.209),-34,-13),("L",(.155,.163,.208),-22,17)),
    "nectarnurse": (("R",(-.153,.162,.192),-28,-31),("L",(.177,.139,.201),0,10)),
    "stridulator": (("R",(-.184,.121,.141),0,0),("L",(.178,.118,.155),0,0)),
    "basaltbulwark": (("R",(-.256,.087,.217),-31,-28),("L",(.333,.069,.154),0,19)),
    "cindermaul": (("R",(-.248,.162,.198),-39,-20),("L",(.263,.098,.202),-26,24)),
    "hookbrawler": (("R",(-.246,.106,.179),-22,-62),("L",(.242,.097,.154),-13,25)),
    "flintrunner": (("R",(-.224,.107,.204),-24,-33),("L",(.219,.134,.177),-18,65)),
    "claybinder": (("R",(-.248,.119,.216),-17,-25),("L",(.249,.148,.222),0,18)),
    "hidemender": (("R",(-.186,.155,.217),-37,-64),("L",(.191,.116,.176),0,0)),
    "pebbleslinger": (("R",(-.214,.128,.217),8,-24),("L",(.222,.101,.160),-5,0)),
    "rivetballista": (("R",(-.188,.148,.220),-19,-68),),
    "nacreduelist": (("R",(-.217,.140,.202),-18,-38),("L",(.207,.123,.186),-25,61)),
    "reeflancer": (("R",(-.230,.126,.176),-17,-12),("L",(.176,.064,.125),0,0)),
    "trenchfist": (("R",(-.247,.106,.239),-6,-6),("L",(.239,.089,.210),-12,18)),
    "pearlslinger": (("R",(-.186,.151,.206),-13,-34),("L",(.191,.073,.134),0,0)),
    "trenchlamplighter": (("R",(-.204,.133,.197),6,-36),),
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


def two_hand_carry(joint, rest, at, fore_grip, angle, yaw=0, roll=0):
    turn = (Matrix.Rotation(radians(yaw),4,"Z") @ Matrix.Rotation(radians(angle),4,"X")
            @ Matrix.Rotation(radians(roll),4,"Y"))
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


def sentinel_guard(joint,rest,default):
    chest = rest["Chest"]
    if joint in arm_nodes("R"):
        grip = TO_BLENDER @ (chest+Vector((-.219,.127,.147)))
        return reach_arm(joint,rest,"R",grip,Matrix.Rotation(radians(-65),4,"X"))
    grip = TO_BLENDER @ (chest+Vector((.241,.149,.207)))
    if joint in arm_nodes("L"):
        return reach_arm(joint,rest,"L",grip,Matrix.Identity(4))
    if joint in ("StringTop","StringBottom"):
        s = -1 if joint=="StringTop" else 1
        pivot = TO_BLENDER @ (rest[joint]-rest["OffhandL"])
        return Matrix.Translation(grip+pivot) @ Matrix.Rotation(radians(s*22),4,"Z")
    return default


def oracle_guard(joint,rest,default):
    grip = TO_BLENDER @ (rest["Chest"]+Vector((-.217,.117,.154)))
    turn = Matrix.Rotation(radians(-14),4,"Y")
    if joint in arm_nodes("R"):
        return reach_arm(joint,rest,"R",grip,turn)
    if joint == "HeadGear":
        return Matrix.Translation(grip) @ turn @ Matrix.Translation(
            TO_BLENDER @ (rest["HeadGear"]-rest["WeaponR"]))
    return default


def archer_draw(joint,rest,tier,branch,default):
    grip = TO_BLENDER @ (rest["Chest"]+Vector((.128,.217,.30)))
    draw = .06
    nock = grip+TO_BLENDER @ Vector((0,0,BOW_TIP_Z-draw))
    if joint in arm_nodes("L"):
        return reach_arm(joint,rest,"L",grip,Matrix.Identity(4))
    if joint in arm_nodes("R"):
        return reach_arm(joint,rest,"R",nock,Matrix.Identity(4))
    if joint == "Nock":
        return Matrix.Translation(nock)
    if joint in ("StringTop","StringBottom"):
        half = CELESTIAL_BOW_HALF[tier,branch]
        side = 1 if joint=="StringTop" else -1
        tip = grip+TO_BLENDER @ Vector((0,side*half,BOW_TIP_Z))
        return Matrix.Translation(tip) @ Matrix.Rotation(side*atan2(draw,half),4,"X")
    return default


def hornet_pose(joint,rest,default):
    at = TO_BLENDER @ (rest["Chest"]+Vector((-.170,.152,.234)))
    turn = Matrix.Rotation(radians(-7),4,"X")
    if joint in arm_nodes("R"):
        return reach_arm(joint,rest,"R",at,turn)
    if joint not in HOSE:
        return default
    start = TO_BLENDER @ (rest["BackSpring"]+Vector(HOSE_START))
    end = at+turn.to_3x3() @ (TO_BLENDER @ Vector(HOSE_END))
    control = start.lerp(end,.5)+TO_BLENDER @ Vector((-.22,-.18,0))
    i = HOSE.index(joint)
    def point(t):
        return start*(1-t)**2+control*(2*t*(1-t))+end*t*t
    a,b = point(i/4),point((i+1)/4)
    delta = b-a
    align = Vector((0,0,1)).rotation_difference(delta).to_matrix().to_4x4()
    return Matrix.Translation(a.lerp(b,.5)) @ align @ Matrix.Diagonal((1,1,delta.length+.015,1))


def completion_guard(unit,joint,rest,default):
    if unit=="lastlightcantor":
        for side, local, angle in (("R",(-.181,.180,.212),-9),("L",(.160,.145,.149),0)):
            if joint in arm_nodes(side):
                grip = TO_BLENDER @ (rest["Chest"]+Vector(local))
                return reach_arm(joint,rest,side,grip,Matrix.Rotation(radians(angle),4,"X"))
    if joint in arm_nodes("R"):
        grip = TO_BLENDER @ (rest["Chest"]+Vector((-.150,.149,.207)))
        yaw,roll = (-25,47) if unit=="gravecutter" else (-14,33)
        turn = Matrix.Rotation(radians(yaw),4,"Z") @ Matrix.Rotation(radians(roll),4,"Y")
        return reach_arm(joint,rest,"R",grip,turn)
    return default


def needlewing_carry(joint,rest,default):
    grip = TO_BLENDER @ (rest["Chest"]+Vector((-.173,.191,.204)))
    turn = Matrix.Rotation(radians(-21),4,"X") @ Matrix.Rotation(radians(-21),4,"Z")
    if joint in arm_nodes("R")[:-1]:
        return reach_arm(joint,rest,"R",grip,turn)
    if joint=="HeadGear":
        return Matrix.Translation(grip) @ turn @ Matrix.Translation(
            TO_BLENDER @ (rest["HeadGear"]-rest["HandR"]))
    return default


def cask_carry(joint,rest,default):
    for side,sign in (("R",-1),("L",1)):
        if joint in arm_nodes(side)[:-1]:
            grip = TO_BLENDER @ (rest["WeaponR"]+Vector((sign*.25,0,0)))
            turn = Matrix.Rotation(radians(sign*90),4,"Z")
            return reach_arm(joint,rest,side,grip,turn)
    return default


ABYSSAL_TWO_HAND_CARRY = {
    "abyssharpooner": ((-.136,.152,.197), HARPOON_SUPPORT, -9, -12),
    "undertowherald": ((-.145,.153,.190), CONCH_SUPPORT, -6, -14),
    "foamweaver": ((-.113,.145,.198), CHALICE_SUPPORT, -5, 19),
}


def latest_completion_pose(unit,joint,rest,tier,branch,default):
    if unit in ABYSSAL_TWO_HAND_CARRY:
        at, support, angle, yaw = ABYSSAL_TWO_HAND_CARRY[unit]
        return two_hand_carry(joint,rest,at,support,angle,yaw=yaw)
    if unit=="needlewing":
        return needlewing_carry(joint,rest,default)
    if unit=="gustdrummer":
        return cask_carry(joint,rest,default)
    if unit=="sunlancer":
        return two_hand_carry(joint,rest,(0,.12,.13),(0,0,.20),-59,yaw=6)
    if unit=="starbreaker":
        return two_hand_carry(joint,rest,(0,.17,.10),(0,0,.13),-22,yaw=24)
    if unit=="spinehurler":
        at = TO_BLENDER @ (rest["Chest"]+Vector((-.176,.158,.174)))
        turn = Matrix.Rotation(radians(-55),4,"X")
        if joint in arm_nodes("R"):
            return reach_arm(joint,rest,"R",at,turn)
        if joint=="HeadGear":
            return Matrix.Translation(at) @ turn
    if unit=="dawnmender":
        x = .17 if tier and branch==0 else .14
        for side,sign in (("R",-1),("L",1)):
            if joint in arm_nodes(side):
                grip=TO_BLENDER @ (rest["Chest"]+Vector((sign*x,.171,.250)))
                return reach_arm(joint,rest,side,grip,Matrix.Identity(4))
    for side,local,pitch_angle,yaw in COMPLETION_CARRY.get(unit,()):
        if joint in arm_nodes(side):
            grip=TO_BLENDER @ (rest["Chest"]+Vector(local))
            turn=Matrix.Rotation(radians(yaw),4,"Z") @ Matrix.Rotation(radians(pitch_angle),4,"X")
            return reach_arm(joint,rest,side,grip,turn)
    return default


def studio_pose(unit, joint, rest, tier, branch):
    transform = Matrix.Translation(TO_BLENDER @ rest[joint])
    if unit in COMPLETION_FRAMING:
        return latest_completion_pose(unit,joint,rest,tier,branch,transform)
    if unit == "hornet":
        return hornet_pose(joint,rest,transform)
    if unit == "pyrewarden":
        return two_hand_carry(joint,rest,(-.03,.24,.17),(0,0,.20),-8,yaw=8)
    if unit in {"gravecutter","lastlightcantor","cragclaw"}:
        return completion_guard(unit,joint,rest,transform)
    if unit == "mantis":
        for side,fore in (("L",-85),("R",-67)):
            if joint in arm_nodes(side):
                folded = folded_arm(joint,rest,side,-49,fore,-16)
                if joint in ("WeaponR","OffhandL"):
                    return folded @ Matrix.Rotation(radians(-36 if side=="R" else 27),4,"Z")
                return folded
    if unit == "trident":
        return two_hand_carry(joint,rest,(-.27,.32,.31),(0,0,.18),-20,yaw=50,roll=55)
    if unit == "solarsentinel":
        return sentinel_guard(joint,rest,transform)
    if unit == "luminousoracle":
        return oracle_guard(joint,rest,transform)
    if unit == "celestialarcher":
        return archer_draw(joint,rest,tier,branch,transform)
    if unit == "justiciar":
        for side, upper, fore in (("R",-22,-66),("L",-10,-23)):
            if joint in arm_nodes(side):
                return folded_arm(joint,rest,side,upper,fore)
    if unit == "bulwark":
        return two_hand_carry(joint,rest,(0,.13,.20),(0,.12,0),-14)
    if unit == "crowcaller":
        return two_hand_carry(joint,rest,(0,.23,.23),(0,-.02,.09),5)
    if unit in {"demolisher", "trigger"}:
        for side in ("L","R"):
            if joint in arm_nodes(side):
                if unit=="demolisher":
                    return folded_arm(joint,rest,side,-14,-39)
                return folded_arm(joint,rest,side,-28,-56 if side=="R" else -73,3)
    if unit == "rattletick":
        if joint in ("Back","StringTop"):
            return pitch(rest[joint],-65 if joint=="Back" else -82) @ transform
        if joint in arm_nodes("R"):
            grip = TO_BLENDER @ (rest["Chest"]+Vector((-.29,.10,.21)))
            turn = Matrix.Rotation(radians(-13),4,"X")
            return reach_arm(joint,rest,"R",grip,turn)
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
        "whisperer": 1.42, "bulwark": 1.53, "demolisher": 1.46, "rattletick": 1.45,
        "capybara": 1.47, "trident": 1.97, "solarsentinel": 1.39,
        "luminousoracle": 1.42, "celestialarcher": 1.48, "justiciar": 1.47,
    }.get(unit,1.27)
    if unit in INSECT_AND_PREVIEW_FRAMING:
        spacing = INSECT_AND_PREVIEW_FRAMING[unit][0]
    if unit in COMPLETION_FRAMING:
        spacing = COMPLETION_FRAMING[unit][0]
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
        rear_angle = {"oilpress": -23, "cargocrab": 23, "smuggler": -15,
                      "rattletick": 29, "capybara": 18, "cindermaul": 10, "hookbrawler": -10,
                      "claybinder": 11, "hidemender": 12, "abyssharpooner": 8, "foamweaver": -12,
                      "rivetballista": 14}.get(unit,0)
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
    if unit in {"whisperer","bulwark","demolisher","crowcaller","trigger","rattletick"}:
        scale,height,aim = {
            "whisperer": (7.50,570,.95), "bulwark": (8.15,565,.56),
            "demolisher": (7.70,590,.67), "crowcaller": (6.70,665,.72),
            "trigger": (6.63,640,.75), "rattletick": (7.75,605,.76),
        }[unit]
    if unit in {"capybara","trident","solarsentinel","luminousoracle","celestialarcher","justiciar"}:
        scale,height,aim = {
            "capybara": (7.72,545,.57), "trident": (10.83,665,.94),
            "solarsentinel": (7.42,655,.78), "luminousoracle": (7.55,720,.87),
            "celestialarcher": (7.76,690,.83), "justiciar": (7.65,670,.79),
        }[unit]
    if unit in INSECT_AND_PREVIEW_FRAMING:
        _,scale,height,aim = INSECT_AND_PREVIEW_FRAMING[unit]
    if unit in COMPLETION_FRAMING:
        _,scale,height,aim = COMPLETION_FRAMING[unit]
    studio((0,-7.8,2.25),(0,0,aim),scale,(1920,height),OUT / f"{unit}-lineup.png")
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    units = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    for unit in units or STUDIO_ANGLES:
        render(unit, STUDIO_ANGLES[unit])
