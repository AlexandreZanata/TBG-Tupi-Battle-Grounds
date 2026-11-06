"""Fit sandstone quarry portals, woven market roofs and carved prows to the shared rigs."""

from aux_parts import component
from mesh import box, pebble, surface, tube
from offshore_sites import pivot, place as place_sites

DESERT_MATERIALS = {
    "cliff":"dr-stone", "cliff-shadow":"dr-stone-dark", "stratum":"dr-stone-light",
    "stone-cool":"dr-stratum", "oak":"dr-oak", "wood":"dr-wood", "wood-end":"dr-grain",
    "hood":"dr-cloth", "merchant-cloth":"dr-cloth", "market-cloth":"dr-canvas",
    "linen":"dr-linen", "ore":"dr-ore", "ore-light":"dr-ore-light",
}


def place(parent):
    place_sites(parent,DESERT_MATERIALS,decorate,"Desert Relics")


def decorate(group,is_mine):
    if is_mine:
        component(group,"Carved sandstone portal",(0,0,0),portal,pivot(group,"Rock outcrop pivot"))
        return
    component(group,"Reed oasis canopy",(0,0,0),canopy,pivot(group,"Canvas awning pivot"))
    component(group,"Carved oasis prow",(0,0,0),prow,pivot(group,"Merchant caravel pivot"))


def portal(group):
    for side in (-1,1):
        for i in range(3):
            box(group,"Individual sandstone adit jamb",(-.230,side*.350,.131+i*.177),
                (.193,.159,.169),"dr-stone-light" if i%2 else "dr-stone",.018)
        box(group,"Gold inset on the quarry capital",(-.223,side*.350,.641),
            (.212,.189,.054),"dr-gold-dark",.012)
    box(group,"Broad sandstone quarry lintel",(-.222,0,.713),(.244,.925,.149),"dr-stone-light",.027)
    box(group,"Upper step of the quarry lintel",(-.224,0,.809),(.215,.799,.070),"dr-stone",.014)
    for y in (-.263,-.132,0,.132,.263):
        box(group,"Turquoise glyph on the quarry lintel",(-.093,y,.726),
            (.020,.047,.060),"dr-turquoise",.007)
    for x,y,z,width in ((-.69,-.672,.130,.50),(-.76,-.592,.302,.37)):
        box(group,"Lower dressed stone beside the quarry",(x,y,z),(width,.162,.141),
            "dr-stone-light",.021,angle=.29)


def canopy(group):
    for i in range(18):
        x = (i-8.5)*.068
        tube(group,"Individual split reed over the market awning",[(x,-.452,-.019),(x,-.225,.070),
             (x,0,.152),(x,.220,.071),(x,.437,-.018)],
             [.015,.016,.016,.016,.014],"dr-grain" if i%3 else "dr-reed",sides=5)
    for y,z in ((-.446,-.009),(-.255,.069),(0,.165),(.251,.069),(.434,-.009)):
        tube(group,"Turquoise lashing across the reed roof",[(-.625,y,z),(.625,y,z)],
             [.012,.012],"dr-turquoise-dark",sides=6)
    for y in (-.468,.453):
        box(group,"Woven turquoise market valance",(0,y,-.060),(1.245,.037,.083),
            "dr-turquoise",.009)
        for x in (-.506,-.253,0,.253,.506):
            box(group,"Short gold trim on the woven valance",(x,y-.021,-.057),
                (.056,.019,.058),"dr-gold",.006)


def prow(group):
    verts = [(-.256,.461,.151),(0,.924,.212),(.256,.461,.151),
             (-.144,.410,-.073),(0,.793,-.081),(.144,.410,-.073),(0,.631,.183)]
    surface(group,"Oasis prow carved turquoise cap",verts,
            [(0,3,4,1),(1,4,5,2),(3,5,4),(0,1,6),(1,2,6),(2,0,6)],
            ["dr-turquoise-dark","dr-turquoise-dark","dr-oak","dr-turquoise",
             "dr-turquoise","dr-turquoise-light"])
    tube(group,"Oasis prow curved gold cutwater",[(0,.789,-.079),(0,.884,.070),(0,.936,.204),
         (0,.917,.298),(0,.846,.318)],[.020,.027,.033,.026,.010],"dr-gold",sides=7)
    for side in (-1,1):
        tube(group,"Oasis prow carved side scroll",[(side*.279,.401,.133),(side*.242,.510,.144),
             (side*.130,.735,.173),(side*.032,.883,.220),(0,.924,.262)],
             [.019,.021,.018,.014,.009],"dr-gold-light",sides=6)
        pebble(group,"Oasis prow turquoise eye inset",(side*.126,.718,.178),(.029,.024,.027),
               "dr-turquoise-dark",sides=6,accent="dr-turquoise-light")
