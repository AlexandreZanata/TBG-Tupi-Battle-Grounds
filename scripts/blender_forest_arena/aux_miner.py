"""Build and animate the Miner, its mine and two cumulative upgrades."""

from math import cos, sin

import bpy
from mathutils import Vector

from aux_parts import FPS, burst, component, harbor, pose, reveal, scale, slide, sweep, walk, work_rig
from hold_parts import banner, braces, disc
from mesh import box, pebble, surface, tube

BUILD = 12
TIER_2 = 17
TIER_3 = 22


def miner(group,root,bridge,seat):
    harbor(group,root,bridge,(1.12,.89),BUILD)
    body = component(group,"Rock outcrop",(0,0,0),outcrop,root)
    timbers = component(group,"Shored mine entrance",(0,0,0),shoring,root)
    rail = component(group,"Iron rails and sleepers",(0,0,0),rails,root)
    lamp = component(group,"Suspended lantern",(-.15,-.22,.81),lantern,root)
    cart = component(group,"Ore cart",(.53,.025,0),minecart,root)
    ore = component(group,"Ore load",(0,0,.22),lambda g: crystals(g,.55),cart)
    for obj,a,b,d in ((body,4.6,6.7,-.30),(timbers,6.45,8.85,.23),
                      (rail,7.5,9.35,.13),(lamp,9.2,10.4,.13),(cart,9.85,11.3,.20),
                      (ore,11.4,13.8,.05)):
        reveal(obj,a,b,drop=d)
    rig = work_rig(group,root,"Mineiro",(1.02,-.35,0),"miner")
    wrist = rig[3][0][2]
    component(group,"Forged pickaxe",(0,0,0),pick,wrist)
    reveal(rig[0],9.72,10.17)
    walk(rig,10.15,11.98,(1.01,-.40,0),(.43,-.40,0),180)
    impacts = strokes(rig)
    scene = bpy.context.scene
    scene.frame_set(round(impacts[0]*FPS)+1)
    tip = root.matrix_world.inverted_safe() @ wrist.matrix_world @ Vector((.329,0,.199))
    scene.frame_set(1)
    strike_pad = component(group,"Exposed ore seam",(tip.x,tip.y,max(.038,tip.z-.12)),
                           lambda g: crystals(g,.53),root)
    reveal(strike_pad,8.9,10.4,drop=.12)
    burst(group,root,tuple(tip),impacts)
    print("Mining impact at",tuple(round(value,3) for value in tip),flush=True)
    work_motion(lamp,cart,ore)
    upgrades(group,root,seat)


def outcrop(group):
    for i,(x,y,z,sx,sy,sz) in enumerate((
        (-.60,-.38,.47,.53,.31,.75),(-.69,.36,.48,.45,.33,.84),
        (-.73,-.06,.78,.41,.43,.64),(-.59,.47,.24,.44,.27,.44),
        (-.48,-.56,.25,.36,.24,.43),(-.47,.11,.85,.39,.48,.30),
    )):
        pebble(group,"Fractured slate outcrop",(x,y,z),(sx,sy,sz),
               "cliff" if i%3 else "cliff-shadow",sides=8,accent="stratum",phase=i*.47)
    points = [(-.32,y,z) for y,z in ((-.30,0),(.30,0),(.29,.49),(.21,.69),(0,.77),(-.22,.66),(-.30,.48))]
    surface(group,"Recessed adit shadow",points,[tuple(range(len(points)))],["soot"])


def shoring(group):
    for y in (-.296,.295):
        tube(group,"Adit oak upright",[(-.27,y,.05),(-.26,y,.30),(-.29,y,.70)],
             [.067,.056,.062],"oak",sides=7,accent="wood")
        braces(group,(-.24,y,.52),(-.24,y*.54,.72),key="wood",width=.042)
    box(group,"Chamfered adit lintel",(-.27,0,.741),(.20,.74,.143),"wood",.026)
    for y in (-.30,.30):
        box(group,"Iron clamp around post",(-.199,y,.67),(.105,.12,.061),"gunmetal",.013)
    for x,y in ((-.36,-.62),(-.03,.51),(-.75,.51)):
        pebble(group,"Loose slate at the portal",(x,y,.045),(.18,.13,.12),"stone-cool",
               sides=7,accent="stratum")


