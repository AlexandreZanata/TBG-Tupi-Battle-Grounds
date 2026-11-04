"""Fit slag mines and shielded trade barges to the established auxiliary motions."""

from aux_parts import component
from mesh import box, material, pebble, surface, tube
from offshore_sites import pivot, place as place_sites

FOUNDRY_MATERIALS={
    "cliff":"basalt", "cliff-shadow":"basalt-deep", "stratum":"basalt-cut",
    "stone-cool":"cooled-slag", "oak":"charred-timber", "wood":"tempered-wood",
    "wood-end":"hot-grain", "hood":"foundry-iron", "merchant-cloth":"hot-grain",
    "market-cloth":"kiln-canvas", "linen":"ash-linen", "ore":"volcanic-ore",
    "ore-light":"volcanic-ore-light",
}


def place(parent):
    place_sites(parent,FOUNDRY_MATERIALS,decorate,"Volcanic Forge")


def decorate(group,is_mine):
    if is_mine:
        rock=pivot(group,"Rock outcrop pivot")
        component(group,"Slag mine vent",(0,0,0),mine_vent,rock)
        return
    awning=pivot(group,"Canvas awning pivot")
    ship=pivot(group,"Merchant caravel pivot")
    hull=next(obj for obj in ship.children if obj.name.startswith("Open carvel-planked hull"))
    for slot in hull.material_slots:
        slot.material=material("foundry-copper" if slot.material.name=="bronze" else "foundry-iron")
    component(group,"Foundry market shelter",(0,0,0),shelter,awning)
    component(group,"Barge heat shield",(0,0,0),heat_shield,ship)


def mine_vent(group):
    for x,y,h,r in ((-.97,-.47,.73,.162),(-.89,-.41,.97,.169),(-.76,-.51,.57,.134),
                    (-.92,.40,.89,.146)):
        tube(group,"Angular basalt at the shored mine face",[(x,y,.057),(x+.018,y,.41*h),
             (x+.039,y-.026,h)],[r*1.11,r,r*.85],"basalt-deep",sides=6,accent="basalt-cut")
        tube(group,"Fresh ore at the broken basalt crown",[(x+.039,y-.026,h-.031),
             (x+.041,y-.027,h+.018)],[r*.84,r*.71],"volcanic-ore",sides=6,accent="ember-seam")
    tube(group,"Slag vein set into the side of the adit",[
        (-.67,-.68,.139),(-.60,-.677,.263),(-.65,-.655,.390),(-.77,-.617,.465),
    ],[.017,.023,.014,.004],"ember-seam",sides=5)
    pebble(group,"Warm ore fragments under the pillar seam",(-.76,-.66,.050),
           (.198,.102,.112),"cooled-slag",sides=7,accent="volcanic-ore-light")


def shelter(group):
    for x in (-.52,-.26,0,.26,.52):
        tube(group,"Copper ribs fitted over the canvas shelter",[(x,-.444,-.037),
             (x,-.215,.080),(x,0,.142),(x,.218,.078),(x,.429,-.030)],
             [.017,.018,.018,.017,.012],"foundry-copper",sides=6)
    box(group,"Steel edge on the market awning",(0,-.446,-.046),(1.19,.044,.070),
        "foundry-iron",.012)
    for x in (-.50,.50):
        box(group,"Brass awning rivet shoe",(x,-.471,-.044),(.084,.025,.065),
            "foundry-brass",.009)


def heat_shield(group):
    verts=[(-.31,.464,.098),(0,.918,.105),(.31,.464,.098),
           (-.15,.373,-.092),(0,.775,-.096),(.15,.373,-.092)]
    surface(group,"Forged prow shroud",verts,[(0,1,4,3),(1,2,5,4),(0,3,5,2),(3,4,5)],
            ["foundry-copper","foundry-copper","foundry-iron","basalt-deep"])
    for s in (-1,1):
        tube(group,"Copper plate seam along the barge",[
            (s*.264,-.530,.114),(s*.354,-.238,.126),(s*.391,.108,.143),
            (s*.310,.429,.152),(0,.902,.139),
        ],[.014,.017,.017,.016,.012],"foundry-brass",sides=6)
        for y in (-.327,-.125,.091):
            box(group,"Replaceable barge heat-shield plate",(s*(.341+.056*(y+.32)),y,.094),
                (.033,.166,.113),"foundry-iron",.011)
    tube(group,"Armored center prow seam",[(0,.767,-.090),(0,.881,.025),(0,.919,.124)],
         [.018,.022,.014],"foundry-iron",sides=7)
