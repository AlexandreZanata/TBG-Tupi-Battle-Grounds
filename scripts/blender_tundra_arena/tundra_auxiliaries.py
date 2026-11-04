"""Fit winter details to both animated auxiliary precincts and their moving pivots."""

from aux_parts import component
from mesh import box, pebble, surface, tube
from offshore_sites import pivot, place as place_sites

WINTER_MATERIALS = {
    "cliff": "frost-granite", "cliff-shadow": "frost-basalt", "stratum": "glacier-dark",
    "stone-cool": "frost-ashlar", "oak": "winter-timber", "wood": "winter-wood",
    "wood-end": "winter-grain", "hood": "winter-cloth", "merchant-cloth": "winter-grain",
    "market-cloth": "winter-cloth", "linen": "winter-linen", "ore": "winter-crystal-dark",
    "ore-light": "winter-crystal",
}


def place(parent):
    place_sites(parent,WINTER_MATERIALS,decorate,"Frozen Tundra")


def decorate(group,is_mine):
    if is_mine:
        rock=pivot(group,"Rock outcrop pivot")
        component(group,"Winter mine face",(0,0,0),mine_snow,rock)
    else:
        awning=pivot(group,"Canvas awning pivot")
        ship=pivot(group,"Merchant caravel pivot")
        component(group,"Winter market shelter",(0,0,0),awning_snow,awning)
        component(group,"Icebreaker's bow shoe",(0,0,0),bow_shoe,ship)


def mine_snow(group):
    for i,(x,y,z,rx,ry,rz) in enumerate((
        (-.73,-.06,1.116,.34,.37,.12),(-.68,.36,.923,.33,.23,.12),
        (-.60,-.37,.88,.39,.21,.13),(-.47,.11,.984,.29,.37,.10),
    )):
        pebble(group,"Separate wind-packed mine snow cap",(x,y,z),(rx,ry,rz),
               "snow-light",sides=8,accent="snow-shadow",phase=i*.47)
    for x,y,z,length in ((-.58,-.53,.92,.28),(-.77,-.55,.85,.20),(-.90,-.53,.82,.17)):
        tube(group,"Hanging adit-side ice needle",[(x,y,z),(x+.014,y-.021,z-length*.30),
             (x+.031,y-.045,z-length)],[.039,.030,.003],"glacier",sides=5,accent="glacier-light")
    tube(group,"Ice vein beside the entrance",[
        (-.48,-.61,.102),(-.56,-.625,.29),(-.55,-.623,.432),(-.70,-.602,.51),
    ],[.024,.026,.019,.007],"winter-crystal-dark",sides=5,accent="winter-crystal")


def awning_snow(group):
    for i,x in enumerate((-.57,-.37,-.15,.08,.30,.52)):
        pebble(group,"Patch of snow lying on the canvas",(x,.049*(i%2),.127),
               (.15,.111,.047),"snow",sides=7,accent="snow-light",phase=i)
    for x in (-.55,-.32,.02,.38,.55):
        tube(group,"Winter canvas binding",[(x,-.443,-.033),(x,-.453,-.147)],
             [.013,.010],"winter-linen",sides=5)
    box(group,"Insulated timber front binding",(0,-.455,-.037),(.89,.031,.042),
        "winter-timber",.011)


def bow_shoe(group):
    vertices=[(-.23,.51,.085),(0,.923,.102),(.23,.51,.085),
              (-.108,.37,-.104),(0,.775,-.079),(.108,.37,-.104)]
    surface(group,"Copper icebreaker shoe and reinforced stem",vertices,
            [(0,1,4,3),(1,2,5,4),(0,3,5,2),(3,4,5)],
            ["winter-copper","winter-copper","winter-timber","iron"])
    tube(group,"Forged stem runner",[(0,.750,-.093),(0,.889,.036),(0,.927,.114)],
         [.021,.022,.015],"iron",sides=7)
