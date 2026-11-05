"""Fit celestial quarry masonry, an azure canopy and gilt prows to the shared rigs."""

from math import cos, pi, sin

from aux_parts import component
from celestial_architecture import gem
from mesh import box, pebble, surface, tube
from offshore_sites import pivot, place as place_sites

QUARRY_MATERIALS = {
    "cliff":"cel-stone", "cliff-shadow":"cel-ivory-shade", "stratum":"cel-ivory",
    "stone-cool":"cel-ivory-shade", "oak":"cel-oak", "wood":"cel-wood", "wood-end":"cel-grain",
    "hood":"cel-azure-deep", "merchant-cloth":"cel-canvas", "market-cloth":"cel-canvas",
    "linen":"cel-cloth", "ore":"cel-crystal", "ore-light":"cel-crystal-light",
}


def place(parent):
    place_sites(parent,QUARRY_MATERIALS,decorate,"Celestial Citadel")


def decorate(group,is_mine):
    if is_mine:
        component(group,"Ivory quarry portal",(0,0,0),portal,pivot(group,"Rock outcrop pivot"))
        return
    component(group,"Gilded market canopy",(0,0,0),canopy,pivot(group,"Canvas awning pivot"))
    component(group,"Ceremonial cutter prow",(0,0,0),prow,pivot(group,"Merchant caravel pivot"))


def portal(group):
    for s in (-1,1):
        box(group,"Quarried ivory portal plinth",(-.227,s*.355,.066),(.203,.174,.143),"cel-stone",.020)
        tube(group,"Slender gilt-framed adit jamb",[(-.224,s*.349,.132),(-.220,s*.349,.56),
             (-.237,s*.347,.677)],[.065,.057,.083],"cel-ivory",sides=7,accent="cel-stone")
        box(group,"Gilt capital at the portal spring",(-.225,s*.350,.681),(.184,.167,.061),"cel-gold",.011)
    for i in range(9):
        a,b = (i+.038)*pi/9,(i+.962)*pi/9
        profile = [(.337*r*cos(t),.675+.207*r*sin(t)) for r,t in ((1,a),(1.26,a),(1.26,b),(1,b))]
        verts = [(x,y,z) for x in (-.358,-.133) for y,z in profile]
        surface(group,"Carved quarry arch facing",verts,
                [(3,2,1,0),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(0,4,7,3)],
                ["cel-gold-light" if i==4 else "cel-ivory" if i%2 else "cel-stone"]*6)
    for x,y,z,w in ((-.70,-.69,.129,.49),(-.76,-.60,.301,.32)):
        box(group,"Worked stone course below the crystal vein",(x,y,z),(w,.166,.147),
            "cel-ivory",.022,angle=.30)
    gem(group,"Set azure crystal on the quarry cornice",(-.165,0,.939),.044,.117)


def canopy(group):
    roof = ((-.460,-.017),(-.263,.062),(0,.160),(.243,.065),(.435,-.019))
    for i in range(8):
        x = -.607+i*.152
        verts = [(x+dx,y,z) for dx in (0,.150) for y,z in roof]
        surface(group,"Separate blue market roof panel",verts,[(j,j+1,j+6,j+5) for j in range(4)],
                ["cel-azure" if i%3 else "cel-azure-light"]*4)
    for x in (-.599,-.297,0,.297,.599):
        tube(group,"Gilt rib across the azure roof",[(x,y,z+.012) for y,z in roof],
             [.016,.015,.016,.015,.016],"cel-gold-light",sides=6)
    for y in (-.465,.441):
        box(group,"Ivory rain eave",(0,y,-.026),(1.246,.052,.061),"cel-ivory",.012)
        for x in (-.51,0,.51):
            box(group,"Gold fitting on the rain eave",(x,y-.026,-.025),(.076,.029,.059),
                "cel-gold",.010)
    tube(group,"Gilt ridge along the awning",[(-.624,0,.172),(.627,0,.172)],
         [.018,.018],"cel-gold",sides=6)


def prow(group):
    verts = [(-.258,.458,.153),(0,.915,.206),(.258,.458,.153),
             (-.146,.409,-.074),(0,.790,-.081),(.146,.409,-.074),(0,.634,.180)]
    surface(group,"Celestial prow fitted blue nose",verts,
            [(0,3,4,1),(1,4,5,2),(3,5,4),(0,1,6),(1,2,6),(2,0,6)],
            ["cel-azure-deep","cel-azure-deep","cel-oak","cel-azure","cel-azure","cel-azure-light"])
    tube(group,"Celestial prow gilt cutwater",[(0,.788,-.080),(0,.886,.067),(0,.928,.193),
         (0,.899,.295),(0,.837,.302)],[.019,.027,.032,.027,.011],"cel-gold",sides=7)
    for s in (-1,1):
        tube(group,"Celestial prow curved wing trim",[(s*.287,.383,.134),(s*.252,.492,.153),
             (s*.143,.685,.164),(s*.045,.848,.201),(0,.904,.251)],
             [.020,.022,.018,.016,.012],"cel-gold-light",sides=6)
        pebble(group,"Celestial prow blue stud in gold trim",(s*.118,.718,.174),(.030,.027,.028),
               "cel-crystal",sides=6,accent="cel-crystal-light")
