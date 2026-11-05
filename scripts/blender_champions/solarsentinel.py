"""Forge a shield-bearing sentinel with two separately hinged sun panels."""

from math import cos, sin, tau

from bloomkeeper import leaf

TIP = (0,0,.73)


def solarsentinel(m,tier,branch,*,production=False):
    fortress = tier > 0 and branch == 0
    guard = tier > 0 and branch == 1
    cloth = "sun_teal" if guard else "sun_navy"
    metal = "sun_cream" if fortress else "edge" if guard else "brass"
    armor(m,cloth,metal,fortress,guard,tier)
    helmet(m,metal,fortress,guard,tier)
    spear(m,metal,guard)
    shield(m,metal,fortress,guard,tier)


def armor(m,cloth,metal,fortress,guard,tier):
    c = "Chest"
    m.loft("Layered sentinel cuirass",c,"brass",[
        (-.041,.112,.091,-.002),(.057,.143,.107,.002),(.162,.168,.126,.002),
        (.256,.180,.114,-.010),(.326,.123,.082,-.013),(.353,.085,.069,0),
    ],n=15)
    for s in (-1,1):
        m.tube("Cuirass molded edge",c,metal,[(s*.088,.037,.097),(s*.124,.179,.112),
               (s*.141,.276,.096),(s*.071,.328,.069)],[.013,.018,.021,.012],n=7)
    m.loft("Split sentinel skirt","Hips",cloth,[(-.207,.182,.143,-.005),
           (-.116,.161,.126,-.003),(.002,.134,.106,0),(.063,.142,.113,0)],n=15,opening=.42,fold=.09)
    m.loft("Trailing armored cloak","BackSpring",cloth,[
        (-.302,.181,.064,-.094),(-.218,.158,.060,-.078),(-.080,.116,.047,-.039),
        (.081,.119,.041,.010),
    ],n=13,opening=1.45,fold=.11)
    for side,s in (("L",1),("R",-1)):
        arm,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Cuirass-linked upper sleeve",arm,cloth,[(0,.034,0),(0,-.064,0),(0,-.171,0)],
               [.064,.066,.046],n=9)
        m.ell("Layered sun-guard pauldron",arm,metal,(s*.029,.009,.006),
              (.118 if fortress else .099,.070,.098),segments=12)
        if guard:
            leaf(m,"Sweep of the shoulder guard",arm,"sun_teal",(s*.014,.026,.053),
                 (s*.159,.102,.010),.059)
        if fortress and tier==2:
            m.tube("Second heavy shoulder lamella",arm,"sun_cream",[(s*.036,-.036,.074),
                   (s*.108,-.056,.026),(s*.117,-.079,-.050)],[.025,.036,.018],n=8)
        m.tube("Forearm vambrace",fore,metal,[(0,.018,0),(0,-.038,.006),(0,-.147,0)],
               [.050,.061,.044],n=9)
        m.ell("Closed armored grip",hand,"hide",(0,-.020,.008),(.045,.048,.041),segments=10)
        for node in (thigh,shin):
            m.tube("Cloaked armored leg",node,cloth,[(0,.027,0),(0,-.087,0),(0,-.192,0)],
                   [.057,.054,.040],n=8)
        m.box("Forged shin plate",shin,metal,(0,-.074,.044),(.090,.156,.046),.013)
        m.ell("Forged sentinel boot",foot,"brass",(0,-.024,.051),(.058,.046,.108),segments=11)


def helmet(m,metal,fortress,guard,tier):
    h = "Head"
    m.loft("Closed sentinel helmet",h,"sun_cream",[
        (-.126,.086,.078,.012),(-.073,.120,.110,.007),(.018,.133,.117,-.003),
        (.113,.111,.096,-.011),(.168,.066,.068,-.026),
    ],n=15)
    m.box("Dark visor slit",h,"sun_navy",(0,.024,.114),(.190,.043,.024),.008)
    m.tube("Sculpted brow and nose guard",h,metal,[(0,-.081,.134),(0,.047,.137),
           (0,.107,.112)],[.018,.024,.016],n=7)
    for s in (-1,1):
        m.tube("Fluted cheek guard",h,"brass",[(s*.112,.056,.065),(s*.113,-.015,.079),
               (s*.086,-.089,.105)],[.020,.028,.016],n=8)
    halo = .253 if guard and tier==2 else .195
    m.rim("Secondary sun halo","HeadGear","sun_teal" if guard else "brass",
          (0,.033,-.104),(halo,halo),.015)
    if guard and tier==2:
        for s in (-1,1):
            leaf(m,"Winglet on the sun halo","HeadGear","sun_cream",(s*.178,-.051,-.096),
                 (s*.295,.157,-.109),.054)


def spear(m,metal,guard):
    w = "WeaponR"
    m.tube("Balanced sentinel spear shaft",w,"darkwood",[(0,0,-.290),(0,0,0),(0,0,.575)],
           [.022,.024,.025],n=10)
    m.tube("Leather spear grip",w,"hide",[(0,0,-.059),(0,0,.070)],[.030,.030],n=9)
    m.tube("Cast head collar",w,"brass",[(0,0,.496),(0,0,.570)],[.031,.036],n=9)
    leaf(m,"Forged spear blade at contact",w,metal,(0,0,.529),TIP,.056,facing=(0,1,0))
    if guard:
        for s in (-1,1):
            leaf(m,"Spear head winglet",w,"sun_teal",(0,0,.529),(s*.065,0,.618),.021,facing=(0,1,0))


def shield(m,metal,fortress,guard,tier):
    w = "OffhandL"
    r = .255 if fortress else .232
    m.tube("Round shield blank",w,"brass",[(0,0,.037),(0,0,.075)],[r,r],n=26)
    m.tube("Recessed round shield face",w,"sun_teal" if guard else "sun_cream",
           [(0,0,.074),(0,0,.093)],[r*.85,r*.87],n=26)
    m.rim("Raised outer shield rim",w,metal,(0,0,.084),(r,r),.014)
    m.ell("Convex boss of the sun disk",w,"gold",(0,0,.109),(.069,.069,.040),segments=14)
    for i in range(8):
        a = tau*i/8
        leaf(m,"Raised radial shield ray",w,metal,(.083*sin(a),.083*cos(a),.103),
             (r*1.16*sin(a),r*1.16*cos(a),.089),.033)
    for node,s in (("StringTop",-1),("StringBottom",1)):
        reach = .131 if guard else .100
        leaf(m,"Independently hinged sun vane",node,"sun_teal" if guard else "gold",
             (s*.019,-.148,.094),(s*reach,.186,.096),.059)
        if guard:
            leaf(m,"Raised vane edge",node,"edge",(s*.041,-.094,.115),(s*.128,.155,.116),.027)
    if fortress:
        for s in (-1,1):
            leaf(m,"Reinforced fortress shield spur",w,"sun_cream",(s*.129,.153,.102),
                 (s*.205,.305,.094),.043)
        if tier==2:
            leaf(m,"Raised ivory shield crest",w,"sun_cream",(0,.177,.115),(0,.437,.098),.073)
