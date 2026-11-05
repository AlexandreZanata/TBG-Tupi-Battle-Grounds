"""Build a plated maul bearer with a visible two-hand shaft and star-shaped hammer."""

from math import cos, sin, tau

from bloomkeeper import leaf
from completion_people import limbs

FRONT_GRIP=.13
STRIKE=.30


def starbreaker(m,tier,branch,*,production=False):
    maul = tier > 0 and branch == 0
    nova = tier > 0 and branch == 1
    ceramic = "sun_cream" if maul else "star_cobalt" if nova else "star_ceramic"
    body(m,ceramic,maul,nova,tier)
    helmet(m,ceramic,maul,nova,tier)
    limbs(m,"sun_navy",ceramic,arm=.20,width=.069,skin="iron")
    hammer(m,ceramic,maul,nova,tier)


def body(m,ceramic,maul,nova,tier):
    m.loft("Thick plated maul-bearer torso","Chest",ceramic,[
        (-.066,.135,.112,0),(.040,.160,.130,-.006),(.149,.209,.140,0),
        (.251,.210,.128,-.010),(.324,.113,.087,-.013),
    ],n=13)
    m.box("Broad hammered hip belt","Hips","brass",(0,.039,.068),(.281,.076,.134),.014)
    for side,s in (("L",1),("R",-1)):
        m.ell("Heavy hammer-side pauldron",f"UpperArm{side}",ceramic,(s*.021,-.009,.011),
              (.100+.026*maul,.079,.107),segments=12)
        m.tube("Raised rectangular chest rib","Chest","brass",[(s*.105,.095,.113),
               (s*.150,.191,.130),(s*.158,.265,.103)],[.020,.025,.017],n=8)
        if maul:
            leaf(m,"Shoulder constellation spur","Chest","gold",(s*.117,.283,-.034),
                 (s*.255,.381,-.055),.057)
        if nova:
            leaf(m,"Forward cobalt pauldron ridge","Chest","star_cobalt",(s*.138,.252,.041),
                 (s*.269,.286,.006),.063)


def helmet(m,ceramic,maul,nova,tier):
    h="Head"
    m.loft("Faceted closed hammer helmet",h,ceramic,[
        (-.127,.081,.075,.010),(-.059,.112,.100,.003),(.044,.121,.105,-.002),
        (.112,.110,.097,-.008),(.150,.067,.071,-.021),
    ],n=10)
    m.box("Dark recess in closed visor",h,"ink",(0,.024,.108),(.176,.060,.033),.008)
    for s in (-1,1):
        m.box("Slit with starlight",h,"soul" if nova else "sap",(s*.049,.030,.127),
              (.053,.016,.011),.004)
    for i in range(5):
        a=tau*i/5
        leaf(m,"Star-crown embossed petal",h,"brass",(.039*sin(a),.073+.032*cos(a),.111),
             (.094*sin(a),.075+.095*cos(a),.121),.018)
    if maul and tier==2:
        leaf(m,"Raised hammer-crown crest","HeadGear","gold",(0,.086,.038),
             (0,.270,-.077),.053,facing=(1,0,0))


def hammer(m,ceramic,maul,nova,tier):
    w="WeaponR"
    m.tube("Long balanced hammer haft",w,"sun_navy",[(0,0,-.142),(0,0,0),
           (0,0,FRONT_GRIP),(0,0,STRIKE),(0,0,.493)],[.027,.029,.029,.026,.035],n=10)
    m.tube("Two-hand binding along the haft",w,"hide",[(0,0,-.056),(0,0,.185)],[.036,.036],n=9)
    width=.412 if maul else .340
    m.box("Chamfered star maul head",w,ceramic,(0,0,.574),(width,.255,.249),.043)
    for s in (-1,1):
        m.box("Forged hammer face plate",w,"brass",(s*width*.482,0,.573),
              (.059,.276,.272),.018)
        if maul and tier==2:
            leaf(m,"Raised constellation hammer edge",w,"gold",(s*.108,.113,.541),
                 (s*.208,.175,.709),.057,facing=(0,1,0))
    if nova:
        r=.086 if tier==2 else .065
        m.ell("Held nova crystal",w,"soul",(0,0,.713),(r,r,r*.92),segments=13)
        for s in (-1,1):
            m.tube("Bronze nova cage spar",w,"brass",[(s*.082,.046,.624),(s*.123,.048,.712),
                   (s*.045,.008,.789)],[.018,.020,.011],n=8)
            if tier==2:
                leaf(m,"Raised starforge cage wing",w,"star_cobalt",(s*.086,.009,.650),
                     (s*.197,.028,.782),.043,facing=(0,1,0))
