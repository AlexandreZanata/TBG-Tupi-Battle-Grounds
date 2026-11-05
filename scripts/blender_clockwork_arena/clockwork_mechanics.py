"""Model toothed gears, furnace fittings and the foundry's coastal machinery."""

from math import cos, sin, tau

from mesh import box, pebble, surface, tube

GEAR_MODULE = .08


def gear(group,teeth):
    radius = GEAR_MODULE*teeth*.5
    count = teeth*4
    profile = [(tau*(tooth+t)/teeth,r) for tooth in range(teeth)
               for t,r in ((0,radius-.10),(.24,radius+.08),(.56,radius+.08),(.80,radius-.10))]
    inner = radius*.72
    vertices = [(r*cos(a),y,r*sin(a)) for y in (-.093,.093)
                for ring in (0,1) for a,outer in profile for r in (outer if ring else inner,)]
    faces,keys = [],[]
    for i in range(count):
        j = (i+1)%count
        faces.extend(((i,count+i,count+j,j),
                      (2*count+j,3*count+j,3*count+i,2*count+i),
                      (count+i,3*count+i,3*count+j,count+j),
                      (i,j,2*count+j,2*count+i)))
        keys.extend(("cw-brass","cw-brass-dark","cw-brass-light" if i%4 in (0,1) else "cw-brass",
                     "cw-brass-dark"))
    surface(group,f"{teeth}-tooth extruded brass rim",vertices,faces,keys)
    for i in range(6):
        a = i*tau/6
        tube(group,"Radial iron drive spoke",[(radius*.20*cos(a),0,radius*.20*sin(a)),
             (radius*.755*cos(a),0,radius*.755*sin(a))],
             [radius*.091,radius*.067],"cw-iron",sides=6,accent="cw-steel")
    tube(group,"Separate machined gear hub",[(0,-.148,0),(0,.139,0)],
         [radius*.23,radius*.23],"cw-iron",sides=12,accent="cw-steel")
    tube(group,"Brass hub axle cap",[(0,-.169,0),(0,-.146,0)],
         [radius*.125,radius*.133],"cw-brass-light",sides=10)
    for i in range(4):
        a = (i+.5)*tau/4
        pebble(group,"Steel bolt on the gear hub",(radius*.173*cos(a),-.153,radius*.173*sin(a)),
               (.022,.014,.022),"cw-steel-light",sides=6)


def rivets(group,center,half,step):
    x,y,z = center
    count = max(1,round(half*2/step))
    for i in range(count+1):
        pebble(group,"Individual domed brass rivet",(x-half+i*half*2/count,y,z),
               (.027,.015,.025),"cw-brass",sides=6)


def grille(group,at,width,height):
    x,y,z = at
    box(group,"Recessed furnace opening",(x,y,z),(width,.075,height),"cw-char",.026)
    box(group,"Hot iron behind the furnace bars",(x,y-.046,z),(width*.83,.022,height*.79),"cw-furnace",.014)
    for i in range(6):
        u = x+(i-2.5)*width*.143
        box(group,"Separate dark furnace grille bar",(u,y-.066,z),
            (width*.052,.051,height*.93),"cw-iron-dark",.008)
    for s in (-1,1):
        box(group,"Bolted brass furnace grille frame",(x,y-.076,z+s*height*.51),
            (width*1.10,.058,.052),"cw-brass-dark",.009)
    rivets(group,(x,y-.110,z+height*.51),width*.48,width*.31)


def boiler(group):
    for x in (-.36,.36):
        box(group,"Boiler bed rail",(x,0,.073),(.133,.83,.151),"cw-iron-dark",.019)
    tube(group,"Riveted horizontal boiler shell",[(-.46,0,.37),(-.40,0,.37),(.43,0,.37),(.50,0,.37)],
         [.208,.281,.281,.198],"cw-iron",sides=10,accent="cw-steel")
    for x in (-.34,.29):
        tube(group,"Separate brass boiler strap",[(x-.031,0,.37),(x+.031,0,.37)],
             [.293,.293],"cw-brass-dark",sides=10,accent="cw-brass")
    tube(group,"Copper steam outlet",[(.15,0,.605),(.15,0,.82),(.15,-.16,.891)],
         [.042,.043,.043],"cw-copper",sides=7)
    tube(group,"Small boiler pressure face",[(.461,-.153,.475),(.483,-.183,.475)],
         [.079,.079],"cw-brass",sides=10)
    pebble(group,"Ivory face of the pressure dial",(.489,-.193,.475),(.055,.015,.055),"cw-linen",sides=9)


