"""Build a round-eyed pearl scout and a hand-mounted shell sling."""

from abyssal_parts import bow_arc, fins, mantle, marine
from bloomkeeper import leaf

RELEASE=(0,.12,.28)


def pearlslinger(m,tier,branch,*,production=False):
    tide = tier > 0 and branch == 0
    rain = tier > 0 and branch == 1
    shell = "gold" if tide else "bell_rose" if rain else "fish_silver"
    trim = "sun_cream" if tide else "petal_light" if rain else "bone"
    cloth = "oilshade" if tide else "leech_plum" if rain else "smuggler_rust"
    marine(m,"pearlslinger","wasp_aqua","sun_cream",cloth,face=(.121,.131,.106),eyes=.081,snout=.129)
    fins(m,shell,trim,.203+.045*rain,layers=3)
    mantle(m,cloth,trim,.152,.233+.070*(rain and tier==2))
    sling(m,tier,tide,rain,shell,trim)
    for s in (-1,1):
        m.ell("Visible pearl pouch","Hips",cloth,(s*.124,-.034,.062),(.056,.069,.065),segments=11)
        m.ell("Pearl nestled in pouch","Hips",trim,(s*.131,.017,.085),(.036,.034,.032),segments=10)
        if tide:
            m.rim("Gold pearl-seeker lens frame","Head",shell,(s*.081,.034,.139),(.040,.039),.008,segments=11)
    if tide and tier==2:
        m.ell("Raised tide eye diadem","Head","sap",(0,.117,.114),(.036,.037,.023),segments=11)


def sling(m,tier,tide,rain,shell,trim):
    n="WeaponR"
    m.tube("Pearl sling handle through palm",n,"wood",[(0,-.095,0),(0,.056,.035),
           (0,.117,.194)],[.025,.030,.038],n=8)
    bow_arc(m,n,shell,.117,.183,.149 if tide else .119,.104)
    m.ell("Pearl at the native release origin",n,trim,RELEASE,(.043,.039,.040),segments=12)
    if rain:
        for s in (-1,1):
            leaf(m,"Divided shell cup wing",n,shell,(s*.071,.110,.163),
                 (s*(.190+.028*(tier==2)),.208,.263),.054)
            if tier==2:
                m.ell("Reserve pearl in outer shell cup",n,trim,(s*.132,.152,.235),
                      (.033,.029,.030),segments=9)
