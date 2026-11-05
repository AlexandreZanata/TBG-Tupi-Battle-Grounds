"""Sculpt a resonant monster with a ribbed, chest-mounted sound chamber."""

from bloomkeeper import leaf
from completion_people import limbs

THROAT = (0,0,.30)


def gorgehowl(m,tier,branch,*,production=False):
    deep = tier > 0 and branch == 0
    split = tier > 0 and branch == 1
    shell = "howl_clay" if deep else "howl_glaze" if split else "fortress_basalt"
    chamber(m,shell,deep,split,tier)
    skull(m,shell,deep,split,tier)
    limbs(m,shell,"slag",width=.075,skin=shell)


def chamber(m,shell,deep,split,tier):
    c="Chest"
    m.loft("Rounded cracked resonator torso",c,shell,[
        (-.083,.136,.117,-.013),(.016,.186,.145,-.014),(.152,.220,.148,-.003),
        (.257,.206,.127,-.006),(.336,.112,.096,0),
    ],n=14,fold=.077)
    m.ell("Deep acoustic cavity",c,"ink",(0,.063,.161),(.133,.144,.073),segments=13)
    for s in (-1,1):
        for i in range(4):
            y=.231-i*.056
            x=.100+.020*i
            m.tube("Curved acoustic rib",c,"sun_cream" if deep else "edge" if split else "oldbone",
                   [(s*.186,y+.009,.102),(s*x,y,.196),(s*.045,y-.023,.219)],
                   [.020,.030,.014],n=8)
        if deep:
            leaf(m,"Ivory side amplifier plate",c,"sun_cream",(s*.139,.081,.082),
                 (s*.263,.308,.025),.062)
        if split:
            m.tube("Secondary glaze sound pipe",c,"howl_glaze",[(s*.101,.018,.161),
                   (s*.181,.065,.205),(s*.247,.158,.170)],[.038,.046,.053],n=9)
            if tier==2:
                leaf(m,"Raised split-echo outlet fin",c,"soul",(s*.199,.096,.155),
                     (s*.315,.242,.114),.035)
    m.tube("Chest sound nozzle aligned to the attack",c,"oldbone",[(0,0,.132),(0,0,.208),
           (0,0,THROAT[2]-.014)],[.055,.049,.069],n=12)
    m.rim("Sound chamber's flared rim",c,"edge" if split else "brass",
          (0,0,THROAT[2]-.010),(.055,.055),.010,segments=18)
    m.ell("Recessed sound outlet",c,"ink",(0,0,.286),(.047,.047,.006),segments=11)
    if deep and tier==2:
        for s in (-1,1):
            leaf(m,"Tall ivory resonator spike",c,"sun_cream",(s*.174,.260,-.058),
                 (s*.273,.459,-.107),.051)


def skull(m,shell,deep,split,tier):
    h="Head"
    m.ell("Hollow monster skull",h,shell,(0,-.005,.015),(.135,.126,.107),segments=14)
    m.ell("Dark howling mouth",h,"ink",(0,-.059,.099),(.075,.047,.036),segments=12)
    for s in (-1,1):
        m.ell("Inset lantern eye",h,"sap" if deep else "soul",(s*.055,.033,.102),
              (.022,.020,.012),segments=9)
        m.tube("Spiral shell horn",h,"sun_cream" if deep else shell,[(s*.104,.033,-.011),
               (s*.166,.121,-.028),(s*.176,.211+.046*split,-.019),(s*.124,.249+.047*split,.040)],
               [.051,.059,.034,.005],n=9)
        m.tube("Carved cheek brace",h,shell,[(s*.120,-.010,.040),(s*.102,-.065,.083),
               (s*.066,-.088,.091)],[.023,.031,.014],n=7)