def rails(group):
    for i in range(8):
        box(group,"Set timber sleeper",(-.26+i*.171,0,.020),(.102,.455,.041),"oak",.009)
    for y in (-.151,.151):
        box(group,"Continuous iron rail",(.33,y,.055),(1.34,.039,.058),"gunmetal",.008)
        box(group,"Rail running surface",(.33,y,.089),(1.34,.032,.016),"silver",.004)


def minecart(group):
    box(group,"Ore wagon underframe",(0,0,.115),(.37,.33,.068),"gunmetal",.018)
    for side in (-1,1):
        y = side*.163
        for x in (-.128,.128):
            disc(group,"Flanged cast wheel",(x,y,.098),.067,.041,"gunmetal",sides=10)
            disc(group,"Cart axle cap",(x,y+side*.027,.098),.033,.019,"bronze",sides=8)
        box(group,"Angled oak wagon side",(0,side*.165,.237),(.44,.047,.22),"wood",.017)
        for x in (-.143,.143):
            box(group,"Riveted wagon strap",(x,side*.190,.238),(.044,.016,.21),"gunmetal",.007)
        box(group,"Wagon buffer panel",(side*.202,0,.231),(.047,.33,.214),"oak",.018)


def lantern(group):
    tube(group,"Suspension link",[(0,0,0),(0,0,-.117)],[.018,.018],"gunmetal",7)
    disc(group,"Lantern base",(0,0,-.25),.062,.034,"gunmetal",axis="z",sides=8)
    tube(group,"Lit lantern glass",[(0,0,-.241),(0,0,-.141)],[.043,.035],"glow",sides=8)
    for y in (-.047,.047):
        tube(group,"Lantern cage",[(0,y,-.252),(0,y,-.133)],[.012,.012],"iron",6)
    tube(group,"Lantern cap",[(0,0,-.146),(0,0,-.112)],[.059,.026],"bronze",sides=8)


def crystals(group,size=1):
    for i,(x,y,h,w) in enumerate(((-.12,-.04,.19,.068),(.03,.08,.28,.079),(.14,-.055,.16,.052),
                                 (-.04,-.10,.15,.061),(.10,.105,.13,.047))):
        x,y,h,w = (k*size for k in (x,y,h,w))
        tube(group,"Faceted copper-green ore",[(x,y,-.025),(x,y,h*.61),(x+.030,y-.016,h)],
             [w,w*.81,.005],"ore",sides=6,accent="ore-light")


def pick(group):
    tube(group,"Ash pick handle",[(-.105,0,-.123),(.15,0,.18),(.218,0,.255)],
         [.018,.018,.017],"wood-end",sides=8)
    tube(group,"Forged pick head",[(.125,0,.344),(.189,0,.293),(.236,0,.250),(.329,0,.199)],
         [.008,.030,.034,.005],"gunmetal",sides=8,accent="silver")


def strokes(rig):
    root,chest,neck,arms,legs = rig
    impact = []
    for start,end,seconds in ((12,16.9,1.35),(17,21.9,1.10),(22,27.6,.90)):
        t = start
        while t+seconds <= end:
            for u,lean,shoulder,elbow,head in (
                (0,-5,-14,-24,4),(.29,-17,-69,-52,12),
                (.45,-10,-80,-42,9),(.56,17,26,8,-15),
                (.70,12,13,-6,-8),(1,-5,-14,-24,4),
            ):
                time = t+seconds*u
                pose(chest,time,(0,lean,0))
                pose(neck,time,(0,head,0))
                pose(arms[0][0],time,(0,shoulder,-8))
                pose(arms[0][1],time,(0,elbow,0))
                pose(arms[1][0],time,(0,shoulder*.38,6))
                pose(arms[1][1],time,(0,elbow*.55,0))
            impact.append(t+seconds*.56)
            t += seconds
    for obj in (chest,neck,arms[0][0],arms[0][1],arms[1][0],arms[1][1]):
        pose(obj,28.2,(0,0,0))
    return impact


def work_motion(lamp,cart,ore):
    for i in range(66):
        t = 10.4+i*.258
        q = min(1,max(0,(28.1-t)*2))
        pose(lamp,t,(3*cos(t*4)*q,0,7*sin(t*4)*q))
        if t>=11.3:
            span = .047 if t<TIER_2 else .157
            slide(cart,t,(.57+span*sin(t*2.15)*q,.025,0))
    for t,size in ((15.8,.87),(16.8,1),(20.6,.93),(22.6,1.15),(27.4,1.19),(28.1,1.19)):
        scale(ore,t,size)


