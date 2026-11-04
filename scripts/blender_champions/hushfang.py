"""Sculpt a low saber-cat around its four planted chains and independent tail."""

from bloomkeeper import leaf

BITE = (0,0,.30)


def hushfang(m,tier,branch,*,production=False):
    stalk = tier > 0 and branch == 0
    midnight = tier > 0 and branch == 1
    fur = "fang_warm" if stalk else "fang_blue" if midnight else "steel_dark"
    trunk(m,fur,stalk,midnight,tier)
    head(m,fur,stalk,midnight,tier)
    for side,s in (("L",1),("R",-1)):
        leg(m,(f"UpperArm{side}",f"Forearm{side}",f"Hand{side}"),s,fur,stalk,midnight)
        leg(m,(f"Thigh{side}",f"Shin{side}",f"Foot{side}"),s,fur,False,False)


def trunk(m,fur,stalk,midnight,tier):
    m.ell("Rounded feline haunch","Hips",fur,(0,.003,-.047),(.169,.147,.225),segments=14)
    m.ell("Low flexible feline midsection","Spine",fur,(0,-.006,.052),(.162,.123,.218),segments=14)
    m.ell("Broad forward feline chest","Chest",fur,(0,.003,.044),(.190,.169,.230),segments=15)
    m.tube("Curved articulated feline tail","BackSpring",fur,[(0,.014,.145),
           (.025,.008,-.129),(.046,.013,-.342),(.060,.011,-.556),
           (.072,.066,-.691),(.049,.144,-.748)],
           [.054,.049,.045,.037,.027,.007],n=10)
    for s in (-1,1):
        for i in range(2):
            z = -.067+i*.091
            m.tube("Raised narrow coat stripe","Chest","steel_dark" if stalk else "ink",
                   [(s*.122,.127,z),(s*.168,.083,z+.018),(s*.181,.021,z+.037)],
                   [.008,.011,.005],n=6)
        if midnight:
            leaf(m,"Front flint shoulder crescent","Chest","crag_fissure",(s*.108,.103,.118),
                 (s*.211,.054,-.099),.056)
        if tier==2:
            for i in range(2):
                leaf(m,"Raised collar shard over the forelegs","Chest","sun_cream" if stalk else "soul",
                     (s*.082,.124,.061+i*.074),(s*.172,.213,.022+i*.074),.045)


def head(m,fur,stalk,midnight,tier):
    h = "Head"
    m.ell("Low predatory feline skull",h,fur,(0,.009,.035),(.154,.128,.148),segments=15)
    m.ell("Long feline muzzle",h,"hide" if stalk else fur,(0,-.039,.171),(.108,.068,.109),segments=13)
    m.ell("Dark nose at native bite",h,"ink",(0,-.008,.266),(.068,.038,BITE[2]-.266),segments=12)
    for s in (-1,1):
        m.tube("Low pointed feline ear",h,fur,[(s*.104,.062,-.020),(s*.128,.122,-.047),
               (s*.148,.182,-.060)],[.044,.036,.003],n=8)
        m.ell("Night hunter eye recess",h,"ink",(s*.104,.037,.147),(.043,.027,.034),segments=10)
        m.ell("Narrow saber-cat eye",h,"soul" if midnight else "sap",(s*.119,.039,.170),
              (.025,.014,.020),segments=9)
        m.tube("Predator brow",h,fur,[(s*.059,.069,.164),(s*.116,.075,.155),
               (s*.141,.050,.117)],[.017,.028,.011],n=7)
        end = -.222 if stalk and tier==2 else -.190 if stalk else -.155
        m.tube("Saber fang",h,"sun_cream",[(s*.077,-.052,.199),(s*.091,-.109,.231),
               (s*.081,end,.252)],[.026,.024,.002],n=8)
        if midnight and tier==2:
            leaf(m,"Midnight swept facial flint",h,"crag_fissure",(s*.078,.062,.082),
                 (s*.173,.152,-.043),.038)


def leg(m,nodes,s,fur,stalk,midnight):
    upper,lower,foot = nodes
    m.tube("Feline articulated upper leg",upper,fur,[(0,.033,0),(s*.011,-.068,.004),
           (0,-.182,0)],[.070,.078,.041],n=9)
    m.tube("Tapered feline shin",lower,fur,[(0,.022,0),(0,-.079,.008),(0,-.181,0)],
           [.048,.050,.036],n=9)
    m.ell("Wide silent padded paw",foot,fur,(0,-.005,.043),(.057,.038,.089),segments=11)
    for x in (-.032,0,.032):
        m.tube("Small curved paw claw",foot,"oldbone",[(x,-.004,.097),(x,-.012,.133),(x,-.018,.149)],
               [.013,.012,.002],n=6)
    if stalk or midnight:
        leaf(m,"Protective plate above front paw",lower,"oldbone" if stalk else "crag_fissure",
             (s*.009,-.152,.032),(s*.061,-.015,.049),.039)
