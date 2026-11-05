"""Build eight independently moving spider legs and a braced web spinner."""

from bloomkeeper import leaf

LEGS = (("UpperArm",.20),("Forearm",.06),("Thigh",-.08),("Shin",-.22))
MUZZLE = (0,0,.14)


def weaver(m,tier,branch,*,production=False):
    extended = tier > 0 and branch == 0
    guard = tier > 0 and branch == 1
    shell = "weaver_claret" if guard else "weaver_violet"
    abdomen(m,shell,extended,guard,tier)
    head(m,shell,extended,guard,tier)
    for side,s in (("L",1),("R",-1)):
        for joint,z in LEGS:
            leg(m,f"{joint}{side}",s,z,shell,extended,guard)


def abdomen(m,shell,extended,guard,tier):
    c = "Chest"
    m.ell("Round spider abdomen",c,shell,(0,.007,-.133),(.234,.187,.303),segments=16,rings=8)
    m.ell("Connecting cephalothorax",c,"violet_dark",(0,-.044,.198),(.161,.112,.197),segments=14)
    for s in (-1,1):
        for i in range(3):
            z = -.306+.121*i
            m.tube("Sweeping abdomen ridge",c,"sun_cream" if extended else "horn_stone" if guard else "violet_dark",
                   [(s*.021,.177,z),(s*.127,.159,z-.017),(s*.188,.109,z+.013)],
                   [.017,.026,.010],n=8)
        if guard:
            leaf(m,"Claret abdomen side shield",c,"weaver_claret",(s*.146,.055,.039),
                 (s*.274,.141,-.282),.081)
        if extended:
            leaf(m,"Ivory silk saddle tine",c,"sun_cream",(s*.097,.161,-.019),
                 (s*.230,.210,-.210),.046)
        if extended and tier==2:
            m.tube("Rear silk rack riser","BackSpring","gold",[(s*.124,.100,-.199),
                   (s*.159,.242,-.158),(s*.121,.289,-.096)],[.025,.022,.014],n=8)
    if extended and tier==2:
        m.tube("Exposed silk spool axle","BackSpring","gold",[(-.148,.266,-.116),(.148,.266,-.116)],
               [.026,.026],n=8)
        m.tube("Wound silk spool","BackSpring","sun_cream",[(-.103,.266,-.116),(.103,.266,-.116)],
               [.052,.052],n=11)


def head(m,shell,extended,guard,tier):
    h,w = "Head","WeaponR"
    m.ell("Overlapping front collar",h,"violet_dark",(0,-.001,-.077),(.133,.091,.135),segments=12)
    m.ell("Low spider face",h,shell,(0,.009,.026),(.160,.092,.113),segments=15)
    for s in (-1,1):
        for x,y,z,r in ((.040,.037,.120,.027),(.095,.028,.112,.021),(.130,.012,.087,.015)):
            m.ell("Small independent spider eye",h,"venom" if extended else "bell_eye" if guard else "soul",
                  (s*x,y,z),(r,r*.80,r*.69),segments=9)
        m.tube("Spider chelicera",h,"brass" if guard else "oldbone",[(s*.091,-.049,.081),
               (s*.086,-.083,.134),(s*.060,-.091,.166)],[.031,.027,.005],n=8)
        if guard and tier==2:
            leaf(m,"Guard nozzle wing",w,"gold",(s*.035,-.015,.019),
                 (s*.090,.013,.157),.033)
    r = .042 if extended else .034
    m.tube("Low web spinner body",w,"brass",[(0,0,-.022),(0,0,.061),(0,0,.124)],
           [r*.91,r*.92,r],n=11)
    m.rim("Web spinner muzzle ring",w,"edge",(0,0,MUZZLE[2]-.006),(r,r),.006,segments=14)
    m.ell("Recessed spinner outlet",w,"ink",(0,0,.130),(r*.76,r*.76,.004),segments=9)


def leg(m,node,s,z,shell,extended,guard):
    spread = .352 if guard and z>0 else .299
    step = z*.46
    m.tube("Rising spider thigh on its own pivot",node,shell,[(0,.005,0),
           (s*.131,.072,step*.30),(s*(spread-.043),.029,step)],
           [.044,.048,.032],n=8)
    m.tube("Bent tapered spider shin",node,"sun_cream" if extended else "violet_dark",[
        (s*(spread-.046),.055,step),(s*(spread+.058),-.087,step*1.32),
        (s*(spread+.086),-.220,step*1.56),(s*(spread+.121),-.298,step*1.63),
    ],[.034,.044,.028,.004],n=8)
    m.ell("Joint cap on raised spider knee",node,"brass" if guard else shell,
          (s*(spread-.030),.037,step),(.039,.034,.038),segments=9)
