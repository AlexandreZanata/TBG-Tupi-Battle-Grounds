"""Fit cast-iron mine shoring, cart plates and market machinery to the auxiliary rigs."""


from aux_parts import component
from mesh import box, pebble, surface, tube
from offshore_sites import pivot, place as place_sites

FOUNDRY_MATERIALS = {
    "cliff":"cw-rock", "cliff-shadow":"cw-rock-dark", "stratum":"cw-stratum",
    "stone-cool":"cw-rock-light", "oak":"cw-oak", "wood":"cw-wood", "wood-end":"cw-grain",
    "hood":"cw-workcoat", "merchant-cloth":"cw-workcoat", "market-cloth":"cw-canvas",
    "linen":"cw-linen", "ore":"cw-ore", "ore-light":"cw-ore-light",
}


def place(parent):
    place_sites(parent,FOUNDRY_MATERIALS,decorate,"Clockwork Bastion")


def decorate(group,is_mine):
    if is_mine:
        component(group,"Bolted quarry portal",(0,0,0),portal,pivot(group,"Rock outcrop pivot"))
        component(group,"Mine cart side armor",(0,0,0),cart_plates,pivot(group,"Ore cart pivot"))
        return
    component(group,"Ribbed foundry canopy",(0,0,0),canopy,pivot(group,"Canvas awning pivot"))
    component(group,"Ironclad trade prow",(0,0,0),prow,pivot(group,"Merchant caravel pivot"))


def portal(group):
    for s in (-1,1):
        box(group,"Cast iron adit upright",(-.215,s*.352,.344),(.173,.128,.663),"cw-iron",.025)
        for z in (.121,.546):
            box(group,"Bolted bronze adit brace",(-.195,s*.353,z),(.193,.164,.079),"cw-brass-dark",.012)
            pebble(group,"Head of an adit reinforcement bolt",(-.092,s*.353,z),
                   (.017,.030,.030),"cw-brass-light",sides=6)
    box(group,"Separate machined quarry lintel",(-.221,0,.728),(.225,.89,.161),"cw-steel",.028)
    for y in (-.28,-.14,0,.14,.28):
        pebble(group,"Brass rivet along the quarry lintel",(-.102,y,.735),
               (.017,.025,.025),"cw-brass",sides=6)
    for x,y,z,length in ((-.69,-.663,.137,.49),(-.75,-.591,.303,.36)):
        box(group,"Iron lower mine-face patch",(x,y,z),(length,.161,.131),"cw-iron",.023,angle=.28)
    tube(group,"Copper drain outside the mining approach",[(-.88,-.62,.074),(-.88,-.59,.47),
         (-.89,-.48,.66),(-.77,-.42,.74)],[.028,.029,.030,.025],"cw-copper",sides=7)


def cart_plates(group):
    for s in (-1,1):
        box(group,"Individual iron ore-cart side plate",(0,s*.197,.238),
            (.371,.025,.149),"cw-iron",.014)
        for x in (-.126,.126):
            pebble(group,"Ore-cart side plate rivet",(x,s*.215,.238),(.017,.012,.017),
                   "cw-brass",sides=6)
        box(group,"Brass upper ore-cart lip",(0,s*.194,.328),(.427,.041,.034),"cw-brass-dark",.009)


def canopy(group):
    roof = ((-.468,-.016),(-.242,.081),(0,.163),(.233,.077),(.442,-.019))
    for i in range(8):
        x = -.601+i*.149
        vertices = [(x+dx,y,z) for dx in (0,.144) for y,z in roof]
        surface(group,"Individual market iron roof panel",vertices,[(j,j+1,j+6,j+5) for j in range(4)],
                ["cw-iron" if i%3==0 else "cw-steel"]*4)
        tube(group,"Brass raised seam across the market roof",[(x,y,z+.010) for y,z in roof],
             [.013]*5,"cw-brass",sides=6)
    for y in (-.472,.448):
        box(group,"Rolled iron market eave",(0,y,-.030),(1.242,.058,.071),"cw-iron",.013)
        for x in (-.49,0,.49):
            box(group,"Brass fitting on the market eave",(x,y-.030,-.028),(.073,.027,.059),"cw-brass",.010)


def prow(group):
    vertices = [(-.255,.462,.157),(0,.921,.210),(.255,.462,.157),
                (-.144,.412,-.075),(0,.795,-.083),(.144,.412,-.075),(0,.632,.184)]
    surface(group,"Clockwork prow iron armor cap",vertices,
            [(0,3,4,1),(1,4,5,2),(3,5,4),(0,1,6),(1,2,6),(2,0,6)],
            ["cw-iron","cw-iron","cw-char","cw-steel","cw-steel","cw-brass-dark"])
    tube(group,"Clockwork prow brass cutwater",[(0,.789,-.081),(0,.884,.074),(0,.935,.203),
         (0,.921,.271)],[.020,.029,.029,.013],"cw-brass",sides=7)
    for s in (-1,1):
        tube(group,"Clockwork prow external armor seam",[(s*.279,.411,.139),(s*.212,.558,.149),
             (s*.111,.748,.183),(0,.924,.212)],[.018,.017,.015,.010],"cw-brass-light",sides=6)
        pebble(group,"Clockwork prow separate rivet",(s*.168,.638,.169),(.029,.025,.026),
               "cw-brass",sides=6)
