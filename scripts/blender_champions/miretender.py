"""Build a stocky wetland healer with a fitted broth bowl and release ladle."""

from math import cos, sin, tau

from bloomkeeper import leaf
from completion_people import limbs

RELEASE = (0,0,.25)


def miretender(m,tier,branch,*,production=False):
    broth = tier > 0 and branch == 0
    reeds = tier > 0 and branch == 1
    cloth = "mire_amber" if broth else "mire_reed" if reeds else "moss"
    body(m,cloth,broth,reeds,tier)
    face(m,cloth,broth,reeds,tier)
    limbs(m,cloth,"leather",leg=.38*.85,width=.065,skin="ogre")
    utensils(m,broth,reeds,tier)


def body(m,cloth,broth,reeds,tier):
    m.loft("Rounded marsh keeper's coat","Chest",cloth,[
        (-.052,.142,.112,0),(.054,.179,.140,-.006),(.159,.185,.133,-.013),
        (.248,.181,.112,-.012),(.332,.114,.082,-.002),
    ],n=16,fold=.072)
    m.loft("Woven wetland apron","Hips","wicker",[
        (-.212,.174,.116,.039),(-.133,.162,.103,.039),(.024,.151,.093,.025),
    ],n=14,opening=1.31,fold=.14)
    for s in (-1,1):
        m.tube("Apron shoulder cord","Chest","darkwood",[(s*.124,.314,-.012),
               (s*.097,.210,.111),(s*.134,.041,.136)],[.016,.024,.019],n=7)
        if reeds:
            for i in range(3):
                leaf(m,"Reed collar blade","Chest","mire_reed",(s*.101,.286,.046-i*.040),
                     (s*(.235-i*.023),.387-i*.027,-.016-i*.036),.044)
        if broth:
            m.ell("Belt broth flask","Hips","mire_amber",(s*.149,-.018,.112),
                  (.041,.066,.037),segments=10)
            m.box("Leather flask stopper","Hips","wood",(s*.149,.045,.110),(.046,.033,.041),.007)
        if reeds and tier==2:
            m.tube("Long back marsh reed","BackSpring","capy_reed",[(s*.062,-.113,-.017),
                   (s*.110,.092,-.070),(s*.170,.321,-.097)],[.013,.014,.007],n=7)
            m.ell("Brown marsh seed head","BackSpring","wood",(s*.159,.280,-.088),(.020,.049,.021),
                  segments=9)


def face(m,cloth,broth,reeds,tier):
    h="Head"
    m.ell("Wide ogre healer skull",h,"ogre",(0,.006,.016),(.160,.120,.118),segments=15)
    m.ell("Wide mottled healer muzzle",h,"ogre_shade",(0,-.046,.097),(.110,.066,.077),segments=13)
    m.ell("Rounded ogre nose",h,"ogre",(0,-.009,.158),(.045,.038,.034),segments=11)
    for s in (-1,1):
        m.ell("Rounded eye under brim",h,"linen",(s*.075,.030,.116),(.036,.025,.021),segments=10)
        m.ell("Ogre pupil",h,"ink",(s*.079,.029,.133),(.013,.016,.008),segments=8)
        leaf(m,"Blunt pointed swamp ear",h,"ogre",(s*.129,-.027,.017),(s*.218,.039,-.014),.047)
    m.loft("Round woven forager hat","HeadGear","wicker",[(.068,.136,.100,-.014),
           (.081,.215,.164,-.006),(.113,.203,.158,-.009),(.138,.136,.103,-.017),
           (.189,.100,.086,-.022)],n=16)
    m.loft("Color of the woven hat binding","HeadGear",cloth,[(.107,.170,.134,-.013),
           (.131,.154,.118,-.014)],n=15)
    if broth and tier==2:
        leaf(m,"Tall drying leaf on the hat","HeadGear","sun_cream",(.099,.141,-.037),
             (.216,.262,-.079),.046)


def utensils(m,broth,reeds,tier):
    b,w = "OffhandL","WeaponR"
    r = .125 if broth else .109
    m.loft("Clay basin profile",b,"mire_amber" if broth else "mire_reed" if reeds else "wood",
           [(-.090,.043,.042,.048),(-.073,.063,.058,.050),(-.028,r*.91,r*.77,.050),
            (.022,r,r*.84,.050)],n=16)
    rim=[(r*cos(i*tau/16),.023,.050+r*.84*sin(i*tau/16)) for i in range(17)]
    m.tube("Rolled clay bowl lip",b,"brass",rim,[.009]*17,n=7)
    m.ell("Broth surface",b,"soul" if reeds else "sap",(0,.018,.050),(r*.83,.012,r*.68),segments=13)
    m.tube("Wrapped ladle handle",w,"darkwood",[(0,0,-.083),(0,0,0),(0,0,.180)],
           [.022,.024,.018],n=9)
    m.ell("Ladle scoop at release",w,"brass" if broth else "wood",(0,-.009,.185),
          (.059,.034,RELEASE[2]-.185),segments=12)
    if tier==2:
        for s in (-1,1):
            m.tube("Reinforced bowl carrying loop",b,"gold" if broth else "capy_reed",
                   [(s*r*.77,.011,.046),(s*r*1.26,.053,.050),(s*r*1.23,-.004,.080),
                    (s*r*.79,-.041,.078)],[.011,.015,.013,.007],n=7)
