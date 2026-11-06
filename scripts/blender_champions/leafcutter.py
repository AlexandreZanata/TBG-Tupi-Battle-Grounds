"""Build a six-legged leaf cutter with a wrist-mounted serrated tool."""

from bloomkeeper import leaf
from completion_people import limbs
from insect_legs import auxiliary_legs

STRIKE=(0,0,.30)


def leafcutter(m,tier,branch,*,production=False):
    deep = tier > 0 and branch == 0
    amber = tier > 0 and branch == 1
    shell = "leaf_ant" if not amber else "wasp_amber"
    body(m,shell,deep,amber,tier)
    face(m,shell,deep,amber,tier)
    limbs(m,shell,"pine",arm=.17*.95,width=.047,skin="pine")
    auxiliary_legs(m,shell,"sun_cream" if deep else "hide")
    saw(m,deep,amber,tier)


def body(m,shell,deep,amber,tier):
    m.loft("Leafcutter shell across thorax","Chest",shell,[
        (-.058,.085,.071,0),(.053,.116,.090,-.009),(.160,.145,.105,0),
        (.268,.157,.090,.006),(.322,.092,.065,.017),
    ],n=13)
    m.tube("Swept leafcutter gaster","Hips",shell,[(0,.008,.006),(0,.020,-.091),
           (0,-.016,-.210),(0,-.065,-.320)],[.063,.113,.108,.019],n=12)
    for s in (-1,1):
        if deep:
            leaf(m,"Ivory shoulder cuticle","Chest","sun_cream",(s*.065,.231,.053),
                 (s*.202,.331,-.038),.061)
        if amber:
            leaf(m,"Amber angular hip shell","Hips","wasp_amber",(s*.072,.063,-.016),
                 (s*.165,-.058,-.228),.051)


def face(m,shell,deep,amber,tier):
    h="Head"
    m.ell("Wedge-shaped ant face",h,shell,(0,.009,.026),(.131,.118,.112),segments=14)
    for s in (-1,1):
        m.ell("Ant compound eye socket",h,"ink",(s*.100,.032,.097),(.045,.039,.034),segments=10)
        m.ell("Glossy ant eye",h,"gold",(s*.114,.036,.121),(.024,.024,.015),segments=9)
        m.tube("Long bent antenna",h,"pine",[(s*.059,.102,.029),(s*.105,.187,.042),
               (s*.182,.228,.079)],[.016,.017,.004],n=7)
        m.tube("Curved ant mandible",h,"sun_cream" if deep else "gold",[(s*.094,-.034,.087),
               (s*.111,-.068,.148),(s*.054,-.083,.198)],[.028,.028,.004],n=8)
    if tier==2:
        leaf(m,"Central blade-like ant crest",h,"sun_cream" if deep else "gold",(0,.079,.053),
             (0,.251,-.096),.053,facing=(1,0,0))


def saw(m,deep,amber,tier):
    w="WeaponR"
    m.tube("Carved blade wrist grip",w,"hide",[(0,0,-.061),(0,0,.091)],[.026,.027],n=9)
    m.tube("Cutting tool along the native strike line",w,"iron",[(0,0,.057),(0,0,.166),
           STRIKE,(0,0,.515 if deep else .429)],[.034,.048,.043,.005],n=9)
    leaf(m,"Thick tapered leaf saw blade",w,"sun_cream" if deep else "gold" if amber else "edge",
         (0,0,.077),(0,0,.524 if deep else .441),.065 if deep else .047,facing=(0,1,0))
    for i in range(4+(tier==2)):
        z=.120+.055*i
        leaf(m,"Separately cut blade serration",w,"gold" if amber else "sun_cream",
             (.029,-.003,z),(.100+.018*deep,0,z+.045),.025,facing=(0,1,0))
    if amber and tier==2:
        leaf(m,"Amber rear hook",w,"wasp_amber",(-.035,.003,.247),(-.126,0,.386),.047,
             facing=(0,1,0))
