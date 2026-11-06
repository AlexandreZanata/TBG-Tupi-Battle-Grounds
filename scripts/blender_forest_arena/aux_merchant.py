"""Build and animate the Merchant, the pier, market and caravel deliveries."""

from math import cos, pi, sin, tau

from mathutils import Vector

from aux_parts import END, SEA, burst, component, harbor, key, pose, reveal, scale, slide, sweep, walk, work_rig
from hold_parts import banner, braces, disc
from mesh import box, pebble, surface, tube

BUILD = 3
DELIVERY = 3.2
UPGRADES = (10,19)
DELIVERIES = (5.2,13.0,23.1)


def merchant(group,root,bridge,seat):
    harbor(group,root,bridge,(1.05,.71),BUILD)
    pier = component(group,"Dock apron",(0,-.68,0),jetty,root)
    stall = component(group,"Trading stall frame",(-.27,.23,0),frame,root)
    cloth = component(group,"Canvas awning",(-.27,.23,.95),awning,root)
    cargo = component(group,"Trade stock",(-.31,.23,.04),stock,root)
    dock_hoist = component(group,"Dock unloading crane",(-.70,-.48,0),hoist,root)
    for obj,a,b,d in ((pier,.75,1.39,.20),(stall,1.32,2.25,.14),
                      (cloth,1.80,2.41,.20),(cargo,2.12,2.74,.22),(dock_hoist,2.30,2.96,.15)):
        reveal(obj,a,b,drop=d)
    worker = work_rig(group,root,"Mercador",(.38,.49,0),"merchant")
    component(group,"Brass spyglass",(0,0,0),spyglass,worker[3][0][2])
    reveal(worker[0],2.13,2.58)
    walk(worker,2.42,3.25,(.40,.48,0),(.27,-.41,0),-90)
    ship = component(group,"Merchant caravel",(0,-3.38,SEA+.05),caravel,root)
    reveal(ship,2.5,3.24)
    crate = component(group,"Delivered parcel",(0,-1.34,-.89),parcel,root)
    rope = component(group,"Working crane line",(-.17,-.94,.91),
                     lambda g: tube(g,"Suspended rope",[(0,0,0),(0,0,-1)],
                                    [.010,.010],"linen",sides=6),root)
    animate_market(worker,ship,crate,rope,cloth)
    upgrades(group,root,seat)


def jetty(group):
    for x in (-.27,.27):
        tube(group,"Dock bollard",[(x,-.35,-1.30),(x,-.35,.30)],[.059,.061],"oak",sides=7)
        for z in (.11,.15,.19):
            disc(group,"Mooring rope wrap",(x,-.35,z),.069,.025,"linen",axis="z",sides=9)
    for i in range(6):
        box(group,"Dock apron tread",(0,-.066-i*.085,-.040),(.65,.083,.085),"wood",.012)


def frame(group):
    for x in (-.53,.53):
        for y in (-.33,.33):
            tube(group,"Squared stall upright",[(x,y,.008),(x,y,.94)],[.056,.051],"oak",sides=4)
            braces(group,(x,y,.48),(x*.68,y,.79),key="wood",width=.036)
    for y in (-.31,.31):
        box(group,"Counter bearer",(0,y,.49),(1.16,.103,.095),"oak",.015)
    box(group,"Broad merchant counter",(0,-.22,.56),(1.12,.33,.092),"wood-end",.028)
    box(group,"Rear display shelf",(0,.253,.44),(1.04,.195,.055),"oak",.016)
    for y in (-.34,.34):
        braces(group,(-.60,y,.89),(.60,y,.89),key="wood",width=.046)


