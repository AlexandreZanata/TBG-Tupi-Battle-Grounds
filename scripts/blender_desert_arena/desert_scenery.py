"""Sculpt palms, open pottery, low dunes and separate sandstone islets."""

from math import cos, pi, sin, tau
from random import Random

from mesh import box, pebble, surface, tube
from terrain import rows


def palm(group,young=False):
    height = 1.34 if young else 1.91
    trunk = [(.12*t*t,.027*sin(t*pi),height*t) for t in (0,.24,.50,.76,1)]
    tube(group,"Bent faceted palm trunk",trunk,[.105,.095,.075,.056,.043],
         "dr-trunk",sides=7,accent="dr-grain")
    for i in range(7 if young else 10):
        t = (i+.5)/(7 if young else 10)
        x,y,z = .12*t*t,.027*sin(t*pi),height*t
        tube(group,"Raised dry band on the palm trunk",[(x,y,z),(x+.004,y,z+.034)],
             [.108-.065*t,.102-.065*t],"dr-trunk",sides=7,accent="dr-grain")
    count = 7 if young else 9
    for i in range(count):
        a = i*tau/count+.17
        length = (.70 if young else 1.04)*(.88+.12*sin(i*1.93))
        frond(group,(.12,0,height),a,length,i)
    for x,y,z in ((.064,-.052,height-.085),(.16,-.02,height-.101),(.11,.061,height-.07)):
        pebble(group,"Separate cluster of palm fruit",(x,y,z),(.067,.062,.079),
               "dr-fruit",sides=6)


def frond(group,at,angle,length,variant):
    x,y,z = at
    dx,dy = cos(angle),sin(angle)
    verts = []
    for t,width,h in ((0,.025,0),(.25,.135,.11),(.55,.167,.08),(.80,.108,-.075),(1,.004,-.251)):
        for side in (-1,0,1):
            verts.append((x+length*t*dx-side*width*dy,y+length*t*dy+side*width*dx,
                          z+length*h+(.025 if side==0 else 0)))
    faces = [(i*3+j,(i+1)*3+j,(i+1)*3+j+1,i*3+j+1) for i in range(4) for j in range(2)]
    keys = ["dr-palm-light" if (i+variant)%3==0 else "dr-palm" for i in range(len(faces))]
    surface(group,"Individually folded green palm frond",verts,faces,keys)
    tube(group,"Thin palm frond midrib",[(x,y,z+.027),(x+length*.55*dx,y+length*.55*dy,z+length*.08+.028),
         (x+length*dx,y+length*dy,z-length*.251+.025)],
         [.014,.010,.002],"dr-palm-dark",sides=5)
    for t in (.30,.48,.66):
        center = (x+length*t*dx,y+length*t*dy,z+length*(.16-.23*t))
        for side in (-1,1):
            u,v,w = center
            tip = (u+length*.13*dx-side*.14*dy,v+length*.13*dy+side*.14*dx,w-.041)
            surface(group,"Small tapered palm leaflet",[(u,v,w),tip,
                    (u+length*.09*dx,v+length*.09*dy,w+.012)],[(0,1,2)],
                    ["dr-palm" if variant%2 else "dr-palm-light"])


def pot(group):
    profile = ((.113,.023),(.176,.10),(.220,.244),(.204,.344),(.137,.436),(.141,.469),
               (.163,.488),(.164,.513),(.117,.513),(.112,.465),(.150,.265),(.071,.079))
    rings = [[(r*cos(i*tau/12),r*sin(i*tau/12),z) for i in range(12)] for r,z in profile]
    rows(group,"Open terracotta jar with an inner wall",rings,
         lambda band,i:"dr-clay-dark" if band>=8 else "dr-clay-light" if band in (0,5,6,7) else "dr-clay",
         floor=True)
    for z,r in ((.224,.222),(.292,.215)):
        verts = [(r*cos(i*tau/12),r*sin(i*tau/12),z+dz) for dz in (0,.023) for i in range(12)]
        surface(group,"Turquoise band painted around the jar",verts,
                [(i,(i+1)%12,(i+1)%12+12,i+12) for i in range(12)],["dr-turquoise"]*12)


def pottery_group(group):
    pot(group)
    for obj in tuple(group.objects):
        obj.location.x = -.19
    pebble(group,"Stoppered small sandstone flask",(.256,-.087,.147),(.12,.119,.213),
           "dr-stone-light",sides=8,accent="dr-stone")
    tube(group,"Bronze stopper on the small flask",[(.256,-.087,.255),(.256,-.087,.310)],
         [.054,.047],"dr-bronze",sides=7)
    box(group,"Folded woven cloth under the pottery",(.025,.041,.019),(.79,.61,.038),"dr-canvas",.012)
    for x in (-.27,-.14,.12,.25):
        box(group,"Turquoise stripe in the woven cloth",(x,-.102,.042),(.026,.48,.007),"dr-turquoise-dark")