def pipe_cluster(group):
    for i,(x,y,height) in enumerate(((-.28,0,.81),(0,.12,1.11),(.28,.04,.67))):
        tube(group,"Individual bent copper exhaust",[(x,y,.02),(x,y,height-.12),
             (x+.02,y-.072,height),(x+.073,y-.20,height)],
             [.062,.062,.069,.073],"cw-copper",sides=8,accent="cw-brass-dark")
        tube(group,"Rolled rim on copper exhaust",[(x+.068,y-.177,height),(x+.077,y-.218,height)],
             [.087,.087],"cw-brass",sides=8)
        for z in (.16,height*.65):
            tube(group,"Pipe mounting collar",[(x,y,z),(x,y,z+.042)],
                 [.080,.080],"cw-iron",sides=8)
    box(group,"Shared slate manifold base",(0,.03,.037),(.85,.60,.103),"cw-rock-dark",.027)


def crane(group):
    box(group,"Coastal crane concrete footing",(0,0,.095),(.81,.74,.211),"cw-rock",.045)
    box(group,"Brass turntable below the crane",(0,0,.246),(.55,.52,.135),"cw-brass-dark",.029)
    tube(group,"Triangulated coastal crane mast",[(0,0,.30),(0,0,1.54)],
         [.139,.098],"cw-iron",sides=4,accent="cw-steel")
    for s in (-1,1):
        tube(group,"Crane side brace",[(s*.27,0,.32),(0,0,1.41)],[.037,.037],"cw-brass-dark",sides=6)
    tube(group,"Crane raised boom",[(-.38,0,1.43),(.82,0,1.85),(1.14,0,1.71)],
         [.074,.072,.046],"cw-iron",sides=4,accent="cw-steel")
    tube(group,"Upper boom tension tie",[(-.31,0,1.48),(0,0,1.96),(1.13,0,1.73)],
         [.023,.023,.020],"cw-brass",sides=6)
    tube(group,"Static crane hoist cable",[(1.10,0,1.73),(1.10,0,.58)],[.012,.012],"cw-iron-dark",sides=5)
    tube(group,"Forged crane hook",[(1.10,0,.62),(1.10,0,.44),(1.18,0,.40),(1.22,0,.47)],
         [.023,.026,.026,.014],"cw-brass",sides=6)
    box(group,"Rear iron crane counterweight",(-.37,0,1.27),(.33,.35,.38),"cw-iron-dark",.025)


def crate(group):
    box(group,"Riveted foundry supply chest",(0,0,.203),(.67,.54,.42),"cw-iron",.038)
    for x in (-.22,.22):
        box(group,"Brass band across the supply chest",(x,0,.419),(.058,.532,.034),"cw-brass",.008)
        box(group,"Brass band on the front of the supply chest",(x,-.276,.205),
            (.058,.028,.375),"cw-brass-dark",.007)
    box(group,"Steel chest latch",(0,-.298,.286),(.108,.044,.139),"cw-steel",.013)
    rivets(group,(0,-.281,.052),.274,.15)


def lamp(group):
    box(group,"Cast-iron lamp foot",(0,0,.060),(.35,.35,.131),"cw-iron-dark",.023)
    tube(group,"Faceted brass lamp stem",[(0,0,.119),(0,0,.378)],
         [.069,.050],"cw-brass-dark",sides=7)
    tube(group,"Amber lamp glass",[(0,0,.368),(0,0,.563)],[.090,.079],"cw-amber",sides=8)
    for x,y in ((-.091,0),(.091,0),(0,-.091),(0,.091)):
        tube(group,"Separate lantern cage rod",[(x,y,.355),(x*.92,y*.92,.585)],
             [.015,.015],"cw-iron",sides=6)
    tube(group,"Swept iron lamp hood",[(0,0,.552),(0,0,.608),(.02,0,.684)],
         [.132,.133,.017],"cw-iron",sides=8,accent="cw-steel")


def railing(group):
    for x in (-.49,0,.49):
        box(group,"Riveted rail post base",(x,0,.044),(.178,.18,.098),"cw-rock-dark",.015)
        tube(group,"Thin iron foundry rail post",[(x,0,.085),(x,0,.418)],
             [.035,.031],"cw-iron",sides=6,accent="cw-steel")
    for z in (.215,.371):
        box(group,"Bronze-edged foundry rail",(0,0,z),(1.056,.044,.045),
            "cw-brass-dark" if z>.3 else "cw-iron",.007)
