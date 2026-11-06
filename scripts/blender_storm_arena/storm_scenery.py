"""Sculpt wind-swept heath, slate stacks, surf and low storm banks."""

from math import cos, sin, tau
from random import Random

from mesh import box, pebble, surface, tube
from terrain import rows


def wind_pine(group):
    tube(group,"Wind-bent pine trunk",[(0,0,-.017),(.105,0,.52),(.31,.025,1.09),(.54,.035,1.85)],
         [.105,.091,.059,.022],"storm-oak",sides=7)
    for i in range(4):
        z,x = .32+i*.29,.115+i*.119
        r = .62-i*.108
        tube(group,"Swept faceted branch tier",[(x-.145,0,z),(x,0,z+.134),(x+.43,0,z+.63)],
             [r*.36,r,.028],"storm-pine",sides=7,accent="storm-pine-light")
    tube(group,"Trailing low pine bough",[(.10,-.027,.64),(.47,-.079,.61),(.77,-.087,.83)],
         [.09,.275,.020],"storm-pine-dark",sides=7,accent="storm-pine")


def silver_grass(group):
    rng = Random(191)
    verts,faces,keys = [],[],[]
    for i in range(21):
        x,y = rng.uniform(-.27,.23),rng.uniform(-.19,.19)
        height = rng.uniform(.16,.39)
        lean,w = rng.uniform(.088,.228),rng.uniform(.010,.024)
        a = len(verts)
        verts.extend(((x-w,y,0),(x+w,y,0),(x+.42*lean+w*.6,y+.028,height*.60),
                      (x+.42*lean-w*.6,y+.020,height*.60),(x+lean,y+.029,height)))
        faces.extend(((a,a+1,a+2,a+3),(a+3,a+2,a+4)))
        keys.extend(("storm-grass-dark","storm-grass-light" if i%4==0 else "storm-grass"))
    surface(group,"Twenty-one swept silver grass blades",verts,faces,keys)


def slate_boulder(group):
    for i,(x,y,z,sx,sy,sz) in enumerate(((-.17,.04,.11,.37,.29,.38),(.23,-.10,.04,.24,.19,.21),
                                        (-.38,.23,.035,.17,.14,.15))):
        pebble(group,"Chipped slate erratic",(x,y,z),(sx,sy,sz),
               "storm-slate" if i%2 else "storm-slate-deep",sides=7,accent="storm-slate-light",phase=i*.8)
    for s in (-1,1):
        tube(group,"Exposed shallow quartz band",[(-.458,-.105,.162),(-.175,-.193,.210),
             (.060,-.157,.192+s*.039)],[.008,.012,.006],"storm-shale",sides=5)


def reef(group):
    for i,(x,y,z,sx,sy,sz) in enumerate(((-.41,0,.08,.38,.36,.43),(.07,.12,.32,.50,.42,.69),
                                        (.47,.18,.21,.36,.35,.51),(.63,-.03,-.028,.27,.31,.24))):
        pebble(group,"Jagged slate skerry",(x,y,z),(sx,sy,sz),
               "storm-slate" if i%2 else "storm-slate-deep",sides=6,accent="storm-slate-light",phase=i*.51)
    surface(group,"Small patch of foam between the stacks",
            [(-.58,-.23,.036),(-.19,-.49,.037),(.37,-.38,.038),(.13,-.36,.038),(-.22,-.39,.038)],
            [(0,1,2,3,4)],["storm-foam"])


def relay_islet(group):
    rings = []
    for sx,sy,z in ((3.26,1.72,-.39),(3.51,1.83,-.11),(3.27,1.65,.14),(2.98,1.49,.31)):
        rings.append([(sx*cos(i*tau/19)*(1+.11*sin(i*2.01)),
                       sy*sin(i*tau/19)*(1+.084*cos(i*1.39)),z+.052*sin(i*1.70))
                      for i in range(19)])
    keys = ("storm-slate-deep","storm-slate","storm-slate-edge")
    rows(group,"Shorn slate of the relay islet",rings,
         lambda band,i:keys[band] if i%4 else "storm-slate-light",floor=True)
    for x,y,z in ((-2.64,-.61,.11),(2.68,-.40,.15),(2.21,.73,.25),(-1.99,.76,.18)):
        pebble(group,"Loosened relay islet fragment",(x,y,z),(.45,.34,.38),
               "storm-slate",sides=6,accent="storm-slate-light",phase=x)


def whitecap(group):
    for i,(x,y,length) in enumerate(((-.63,-.05,.84),(.14,.14,.71),(-.34,.40,1.13),(.47,-.26,.63))):
        outline = [(x,y,.013),(x+length*.42,y+.035,.017),(x+length,y+.08,.012),
                   (x+length*.86,y+.116,.017),(x+length*.39,y+.101,.025),
                   (x+length*.085,y+.027,.015)]
        surface(group,"Low diagonal storm whitecap",outline,[(0,1,4,5),(1,2,3,4)],
                ["storm-foam" if i%2==0 else "storm-swell","storm-swell"])


def cloud_bank(group):
    lobes = ((-2.28,.09,.03,2.38,.81,.67),(-.88,.42,.24,2.55,1.10,.97),
             (.68,.33,.15,2.65,.91,.76),(2.59,.06,.04,1.90,.71,.45))
    for i,(x,y,z,sx,sy,sz) in enumerate(lobes):
        cloud = pebble(group,"Overlapping elongated polygonal cloud bank",(x,y,z),(sx,sy,sz),
                       "storm-cloud" if i%2 else "storm-cloud-dark",sides=9,
                       accent="storm-cloud-edge",phase=i*.92)
        cloud.visible_shadow = False
        cloud.visible_diffuse = False
        cloud.visible_glossy = False


def wreck(group):
    for x,y in ((-.40,-.17),(.46,-.13),(.57,.37)):
        tube(group,"Split remnant of a pier pile",[(x,y,-.39),(x,y,.16),(x+.064,y+.023,.27)],
             [.061,.063,.046],"storm-oak",sides=6,accent="storm-grain")
    for i in range(4):
        x = -.44+i*.172
        box(group,"Narrow waterlogged dock plank",(x,.06+i*.037,.11+i*.028),
            (.161,.56-i*.071,.062),"storm-wood" if i%2 else "storm-grain",.009,angle=-.20+i*.131)
    tube(group,"Loose bronze dock strap",[(-.51,-.22,.25),(-.22,-.12,.24),(.12,.02,.20)],
         [.023,.019,.017],"storm-copper",sides=5)
