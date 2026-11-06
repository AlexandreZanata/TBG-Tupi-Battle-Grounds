"""Sculpt a wide beetle with a sprung carapace and hinged mandibles."""

from bloomkeeper import leaf
from insect_legs import legs

BITE = (0,-.05,.31)


def jawguard(m,tier,branch,*,production=False):
    plate = tier > 0 and branch == 0
    swarm = tier > 0 and branch == 1
    shell = "sun_cream" if plate else "beetle_coral" if swarm else "beetle_green"
    carapace(m,shell,plate,swarm,tier)
    mask(m,shell,swarm)
    for side,s,joint in (("L",1,"OffhandL"),("R",-1,"WeaponR")):
        mandible(m,joint,s,swarm,tier)
        limb(m,side,shell)
        legs(m,side,.14,"pine",shell,.074)


def carapace(m,shell,plate,swarm,tier):
    m.loft("Compact beetle thorax","Chest","pine",[
        (-.073,.167,.138,.002),(.040,.220,.171,0),(.170,.249,.163,0),
        (.266,.218,.132,.011),(.324,.115,.095,.029),
    ],n=14)
    m.ell("Heavy carapace underlayer","BackSpring","hide",(0,.070,-.118),
          (.310,.267,.155),segments=15,rings=8)
    for s in (-1,1):
        m.ell("Split curved wing casing","BackSpring",shell,(s*.139,.088,-.125),
              (.213,.287,.190),segments=15,rings=8)
        m.tube("Tapered outer shell rim","BackSpring","oldbone" if plate else "brass",
               [(s*.040,-.152,-.202),(s*.238,-.092,-.233),(s*.342,.073,-.165),
                (s*.249,.286,-.136),(s*.082,.359,-.115)],[.021,.031,.031,.025,.014],n=8)
        if plate:
            leaf(m,"Raised limestone side carapace","BackSpring","sun_cream",(s*.180,-.129,-.265),
                 (s*.350,.283,-.221),.101)
        if swarm:
            for i in range(3):
                leaf(m,"Raked swarm shoulder barb","Chest","beetle_coral",(s*.156,.216,.011-i*.044),
                     (s*(.286+i*.022),.333-i*.024,-.031-i*.049),.048)
    m.tube("Central dark seam between the wing casings","BackSpring","hide",[
        (0,-.136,-.266),(0,-.064,-.291),(0,.082,-.306),(0,.242,-.285),(0,.353,-.203),
    ],[.012,.015,.016,.015,.010],n=7)
    if plate and tier==2:
        leaf(m,"Tall armored dorsal fin","BackSpring","sun_cream",(0,.148,-.246),(0,.540,-.219),
             .113,facing=(1,0,0))


def mask(m,shell,swarm):
    h = "Head"
    m.ell("Wide plated beetle face",h,"pine",(0,.018,.021),(.195,.138,.162),segments=16)
    m.tube("Single hooked beetle horn","HeadGear",shell,[(0,.060,.033),(0,.163,.072),
           (0,.249,.111),(0,.286,.169)],[.072,.069,.041,.006],n=10)
    for s in (-1,1):
        m.ell("Inset beetle eye",h,"ink",(s*.137,.027,.117),(.057,.041,.042),segments=11)
        m.ell("Amber beetle eye lens",h,"forge" if swarm else "sap",(s*.157,.029,.141),
              (.029,.023,.025),segments=9)
        m.tube("Brow sweeping into the central horn",h,shell,[(0,.082,.100),(s*.106,.090,.128),
               (s*.173,.060,.096)],[.027,.046,.019],n=8)
    m.ell("Recess behind moving jaws",h,"ink",(0,-.062,.164),(.113,.064,.048),segments=12)


def mandible(m,node,s,swarm,tier):
    end = (-s*.116,.030,.150)
    m.tube("Hooked bite on its own jaw pivot",node,"beetle_coral" if swarm else "oldbone",[
        (0,.006,-.028),(s*.050,-.006,.048),(s*.026,.005,.128),end,
    ],[.053,.053+.018*swarm,.034,.004],n=10)
    if swarm and tier==2:
        leaf(m,"Outer serrated fang",node,"brass",(s*.044,.024,.035),
             (s*.113,.009,.193),.047,facing=(0,1,0))


def limb(m,side,shell):
    arm,fore,hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
    m.tube("Chitin shoulder arm",arm,shell,[(0,.032,0),(0,-.059,0),(0,-.183,0)],
           [.081,.104,.055],n=9)
    m.tube("Dark elbow and forearm",fore,"pine",[(0,.025,0),(0,-.065,0),(0,-.164,0)],
           [.054,.076,.045],n=9)
    m.ell("Armored beetle forepaw",hand,shell,(0,-.023,.021),(.060,.069,.051),segments=10)
