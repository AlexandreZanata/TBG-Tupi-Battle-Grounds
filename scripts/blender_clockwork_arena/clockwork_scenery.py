"""Sculpt foundry foundations, sparse heath and offshore supply debris."""

from math import cos, sin, tau
from random import Random

from mesh import box, pebble, surface, tube
from terrain import rows


def foundry_islet(group):
    rings = []
    for rx,ry,z in ((3.75,2.08,-.43),(3.89,2.18,-.06),(3.56,1.93,.26),(3.25,1.67,.50)):
        rings.append([(rx*cos(a)*(1+.074*sin(i*1.82)),ry*sin(a)*(1+.088*cos(i*1.29)),
                       z+.038*sin(i*1.67)) for i in range(24) for a in (i*tau/24,)])
    keys = ("cw-rock-dark","cw-rock","cw-stratum")
    rows(group,"Layered foundry island rock",rings,
         lambda band,i:keys[band] if (i+band)%7 else "cw-rock-light",floor=True)
    for x in (-2.68,2.68):
        box(group,"Iron retaining buttress below the foundry",(x,-1.33,.227),
            (.262,.35,.663),"cw-iron",.035)
        box(group,"Brass cap on the retaining buttress",(x,-1.33,.581),
            (.291,.385,.063),"cw-brass-dark",.013)


def crane_islet(group):
    outline = [(-.96,-.41),(-.59,-.77),(.63,-.66),(1.03,-.21),(.89,.57),(-.54,.69)]
    rings = [[(x*s,y*s,z+.043*sin(i*1.87)) for i,(x,y) in enumerate(outline)]
             for s,z in ((1,-.27),(1.10,.01),(.93,.42),(.75,.61))]
    keys = ("cw-rock-dark","cw-rock","cw-rock-light")
    rows(group,"Independent slate crane island",rings,lambda band,i:keys[band],floor=True)
    box(group,"Bolted upper crane island plate",(0,0,.645),(1.35,1.05,.093),"cw-steel",.032)


def rubble(group):
    for i,(x,y,z,sx,sy,sz) in enumerate(((-.19,.04,.09,.32,.27,.34),(.21,-.06,.06,.23,.19,.24),
                                        (-.39,.21,.04,.14,.13,.16))):
        pebble(group,"Broken slate at the foundry perimeter",(x,y,z),(sx,sy,sz),
               "cw-rock" if i%2 else "cw-rock-dark",sides=7,accent="cw-rock-light",phase=i*.69)
    box(group,"Rusty strap discarded across the rubble",(-.11,-.139,.17),(.45,.043,.030),
        "cw-rust",.004,angle=.28)


def grass(group):
    rng = Random(741)
    verts,faces,keys = [],[],[]
    for i in range(17):
        x,y = rng.uniform(-.22,.22),rng.uniform(-.15,.15)
        h,w = rng.uniform(.13,.30),rng.uniform(.009,.021)
        lean = rng.uniform(-.04,.12)
        a = len(verts)
        verts.extend(((x-w,y,0),(x+w,y,0),(x+lean,y+.019,h*.70),(x+lean+.033,y+.023,h)))
        faces.extend(((a,a+1,a+2),(a+1,a+3,a+2)))
        keys.extend(("cw-grass-dark","cw-grass-light" if i%3==0 else "cw-grass"))
    surface(group,"Sparse grass around foundry footings",verts,faces,keys)


def dead_tree(group):
    tube(group,"Dry foundry-perimeter trunk",[(0,0,-.027),(.043,.025,.53),(.018,.015,1.12),
         (.164,.055,1.47)],[.101,.078,.046,.014],"cw-oak",sides=7,accent="cw-grain")
    for points,radii in (
        (((.035,.017,.61),(-.238,.055,.89),(-.303,.113,1.16)),(.049,.035,.009)),
        (((.033,.022,.83),(.322,.093,1.05),(.472,.087,1.09)),(.038,.024,.007)),
        (((.01,.01,1.07),(-.101,-.073,1.29)),(.031,.007)),
    ):
        tube(group,"Separate dry branch above the slate",points,radii,"cw-oak",sides=6,accent="cw-grain")


def supply_raft(group):
    for x in (-.38,.38):
        tube(group,"Waterlogged floating timber",[(x,-.52,.012),(x,.51,.012)],
             [.091,.082],"cw-oak",sides=7,accent="cw-grain")
    for i in range(5):
        box(group,"Loose supply raft deck board",(0,(i-2)*.155,.109),
            (.94,.15,.071),"cw-wood" if i%2 else "cw-grain",.012,angle=.029*i)
    box(group,"Iron strap over the floating boards",(0,-.18,.157),(.90,.041,.027),"cw-iron",.006)


def ripple(group):
    for i,(x,y,length) in enumerate(((-.47,-.03,.80),(.13,.14,.67),(-.32,.35,.98))):
        points = [(x,y,.013),(x+length*.42,y+.032,.017),(x+length,y+.060,.013),
                  (x+length*.80,y+.087,.017),(x+length*.33,y+.058,.019)]
        surface(group,"Thin broken highlight on the foundry sea",points,[(0,1,4),(1,2,3,4)],
                ["cw-ripple" if i%2==0 else "cw-shallows","cw-shallows"])
