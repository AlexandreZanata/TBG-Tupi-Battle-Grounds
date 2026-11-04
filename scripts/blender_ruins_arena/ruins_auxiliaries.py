"""Fit quarry adits, reed canopies and carved boats to the offshore auxiliary rig."""

from math import cos, pi, sin

from aux_parts import component
from mesh import box, pebble, surface, tube
from offshore_sites import pivot, place as place_sites

QUARRY_MATERIALS={
    "cliff":"limestone-dark", "cliff-shadow":"ruin-shadow", "stratum":"limestone-seam",
    "stone-cool":"limestone", "oak":"ruin-oak", "wood":"ruin-wood", "wood-end":"ruin-grain",
    "hood":"ruin-jade-dark", "merchant-cloth":"ruin-market", "market-cloth":"ruin-linen",
    "linen":"ruin-linen", "ore":"ruin-ore", "ore-light":"ruin-ore-light",
}


def place(parent):
    place_sites(parent,QUARRY_MATERIALS,decorate,"Ancient Ruins")


def decorate(group,is_mine):
    if is_mine:
        component(group,"Ruin adit stonework",(0,0,0),adit,pivot(group,"Rock outcrop pivot"))
        return
    component(group,"Reed market canopy",(0,0,0),canopy,pivot(group,"Canvas awning pivot"))
    component(group,"Carved lagoon prow",(0,0,0),prow,pivot(group,"Merchant caravel pivot"))


def adit(group):
    for s in (-1,1):
        for i in range(3):
            box(group,"Separate quarry entrance jamb",(-.254,s*.371,.135+i*.185),
                (.195,.174,.174),"limestone-light" if i%2 else "limestone",.020)
    for i in range(9):
        a,b=(i+.05)*pi/9,(i+.95)*pi/9
        section=[(.324*r*cos(t),.51+.307*r*sin(t)) for r,t in ((1,a),(1.31,a),(1.31,b),(1,b))]
        verts=[(x,y,z) for x in (-.376,-.157) for y,z in section]
        surface(group,"Separate quarried adit arch stone",verts,
                [(3,2,1,0),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(0,4,7,3)],
                ["ruin-jade" if i==4 else "limestone-light" if i%2 else "limestone"]*6)
    for x,y,z,length in ((-.66,-.682,.13,.55),(-.69,-.616,.30,.44),(-.74,-.586,.48,.36)):
        box(group,"Exposed courses of quarry limestone",(x,y,z),(length,.170,.151),
            "limestone-light",.023,angle=.21)
    pebble(group,"Moss on the partly restored adit",(-.409,-.345,.656),(.136,.135,.044),
           "ruin-moss",accent="ruin-leaf")


def canopy(group):
    for i in range(17):
        x=(i-8)*.070
        tube(group,"Separate split reed across the awning",[(x,-.441,-.024),(x,-.221,.061),
             (x,0,.139),(x,.221,.061),(x,.441,-.024)],[.015,.016,.017,.016,.015],
             "ruin-reed" if i%3 else "ruin-grain",sides=5)
    for y,z in ((-.435,-.018),(-.251,.063),(0,.151),(.251,.063),(.435,-.018)):
        tube(group,"Cross-lashing over the reed roof",[(-.607,y,z),(.607,y,z)],
             [.012,.012],"ruin-jade-dark",sides=5)
    box(group,"Patterned jade edge under the canopy",(0,-.446,-.053),(1.19,.042,.073),
        "ruin-market",.010)
    for x in (-.496,-.248,0,.248,.496):
        box(group,"Bronze reed awning fastener",(x,-.474,-.052),(.054,.025,.047),
            "ruin-bronze",.007)


def prow(group):
    verts=[(-.225,.511,.074),(0,.938,.191),(.225,.511,.074),
           (-.109,.485,-.064),(0,.822,-.074),(.109,.485,-.064),
           (0,.642,.105)]
    surface(group,"Lagoon prow cap",verts,
            [(0,3,4,1),(1,4,5,2),(3,5,4),(0,1,6),(1,2,6),(2,0,6)],
            ["ruin-jade-dark","ruin-jade-dark","ruin-oak","ruin-copper","ruin-copper","ruin-jade"])
    tube(group,"Lagoon prow bronze ridge",[(0,.805,-.059),(0,.898,.064),(0,.949,.176),
         (0,.920,.279),(0,.867,.270)],[.016,.025,.031,.028,.016],"ruin-bronze",sides=7)
    for s in (-1,1):
        tube(group,"Lagoon prow jade keel edge",[(s*.279,.371,.105),(s*.204,.537,.123),
             (s*.096,.768,.168),(0,.936,.192)],[.016,.017,.014,.012],"ruin-jade",sides=6)
        pebble(group,"Lagoon prow copper eye",(s*.085,.786,.167),(.030,.026,.029),
               "ruin-bronze",sides=6,accent="ruin-jade-light")
