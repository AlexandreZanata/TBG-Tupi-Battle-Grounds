"""Tailor an eclipse duelist with independently carried sun and moon blades."""

from bloomkeeper import leaf
from completion_people import face, limbs

STRIKE=(0,0,.30)


def eclipsedancer(m,tier,branch,*,production=False):
    day = tier > 0 and branch == 0
    night = tier > 0 and branch == 1
    silk = "eclipse_sunset" if day else "eclipse_moon" if night else "sun_navy"
    armor = "brass" if day else "archer_amethyst" if night else "iron"
    tunic(m,silk,armor,day,night,tier)
    portrait(m,silk,armor,day,night,tier)
    limbs(m,silk,"steel_dark",width=.049)
    sword(m,"WeaponR",-1,day,night,tier)
    sword(m,"OffhandL",1,day,night,tier)


def tunic(m,silk,armor,day,night,tier):
    m.loft("Light asymmetric duelist bodice","Chest",silk,[
        (-.052,.101,.086,0),(.061,.121,.096,-.005),(.167,.140,.106,0),
        (.251,.158,.107,-.009),(.320,.084,.067,0),
    ],n=14,fold=.053)
    m.loft("Swirling skirt attached at the belt","Hips",silk,[
        (-.271,.166,.125,.008),(-.179,.153,.111,-.010),(-.052,.122,.097,-.008),
        (.037,.113,.094,0),
    ],n=14,opening=.50,fold=.155)
    m.tube("Asymmetric wrap over duelist bodice","Chest",armor,[(-.118,.287,.075),
           (-.058,.195,.115),(.055,.087,.103),(.110,.042,.095)],[.021,.031,.026,.016],n=8)
    for s in (-1,1):
        leaf(m,"Swirling silk tail","BackSpring",silk,(s*.051,.083,-.013),
             (s*(.208+.020*tier),-.304,-.127),.068)
        if day or night:
            leaf(m,"Rising shoulder petal","Chest",armor,(s*.103,.273,.071),
                 (s*.233,.337,-.051),.053)


def portrait(m,silk,armor,day,night,tier):
    face(m)
    h="Head"
    m.loft("Sculpted duelist hair","HeadGear","hide",[(.035,.105,.082,-.028),
           (.093,.119,.095,-.025),(.148,.104,.082,-.041),(.168,.065,.057,-.057)],n=14)
    m.ell("Coiled rear hair knot","HeadGear","hide",(0,.099,-.116),(.072,.069,.075),segments=11)
    m.tube("Half-moon headband",h,armor,[(-.105,.067,.067),(-.072,.091,.103),
           (0,.096,.116),(.072,.091,.103),(.105,.067,.067)],[.011,.015,.020,.015,.011],n=7)
    if tier==2:
        leaf(m,"Tall diadem fin over temple","HeadGear",armor,(.070,.093,.061),
             (.149,.250,-.020),.045)


def sword(m,w,s,day,night,tier):
    m.tube("Wrapped dual sword grip",w,"hide",[(0,0,-.079),(0,0,.077)],[.027,.027],n=9)
    m.tube("Crossguard flare",w,"gold" if day else "brass",[(s*.073,0,.052),
           (0,0,.075),(-s*.061,0,.058)],[.015,.024,.010],n=8)
    tip=.510 if day and s<0 else .434 if night and s>0 else .377 if s<0 else .332
    tint = "sun_cream" if day else "archer_amethyst" if night else "edge"
    m.tube("Long curved sword spine",w,tint,[(0,0,.079),(s*.025,0,.182),STRIKE,
           (-s*.061,0,tip)],[.031,.040,.032,.003],n=9)
    leaf(m,"Drawn sharp sword edge",w,"sun_cream" if day else "edge",
         (s*.016,-.009,.087),(-s*.061,-.007,tip),.046,facing=(0,1,0))
    if (day and s<0) or (night and s>0):
        leaf(m,"Solar fork" if day else "Lunar crescent hook",w,"gold" if day else "ritual",
             (s*.029,0,.153),(s*(.094 if tier==2 else .068),0,.277),.039,facing=(0,1,0))
