"""Fit storm shoring, a seamed market roof and a forged prow to the auxiliary rigs."""

from math import cos, pi, sin

from aux_parts import component
from mesh import box, pebble, surface, tube
from offshore_sites import pivot, place as place_sites

STORM_MATERIALS = {
    "cliff":"storm-slate", "cliff-shadow":"storm-slate-deep", "stratum":"storm-slate-light",
    "stone-cool":"storm-slate-edge", "oak":"storm-oak", "wood":"storm-wood", "wood-end":"storm-grain",
    "hood":"storm-pine-dark", "merchant-cloth":"storm-oilcloth", "market-cloth":"storm-oilcloth",
    "linen":"storm-cloth", "ore":"storm-ore", "ore-light":"storm-ore-light",
}


def place(parent):
    place_sites(parent,STORM_MATERIALS,decorate,"Storm Valley")


def decorate(group,is_mine):
    if is_mine:
        component(group,"Forged storm adit",(0,0,0),adit,pivot(group,"Rock outcrop pivot"))
        return
    component(group,"Storm standing-seam canopy",(0,0,0),canopy,pivot(group,"Canvas awning pivot"))
    component(group,"Storm cutter prow",(0,0,0),prow,pivot(group,"Merchant caravel pivot"))


def adit(group):
    for s in (-1,1):
        tube(group,"Bent slate adit reinforcement",[(-.198,s*.350,.026),(-.203,s*.353,.584),
             (-.225,s*.333,.697)], [.059,.060,.065],"storm-iron",sides=6,accent="storm-steel")
        for z in (.178,.551):
            box(group,"Copper adit restraint band",(-.188,s*.353,z),(.121,.136,.064),
                "storm-copper",.011)
    for i in range(7):
        a,b = (i+.036)*pi/7,(i+.964)*pi/7
        arch = [(.343*r*cos(t),.624+.235*r*sin(t)) for r,t in ((1,a),(1.25,a),(1.25,b),(1,b))]
        verts = [(x,y,z) for x in (-.337,-.123) for y,z in arch]
        surface(group,"Separate forged adit arch plate",verts,
                [(3,2,1,0),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(0,4,7,3)],
                ["storm-copper" if i==3 else "storm-steel"]*6)
    for x,y,z in ((-.71,-.65,.160),(-.65,-.61,.314)):
        box(group,"Forged panel beside the adit",(x,y,z),(.43,.167,.135),"storm-iron",.017,angle=.26)
        pebble(group,"Copper rivet on the storm adit panel",(x+.114,y-.082,z),
               (.031,.019,.032),"storm-brass",sides=6)
    tube(group,"Insulated grounding cable across the mine face",[(-.86,-.67,.074),(-.85,-.63,.38),
         (-.92,-.52,.63),(-.88,-.42,.75)], [.022,.021,.025,.019],"storm-copper",sides=7)


def canopy(group):
    roof = ((-.464,-.030),(-.235,.086),(0,.167),(.235,.086),(.442,-.030))
    for i in range(8):
        x = -.601+i*.149
        verts = [(x+dx,y,z) for dx in (0,.143) for y,z in roof]
        faces = [(j,j+1,6+j,5+j) for j in range(4)]
        surface(group,"Separate pitched storm canopy plate",verts,faces,
                ["storm-iron" if i%3==0 else "storm-steel"]*4)
        tube(group,"Raised copper standing seam",[(x,y,z+.007) for y,z in roof],
             [.012]*5,"storm-copper",sides=6)
    for y,z in ((-.468,-.032),(.444,-.032)):
        box(group,"Rolled rain eave",(0,y,z),(1.248,.055,.063),"storm-iron",.012)
        for x in (-.49,0,.49):
            box(group,"Copper eave corner clasp",(x,y-.029,z),(.073,.031,.058),
                "storm-brass",.009)


def prow(group):
    verts = [(-.25,.45,.167),(0,.912,.213),(.25,.45,.167),
             (-.147,.403,-.073),(0,.789,-.078),(.147,.403,-.073),(0,.619,.191)]
    surface(group,"Storm prow riveted cap",verts,
            [(0,3,4,1),(1,4,5,2),(3,5,4),(0,1,6),(1,2,6),(2,0,6)],
            ["storm-iron","storm-iron","storm-char","storm-steel","storm-steel","storm-copper"])
    tube(group,"Storm prow bronze keel stem",[(0,.783,-.073),(0,.881,.078),
         (0,.931,.192),(0,.913,.263)], [.018,.025,.022,.010],"storm-copper",sides=7)
    for s in (-1,1):
        tube(group,"Storm prow external armor seam",[(s*.280,.407,.140),(s*.214,.553,.143),
             (s*.121,.712,.177),(0,.916,.211)], [.017,.016,.013,.010],"storm-brass",sides=6)
        pebble(group,"Storm prow domed rivet",(s*.177,.624,.160),(.027,.023,.025),
               "storm-copper",sides=6)
