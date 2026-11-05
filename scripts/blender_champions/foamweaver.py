"""Shape the foam healer's two-handed shell chalice and thread-covered mantle."""

from abyssal_parts import fins, mantle, marine
from bloomkeeper import leaf

RELEASE=(0,.12,.28)
SUPPORT=(.21,-.01,.01)


def foamweaver(m,tier,branch,*,production=False):
    spring = tier > 0 and branch == 0
    threads = tier > 0 and branch == 1
    shell = "sun_cream" if spring else "mantis_jade" if threads else "fish_silver"
    bead = "sap" if spring else "soul" if threads else "wasp_wing"
    cloth = "oracle_saffron" if spring else "dawn_teal" if threads else "ritual_indigo"
    marine(m,"foamweaver","wasp_aqua","sun_cream",cloth,face=(.105,.112,.098),eyes=.073,snout=.126)
    fins(m,shell,bead,.206+.029*threads,layers=3)
    mantle(m,cloth,shell,.166+.055*(threads and tier==2),.345)
    chalice(m,shell,bead,tier,spring,threads)
    for s in (-1,1):
        leaf(m,"Overlapping scallop cloth at the waist","Hips",cloth,(s*.055,.033,.103),
             (s*.095,-.196,.110),.051)
        if threads:
            m.tube("Raised curved mantle thread","BackSpring",bead,[(s*.040,.066,-.013),
                   (s*.127,-.092,-.047),(s*(.141+.041*(tier==2)),-.295,-.113)],[.012,.011,.005],n=7)
            if tier==2:
                leaf(m,"Open wave crest on the mantle","BackSpring",shell,(s*.067,.052,-.013),
                     (s*.251,.154,-.077),.071)
    if spring and tier==2:
        for s in (-1,1):
            leaf(m,"Nacre spring circlet petal","Head",shell,(s*.044,.086,.061),
                 (s*.103,.179,-.043),.043)


def chalice(m,shell,bead,tier,spring,threads):
    n="WeaponR"
    m.tube("Chalice stem crosses the main palm",n,"wood",[(0,-.062,.003),(0,.034,.062),
           (0,.078,.168)],[.031,.034,.044],n=9)
    m.loft("Hollow open scallop bowl",n,shell,[
        (.047,.040,.052,.176),(.076,.096,.096,.180),
        (.123,.144,.126,.183),(.123,.127,.110,.183),
        (.077,.064,.064,.180),
    ],n=14)
    m.ell("Foam cap at the spell origin",n,bead,RELEASE,(.043,.041,.042),segments=11)
    m.tube("Cup handle extending to the support hand",n,shell,[(.103,.099,.118),
           (.155,.025,.066),SUPPORT,(.151,-.006,.040)],[.019,.023,.023,.017],n=8)
    for s in (-1,1):
        if spring:
            leaf(m,"Ivory scallop petal",n,shell,(s*.067,.094,.219),
                 (s*.157,.196+.037*(tier==2),.238),.043)
        if threads:
            m.tube("Separate curved foam filament",n,bead,[(s*.065,.098,.211),
                   (s*.120,.155,.267),(s*.141,.208+.035*(tier==2),.229)],
                   [.011,.011,.003],n=7)