def sandstone_outcrop(group):
    for i,(x,y,z,sx,sy,sz) in enumerate(((-.19,.04,.15,.38,.30,.41),(.23,-.074,.068,.24,.23,.22),
                                       (-.39,.24,.045,.18,.15,.17))):
        pebble(group,"Weathered sandstone fragment",(x,y,z),(sx,sy,sz),
               "dr-stone" if i%2 else "dr-stone-dark",sides=7,accent="dr-stone-light",phase=i*.7)
    tube(group,"Thin pale seam in the sandstone",[(-.431,-.14,.135),(-.133,-.223,.161),
         (.062,-.171,.122)],[.010,.013,.006],"dr-ivory",sides=5)


def dune(group):
    rings = [[(rx*cos(i*tau/13),ry*sin(i*tau/13),z+.020*sin(i*1.39))
              for i in range(13)] for rx,ry,z in ((.75,.40,-.022),(.55,.27,.086),(.29,.12,.178))]
    rows(group,"Low sculpted sand drift",rings,
         lambda band,i:"dr-sand-light" if (i+band)%4 else "dr-sand",floor=True)


def reeds(group):
    rng = Random(861)
    vertices,faces,keys = [],[],[]
    for i in range(15):
        x,y = rng.uniform(-.21,.21),rng.uniform(-.15,.15)
        h = rng.uniform(.17,.34)
        a = len(vertices)
        vertices.extend(((x-.014,y,0),(x+.014,y,0),(x+.07,y+.014,h)))
        faces.append((a,a+1,a+2))
        keys.append("dr-reed" if i%3 else "dr-palm-light")
    surface(group,"Sparse oasis grass blades",vertices,faces,keys)


def temple_islet(group):
    rings = []
    for rx,ry,z in ((3.91,2.19,-.44),(4.02,2.28,-.07),(3.78,2.00,.244),(3.40,1.74,.482)):
        rings.append([(rx*cos(a)*(1+.070*sin(i*1.83)),ry*sin(a)*(1+.075*cos(i*1.31)),
                       z+.041*sin(i*1.69)) for i in range(24) for a in (i*tau/24,)])
    keys = ("dr-stone-dark","dr-stone","dr-stone-light")
    rows(group,"Layered sandstone below the temple",rings,
         lambda band,i:keys[band] if (i+band)%7 else "dr-stratum",floor=True)


def coastal_skerry(group):
    for i,(x,y,z,sx,sy,sz) in enumerate(((-.32,.01,.072,.37,.35,.31),(.10,.09,.254,.50,.39,.59),
                                       (.50,-.035,.095,.32,.30,.36))):
        pebble(group,"Sandstone skerry in the blue sea",(x,y,z),(sx,sy,sz),
               "dr-stone" if i%2 else "dr-stone-dark",sides=6,accent="dr-stone-light",phase=i*.49)


def ripple(group):
    for i,(x,y,length) in enumerate(((-.45,-.04,.80),(.13,.13,.71),(-.33,.32,.97))):
        points = [(x,y,.013),(x+length*.43,y+.032,.017),(x+length,y+.061,.013),
                  (x+length*.79,y+.084,.017),(x+length*.33,y+.058,.020)]
        surface(group,"Long thin highlight on the desert sea",points,[(0,1,4),(1,2,3,4)],
                ["dr-ripple" if i%2==0 else "dr-shallows","dr-shallows"])


def oasis_lamp(group):
    box(group,"Sandstone lamp foot",(0,0,.060),(.35,.35,.132),"dr-stone-dark",.024)
    tube(group,"Turquoise lamp pedestal",[(0,0,.124),(0,0,.371),(0,0,.431)],
         [.083,.057,.109],"dr-turquoise-dark",sides=8,accent="dr-turquoise")
    tube(group,"Amber glow inside the oasis lantern",[(0,0,.421),(0,0,.582)],
         [.082,.074],"dr-amber",sides=8)
    for x,y in ((-.086,0),(.086,0),(0,-.086),(0,.086)):
        tube(group,"Gold lantern cage rib",[(x,y,.413),(x*.91,y*.91,.598)],
             [.015,.014],"dr-gold-dark",sides=6)
    tube(group,"Turquoise lantern cover",[(0,0,.576),(0,0,.620),(0,0,.704)],
         [.128,.128,.016],"dr-turquoise",sides=8,accent="dr-turquoise-light")


def balustrade(group):
    box(group,"Sandstone balustrade foundation",(0,0,.053),(1.105,.248,.121),"dr-stone-dark",.021)
    for x in (-.453,0,.453):
        tube(group,"Carved low sandstone garden post",[(x,0,.106),(x,0,.371)],
             [.068,.057],"dr-stone",sides=6,accent="dr-stone-light")
    box(group,"Turquoise coping across the balustrade",(0,0,.405),(1.114,.193,.079),"dr-turquoise",.015)
    for x in (-.449,.449):
        box(group,"Gold inset at each end of the rail",(x,0,.449),(.166,.192,.024),"dr-gold",.008)
