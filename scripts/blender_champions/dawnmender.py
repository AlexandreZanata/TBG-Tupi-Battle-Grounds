"""Tailor a dawn healer with a two-grip lantern and a forward light outlet."""

from math import cos, sin, tau

from bloomkeeper import leaf
from completion_people import face, limbs

RELEASE=(0,0,.25)


def dawnmender(m,tier,branch,*,production=False):
    sanctuary = tier > 0 and branch == 0
    beacon = tier > 0 and branch == 1
    cloth = "dawn_peach" if sanctuary else "dawn_teal" if beacon else "sun_cream"
    costume(m,cloth,sanctuary,beacon,tier)
    limbs(m,cloth,"brass",arm=.19,width=.049)
    lantern(m,sanctuary,beacon,tier)


def costume(m,cloth,sanctuary,beacon,tier):
    m.loft("Soft fitted healer bodice","Chest",cloth,[
        (-.052,.116,.086,0),(.062,.128,.095,-.003),(.163,.144,.103,-.006),
        (.262,.149,.093,-.006),(.320,.080,.062,0),
    ],n=14,fold=.047)
    m.loft("Split light healer skirt","Hips",cloth,[
        (-.320,.177,.132,.006),(-.220,.159,.123,.002),(-.099,.138,.099,-.006),
        (.022,.126,.093,-.005),
    ],n=15,opening=.52,fold=.13)
    face(m)
    h="HeadGear"
    m.loft("Hood with visible face","Head","sun_cream" if sanctuary else "sun_navy" if beacon else cloth,
           [(-.099,.090,.084,-.012),(-.021,.113,.113,-.016),(.097,.119,.111,-.021),
            (.166,.081,.087,-.031),(.210,.044,.056,-.063)],n=15,opening=.81)
    for s in (-1,1):
        m.tube("Embroidered double stole","Chest","sun_cream" if beacon else "brass",[
            (s*.071,.306,.048),(s*.089,.217,.105),(s*.069,.112,.116),(s*.097,.010,.105),
        ],[.013,.021,.018,.011],n=7)
        if sanctuary or beacon:
            leaf(m,"Dawn shoulder petal","Chest","dawn_teal" if beacon else "gold",
                 (s*.082,.275,.045),(s*.228,.335,-.048),.052)
    if tier==2:
        m.rim("Fine dawn halo",h,"soul" if beacon else "brass",(0,.037,-.102),(.180,.191),.013)


def lantern(m,sanctuary,beacon,tier):
    w="WeaponR"
    x=.17 if sanctuary else .14
    r=.153 if sanctuary else .126
    metal="gold" if sanctuary else "dawn_teal" if beacon else "brass"
    m.tube("Crossbar between the two lantern grips",w,"hide",[(0,0,0),(x,0,0),(2*x,0,0)],
           [.028,.029,.028],n=10)
    m.rim("Worked solar lantern frame",w,metal,(x,.026,.027),(r,r),.017)
    m.ell("Suspended lantern heart",w,"soul" if beacon else "sap",(x,.021,.056),
          (.064,.067,.061),segments=14)
    for i in range(6):
        a=tau*i/6
        leaf(m,"Pressed flame petal on the lantern",w,"sun_cream",
             (x+.060*sin(a),.026+.060*cos(a),.051),
             (x+r*sin(a),.026+r*cos(a),.044),.027)
    m.tube("Curved light guide to native release",w,metal,[(x,-.019,.054),
           (x*.44,-.033,.127),(0,0,.207)],[.029,.026,.028],n=8)
    m.rim("Lantern release lens bezel",w,"brass",(0,0,.233),(.036,.035),.011,segments=16)
    m.ell("Light at native release",w,"soul" if beacon else "sap",(0,0,.222),
          (.030,.029,RELEASE[2]-.222),segments=10)
    if sanctuary and tier==2:
        leaf(m,"Tall sanctuary lamp finial",w,"gold",(x,.134,.022),(x,.288,.015),.049)
    if beacon:
        m.rim("Outer teal beacon ring",w,"dawn_teal",(x,.025,.045),(.178,.181),.011)
        if tier==2:
            for s in (-1,1):
                leaf(m,"Aurora chorus petals",w,"soul",(x+s*.116,-.031,.043),
                     (x+s*.213,.173,.029),.049)
