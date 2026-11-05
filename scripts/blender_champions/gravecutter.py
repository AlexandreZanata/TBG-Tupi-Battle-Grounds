"""Sculpt an undead duelist with linked iron, layered rags and an anchored sickle."""

from math import cos, sin, tau

from bloomkeeper import leaf

TIP = (0,0,.30)


def gravecutter(m,tier,branch,*,production=False):
    grave = tier > 0 and branch == 0
    reaver = tier > 0 and branch == 1
    cloth = "gravecutter_ochre" if grave else "gravecutter_teal" if reaver else "showman_coat"
    costume(m,cloth,grave,reaver,tier)
    skull(m,grave,reaver,tier)
    sickle(m,grave,reaver,tier)


def costume(m,cloth,grave,reaver,tier):
    m.loft("Grave duelist's fitted chest coat","Chest",cloth,[
        (-.043,.113,.088,0),(.052,.136,.093,-.004),(.157,.154,.102,0),
        (.250,.179,.103,-.005),(.327,.101,.073,-.008),
    ],n=14,fold=.078)
    for s in (-1,1):
        x = s*.089
        m.plate("Ragged tail secured to the hip","Hips",cloth,
                [(x-.053,-.008),(x-.072,-.201),(x+.004,-.177),(x+.052,-.223),(x+.071,-.020)],
                (-.091,-.054),bevel=.005)
        m.tube("Diagonal coat lapel","Chest","oldbone",[(s*.033,.087,.098),
               (s*.104,.197,.112),(s*.140,.275,.091)],[.014,.022,.016],n=7)
        side = "L" if s>0 else "R"
        upper,fore,hand,thigh,shin,foot = (f"{p}{side}" for p in
                                           ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Threadbare duelist sleeve",upper,cloth,[(0,.026,0),(0,-.069,0),(0,-.174,0)],
               [.056,.065,.042],n=9)
        m.tube("Skeletal forearm between links",fore,"oldbone",[(0,.022,0),(0,-.055,0),(0,-.153,0)],
               [.039,.043,.031],n=8)
        m.ell("Bone grip beneath sickle",hand,"bone",(0,-.019,.011),(.044,.044,.034),segments=9)
        for joint in (thigh,shin):
            m.tube("Tight rag-wrapped leg",joint,"hide",[(0,.025,0),(0,-.087,0),(0,-.192,0)],
                   [.053,.051,.037],n=8)
        m.ell("Duelist boot",foot,"hide",(0,-.021,.044),(.055,.046,.102),segments=10)
    for i in range(6):
        y = -.009-i*.024
        if i%2:
            m.rim("Interlocked forearm chain link","ForearmR","copper",(0,y,.044),(.020,.019),.006,segments=10)
        else:
            points=[(.012,y+.018*sin(k*tau/10),.043+.020*cos(k*tau/10)) for k in range(11)]
            m.tube("Alternating forearm chain link","ForearmR","iron",points,[.006]*11,n=6)
    if grave:
        m.ell("Ivory coffin-shoulder plate","UpperArmR","sun_cream",(-.023,.003,.010),
              (.107,.077,.097),segments=11)
        if tier==2:
            leaf(m,"Spade crest over the right shoulder","UpperArmR","bone",(-.010,.025,.010),
                 (-.161,.142,-.044),.068)
    if reaver:
        for x in (-.119,.119):
            m.tube("Iron soul-chain yoke","Chest","soul",[(x,.276,.066),(x*.69,.211,.128),
                   (x*.29,.125,.118)],[.008,.010,.006],n=7)


def skull(m,grave,reaver,tier):
    h = "Head"
    m.ell("Tapered bone skull",h,"bone",(0,.009,.014),(.107,.119,.099),segments=14)
    m.ell("Recessed dark jaw",h,"ink",(0,-.073,.068),(.070,.035,.045),segments=10)
    m.loft("Low open grave hood",h,"gravecutter_ochre" if grave else "gravecutter_teal" if reaver else "plum",
           [(-.086,.095,.080,-.022),(.028,.119,.111,-.014),(.114,.092,.087,-.019),
            (.152,.059,.058,-.041)],n=13,opening=.78)
    for s in (-1,1):
        m.ell("Dark duelist eye socket",h,"ink",(s*.046,.016,.101),(.038,.026,.017),segments=9)
        m.ell("Grave pupil",h,"soul" if reaver else "sap",(s*.047,.016,.113),
              (.013,.011,.007),segments=8)
    for x in (-.037,-.012,.012,.037):
        m.box("Separately cut grave tooth",h,"sun_cream",(x,-.067,.106),(.016,.025,.015),.003)
    if reaver and tier==2:
        m.rim("Spectral collar around the hood",h,"soul",(0,.031,-.046),(.154,.161),.013,segments=21)


def sickle(m,grave,reaver,tier):
    w = "WeaponR"
    m.tube("Grave sickle hand grip",w,"hide",[(0,0,-.064),(0,0,.109)],[.028,.029],n=10)
    m.tube("Forged sickle collar",w,"brass",[(0,0,.063),(0,0,.121)],[.034,.038],n=9)
    m.tube("Hooked sickle edge on contact",w,"bone" if grave else "edge",[
        (0,0,.100),(.107,0,.147),(.158 if grave else .125,0,.230),
        (.105,0,.310),TIP,
    ],[.035,.052,.044,.028,.003],n=10)
    if tier==2:
        leaf(m,"Second tooth on sickle hook",w,"soul" if reaver else "sun_cream",
             (.114,.009,.201),(.060,.015,.361),.044,facing=(0,1,0))