def upgrades(group,root,seat):
    winch = component(group,"Level 2 timber winch",(-.73,.58,0),winch_frame,root)
    drum = component(group,"Winch drum",(0,0,.81),pulley,winch)
    bucket = component(group,"Ore bucket",(.23,-.036,.26),ore_bucket,winch)
    heap = component(group,"Level 2 ore sorting tray",(.48,.57,.04),ore_tray,root)
    for obj,lag in ((winch,0),(heap,.31)):
        reveal(obj,TIER_2+lag,TIER_2+.73+lag,drop=.16)
    sweep(group,root,TIER_2,(1.09,.85))
    head = component(group,"Level 3 mine headframe",(-.73,.14,.54),headframe,root)
    wheel = component(group,"Headframe upper sheave",(0,0,1.11),pulley,head)
    crown = component(group,"Level 3 crystal seam",(-.43,-.51,.65),
                      lambda g: crystals(g,1.44),root)
    flag = component(group,"Mine pennant",(-.55,.46,1.30),
                     lambda g: mine_banner(g,seat),root)
    rig = work_rig(group,root,"Mining apprentice",(.52,.59,0),"miner",size=.75)
    component(group,"Apprentice shovel",(0,0,0),shovel,rig[3][0][2])
    for obj,lag in ((head,0),(crown,.19),(flag,.37)):
        reveal(obj,TIER_3+lag,TIER_3+.69+lag,drop=.18)
    scale(rig[0],0,.0001)
    scale(rig[0],22.22,.0001)
    scale(rig[0],22.84,.75)
    sweep(group,root,TIER_3,(1.13,.90))
    for i in range(39):
        t = 17.75+i*.266
        q = min(t,27.80)
        pose(drum,t,((q-17.75)*164,0,0))
        slide(bucket,t,(.23,-.036,.30+.18*sin((q-17.75)*2.8)))
        if t>=22.82:
            pose(wheel,t,(-(q-22.82)*207,0,0))
            pose(flag,t,(4*sin(t*3.2),6*sin(t*2.3),0))
            pose(rig[1],t,(0,18*sin((q-22.82)*4.1),0))
            pose(rig[3][0][0],t,(0,-22-32*sin((q-22.82)*4.1),0))


def winch_frame(group):
    for y in (-.117,.117):
        braces(group,(-.10,y,.03),(0,y,.84),key="wood",width=.046)
        braces(group,(.13,y,.03),(0,y,.84),key="wood",width=.046)
    braces(group,(-.10,-.13,.07),(.13,.13,.73),key="oak",width=.031)


def pulley(group):
    for x in (-.092,.092):
        disc(group,"Slotted winch cheek",(x,0,0),.12,.037,"bronze",axis="x",sides=12)
    tube(group,"Wound rope core",[(-.105,0,0),(.105,0,0)],[.077,.077],"oak",sides=10)
    braces(group,(.115,0,0),(.127,.083,.114),key="gunmetal",width=.022)


def ore_bucket(group):
    tube(group,"Tapered bucket",[(0,0,-.055),(0,0,.095)],[.073,.100],"wood",sides=10,accent="oak")
    disc(group,"Iron bucket lip",(0,0,.084),.104,.026,"gunmetal",axis="z",sides=10)
    tube(group,"Bucket suspension rope",[(0,0,.083),(-.05,0,.49)],[.012,.012],"linen",sides=6)


def ore_tray(group):
    box(group,"Shallow sorting tray",(0,0,0),(.44,.31,.071),"oak",.016)
    crystals(group,.78)


def headframe(group):
    for y in (-.19,.19):
        braces(group,(-.20,y,0),(-.067,y,.985),width=.069)
        braces(group,(.26,y,.02),(.063,y,1.0),width=.064)
        braces(group,(-.14,y,.26),(.18,y,.64),key="wood",width=.040)
    box(group,"Capped headframe crossbeam",(0,0,1.10),(.32,.48,.12),"wood",.025)
    for y in (-.18,.18):
        tube(group,"Haul rope",[(.041,y,.25),(.02,y,1.12)],[.011,.011],"linen",sides=6)


def mine_banner(group,seat):
    tube(group,"Mine standard pole",[(0,0,-.52),(0,0,.43)],[.029,.021],"oak",sides=7)
    banner(group,seat,.10,-.018,.35,.29,.33)


def shovel(group):
    tube(group,"Shovel handle",[(-.10,0,-.11),(.29,0,-.29)],[.017,.016],"wood",sides=7)
    box(group,"Forged scoop",(.345,0,-.308),(.139,.117,.028),"gunmetal",.019,-.18)
