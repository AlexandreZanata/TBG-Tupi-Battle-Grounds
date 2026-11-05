"""Model the Artillery gun deck, heavy cannon and rapid loading equipment."""

from math import cos, sin, tau

from hold_parts import banner, bore, braces, disc, footing, gear, layer, mechanism, pier, plaque, profile_x
from mesh import box, pebble, surface, tube

CANNON = {"Base": (.12,1.0), "A1": (.17,1.15), "A2": (.24,1.4), "B1": (.12,1.0), "B2": (.12,1.0)}


def artillery(root,form,seat):
    deck(layer(root,"Timber gun deck"),seat)
    r,length = CANNON[form]
    muzzle = length*.60 + r*.50
    cannon = layer(root,"Aiming and recoil assembly")
    mechanism(cannon,"Swivel carriage",(0,0,0),
              lambda g: carriage(g,r,form=="A2"),motion="Yaw about deck center")
    mechanism(cannon,"Cannon barrel",(.05,0,1.05+r),
              lambda g: barrel(g,r,length,muzzle,form),pitch=.12,
              muzzle=(muzzle,0,0),motion="Aim and recoil")
    for y in ((-.47,.47) if form in ("B1","B2") else (-.47,)):
        mechanism(cannon,"Breech drive gear",(-.32,y,1.04),
                  lambda g: gear(g,.174,9),motion="Notch on release")
    upgrade = layer(root,"Fortification attachments")
    if form in ("A1","A2"):
        heavy(upgrade,form=="A2")
    if form in ("B1","B2"):
        loader(upgrade,form=="B2")


def deck(group,seat):
    footing(group,1.88,1.72)
    for x in (-.68,.68):
        for y in (-.55,.55):
            pier(group,x,y,.15,.26,.57,rows=3)
    for y in (-.56,.56):
        for sign in (-1,1):
            braces(group,(sign*.68,y,.26),(-sign*.68,y,.69),width=.062)
    for x in (-.68,.68):
        braces(group,(x,-.56,.29),(x,.56,.69),width=.052)
    box(group,"Gun deck substructure",(0,0,.747),(1.81,1.56,.183),"oak",.045)
    for i in range(8):
        box(group,"Gun deck plank",(0,-.675+i*.193,.842),(1.76,.188,.085),
            "wood" if i%3 else "wood-end",.015)
    banner(group,seat,.43,-.784,.793,.45,.37)
    for x in (-.61,-.33):
        y = -.49 if x<-.5 else .55
        tube(group,"Powder barrel",[(x,y,.88),(x,y,.94),(x,y,1.16),(x,y,1.21)],
             [.116,.137,.132,.108],"wood",sides=10,accent="oak")
        for z in (.943,1.143):
            disc(group,"Powder barrel hoop",(x,y,z),.134,.039,"iron",axis="z",sides=10)
    for x,y in ((-.72,.37),(-.63,.22),(-.52,.36)):
        pebble(group,"Cast cannon shot",(x,y,.921),(.063,.063,.060),"gunmetal",sides=7)


def carriage(group,r,grand):
    axle_z = 1.05+r
    for sign in (-1,1):
        y = sign*(r+.060)
        plaque(group,"Shaped oak cheek",[(-.58,.86),(.39,.86),(.30,1.02),
               (.13,axle_z+.014),(-.17,axle_z+.014),(-.40,.96)],y,.111,"oak",.025)
        wheel_y = y+sign*.099
        for x,z,rad in ((.19,.998,.148),(-.34,.966,.113)):
            disc(group,"Spoked carriage wheel", (x,wheel_y,z),rad,.066,"wood",sides=12,edge="oak")
            disc(group,"Forged hub", (x,wheel_y+sign*.04,z),rad*.36,.065,"gunmetal",sides=9)
            for i in range(5):
                a = i*tau/5
                braces(group,(x,wheel_y+sign*.040,z),
                       (x+rad*.84*cos(a),wheel_y+sign*.040,z+rad*.84*sin(a)),
                       key="wood-end",width=.014)
    tube(group,"Cannon trunnion axle",[(.05,-r-.138,axle_z),(.05,r+.138,axle_z)],
         [.050,.050],"gunmetal",sides=10)
    if grand:
        box(group,"Recoil-slide bed",(-.16,0,.898),(1.31,.78,.112),"gunmetal",.022)


def barrel(group,r,length,muzzle,form):
    back = -length*.40-r*.36
    bore(group,muzzle,r,back,key="gunmetal",edge="bronze" if form!="A2" else "silver")
    for i in range(3 if form=="A2" else 2):
        x = -length*.23 + i*length*.22
        profile_x(group,"Shrink-fit barrel band",
                  [(x-.031,r*.93),(x-.022,r*1.02),(x+.022,r*1.02),(x+.031,r*.93)],
                  ["bronze","silver" if form=="A2" else "bronze","bronze"],sides=16)
    disc(group,"Cascabel knob",(back-.013,0,0),r*.41,.057,"gunmetal",axis="x",sides=12)


def heavy(group,grand):
    for x,y in ((.49,-.54),(.54,.51),(.33,-.61)):
        pebble(group,"Weathered sandbag",(x,y,.91),(.165,.098,.074),"linen",sides=8)
    if grand:
        for y in (-.61,.61):
            box(group,"Iron recoil runner",(-.15,y,.917),(1.47,.083,.088),"silver",.015)
            braces(group,(.54,y,.38),(-.45,y,.73),key="gunmetal",width=.074)
        for x in (-.63,.64):
            box(group,"Recoil stop block",(x,-.61,.965),(.145,.165,.147),"gunmetal",.024)


def loader(group,advanced):
    x,y = -.43,.27
    box(group,"Loader guide frame",(x,y,1.13),(.36,.37,.23),"gunmetal",.025)
    rings = ((1.19,.14),(1.40,.23),(1.454,.24),(1.454,.18),(1.31,.115))
    pts = [(x+r*cos(tau*i/10),y+r*sin(tau*i/10),z) for z,r in rings for i in range(10)]
    faces = [(r*10+i,r*10+(i+1)%10,(r+1)*10+(i+1)%10,(r+1)*10+i)
             for r in range(len(rings)-1) for i in range(10)]
    surface(group,"Open cast loading hopper",pts,faces,["bronze"]*20+["silver"]*10+["soot"]*10)
    for x,y in ((-.48,.28),(-.36,.31),(-.43,.19)):
        pebble(group,"Shot in the hopper",(x,y,1.363),(.057,.057,.058),"gunmetal",sides=7)
    if advanced:
        for x in (-.66,-.49,-.32):
            box(group,"Shot rack iron saddle",(x,-.566,1.065),(.13,.18,.275),"patina",.026)
            pebble(group,"Ready cartridge",(x,-.583,1.228),(.059,.071,.071),"bronze",sides=7)
        mechanism(group,"Indexed flywheel",(-.68,-.24,1.32),
                  lambda g: gear(g,.254,13),motion="Half-tooth advance")