def awning(group):
    vertices = [(x,y,.12-.33*abs(y)+.018*cos(x*12))
                for x in (-.65,-.43,-.22,0,.22,.43,.65) for y in (-.46,-.23,0,.23,.43)]
    faces = [(i*5+j,(i+1)*5+j,(i+1)*5+j+1,i*5+j+1) for i in range(6) for j in range(4)]
    surface(group,"Hand-sewn striped canopy",vertices,faces,
            ["market-cloth" if i//4%2 else "linen" for i in range(24)],smooth=True)
    points = [(x,-.46,.12-.33*.46+.018*cos(x*12)) for x in (-.65,-.43,-.22,0,.22,.43,.65)]
    hem = points+[(x,y,z-.119+.029*cos(x*18)) for x,y,z in points]
    surface(group,"Scalloped awning edge",hem,[(i,i+1,i+8,i+7) for i in range(6)],
            ["market-cloth" if i%2 else "linen" for i in range(6)])


def stock(group):
    for x,z in ((-.38,.55),(-.21,.55),(.32,.45)):
        tube(group,"Glazed trade crock",[(x,-.15,z),(x,-.15,z+.044),(x,-.15,z+.123),
             (x,-.15,z+.164)],[.045,.072,.057,.039],"tile",sides=9,accent="tile-light")
        disc(group,"Stoppered crock lid",(x,-.15,z+.17),.039,.028,"wood-end",axis="z",sides=8)
    for x,y,z in ((.35,-.02,.114),(.37,.17,.126),(-.39,.17,.11)):
        pebble(group,"Tied linen trade sack",(x,y,z),(.128,.112,.158),"linen",
               sides=8,accent="sand-light",phase=x)
    for i in range(3):
        box(group,"Folded bolts of cloth",(.06,-.17,.55+i*.035),(.26,.175,.035),
            ("tile","market-cloth","linen")[i],.012)


def hoist(group):
    tube(group,"Quayside crane post",[(0,0,0),(0,0,.94)],[.069,.058],"oak",sides=7)
    braces(group,(0,0,.91),(.55,-.48,.93),key="wood",width=.054)
    braces(group,(0,0,.40),(.34,-.30,.90),width=.035)
    disc(group,"Quay pulley",(.53,-.46,.90),.065,.045,"bronze",sides=9)


def spyglass(group):
    tube(group,"Bound leather spyglass body",[(-.02,0,0),(.11,0,0),(.23,0,0)],
         [.027,.035,.041],"oak",sides=9,accent="wood")
    for x,r in ((.02,.029),(.11,.037),(.231,.049)):
        disc(group,"Polished spyglass ring",(x,0,0),r,.024,"bronze",axis="x",sides=10)
    disc(group,"Recessed glass",(.244,0,0),.032,.010,"patina",axis="x",sides=10)


def caravel(group):
    outline = [(-.26,-.56),(-.34,-.30),(-.39,.12),(-.28,.49),(0,.85),
               (.28,.49),(.39,.12),(.34,-.30),(.26,-.56)]
    n = len(outline)
    verts = [(x*r,y*r,z+.07*abs(y)) for r,z in ((.58,-.14),(1,.071),(.97,.183),(.82,.157),(.77,.043))
             for x,y in outline]
    faces = [(j*n+i,j*n+(i+1)%n,(j+1)*n+(i+1)%n,(j+1)*n+i)
             for j in range(4) for i in range(n)]
    surface(group,"Open carvel-planked hull",verts,faces+[(tuple(range(4*n,5*n)))],
            ["oak"]*n+["wood"]*n+["bronze"]*n+["wood-end"]*(n+1))
    for y,z in ((-.40,.178),(.25,.125)):
        box(group,"Cargo deck cross bench",(0,y,z),(.58,.121,.068),"wood-end",.017)
    tube(group,"Tall mast",[(0,-.047,.08),(0,-.047,1.37)],[.040,.027],"oak",sides=8)
    tube(group,"Cross-yard",[(-.39,-.036,1.215),(.39,-.036,1.215)],[.027,.026],"wood",sides=7)
    verts = [(x*(.83+.17*v),.19*(1-(x/.355)**2)*sin(pi*v)-.055,1.215-v*.81+.027*cos(x*9))
             for v in (0,.26,.53,.79,1) for x in (-.355,-.177,0,.177,.355)]
    faces = [(j*5+i,j*5+i+1,(j+1)*5+i+1,(j+1)*5+i) for j in range(4) for i in range(4)]
    surface(group,"Wind-filled sail with stitched hem",verts,faces,
            ["sand-light" if i%4 in (0,3) else "linen" for i in range(16)],smooth=True)
    for x,y,z in ((-.35,-.055,1.21),(.35,-.055,1.21),(0,-.052,1.35)):
        tube(group,"Rigging line",[(x,y,z),(x*.50,.47 if x==0 else -.38,.21)],
             [.008,.008],"wood-dark",sides=5)


def parcel(group):
    box(group,"Oak cargo crate",(0,0,0),(.26,.228,.236),"wood",.020)
    for x in (-.09,.09):
        box(group,"Crate strap",(x,0,0),(.029,.246,.25),"oak",.006)
    box(group,"Merchant seal",(0,-.122,.031),(.063,.014,.062),"bronze",.009)


def animate_market(rig,ship,crate,rope,cloth):
    root,chest,head,arms,legs = rig
    hand,elbow,wrist = arms[0]
    scale(crate,0,.0001)
    scale(rope,0,.0001)
    for i in range(83):
        t = 3.27+i*.31
        if t>END:
            break
        delivery = next((s for s in DELIVERIES if s-.22<=t<=s+DELIVERY+.30),None)
        readiness = max(0,.5+.5*sin(t*1.6))
        active = 1 if delivery is not None else readiness*.35
        slide(root,t,(.27,-.41-active*.11,.011*abs(sin(t*4))))
        pose(root,t,(0,0,-90+7*sin(t*1.3)))
        pose(chest,t,(0,-8*active,3*sin(t*2.6)))
        pose(head,t,(0,-5-9*active,9*sin(t*1.5)))
        pose(hand,t,(0,-17-40*active,0))
        pose(elbow,t,(0,-32-36*active,0))
        pose(wrist,t,(0,24+109*active,0))
        pose(arms[1][0],t,(0,-12*active,0))
        pose(cloth,t,(1.7*sin(t*2.4),1.2*cos(t*2.1),0))
        if t>=27.4:
            pose(hand,t,(0,-8,0))
            pose(elbow,t,(0,-12,0))
            pose(wrist,t,(0,0,0))
    for start in DELIVERIES:
        sail_and_unload(ship,crate,rope,start)


def sail_and_unload(ship,crate,rope,start):
    course = ((0,-.47,-3.34,-23),(.30,-.13,-2.81,-12),(.82,0,-1.57,0),
              (1.18,0,-1.53,0),(1.72,0,-1.58,5),(2.05,.20,-1.90,52),
              (2.52,.54,-2.63,144),(DELIVERY,.48,-3.36,179))
    for time,x,y,yaw in course:
        slide(ship,start+time,(x,y,SEA+.055+.034*sin(time*6.8)))
        pose(ship,start+time,(3.2*sin(time*8),3.5*cos(time*7),yaw))
    scale(crate,start+.68,.0001)
    scale(crate,start+.78,1)
    scale(rope,start+.72,.0001)
    for time,at in ((.79,(0,-1.42,-.96)),(1.0,(.06,-1.21,-.65)),
                    (1.27,(.11,-1.02,.24)),(1.65,(.16,-.83,.36)),(1.91,(.17,-.81,.133))):
        slide(crate,start+time,at)
        hook = Vector(at)+Vector((0,0,.12))-rope.location
        turn = Vector((0,0,-1)).rotation_difference(hook).to_euler()
        key(rope,"rotation_euler",start+time,turn)
        key(rope,"scale",start+time,(1,1,hook.length))
    pose(crate,start+.79,(0,0,14))
    pose(crate,start+1.91,(0,0,-14))
    scale(crate,start+2.36,1)
    scale(crate,start+2.68,.0001)
    scale(rope,start+1.99,.0001)


def upgrades(group,root,seat):
    t2,t3 = UPGRADES
    goods = component(group,"Level 2 cargo stacks",(-.67,-.33,.03),goods_stack,root)
    pennant = component(group,"Level 2 quay pennants",(.56,-.47,.59),
                        lambda g: banners(g,seat),root)
    reveal(goods,t2,t2+.67,drop=.19)
    reveal(pennant,t2+.22,t2+.79,drop=.14)
    sweep(group,root,t2,(1.09,.75))
    tent = component(group,"Level 3 market pavilion",(.59,.38,0),
                     lambda g:pavilion(g,seat),root)
    chest = component(group,"Level 3 treasure chest",(.65,-.24,.02),treasure,root)
    lid = component(group,"Hinged treasure lid",(0,.119,.26),treasure_lid,chest)
    reveal(tent,t3,t3+.70,drop=.20)
    reveal(chest,t3+.32,t3+.92,drop=.21)
    sweep(group,root,t3,(1.09,.75))
    for i in range(56):
        t=t2+.80+i*.31
        pose(pennant,t,(4*sin(t*3.7),2*cos(t*3.1),7*sin(t*2.2)))
    last = DELIVERIES[-1]
    for a,angle in ((0,0),(last+.71,0),(last+1.25,-72),(last+2.1,-61),(last+2.76,0)):
        pose(lid,a,(angle,0,0))
    burst(group,root,(.65,-.24,.33),(last+1.19,),coins=True)


def goods_stack(group):
    for x,y,z,size in ((-.05,0,.095,.96),(.17,.02,.09,.75),(-.095,.026,.28,.77)):
        box(group,"Stacked trade crate",(x,y,z),(.26*size,.22*size,.22*size),"wood",.025)
        for sign in (-1,1):
            box(group,"Trade crate slat",(x+sign*.073*size,y-.12*size,z),
                (.036*size,.022,.224*size),"oak",.009)
    for z in (.14,.21,.26):
        box(group,"Tied fabric bundle",(-.18,.14,z),(.23,.163,.058),"market-cloth",.016)


def banners(group,seat):
    tube(group,"Quay pennant mast",[(0,0,-.44),(0,0,.32)],[.028,.023],"oak",sides=7)
    banner(group,seat,-.08,-.021,.285,.26,.28)


def pavilion(group,seat):
    for x in (-.30,.30):
        for y in (-.26,.26):
            tube(group,"Pavilion corner pole",[(x,y,0),(x,y,.83)],[.035,.030],"oak",sides=7)
    vertices = [(-.35,-.32,.81),(.35,-.32,.81),(.35,.32,.81),(-.35,.32,.81),
                (-.04,0,1.12),(.20,0,1.12)]
    surface(group,"Gabled market pavilion roof",vertices,[(0,1,5,4),(2,3,4,5),(3,0,4),(1,2,5)],
            [seat,seat,"linen","trim"])
    banner(group,seat,0,-.337,.795,.30,.32)


def treasure(group):
    box(group,"Oak treasure chest",(0,0,.131),(.38,.263,.262),"oak",.032)
    for x in (-.11,.11):
        box(group,"Bound treasure strap",(x,-.138,.132),(.043,.025,.242),"bronze",.010)
    box(group,"Treasure lockplate",(0,-.155,.185),(.085,.025,.075),"gold",.015)


def treasure_lid(group):
    box(group,"Domed lid", (0,-.118,.034),(.40,.276,.076),"wood-end",.034)
    for x in (-.11,.11):
        box(group,"Lid iron band",(x,-.118,.073),(.044,.247,.023),"bronze",.009)
